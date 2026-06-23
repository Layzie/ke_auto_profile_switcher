//! IOKit-based event-driven device monitoring (macOS only).
//!
//! Watches HID keyboards (USB and Bluetooth alike) via
//! `IOServiceAddMatchingNotification` on the `IOHIDDevice` service class, giving
//! real-time connect/disconnect callbacks with no polling. Only IORegistry
//! metadata is read (the device is never opened with `IOHIDManagerOpen`), so this
//! does **not** require the Input Monitoring TCC permission.

use crate::error::{AppError, Result};
use crate::monitor::{DeviceEvent, DeviceIdentifier, DeviceInfo, DeviceMonitor};
use core_foundation::base::{CFType, TCFType};
use core_foundation::number::CFNumber;
use core_foundation::runloop::{kCFRunLoopDefaultMode, CFRunLoop, CFRunLoopSource};
use core_foundation::string::CFString;
use core_foundation_sys::base::kCFAllocatorDefault;
use core_foundation_sys::dictionary::CFDictionaryRef;
use io_kit_sys::types::{io_iterator_t, io_registry_entry_t};
use io_kit_sys::{
    kIOMasterPortDefault, IOIteratorNext, IONotificationPortCreate, IONotificationPortDestroy,
    IONotificationPortGetRunLoopSource, IONotificationPortRef, IOObjectRelease,
    IORegistryEntryCreateCFProperty, IORegistryEntryGetRegistryEntryID,
    IOServiceAddMatchingNotification, IOServiceGetMatchingServices, IOServiceMatching,
    IOServiceMatchingCallback,
};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::sync::Arc;

// Service classes and IORegistry/HID property keys (stable-ABI strings).
const IOHID_DEVICE_CLASS: &str = "IOHIDDevice";
const IOUSB_DEVICE_CLASS: &str = "IOUSBHostDevice";
const PROP_TRANSPORT: &str = "Transport";
const PROP_PRODUCT_ID: &str = "ProductID";
const PROP_PRODUCT: &str = "Product";
const PROP_PRIMARY_USAGE: &str = "PrimaryUsage";
const PROP_PRIMARY_USAGE_PAGE: &str = "PrimaryUsagePage";
// IOUSBHostDevice property keys, used only by the `check` listing.
const PROP_USB_PRODUCT_ID: &str = "idProduct";
const PROP_USB_PRODUCT_NAME: &str = "USB Product Name";

// Notification type strings (from IOKitKeys.h).
const NOTIFY_FIRST_MATCH: &str = "IOServiceFirstMatch";
const NOTIFY_TERMINATED: &str = "IOServiceTerminate";

// HID usage page/usage identifying a keyboard.
const USAGE_PAGE_GENERIC_DESKTOP: u16 = 0x01;
const USAGE_KEYBOARD: u16 = 0x06;

/// Raw IORegistry HID properties extracted from an `io_object_t`. FFI-free so the
/// mapping logic can be unit-tested with literal fixtures.
#[derive(Debug, Clone, Default, PartialEq)]
struct RawDeviceProps {
    transport: Option<String>,
    product_id: Option<u16>,
    product_name: Option<String>,
    primary_usage: Option<u16>,
    primary_usage_page: Option<u16>,
}

fn is_keyboard(p: &RawDeviceProps) -> bool {
    p.primary_usage_page == Some(USAGE_PAGE_GENERIC_DESKTOP)
        && p.primary_usage == Some(USAGE_KEYBOARD)
}

/// Map raw HID properties to a `DeviceIdentifier`, or `None` if this is not a
/// keyboard we can switch on (non-keyboard, internal SPI device, USB without a
/// product id, or a Bluetooth device without a usable name).
fn props_to_identifier(p: &RawDeviceProps) -> Option<DeviceIdentifier> {
    if !is_keyboard(p) {
        return None;
    }
    let transport = p.transport.as_deref()?;
    if transport.eq_ignore_ascii_case("USB") {
        p.product_id.map(DeviceIdentifier::usb)
    } else if transport.to_ascii_lowercase().starts_with("bluetooth") {
        match &p.product_name {
            Some(name) if !name.is_empty() => Some(DeviceIdentifier::bluetooth(name.clone())),
            _ => None,
        }
    } else {
        None
    }
}

/// Whether an identifier is one we are configured to watch. Reuses
/// `DeviceIdentifier::matches` for Bluetooth bidirectional partial-name matching.
fn is_monitored(id: &DeviceIdentifier, product_ids: &[u16], bt_names: &[String]) -> bool {
    match id {
        DeviceIdentifier::Usb { product_id } => product_ids.contains(product_id),
        DeviceIdentifier::Bluetooth { .. } => bt_names
            .iter()
            .any(|name| id.matches(&DeviceIdentifier::bluetooth(name.clone()))),
    }
}

