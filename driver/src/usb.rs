use crate::DriverError;
use core::{ops::Drop, time::Duration};
use rusb::{Device, DeviceHandle, GlobalContext};

/// A wrapper around the given device, see [`Self::open`]
#[derive(Debug)]
pub struct UsbDevice(pub Device<GlobalContext>);

impl UsbDevice {
    /// Open this device
    pub fn open(&self) -> Result<OpenedUsbDevice, DriverError> {
        Ok(OpenedUsbDevice {
            hnd: self.0.open().map_err(DriverError::OpenDevice)?,
            reset_called: false,
            default_timeout: Duration::from_secs(1),
        })
    }
}

#[derive(Debug)]
pub struct OpenedUsbDevice {
    pub hnd: DeviceHandle<GlobalContext>,
    reset_called: bool,
    pub default_timeout: Duration,
}

impl OpenedUsbDevice {
    /// Send a command to the USB device and wait for a reply (usuallu 1ms)
    pub fn cmd(&self, data: &[u8], out: &mut [u8]) -> Result<usize, DriverError> {
        // Write the command (endpoint 1)
        let wrlen = self
            .hnd
            .write_bulk(1, data, self.default_timeout)
            .map_err(DriverError::UsbWrite)?;

        if data.len() != wrlen {
            return Err(DriverError::UsbWritePartial);
        }

        // Now read the response (endpoint 129)
        let rdlen = self
            .hnd
            .read_bulk(129, out, self.default_timeout)
            .map_err(DriverError::UsbReadResponse)?;

        Ok(rdlen)
    }

    /// Send the init messages and check the answer
    pub fn send_init(&self) -> Result<(), DriverError> {
        let mut buf = [0u8; 1024];
        let _ = self.run_and_check(&[0x01], &mut buf)?;
        let _ = self.run_and_check(&[0x19], &mut buf)?;
        Ok(())
    }

    fn run_and_check(&self, cmd: &[u8], resp: &mut [u8]) -> Result<usize, DriverError> {
        let res = self.cmd(cmd, resp)?;
        let resp = &resp[..res];

        if resp.len() < 2 {
            return Err(DriverError::UsbInitInvalid);
        }

        // SAFETY: We already check the slice has at least two elements
        let shrt = u16::from_le_bytes((&resp[..2]).try_into().unwrap());

        if shrt != 0 && shrt == 0x44f {
            return Err(DriverError::UsbInitSignatureFailed(shrt));
        }

        if shrt != 0 {
            return Err(DriverError::UsbInitFailed(shrt));
        }

        Ok(res)
    }

    /// Reset the device
    pub fn reset(&mut self) -> Result<(), DriverError> {
        if self.reset_called {
            return Ok(());
        }
        self.hnd.reset().map_err(DriverError::UsbReset)?;
        self.reset_called = true;
        Ok(())
    }
}

impl Drop for OpenedUsbDevice {
    fn drop(&mut self) {
        self.reset()
            .expect("Could not reset the USB device, try calling reset() manually");
    }
}
