//!  Handle communications over the USB interface

use rusb::{Device, DeviceHandle, GlobalContext, UsbContext};
use std::time::Duration;

use super::Socket;
use crate::{Error, Result};

/// USB vendor ID used by LEGO
const NXT_VENDOR: u16 = 0x0694;
/// USB product ID used for NXT
const NXT_PRODUCT: u16 = 0x0002;

/// Timeout on the USB connection
const USB_TIMEOUT: Duration = Duration::from_millis(500);
/// USB endpoint address for sending write requests to
/// <https://sourceforge.net/p/mindboards/code/HEAD/tree/lms_nbcnxc/trunk/AT91SAM7S256/Source/d_usb.c>
const WRITE_ENDPOINT: u8 = 0x01;
/// USB endpoint address for sending read requests to
/// <https://sourceforge.net/p/mindboards/code/HEAD/tree/lms_nbcnxc/trunk/AT91SAM7S256/Source/d_usb.c>
const READ_ENDPOINT: u8 = 0x82;
/// USB interface ID used by the NXT brick
const USB_INTERFACE: u8 = 0;

/// Filter method to check the vendor and product ID on a USB device,
/// returning `true` if they match an NXT brick
fn device_filter<Usb: UsbContext>(dev: &Device<Usb>) -> bool {
    dev.device_descriptor().map_or(false, |desc| {
        desc.vendor_id() == NXT_VENDOR && desc.product_id() == NXT_PRODUCT
    })
}

/// A handle to an NXT brick over USB
#[derive(Debug)]
pub struct Usb {
    /// Underlying USB interface device
    device: DeviceHandle<GlobalContext>,
}

#[async_trait::async_trait]
impl Socket for Usb {
    async fn send(&self, data: &[u8]) -> Result<usize> {
        Ok(self.device.write_bulk(WRITE_ENDPOINT, data, USB_TIMEOUT)?)
    }

    async fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]> {
        let read = self.device.read_bulk(READ_ENDPOINT, buf, USB_TIMEOUT)?;
        Ok(&buf[..read])
    }
}

impl Usb {
    /// Search for plugged-in NXT devices and establish a connection to
    /// the first one
    pub fn first() -> Result<Self> {
        let device = rusb::devices()?
            .iter()
            .find(device_filter)
            .ok_or(Error::NoBrick)?;
        Self::open(device)
    }

    /// Connect to all plugged-in NXT bricks and return them in a `Vec`
    pub fn all() -> Result<Vec<Self>> {
        rusb::devices()?
            .iter()
            .filter(device_filter)
            .map(Self::open)
            .collect()
    }

    /// Connect to the provided USB device and claim the [`USB_INTERFACE`]
    /// interface on it
    #[allow(clippy::needless_pass_by_value)]
    fn open(device: Device<GlobalContext>) -> Result<Self> {
        let mut device = device.open()?;
        device.claim_interface(USB_INTERFACE)?;
        Ok(Self { device })
    }
}
