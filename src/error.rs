use std::fmt::Debug;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No NXT brick found")]
    NoBrick,

    #[error("libusb error")]
    Usb(#[from] rusb::Error),

    #[error("device error")]
    Device(#[from] crate::protocol::DeviceError),

    #[error("Parse error")]
    Parse(&'static str),

    #[error("IO error")]
    Io(#[from] std::io::Error),

    #[error("Failed to write packet")]
    Write,
}

pub trait ErrWrap<T> {
    fn wrap(self) -> Result<T>;
}

impl<T> ErrWrap<T> for Option<T> {
    fn wrap(self) -> Result<T> {
        self.ok_or(Error::Parse("Reached end of input"))
    }
}
