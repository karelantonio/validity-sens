/// List of supported device IDs (only one for now, as im testing it on my sensor.
/// They have the format: (vendor, product)
pub const SUPPORTED: &[(u16, u16)] = &[(0x138a, 0x0097)];

#[derive(thiserror::Error, Debug)]
pub enum DriverError {
    #[error("Could not list devices")]
    ListDevices(#[source] rusb::Error),

    #[error("Could not get descriptior for device")]
    DeviceDescription(#[source] rusb::Error),

    #[error("The USB device was not found")]
    GetDeviceNotFound,

    #[error("The USB device was found but is not supported")]
    GetDeviceFoundUnsupported,
}

/// List the supported USB devices, see also: [`SUPPORTED`]
pub fn list_supported_devices() -> Result<Vec<UsbDevice>, DriverError> {
    let devs = rusb::devices().map_err(DriverError::ListDevices)?;
    let mut res = Vec::new();

    for dev in devs.iter() {
        let desc = dev
            .device_descriptor()
            .map_err(DriverError::DeviceDescription)?;

        for (vid, pid) in SUPPORTED {
            if desc.vendor_id() == *vid && desc.product_id() == *pid {
                res.push(UsbDevice(dev));
                break;
            }
        }
    }

    Ok(res)
}

/// Try to get the USB at the given bus number and address
pub fn get_device(busnum: u8, addr: u8) -> Result<UsbDevice, DriverError> {
    let devs = rusb::devices().map_err(DriverError::ListDevices)?;

    for dev in devs.iter() {
        if dev.bus_number() != busnum || dev.address() != addr {
            continue;
        }

        let desc = dev
            .device_descriptor()
            .map_err(DriverError::DeviceDescription)?;

        for (vid, pid) in SUPPORTED {
            if desc.vendor_id() == *vid && desc.product_id() == *pid {
                return Ok(UsbDevice(dev));
            }
        }

        return Err(DriverError::GetDeviceFoundUnsupported);
    }

    Err(DriverError::GetDeviceNotFound)
}

/// A wrapper around the given device, see [`Self::open`]
pub struct UsbDevice(pub rusb::Device<rusb::GlobalContext>);

impl UsbDevice {
    /// Open this device
    pub fn open(&self) -> u8 {
        todo!()
    }
}
