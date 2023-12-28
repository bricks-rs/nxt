use crate::{error::ErrWrap, Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::io::{Cursor, Write};

#[derive(Copy, Clone, Debug, FromPrimitive)]
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
    DirectGetBattLvl = 0x0B,
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
    SystemBtgetadr = 0x9A,
    SystemDeviceinfo = 0x9B,
    SystemDeleteuserflash = 0xA0,
    SystemPollcmdlen = 0xA1,
    SystemPollcmd = 0xA2,
    SystemRenamefile = 0xA3,
    SystemBtfactoryreset = 0xA4,
}

impl Opcode {
    pub fn is_system(&self) -> bool {
        (*self as u8) & 0x80 != 0
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
    EofExpected0x84,
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
    #[error("illegal file name")]
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
    #[error("no active program")]
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
    pub fn error(self) -> Result<()> {
        if let DeviceError::None = self {
            Ok(())
        } else {
            Err(Error::Device(self))
        }
    }
}

#[derive(Copy, Clone, Debug, FromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    Direct = 0x00,
    Systemm = 0x01,
    Reply = 0x02,
    ReplyNotRequired = 0x80,
}

impl TryFrom<u8> for PacketType {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid packet type"))
    }
}

#[derive(Debug)]
pub struct Packet<'buf> {
    pub typ: PacketType,
    pub opcode: Opcode,
    pub data: &'buf [u8],
}

impl<'buf> Packet<'buf> {
    pub fn parse(buf: &'buf [u8]) -> Result<Self> {
        let mut i = buf.iter().copied();

        let typ = i.next().wrap()?;
        let typ = typ.try_into()?;

        let opcode = i.next().wrap()?;
        let opcode = opcode.try_into()?;

        let status = i.next().wrap()?;
        let status = DeviceError::try_from(status)?;
        status.error()?;

        let data = &buf[3..];

        Ok(Self { typ, opcode, data })
    }

    pub fn serialise(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]> {
        let mut cur = Cursor::new(buf);

        cur.write_all(&[self.typ as u8, self.opcode as u8])?;

        let len = cur.position() as usize;
        let buf = cur.into_inner();
        Ok(&buf[..len])
    }

    pub fn read_u16(&self) -> Result<u16> {
        if self.data.len() < 2 {
            Err(Error::Parse("Insufficient data"))
        } else {
            Ok(u16::from_le_bytes([self.data[0], self.data[1]]))
        }
    }
}
