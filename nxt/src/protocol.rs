//! Message protocol for packets sent over USB or Bluetooth

use crate::{error::ErrWrap, Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Write};

/// Length of the filename field, including null terminator
const FILENAME_LEN: usize = 20;

/// Message opcodes; see methods in `lib.rs` for details
/// <https://sourceforge.net/p/mindboards/code/HEAD/tree/lms_nbcnxc/trunk/AT91SAM7S256/Source/c_cmd.c#l676>
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
pub enum Opcode {
    DirectStartProgram = 0x00,
    DirectStopProgram = 0x01,
    DirectPlaySoundFile = 0x02,
    DirectPlayTone = 0x03,
    DirectSetOutState = 0x04,
    DirectSetInMode = 0x05,
    DirectGetOutState = 0x06,
    DirectGetInVals = 0x07,
    DirectResetInVal = 0x08,
    DirectMessageWrite = 0x09,
    DirectResetPosition = 0x0A,
    DirectGetBattLevel = 0x0B,
    DirectStopSound = 0x0C,
    DirectKeepAlive = 0x0D,
    DirectLsGetStatus = 0x0E,
    DirectLsWrite = 0x0F,
    DirectLsRead = 0x10,
    DirectGetCurrProgram = 0x11,
    DirectGetButtonState = 0x12,
    DirectMessageRead = 0x13,
    DirectDatalogRead = 0x19,
    DirectDatalogSetTimes = 0x1A,
    DirectBtGetContactCount = 0x1B,
    DirectBtGetContactName = 0x1C,
    DirectBtGetConnCount = 0x1D,
    DirectBtGetConnName = 0x1E,
    DirectSetProperty = 0x1F,
    DirectGetProperty = 0x20,
    DirectUpdateResetCount = 0x21,
    // gap
    SystemOpenread = 0x80,
    SystemOpenwrite = 0x81,
    SystemRead = 0x82,
    SystemWrite = 0x83,
    SystemClose = 0x84,
    SystemDelete = 0x85,
    SystemFindfirst = 0x86,
    SystemFindnext = 0x87,
    SystemVersions = 0x88,
    SystemOpenwritelinear = 0x89,
    SystemOpenreadlinear = 0x8A,
    SystemOpenwritedata = 0x8B,
    SystemOpenappenddata = 0x8C,
    SystemCropdatafile = 0x8D,
    SystemFindfirstmodule = 0x90,
    SystemFindnextmodule = 0x91,
    SystemClosemodhandle = 0x92,
    SystemIomapread = 0x94,
    SystemIomapwrite = 0x95,
    SystemBootcmd = 0x97,
    SystemSetbrickname = 0x98,
    SystemBtgetaddr = 0x9A,
    SystemDeviceinfo = 0x9B,
    SystemDeleteuserflash = 0xA0,
    SystemPollcmdlen = 0xA1,
    SystemPollcmd = 0xA2,
    SystemRenamefile = 0xA3,
    SystemBtfactoryreset = 0xA4,
    // gap
    SystemResizeDataFile = 0xd0,
    SystemSeekFromStart = 0xd1,
    SystemSeekFromCurrent = 0xd2,
    SystemSeekFromEnd = 0xd3,
}

impl Opcode {
    /// Determine whether the opcode is a system call (`true`) or a
    /// direct command (`false`)
    pub const fn is_system(self) -> bool {
        (self as u8) & 0x80 != 0
    }
}

impl TryFrom<u8> for Opcode {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid opcode"))
    }
}

#[derive(Copy, Clone, Debug, FromPrimitive, thiserror::Error)]
#[repr(u8)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum DeviceError {
    #[error("None")]
    None = 0x00,
    #[error("pending communication transaction in progress")]
    InProgress = 0x20,
    #[error("specified mailbox queue is empty")]
    QueueEmpty = 0x40,
    #[error("no more handles")]
    NoMoreHandles = 0x81,
    #[error("no space")]
    NoSpace = 0x82,
    #[error("no more files")]
    NoMoreFiles = 0x83,
    #[error("end of file expected")]
    EofExpected = 0x84,
    #[error("end of file")]
    Eof = 0x85,
    #[error("not a linear file")]
    NotALinearFile = 0x86,
    #[error("file not found")]
    FileNotFound = 0x87,
    #[error("handle already closed")]
    HandleAlreadyClosed = 0x88,
    #[error("no linear space")]
    NoLinearSpace = 0x89,
    #[error("undefined error")]
    Undefined = 0x8A,
    #[error("file is busy")]
    FileBusy = 0x8B,
    #[error("no write buffers")]
    NoWriteBuffers = 0x8C,
    #[error("append not possible")]
    AppendNotPossible = 0x8D,
    #[error("file is full")]
    FileIsFull = 0x8E,
    #[error("file exists")]
    FileExists = 0x8F,
    #[error("module not found")]
    ModuleNotFound = 0x90,
    #[error("out of bounds")]
    OutOfBounds = 0x91,
    #[error("File does not exist")]
    IllegalName = 0x92,
    #[error("illegal handle")]
    IllegalHandle = 0x93,
    #[error("request failed (i.e. specified file not found)")]
    RequestFailed = 0xBD,
    #[error("unknown command opcode")]
    UnknownCommand = 0xBE,
    #[error("insane packet (?)")]
    InsanePacket = 0xBF,
    #[error("data contains out-of-range values")]
    ValueOutOfRange = 0xC0,
    #[error("communication bus error")]
    BusError = 0xDD,
    #[error("no free memory in communication buffer")]
    BufferFull = 0xDE,
    #[error("specified channel/connection is not valid")]
    InvalidChannel = 0xDF,
    #[error("specified channel/connection not configured or busy")]
    UnconfiguredChannel = 0xE0,
    #[error("No active program")]
    NoActiveProgram = 0xEC,
    #[error("illegal size specified")]
    IllegalSize = 0xED,
    #[error("illegal mailbox queue ID specified")]
    IllegalQueueId = 0xEE,
    #[error("attempted to access invalid field of a structure")]
    InvalidField = 0xEF,
    #[error("bad input or output specified")]
    BadInputOrOutput = 0xF0,
    #[error("insufficient memory available")]
    InsufficientMemory = 0xFB,
    #[error("bad arguments")]
    BadArguments = 0xFF,
}

