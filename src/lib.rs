pub use error::{Error, Result};
use rusb::{Device, DeviceHandle, GlobalContext, UsbContext};
use std::time::Duration;

mod error;
mod protocol;

use protocol::{Opcode, Packet, PacketType};

pub const NXT_VENDOR: u16 = 0x0694;
pub const NXT_PRODUCT: u16 = 0x0002;

const USB_TIMEOUT: Duration = Duration::from_millis(500);
const WRITE_ENDPOINT: u8 = 1;
const READ_ENDPOINT: u8 = 130;

#[derive(Debug)]
pub struct Nxt {
    device: DeviceHandle<GlobalContext>,
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
        let device = rusb::devices()?
            .iter()
            .find(device_filter)
            .ok_or(Error::NoBrick)?;
        let desc = device.device_descriptor()?;
        dbg!(desc);
        let conf = device.config_descriptor(0)?;
        dbg!(conf.interfaces().map(|i| i.number()).collect::<Vec<_>>());
        let mut device = device.open()?;
        device.reset()?;
        dbg!(&device.active_configuration());
        // device.set_active_configuration(0)?;
        // dbg!(&device.active_configuration());
        device.claim_interface(0)?;

        Ok(Nxt { device })
    }

    fn send(&self, pkt: &Packet, buf: &mut [u8]) -> Result<()> {
        let pkt = pkt.serialise(buf)?;
        dbg!();
        let written =
            self.device.write_bulk(WRITE_ENDPOINT, pkt, USB_TIMEOUT)?;
        if written != pkt.len() {
            Err(Error::Write)
        } else {
            Ok(())
        }
    }

    fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<Packet<'buf>> {
        dbg!();
        let read = self.device.read_bulk(READ_ENDPOINT, buf, USB_TIMEOUT)?;

        let buf = &buf[..read];
        println!("{buf:x?}");
        Packet::parse(buf)
    }

    pub fn get_battery_level(&self) -> Result<u16> {
        let pkt = Packet {
            typ: PacketType::Direct,
            opcode: Opcode::DirectGetBattLvl,
            data: &[],
        };

        let mut buf = [0; 64];

        self.send(&pkt, &mut buf)?;
        let recv = self.recv(&mut buf)?;
        dbg!(&recv);
        println!("{:x?}", recv.data);

        recv.read_u16()
    }
}
