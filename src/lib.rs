#![allow(clippy::too_many_arguments)]

pub use error::{Error, Result};
use rusb::{Device, DeviceHandle, GlobalContext, UsbContext};
use std::{ops::BitOr, time::Duration};

mod error;
mod protocol;

use protocol::{Opcode, Packet};

pub const NXT_VENDOR: u16 = 0x0694;
pub const NXT_PRODUCT: u16 = 0x0002;

const USB_TIMEOUT: Duration = Duration::from_millis(500);
const WRITE_ENDPOINT: u8 = 1;
const READ_ENDPOINT: u8 = 130;
pub const RUN_FOREVER: u32 = 0;

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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OutPort {
    A = 0,
    B = 1,
    C = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OutMode(u8);
impl OutMode {
    pub const IDLE: Self = Self(0x00);
    pub const ON: Self = Self(0x01);
    pub const BRAKE: Self = Self(0x02);
    pub const REGULATED: Self = Self(0x04);
}

impl BitOr<OutMode> for OutMode {
    type Output = Self;
    fn bitor(self, other: OutMode) -> Self {
        Self(self.0 | other.0)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum RegulationMode {
    #[default]
    Idle = 0,
    Speed = 1,
    Sync = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RunState {
    Idle = 0x00,
    RampUp = 0x10,
    Running = 0x20,
    RampDown = 0x40,
}

impl Nxt {
    pub fn first() -> Result<Self> {
        let device = rusb::devices()?
            .iter()
            .find(device_filter)
            .ok_or(Error::NoBrick)?;
        let mut device = device.open()?;
        device.reset()?;
        device.claim_interface(0)?;

        Ok(Nxt { device })
    }

    fn send(&self, pkt: &Packet, check_status: bool) -> Result<()> {
        let mut buf = [0; 64];
        let serialised = pkt.serialise(&mut buf)?;

        let written =
            self.device
                .write_bulk(WRITE_ENDPOINT, serialised, USB_TIMEOUT)?;
        if written != serialised.len() {
            Err(Error::Write)
        } else {
            if check_status {
                let _recv = self.recv(pkt.opcode)?;
            }
            Ok(())
        }
    }

    fn recv(&self, opcode: Opcode) -> Result<Packet> {
        let mut buf = [0; 64];
        let read =
            self.device
                .read_bulk(READ_ENDPOINT, &mut buf, USB_TIMEOUT)?;

        let buf = &buf[..read];
        println!("{buf:x?}");
        let recv = Packet::parse(buf)?;
        if recv.opcode != opcode {
            Err(Error::ReplyMismatch)
        } else {
            Ok(recv)
        }
    }

    pub fn get_battery_level(&self) -> Result<u16> {
        let pkt = Packet::new(Opcode::DirectGetBattLvl);
        self.send(&pkt, false)?;
        let recv = self.recv(Opcode::DirectGetBattLvl)?;
        recv.read_u16()
    }

    pub fn start_program(&self, name: &str) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectStartProgram);
        pkt.push_filename(name)?;
        self.send(&pkt, true)
    }

    pub fn stop_program(&self) -> Result<()> {
        let pkt = Packet::new(Opcode::DirectStopProgram);
        self.send(&pkt, true)
    }

    pub fn play_sound(&self, file: &str, loop_: bool) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectPlaySoundFile);
        pkt.push_bool(loop_);
        pkt.push_filename(file)?;
        self.send(&pkt, true)
    }

    pub fn play_tone(&self, freq: u16, duration_ms: u16) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectPlayTone);
        pkt.push_u16(freq);
        pkt.push_u16(duration_ms);
        self.send(&pkt, true)
    }

    pub fn set_output_state(
        &self,
        port: OutPort,
        power: i8,
        mode: OutMode,
        regulation_mode: RegulationMode,
        turn_ratio: i8,
        run_state: RunState,
        tacho_limit: u32,
    ) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectSetOutState);
        pkt.push_u8(port as u8);
        pkt.push_i8(power);
        pkt.push_u8(mode.0);
        pkt.push_u8(regulation_mode as u8);
        pkt.push_i8(turn_ratio);
        pkt.push_u8(run_state as u8);
        pkt.push_u32(tacho_limit);
        self.send(&pkt, true)
    }
}