/// Read a numeric IORegistry property as `u16`, or `None` if absent/non-numeric.
unsafe fn read_u16_prop(entry: io_registry_entry_t, key: &str) -> Option<u16> {
    read_number_prop(entry, key).map(|n| n as u16)
}

/// Fetch an IORegistry property as an owned `CFType`, or `None` if absent.
/// `IORegistryEntryCreateCFProperty` follows the Create Rule, so the returned
/// reference is wrapped with `wrap_under_create_rule` (released on drop).
unsafe fn read_property(entry: io_registry_entry_t, key: &str) -> Option<CFType> {
    let cf_key = CFString::new(key);
    let value = IORegistryEntryCreateCFProperty(
        entry,
        cf_key.as_concrete_TypeRef(),
        kCFAllocatorDefault,
        0,
    );
    if value.is_null() {
        None
    } else {
        Some(CFType::wrap_under_create_rule(value))
    }
}

unsafe fn read_number_prop(entry: io_registry_entry_t, key: &str) -> Option<i64> {
    read_property(entry, key)?.downcast::<CFNumber>()?.to_i64()
}

unsafe fn read_string_prop(entry: io_registry_entry_t, key: &str) -> Option<String> {
    Some(
        read_property(entry, key)?
            .downcast::<CFString>()?
            .to_string(),
    )
}

unsafe fn extract_props(entry: io_registry_entry_t) -> RawDeviceProps {
    RawDeviceProps {
        transport: read_string_prop(entry, PROP_TRANSPORT),
        product_id: read_u16_prop(entry, PROP_PRODUCT_ID),
        product_name: read_string_prop(entry, PROP_PRODUCT),
        primary_usage: read_u16_prop(entry, PROP_PRIMARY_USAGE),
        primary_usage_page: read_u16_prop(entry, PROP_PRIMARY_USAGE_PAGE),
    }
}

/// The device's stable registry entry id. Readable even on termination (when
/// properties may already be gone), so it is the key for disconnect lookups.
unsafe fn registry_entry_id(entry: io_registry_entry_t) -> Option<u64> {
    let mut id: u64 = 0;
    if IORegistryEntryGetRegistryEntryID(entry, &mut id) == 0 {
        Some(id)
    } else {
        None
    }
}

/// Run `f` for every IOService matching `class`, releasing each object afterward.
unsafe fn for_each_matching_service<F: FnMut(io_registry_entry_t)>(
    class: &str,
    mut f: F,
) -> Result<()> {
    let cstr =
        CString::new(class).map_err(|_| AppError::Monitor("invalid service class".into()))?;
    let matching = IOServiceMatching(cstr.as_ptr());
    if matching.is_null() {
        return Err(AppError::Monitor("IOServiceMatching returned null".into()));
    }
    let mut iterator: io_iterator_t = 0;
    let kr = IOServiceGetMatchingServices(
        kIOMasterPortDefault,
        matching as CFDictionaryRef,
        &mut iterator,
    );
    if kr != 0 {
        return Err(AppError::Monitor(format!(
            "IOServiceGetMatchingServices failed ({})",
            kr
        )));
    }
    loop {
        let obj = IOIteratorNext(iterator);
        if obj == 0 {
            break;
        }
        f(obj);
        IOObjectRelease(obj);
    }
    IOObjectRelease(iterator);
    Ok(())
}

type SharedCallback = Arc<dyn Fn(DeviceEvent) -> Result<()> + Send + Sync + 'static>;

/// Monitor for HID keyboard connections via IOKit notifications. A single
/// instance watches both USB (by product id) and Bluetooth (by device name).
pub struct IoKitMonitor {
    product_ids: Vec<u16>,
    bt_names: Vec<String>,
}

impl IoKitMonitor {
    pub fn new(product_ids: Vec<u16>, bt_names: Vec<String>) -> Self {
        IoKitMonitor {
            product_ids,
            bt_names,
        }
    }
}

/// Context handed to the C trampolines via `refCon`. Accessed only from the
/// single CFRunLoop thread, so interior mutability uses `RefCell`/`Cell`.
struct MonitorContext {
    product_ids: Vec<u16>,
    bt_names: Vec<String>,
    callback: SharedCallback,
    entry_map: RefCell<HashMap<u64, DeviceIdentifier>>,
    is_initial: Cell<bool>,
}

impl MonitorContext {
    fn dispatch(&self, event: DeviceEvent) {
        if let Err(e) = (self.callback)(event) {
            eprintln!("Monitor callback error: {}", e);
        }
    }
}

