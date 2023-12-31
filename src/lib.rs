#![allow(clippy::too_many_arguments)]

pub use error::{Error, Result};
use rusb::{Device, DeviceHandle, GlobalContext, UsbContext};
use std::{
    io::{Cursor, Write},
    sync::Arc,
    time::Duration,
};

#[cfg(feature = "strum")]
pub use strum::IntoEnumIterator;

mod error;
pub mod motor;
mod protocol;
pub mod sensor;
pub mod system;

use motor::*;
use protocol::{Opcode, Packet};
use sensor::*;
use system::*;

pub const NXT_VENDOR: u16 = 0x0694;
pub const NXT_PRODUCT: u16 = 0x0002;

const USB_TIMEOUT: Duration = Duration::from_millis(500);
// https://sourceforge.net/p/mindboards/code/HEAD/tree/lms_nbcnxc/trunk/AT91SAM7S256/Source/d_usb.c
const WRITE_ENDPOINT: u8 = 0x01;
// https://sourceforge.net/p/mindboards/code/HEAD/tree/lms_nbcnxc/trunk/AT91SAM7S256/Source/d_usb.c
const READ_ENDPOINT: u8 = 0x82;
const USB_INTERFACE: u8 = 0;

pub const RUN_FOREVER: u32 = 0;

const MAX_MESSAGE_LEN: usize = 58;
const MAX_NAME_LEN: usize = 15;
const MAX_INBOX_ID: u8 = 19;

const MOD_DISPLAY: u32 = 0xa0001;
const DISPLAY_DATA_OFFSET: u16 = 119;
pub const DISPLAY_WIDTH: usize = 100;
pub const DISPLAY_HEIGHT: usize = 64;
pub const DISPLAY_DATA_LEN: usize = DISPLAY_WIDTH * DISPLAY_HEIGHT / 8;
const DISPLAY_DATA_CHUNK_SIZE: u16 = 32;
const DISPLAY_NUM_CHUNKS: usize =
    DISPLAY_DATA_LEN / DISPLAY_DATA_CHUNK_SIZE as usize;

