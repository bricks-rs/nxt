#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::fmt::Debug;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No NXT brick found")]
    NoBrick,

    #[cfg(feature = "usb")]
    #[error("libusb error")]
    Usb(#[from] rusb::Error),

    #[cfg(feature = "bluetooth")]
    #[error("bluetooth error")]
    Bluetooth(#[from] bluer::Error),

    #[error("device error")]
    Device(#[from] crate::protocol::DeviceError),

    #[error("Parse error")]
    Parse(&'static str),

    #[error("Serialisation error")]
    Serialise(&'static str),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Failed to write packet")]
    Write,

    #[error("Reply opcode mismatch")]
    ReplyMismatch,

    #[error("Invalid charactors for string")]
    InvalidString(#[from] std::string::FromUtf8Error),

    #[error("Integer out of range for type")]
    IntOutOfRange(#[from] std::num::TryFromIntError),
}

/// Trait for converting an `Option<T>` into a `Result<T>`
pub trait ErrWrap<T> {
    /// Convert `self` into a `Result`
    fn wrap(self) -> Result<T>;
}

impl<T> ErrWrap<T> for Option<T> {
    fn wrap(self) -> Result<T> {
        self.ok_or(Error::Parse("Reached end of input"))
    }
}
