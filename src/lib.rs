pub use error::{Error, Result};
use rusb::{Device, GlobalContext, UsbContext};

mod error;

pub const NXT_VENDOR: u16 = 0x0694;
pub const NXT_PRODUCT: u16 = 0x0002;

#[derive(Debug)]
pub struct Nxt {
    device: Device<GlobalContext>,
}

fn device_filter<Usb: UsbContext>(dev: &Device<Usb>) -> bool {
    if let Ok(desc) = dev.device_descriptor() {
        desc.vendor_id() == NXT_VENDOR && desc.product_id() == NXT_PRODUCT
    } else {
        false
    }
}

impl Nxt {
    pub fn first() -> Result<Self> {
        rusb::devices()?
            .iter()
            .find(device_filter)
            .map(|device| Nxt { device })
            .ok_or(Error::NoBrick)
    }
}