impl TryFrom<u8> for DeviceError {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid status"))
    }
}

impl DeviceError {
    /// Map the error/status value to a `Result`
    pub const fn error(self) -> Result<()> {
        if matches!(self, Self::None) {
            Ok(())
        } else {
            Err(Error::Device(self))
        }
    }
}

/// Types of packet, to differentiate between direct command, system
/// calls and replies
#[derive(Copy, Clone, Debug, FromPrimitive, PartialEq, Eq)]
#[repr(u8)]
#[allow(clippy::missing_docs_in_private_items)]
pub enum PacketType {
    Direct = 0x00,
    System = 0x01,
    Reply = 0x02,
    DirectReplyNotRequired = 0x80,
    SystemReplyNotRequired = 0x80 | 0x01,
}

impl TryFrom<u8> for PacketType {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid packet type"))
    }
}

/// Packet data
#[derive(Debug)]
pub struct Packet {
    /// Header - type of packet
    pub typ: PacketType,
    /// Header - packet opcode
    pub opcode: Opcode,
    /// Serialised packet data
    pub data: Vec<u8>,
    /// Whan parsing, current offset into the packet data
    data_offset: usize,
}

impl Eq for Packet {}
impl PartialEq for Packet {
    // clippy flags the rhs.data_offset as suspicious, but it's correct
    #[allow(clippy::suspicious_operation_groupings)]
    fn eq(&self, rhs: &Self) -> bool {
        self.typ == rhs.typ
            && self.opcode == rhs.opcode
            && self.data[self.data_offset..] == rhs.data[rhs.data_offset..]
    }
}

#[allow(clippy::missing_docs_in_private_items)]
impl Packet {
    pub const fn new(opcode: Opcode) -> Self {
        Self {
            typ: if opcode.is_system() {
                PacketType::System
            } else {
                PacketType::Direct
            },
            opcode,
            data: Vec::new(),
            data_offset: 0,
        }
    }

    pub fn parse(buf: &[u8]) -> Result<Self> {
        let mut pkt = Self {
            typ: 0.try_into()?,
            opcode: 0.try_into()?,
            data: buf.to_vec(),
            data_offset: 0,
        };

        let typ = pkt.read_u8()?;
        pkt.typ = typ.try_into()?;

        let opcode = pkt.read_u8()?;
        pkt.opcode = opcode.try_into()?;

        Ok(pkt)
    }

    pub fn check_status(&mut self) -> Result<()> {
        let status = self.read_u8()?;
        let status = DeviceError::try_from(status)?;
        status.error()
    }