/// Drain a notification iterator and emit events. Draining fully is mandatory or
/// the notification will not re-arm. `connecting` selects first-match (connect)
/// vs terminate (disconnect) handling.
unsafe fn drain_and_dispatch(ctx: &MonitorContext, iterator: io_iterator_t, connecting: bool) {
    let treat_as_initial = connecting && ctx.is_initial.get();
    let mut initial: Vec<DeviceIdentifier> = Vec::new();

    loop {
        let obj = IOIteratorNext(iterator);
        if obj == 0 {
            break;
        }

        if connecting {
            let props = extract_props(obj);
            if let Some(id) = props_to_identifier(&props) {
                if is_monitored(&id, &ctx.product_ids, &ctx.bt_names) {
                    // Only track (and emit) devices whose stable entry id we can
                    // read, so a later termination always maps back to a
                    // Connected/Initial event. In practice the id is always
                    // available; this keeps connect/disconnect symmetric.
                    if let Some(eid) = registry_entry_id(obj) {
                        ctx.entry_map.borrow_mut().insert(eid, id.clone());
                        if treat_as_initial {
                            initial.push(id);
                        } else {
                            ctx.dispatch(DeviceEvent::Connected(id));
                        }
                    }
                }
            }
        } else if let Some(eid) = registry_entry_id(obj) {
            let removed = ctx.entry_map.borrow_mut().remove(&eid);
            if let Some(id) = removed {
                ctx.dispatch(DeviceEvent::Disconnected(id));
            }
        }

        IOObjectRelease(obj);
    }

    if treat_as_initial {
        ctx.is_initial.set(false);
        if !initial.is_empty() {
            ctx.dispatch(DeviceEvent::Initial(initial));
        }
    }
}

unsafe extern "C" fn first_match_callback(refcon: *mut c_void, iterator: io_iterator_t) {
    let ctx = &*(refcon as *const MonitorContext);
    drain_and_dispatch(ctx, iterator, true);
}

unsafe extern "C" fn terminate_callback(refcon: *mut c_void, iterator: io_iterator_t) {
    let ctx = &*(refcon as *const MonitorContext);
    drain_and_dispatch(ctx, iterator, false);
}

/// Register one matching notification and return its (drainable) iterator.
unsafe fn add_matching_notification(
    notify_port: IONotificationPortRef,
    notif_type: &str,
    class: &str,
    callback: IOServiceMatchingCallback,
    refcon: *mut c_void,
) -> Result<io_iterator_t> {
    let cstr_type = CString::new(notif_type)
        .map_err(|_| AppError::Monitor("invalid notification type".into()))?;
    let cstr_class =
        CString::new(class).map_err(|_| AppError::Monitor("invalid service class".into()))?;
    let matching = IOServiceMatching(cstr_class.as_ptr());
    if matching.is_null() {
        return Err(AppError::Monitor("IOServiceMatching returned null".into()));
    }
    let mut iterator: io_iterator_t = 0;
    let kr = IOServiceAddMatchingNotification(
        notify_port,
        cstr_type.as_ptr() as *mut c_char,
        matching as CFDictionaryRef,
        callback,
        refcon,
        &mut iterator,
    );
    if kr != 0 {
        return Err(AppError::Monitor(format!(
            "IOServiceAddMatchingNotification failed ({})",
            kr
        )));
    }
    Ok(iterator)
}

impl DeviceMonitor for IoKitMonitor {
    fn start_monitoring<F>(&self, callback: F) -> Result<()>
    where
        F: Fn(DeviceEvent) -> Result<()> + Send + Sync + 'static,
    {
        if self.product_ids.is_empty() && self.bt_names.is_empty() {
            return Ok(());
        }

        unsafe {
            let notify_port = IONotificationPortCreate(kIOMasterPortDefault);
            if notify_port.is_null() {
                return Err(AppError::Monitor("IONotificationPortCreate failed".into()));
            }

            let ctx_ptr = Box::into_raw(Box::new(MonitorContext {
                product_ids: self.product_ids.clone(),
                bt_names: self.bt_names.clone(),
                callback: Arc::new(callback),
                entry_map: RefCell::new(HashMap::new()),
                is_initial: Cell::new(true),
            }));
            let refcon = ctx_ptr as *mut c_void;

            let iter_first = match add_matching_notification(
                notify_port,
                NOTIFY_FIRST_MATCH,
                IOHID_DEVICE_CLASS,
                first_match_callback,
                refcon,
            ) {
                Ok(it) => it,
                Err(e) => {
                    IONotificationPortDestroy(notify_port);
                    drop(Box::from_raw(ctx_ptr));
                    return Err(e);
                }
            };

            let iter_term = match add_matching_notification(
                notify_port,
                NOTIFY_TERMINATED,
                IOHID_DEVICE_CLASS,
                terminate_callback,
                refcon,
            ) {
                Ok(it) => it,
                Err(e) => {
                    IONotificationPortDestroy(notify_port);
                    drop(Box::from_raw(ctx_ptr));
                    return Err(e);
                }
            };

            let ctx = &*ctx_ptr;
            // Drain the returned iterators: delivers the initial device set and
            // arms both notifications.
            drain_and_dispatch(ctx, iter_first, true);
            drain_and_dispatch(ctx, iter_term, false);

            let source_ref = IONotificationPortGetRunLoopSource(notify_port);
            let source = CFRunLoopSource::wrap_under_get_rule(source_ref);
            CFRunLoop::get_current().add_source(&source, kCFRunLoopDefaultMode);

            // Blocks until the run loop stops (process exit / Ctrl-C).
            CFRunLoop::run_current();

            IONotificationPortDestroy(notify_port);
            drop(Box::from_raw(ctx_ptr));
        }

        Ok(())
    }

    fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let mut devices = Vec::new();
        unsafe {
            for_each_matching_service(IOHID_DEVICE_CLASS, |obj| {
                let props = extract_props(obj);
                if let Some(id) = props_to_identifier(&props) {
                    let desc = props
                        .product_name
                        .clone()
                        .unwrap_or_else(|| id.display_name());
                    devices.push(DeviceInfo::new(id, desc, true));
                }
            })?;
        }
        Ok(devices)
    }
}

/// Print all USB devices (for the `check` command). Mirrors the previous
/// `usb_enumeration`-based output format.
pub fn list_usb_devices() {
    let mut devices: Vec<(u16, String)> = Vec::new();
    let result = unsafe {
        for_each_matching_service(IOUSB_DEVICE_CLASS, |obj| {
            if let Some(pid) = read_u16_prop(obj, PROP_USB_PRODUCT_ID) {
                let name = read_string_prop(obj, PROP_USB_PRODUCT_NAME)
                    .unwrap_or_else(|| "Unknown Device".to_string());
                devices.push((pid, name));
            }
        })
    };

    if let Err(e) = result {
        println!("  Could not list USB devices: {}", e);
        return;
    }
    if devices.is_empty() {
        println!("No USB devices found.");
        return;
    }
    println!("USB Devices:");
    for (pid, name) in devices {
        println!("  Product ID: {}, Name: {}", pid, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(
        transport: &str,
        pid: Option<u16>,
        name: Option<&str>,
        usage_page: u16,
        usage: u16,
    ) -> RawDeviceProps {
        RawDeviceProps {
            transport: Some(transport.to_string()),
            product_id: pid,
            product_name: name.map(|s| s.to_string()),
            primary_usage: Some(usage),
            primary_usage_page: Some(usage_page),
        }
    }

    #[test]
    fn keyboard_usage_detected() {
        assert!(is_keyboard(&props("USB", Some(1), None, 0x01, 0x06)));
        assert!(!is_keyboard(&props("USB", Some(1), None, 0x01, 0x02))); // mouse
        assert!(!is_keyboard(&RawDeviceProps::default()));
    }

    #[test]
    fn usb_keyboard_maps_to_usb_identifier() {
        let id = props_to_identifier(&props("USB", Some(0x1234), Some("KB"), 0x01, 0x06));
        assert_eq!(id, Some(DeviceIdentifier::usb(0x1234)));
    }

    #[test]
    fn bluetooth_keyboard_maps_to_bt_identifier() {
        let bt = props_to_identifier(&props("Bluetooth", None, Some("HHKB-BT"), 0x01, 0x06));
        assert_eq!(bt, Some(DeviceIdentifier::bluetooth("HHKB-BT")));
        // BluetoothLowEnergy transport is also treated as Bluetooth.
        let ble = props_to_identifier(&props(
            "BluetoothLowEnergy",
            None,
            Some("Keychron"),
            0x01,
            0x06,
        ));
        assert_eq!(ble, Some(DeviceIdentifier::bluetooth("Keychron")));
    }

    #[test]
    fn non_keyboard_and_unknown_transport_yield_none() {
        // Not a keyboard.
        assert_eq!(
            props_to_identifier(&props("USB", Some(1), None, 0x01, 0x02)),
            None
        );
        // Internal (SPI) transport is ignored.
        assert_eq!(
            props_to_identifier(&props("SPI", Some(1), Some("Internal"), 0x01, 0x06)),
            None
        );
        // USB keyboard without a product id cannot be addressed.
        assert_eq!(
            props_to_identifier(&props("USB", None, None, 0x01, 0x06)),
            None
        );
    }

    #[test]
    fn is_monitored_usb_exact_bt_partial() {
        let usb = DeviceIdentifier::usb(0x1234);
        assert!(is_monitored(&usb, &[0x1234], &[]));
        assert!(!is_monitored(&usb, &[0x5678], &[]));

        let bt = DeviceIdentifier::bluetooth("HHKB-BT");
        assert!(is_monitored(&bt, &[], &["HHKB".to_string()])); // partial match
        assert!(!is_monitored(&bt, &[], &["Magic".to_string()]));
    }
}
