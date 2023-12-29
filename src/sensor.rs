use crate::{Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum InPort {
    Port1,
    Port2,
    Port3,
    Port4,
}

impl TryFrom<u8> for InPort {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid InPort"))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum SensorType {
    Touch,
}

impl TryFrom<u8> for SensorType {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid SensorType"))
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[repr(u8)]
pub enum SensorMode {
    Celsius,
}

impl TryFrom<u8> for SensorMode {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid SensorMode"))
    }
}

#[derive(Debug)]
pub struct InputValues {
    pub port: InPort,
    pub valid: bool,
    pub calibrated: bool,
    pub sensor_type: SensorType,
    pub sensor_mode: SensorMode,
    pub raw_value: u16,
    pub normalised_value: u16,
    pub scaled_value: i16,
    pub calibrated_value: i16,
}