    pub fn serialise<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]> {
        let mut cur = Cursor::new(buf);

        cur.write_all(&[self.typ as u8, self.opcode as u8])?;
        cur.write_all(&self.data)?;

        let len = usize::try_from(cur.position()).unwrap();
        let buf = cur.into_inner();
        Ok(&buf[..len])
    }

    pub fn push_filename(&mut self, name: &str) -> Result<()> {
        // plus one to allow for null terminator
        if name.len() + 1 > FILENAME_LEN {
            Err(Error::Serialise("Filename too long"))
        } else if !name.is_ascii() {
            Err(Error::Serialise("Filename must be ascii"))
        } else {
            self.data.extend(name.bytes());
            self.data.extend(
                std::iter::once(0).cycle().take(FILENAME_LEN - name.len()),
            );
            Ok(())
        }
    }

    pub fn push_str(&mut self, s: &str, max_len: usize) -> Result<()> {
        if s.as_bytes().len() + 1 > max_len {
            return Err(Error::Serialise("String too long"));
        }

        self.data.extend_from_slice(s.as_bytes());
        // enforce null terminator
        self.data.extend(
            std::iter::once(0)
                .cycle()
                .take(max_len - s.as_bytes().len()),
        );
        Ok(())
    }

    pub fn push_bool(&mut self, val: bool) {
        self.data.push(val.into());
    }

    pub fn push_u8(&mut self, val: u8) {
        self.data.push(val);
    }

    pub fn push_i8(&mut self, val: i8) {
        self.data.push(val.to_le_bytes()[0]);
    }

    pub fn push_u16(&mut self, val: u16) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn push_u32(&mut self, val: u32) {
        self.data.extend_from_slice(&val.to_le_bytes());
    }

    pub fn push_slice(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn read_filename(&mut self) -> Result<String> {
        // Read FILENAME_LEN bytes, remove null terminator, parse
        if self.data_offset + FILENAME_LEN > self.data.len() {
            return Err(Error::Parse("Requested slice too long"));
        }

        let fname =
            &self.data[self.data_offset..self.data_offset + FILENAME_LEN];
        self.data_offset += FILENAME_LEN;
        let fname = fname
            .iter()
            .copied()
            .take_while(|&ch| ch != 0)
            .collect::<Vec<_>>();
        Ok(String::from_utf8(fname)?)
    }

    pub fn read_string(&mut self, max_len: usize) -> Result<String> {
        let mut ret = self.read_slice(max_len)?;
        while ret.ends_with(&[0x00]) {
            ret = ret.strip_suffix(&[0x00]).unwrap();
        }
        let ret = ret.to_vec();

        Ok(String::from_utf8(ret)?)
    }

    pub fn read_bool(&mut self) -> Result<bool> {
        let b0 = *self.data.get(self.data_offset).wrap()?;
        self.data_offset += 1;
        Ok(b0 != 0)
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        let b0 = *self.data.get(self.data_offset).wrap()?;
        self.data_offset += 1;
        Ok(b0)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        let b0 = *self.data.get(self.data_offset).wrap()?;
        self.data_offset += 1;
        Ok(i8::from_le_bytes([b0]))
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        let b0 = self.read_u8()?;
        let b1 = self.read_u8()?;
        Ok(u16::from_le_bytes([b0, b1]))
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        let b0 = self.read_u8()?;
        let b1 = self.read_u8()?;
        Ok(i16::from_le_bytes([b0, b1]))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        let b0 = self.read_u8()?;
        let b1 = self.read_u8()?;
        let b2 = self.read_u8()?;
        let b3 = self.read_u8()?;
        Ok(u32::from_le_bytes([b0, b1, b2, b3]))
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        let b0 = self.read_u8()?;
        let b1 = self.read_u8()?;
        let b2 = self.read_u8()?;
        let b3 = self.read_u8()?;
        Ok(i32::from_le_bytes([b0, b1, b2, b3]))
    }

    /// Read a slice of the specified length out of the packet, or an
    /// error if insufficient data available
    pub fn read_slice(&mut self, len: usize) -> Result<&[u8]> {
        if self.data_offset + len > self.data.len() {
            return Err(Error::Parse("Requested slice too long"));
        }

        let data = &self.data[self.data_offset..self.data_offset + len];
        self.data_offset += len;
        Ok(data)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // gen_test_vectors.py
    const BATT_LEVEL: &[u8] = &[0, 11];
    const BRICK_NAME: &[u8] =
        &[1, 152, 116, 101, 115, 116, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

    #[test]
    fn push_filename_adds_20_bytes() {
        let mut pkt = Packet::new(Opcode::DirectStartProgram);
        assert!(pkt.data.is_empty());
        pkt.push_filename("a_file").unwrap();
        assert_eq!(pkt.data.len(), 20);
        assert_eq!(pkt.data, b"a_file\0\0\0\0\0\0\0\0\0\0\0\0\0\0");

        // try some invalid names
        pkt.push_filename("01234abcde01234abcde").unwrap_err();
        pkt.push_filename("01234abcde01234abcde0").unwrap_err();
    }

    #[test]
    fn batt_level() {
        let pkt = Packet::new(Opcode::DirectGetBattLevel);
        let mut buf = [0; 64];
        let ser = pkt.serialise(&mut buf).unwrap();
        assert_eq!(ser, BATT_LEVEL);
        let de = Packet::parse(BATT_LEVEL).unwrap();
        assert_eq!(de, pkt);
    }

    #[test]
    fn brick_name() {
        let mut pkt = Packet::new(Opcode::SystemSetbrickname);
        pkt.push_str("test", 15).unwrap();
        dbg!(&pkt);
        let mut buf = [0; 64];
        let ser = pkt.serialise(&mut buf).unwrap();
        assert_eq!(ser, BRICK_NAME);
        let de = Packet::parse(BRICK_NAME).unwrap();
        assert_eq!(de, pkt);
    }
}
