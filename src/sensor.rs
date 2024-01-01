//! Types and functionality related to sensor & input functions

use crate::{Error, Result};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "strum", derive(strum_macros::EnumIter))]
#[repr(u8)]
pub enum InPort {
    S1 = 0,
    S2 = 1,
    S3 = 2,
    S4 = 3,
}

impl TryFrom<u8> for InPort {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid InPort"))
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "strum", derive(strum_macros::EnumIter))]
#[repr(u8)]
pub enum SensorType {
    #[default]
    None = 0,
    Switch = 1,
    Temperature = 2,
    Reflection = 3,
    Angle = 4,
    LightActive = 5,
    LightInactive = 6,
    SoundDb = 7,
    SoundDba = 8,
    Custom = 9,
    LowSpeed = 10,
    LowSpeed9v = 11,
    HighSpeed = 12,
    ColourFull = 13,
    ColourRed = 14,
    ColourGreen = 15,
    ColourBlue = 16,
    ColourNone = 17,
    ColourExit = 18,
}

impl TryFrom<u8> for SensorType {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid SensorType"))
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, FromPrimitive)]
#[cfg_attr(feature = "strum", derive(strum_macros::EnumIter))]
#[repr(u8)]
pub enum SensorMode {
    #[default]
    Raw = 0x00,
    Bool = 0x20,
    Edge = 0x40,
    Pulse = 0x60,
    Percent = 0x80,
    Celsius = 0xA0,
    Farenheight = 0xC0,
    Rotation = 0xE0,
}

impl TryFrom<u8> for SensorMode {
    type Error = Error;
    fn try_from(code: u8) -> Result<Self> {
        Self::from_u8(code).ok_or(Error::Parse("Invalid SensorMode"))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
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

impl Display for InputValues {
    fn fmt(&self, fmt: &mut Formatter) -> fmt::Result {
        if self.valid {
            match self.sensor_mode {
                SensorMode::Raw => {
                    write!(fmt, "{}", self.raw_value)
                }
                SensorMode::Bool => {
                    write!(fmt, "{}", self.scaled_value != 0)
                }
                SensorMode::Edge | SensorMode::Pulse => {
                    write!(fmt, "{}", self.scaled_value)
                }
                SensorMode::Percent => {
                    write!(fmt, "{}%", self.scaled_value)
                }
                SensorMode::Celsius => {
                    write!(fmt, "{}°C", self.scaled_value)
                }
                SensorMode::Farenheight => {
                    write!(fmt, "{}°F", self.scaled_value)
                }
                SensorMode::Rotation => {
                    write!(fmt, "{} ticks", self.scaled_value)
                }
            }
        } else {
            write!(fmt, "...")
        }
    }
}