#[derive(Clone, Debug)]
pub struct Nxt {
    device: Arc<DeviceHandle<GlobalContext>>,
    name: String,
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
        Self::open(device)
    }

    pub fn all() -> Result<Vec<Self>> {
        rusb::devices()?
            .iter()
            .filter(device_filter)
            .map(Nxt::open)
            .collect()
    }

    fn open(device: Device<GlobalContext>) -> Result<Self> {
        let mut device = device.open()?;
        device.claim_interface(USB_INTERFACE)?;
        let name = "".into();

        Ok(Nxt {
            device: device.into(),
            name,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
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
        let mut recv = Packet::parse(buf)?;
        recv.check_status()?;
        if recv.opcode != opcode {
            Err(Error::ReplyMismatch)
        } else {
            Ok(recv)
        }
    }

    fn send_recv(&self, pkt: &Packet) -> Result<Packet> {
        self.send(pkt, false)?;
        self.recv(pkt.opcode)
    }

    pub fn get_display_data(&self) -> Result<[u8; DISPLAY_DATA_LEN]> {
        let out = [0; DISPLAY_DATA_LEN];
        let mut cur = Cursor::new(out);
        for chunk_idx in 0..DISPLAY_NUM_CHUNKS {
            let data = self.read_io_map(
                MOD_DISPLAY,
                DISPLAY_DATA_OFFSET
                    + chunk_idx as u16 * DISPLAY_DATA_CHUNK_SIZE,
                DISPLAY_DATA_CHUNK_SIZE,
            )?;
            assert_eq!(data.len(), DISPLAY_DATA_CHUNK_SIZE.into());
            cur.write_all(&data)?;
        }

        Ok(cur.into_inner())
    }

    pub fn get_battery_level(&self) -> Result<u16> {
        let pkt = Packet::new(Opcode::DirectGetBattLevel);
        let mut recv = self.send_recv(&pkt)?;
        recv.read_u16()
    }

    pub fn get_firmware_version(&self) -> Result<FwVersion> {
        let pkt = Packet::new(Opcode::SystemVersions);
        let mut recv = self.send_recv(&pkt)?;
        let prot_min = recv.read_u8()?;
        let prot_maj = recv.read_u8()?;
        let fw_min = recv.read_u8()?;
        let fw_maj = recv.read_u8()?;
        Ok(FwVersion {
            prot: (prot_maj, prot_min),
            fw: (fw_maj, fw_min),
        })
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

    pub fn set_input_mode(
        &self,
        port: InPort,
        sensor_type: SensorType,
        sensor_mode: SensorMode,
    ) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectSetInMode);
        pkt.push_u8(port as u8);
        pkt.push_u8(sensor_type as u8);
        pkt.push_u8(sensor_mode as u8);
        self.send(&pkt, true)
    }

    pub fn get_output_state(&self, port: OutPort) -> Result<OutputState> {
        let mut pkt = Packet::new(Opcode::DirectGetOutState);
        pkt.push_u8(port as u8);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectGetOutState)?;
        let port = recv.read_u8()?.try_into()?;
        let power = recv.read_i8()?;
        let mode = recv.read_u8()?.into();
        let regulation_mode = recv.read_u8()?.try_into()?;
        let turn_ratio = recv.read_i8()?;
        let run_state = recv.read_u8()?.try_into()?;
        let tacho_limit = recv.read_u32()?;
        let tacho_count = recv.read_i32()?;
        let block_tacho_count = recv.read_i32()?;
        let rotation_count = recv.read_i32()?;

        Ok(OutputState {
            port,
            power,
            mode,
            regulation_mode,
            turn_ratio,
            run_state,
            tacho_limit,
            tacho_count,
            block_tacho_count,
            rotation_count,
        })
    }

    pub fn get_input_values(&self, port: InPort) -> Result<InputValues> {
        let mut pkt = Packet::new(Opcode::DirectGetInVals);
        pkt.push_u8(port as u8);
        let mut recv = self.send_recv(&pkt)?;
        // hdr>>  s  p  v  c  ty mo  raw>>  norm>  sc>>  cal>>
        // [2, 7, 0, 0, 1, 0, 1, 20, ff, 3, ff, 3, 0, 0, ff, 3]
        let port = recv.read_u8()?.try_into()?;
        let valid = recv.read_bool()?;
        let calibrated = recv.read_bool()?;
        let sensor_type = recv.read_u8()?.try_into()?;
        let sensor_mode = recv.read_u8()?.try_into()?;
        let raw_value = recv.read_u16()?;
        let normalised_value = recv.read_u16()?;
        let scaled_value = recv.read_i16()?;
        let calibrated_value = recv.read_i16()?;

        Ok(InputValues {
            port,
            valid,
            calibrated,
            sensor_type,
            sensor_mode,
            raw_value,
            normalised_value,
            scaled_value,
            calibrated_value,
        })
    }

    pub fn reset_input_scaled_value(&self, port: InPort) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectResetInVal);
        pkt.push_u8(port as u8);
        self.send(&pkt, true)
    }

    pub fn message_write(&self, inbox: u8, message: &[u8]) -> Result<()> {
        if inbox > MAX_INBOX_ID {
            return Err(Error::Serialise("Invalid mailbox ID"));
        }
        if message.len() > MAX_MESSAGE_LEN {
            return Err(Error::Serialise("Message too long (max 58 bytes)"));
        }

        let mut pkt = Packet::new(Opcode::DirectMessageWrite);
        pkt.push_u8(inbox);
        pkt.push_u8(message.len() as u8 + 1);
        pkt.push_slice(message);
        pkt.push_u8(0);
        self.send(&pkt, true)
    }

    pub fn reset_motor_position(
        &self,
        port: OutPort,
        relative: bool,
    ) -> Result<()> {
        let mut pkt = Packet::new(Opcode::DirectResetPosition);
        pkt.push_u8(port as u8);
        pkt.push_bool(relative);
        self.send(&pkt, true)
    }

    pub fn stop_sound_playback(&self) -> Result<()> {
        let pkt = Packet::new(Opcode::DirectStopSound);
        self.send(&pkt, true)
    }

    pub fn keep_alive(&self) -> Result<u32> {
        let pkt = Packet::new(Opcode::DirectKeepAlive);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectKeepAlive)?;
        recv.read_u32()
    }

    pub fn ls_get_status(&self, port: InPort) -> Result<u8> {
        let mut pkt = Packet::new(Opcode::DirectLsGetStatus);
        pkt.push_u8(port as u8);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectLsGetStatus)?;
        recv.read_u8()
    }

    pub fn ls_write(
        &self,
        port: InPort,
        tx_data: &[u8],
        rx_bytes: u8,
    ) -> Result<()> {
        // unsure what limit should be here
        // TODO fuzz the lengths or check fw source
        if tx_data.len() > u8::MAX as usize {
            return Err(Error::Serialise("Data too long"));
        }

        let mut pkt = Packet::new(Opcode::DirectLsWrite);
        pkt.push_u8(port as u8);
        pkt.push_u8(tx_data.len() as u8);
        pkt.push_u8(rx_bytes);
        pkt.push_slice(tx_data);
        self.send(&pkt, true)
    }

    pub fn ls_read(&self, port: InPort) -> Result<Vec<u8>> {
        let mut pkt = Packet::new(Opcode::DirectLsRead);
        pkt.push_u8(port as u8);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectLsRead)?;
        let len = recv.read_u8()?;
        let data = recv.read_slice(len as usize)?;
        Ok(data.to_vec())
    }

    pub fn get_current_program_name(&self) -> Result<String> {
        let pkt = Packet::new(Opcode::DirectGetCurrProgram);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectGetCurrProgram)?;
        recv.read_filename()
    }

    pub fn message_read(
        &self,
        remote_inbox: u8,
        local_inbox: u8,
        remove: bool,
    ) -> Result<Vec<u8>> {
        let mut pkt = Packet::new(Opcode::DirectMessageRead);
        pkt.push_u8(remote_inbox);
        pkt.push_u8(local_inbox);
        pkt.push_bool(remove);
        self.send(&pkt, false)?;
        let mut recv = self.recv(Opcode::DirectMessageRead)?;
        let _local_inbox = recv.read_u8()?;
        let len = recv.read_u8()?;
        let data = recv.read_slice(len as usize)?;
        Ok(data.to_vec())
    }

    pub fn file_open_read(&self, name: &str) -> Result<FileHandle> {
        let mut pkt = Packet::new(Opcode::SystemOpenread);
        pkt.push_filename(name)?;
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let len = recv.read_u32()?;
        Ok(FileHandle { handle, len })
    }

    pub fn file_open_write(&self, name: &str, len: u32) -> Result<FileHandle> {
        let mut pkt = Packet::new(Opcode::SystemOpenwrite);
        pkt.push_filename(name)?;
        pkt.push_u32(len);
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        Ok(FileHandle { handle, len })
    }

    pub fn file_read(&self, handle: &FileHandle, len: u32) -> Result<Vec<u8>> {
        let mut pkt = Packet::new(Opcode::SystemOpenread);
        pkt.push_u8(handle.handle);
        pkt.push_u32(len);
        let mut recv = self.send_recv(&pkt)?;
        let _handle = recv.read_u8()?;
        let len = recv.read_u8()?;
        let data = recv.read_slice(len as usize)?;
        Ok(data.to_vec())
    }

    pub fn file_write(&self, handle: &FileHandle, data: &[u8]) -> Result<u32> {
        let mut pkt = Packet::new(Opcode::SystemWrite);
        pkt.push_u8(handle.handle);
        pkt.push_slice(data);
        let mut recv = self.send_recv(&pkt)?;
        let _handle = recv.read_u8()?;
        recv.read_u32()
    }

    pub fn file_close(&self, handle: &FileHandle) -> Result<()> {
        let mut pkt = Packet::new(Opcode::SystemClose);
        pkt.push_u8(handle.handle);
        self.send(&pkt, true)
    }

    pub fn file_delete(&self, name: &str) -> Result<()> {
        let mut pkt = Packet::new(Opcode::SystemDelete);
        pkt.push_filename(name)?;
        self.send(&pkt, true)
    }

    pub fn file_find_first(&self, pattern: &str) -> Result<FindFileHandle> {
        let mut pkt = Packet::new(Opcode::SystemFindfirst);
        pkt.push_filename(pattern)?;
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let name = recv.read_filename()?;
        let len = recv.read_u32()?;
        Ok(FindFileHandle { handle, name, len })
    }

    pub fn file_find_next(
        &self,
        handle: &FindFileHandle,
    ) -> Result<FindFileHandle> {
        let mut pkt = Packet::new(Opcode::SystemFindnext);
        pkt.push_u8(handle.handle);
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let name = recv.read_filename()?;
        let len = recv.read_u32()?;
        Ok(FindFileHandle { handle, name, len })
    }

    pub fn file_open_write_linear(
        &self,
        name: &str,
        len: u32,
    ) -> Result<FileHandle> {
        let mut pkt = Packet::new(Opcode::SystemOpenwritelinear);
        pkt.push_filename(name)?;
        pkt.push_u32(len);
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        Ok(FileHandle { handle, len })
    }

    pub fn file_open_write_data(
        &self,
        name: &str,
        len: u32,
    ) -> Result<FileHandle> {
        let mut pkt = Packet::new(Opcode::SystemOpenwritedata);
        pkt.push_filename(name)?;
        pkt.push_u32(len);
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        Ok(FileHandle { handle, len })
    }

    pub fn file_open_append_data(&self, name: &str) -> Result<FileHandle> {
        let mut pkt = Packet::new(Opcode::SystemOpenappenddata);
        pkt.push_filename(name)?;
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let len = recv.read_u32()?;
        Ok(FileHandle { handle, len })
    }

    pub fn module_find_first(&self, pattern: &str) -> Result<ModuleHandle> {
        let mut pkt = Packet::new(Opcode::SystemFindfirstmodule);
        pkt.push_filename(pattern)?;
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let name = recv.read_filename()?;
        let id = recv.read_u32()?;
        let len = recv.read_u32()?;
        let iomap_len = recv.read_u16()?;
        Ok(ModuleHandle {
            handle,
            name,
            id,
            len,
            iomap_len,
        })
    }

    pub fn module_find_next(
        &self,
        handle: &ModuleHandle,
    ) -> Result<ModuleHandle> {
        let mut pkt = Packet::new(Opcode::SystemFindnextmodule);
        pkt.push_u8(handle.handle);
        let mut recv = self.send_recv(&pkt)?;
        let handle = recv.read_u8()?;
        let name = recv.read_filename()?;
        let id = recv.read_u32()?;
        let len = recv.read_u32()?;
        let iomap_len = recv.read_u16()?;
        Ok(ModuleHandle {
            handle,
            name,
            id,
            len,
            iomap_len,
        })
    }

    pub fn module_close(&self, handle: &ModuleHandle) -> Result<()> {
        let mut pkt = Packet::new(Opcode::SystemClosemodhandle);
        pkt.push_u8(handle.handle);
        self.send(&pkt, true)
    }

    pub fn read_io_map(
        &self,
        mod_id: u32,
        offset: u16,
        count: u16,
    ) -> Result<Vec<u8>> {
        let mut pkt = Packet::new(Opcode::SystemIomapread);
        pkt.push_u32(mod_id);
        pkt.push_u16(offset);
        pkt.push_u16(count);
        let mut recv = self.send_recv(&pkt)?;
        let _mod_id = recv.read_u32()?;
        let len = recv.read_u16()?;
        let data = recv.read_slice(len as usize)?;
        Ok(data.to_vec())
    }

    pub fn write_io_map(
        &self,
        mod_id: u32,
        offset: u16,
        data: &[u8],
    ) -> Result<u16> {
        let mut pkt = Packet::new(Opcode::SystemIomapwrite);
        pkt.push_u32(mod_id);
        pkt.push_u16(offset);
        pkt.push_u16(data.len().try_into()?);
        pkt.push_slice(data);
        let mut recv = self.send_recv(&pkt)?;
        let _mod_id = recv.read_u32()?;
        recv.read_u16()
    }

    pub fn boot(&self, sure: bool) -> Result<Vec<u8>> {
        if !sure {
            return Err(Error::Serialise(
                "Are you sure? This is not recoverable",
            ));
        }

        let mut pkt = Packet::new(Opcode::SystemBootcmd);
        pkt.push_slice(b"Let's dance: SAMBA\0");
        let mut recv = self.send_recv(&pkt)?;
        Ok(recv.read_slice(4)?.to_vec())
    }

    pub fn set_brick_name(&self, name: &str) -> Result<()> {
        let mut pkt = Packet::new(Opcode::SystemSetbrickname);
        pkt.push_str(name, MAX_NAME_LEN)?;
        self.send(&pkt, true)
    }

    pub fn get_device_info(&self) -> Result<DeviceInfo> {
        let pkt = Packet::new(Opcode::SystemDeviceinfo);
        let mut recv = self.send_recv(&pkt)?;
        let name = recv.read_string(MAX_NAME_LEN)?;
        let bt_addr = [
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
        ];
        // unused
        recv.read_u8()?;
        let signal_strength = (
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
            recv.read_u8()?,
        );
        let flash = recv.read_u32()?;

        Ok(DeviceInfo {
            name,
            bt_addr,
            signal_strength,
            flash,
        })
    }

    pub fn delete_user_flash(&self) -> Result<()> {
        let pkt = Packet::new(Opcode::SystemDeleteuserflash);
        self.send(&pkt, true)
    }

    pub fn poll_command_length(&self, buf: BufType) -> Result<u8> {
        let mut pkt = Packet::new(Opcode::SystemPollcmdlen);
        pkt.push_u8(buf as u8);
        let mut recv = self.send_recv(&pkt)?;
        let _buf_num = recv.read_u8()?;
        recv.read_u8()
    }

    pub fn poll_command(&self, buf: BufType, len: u8) -> Result<Vec<u8>> {
        let mut pkt = Packet::new(Opcode::SystemPollcmd);
        pkt.push_u8(buf as u8);
        pkt.push_u8(len);
        let mut recv = self.send_recv(&pkt)?;
        let _buf = recv.read_u8()?;
        let len = recv.read_u8()?;
        let data = recv.read_slice(len as usize)?;
        Ok(data.to_vec())
    }

    pub fn bluetooth_factory_reset(&self) -> Result<()> {
        let pkt = Packet::new(Opcode::SystemBtfactoryreset);
        self.send(&pkt, true)
    }
}
