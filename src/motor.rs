//! Types and functionality related to motor functions

use crate::{Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::ops::BitOr;

/// Setting the tacho limit to this value will cause the motor to run
/// forever (or at least until further instruction or the batteries run
/// out)
pub const RUN_FOREVER: u32 = 0;

/// Supported output ports and port combinations. Some APIs accept any
/// combination shorthand, while some only support individual ports.
// supported ports are 0, 1, 2 == A, B, C
// 3 == AB, 4 == AC, 5 == BC, 6 == ABC
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
#[allow(missing_docs)]
pub enum OutPort {
    A = 0,
    B = 1,
    C = 2,
    AB = 3,
    AC = 4,
    BC = 5,
    ABC = 6,
    //0xFF is protocol defined to mean "all ports".
    All = 0xff,
}

impl TryFrom<u8> for OutPort {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid OutPort"))
    }
}

/// Bitflags for output mode settings
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
pub struct OutMode(pub(crate) u8);
impl OutMode {
    /// Idle - do not turn motor
    pub const IDLE: Self = Self(0x00);
    /// On - send power to the motor
    pub const ON: Self = Self(0x01);
    /// When power is set to zero, brake the motor rather than letting
    /// it coast
    pub const BRAKE: Self = Self(0x02);
    /// Try to maintain a constant angular velocity by dynamically
    /// adjusting the power, or try to keep two motors synchronised (see
    /// also [`RegulationMode`]).
    pub const REGULATED: Self = Self(0x04);
}

impl From<u8> for OutMode {
    fn from(code: u8) -> Self {
        Self(code)
    }
}

impl BitOr<Self> for OutMode {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

/// What kind of regulation to perform on the motor power
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum RegulationMode {
    /// Do not regulate the power, just send the commanded amout
    #[default]
    Idle = 0,
    /// Try to maintain a constant angular velocity by dynamically
    /// adjusting the power
    Speed = 1,
    /// Try to synchronise two motors
    Sync = 2,
}

impl TryFrom<u8> for RegulationMode {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid RegulationMode"))
    }
}

/// Whether the motor is running or changing its speed
#[derive(Copy, Clone, Debug, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum RunState {
    /// Don't run the motor
    Idle = 0x00,
    /// Gradually increase the power
    RampUp = 0x10,
    /// Run motor at commanded power
    Running = 0x20,
    /// Gradually slow the motor down
    RampDown = 0x40,
}

impl TryFrom<u8> for RunState {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid RunState"))
    }
}

/// Information returned by the `GetOutputState` API. Includes both the
/// commanded configuration and data read from the rotation counters
#[derive(Debug)]
#[allow(missing_docs)]
pub struct OutputState {
    pub port: OutPort,
    pub power: i8,
    pub mode: OutMode,
    pub regulation_mode: RegulationMode,
    pub turn_ratio: i8,
    pub run_state: RunState,
    pub tacho_limit: u32,
    pub tacho_count: i32,
    pub block_tacho_count: i32,
    pub rotation_count: i32,
}
