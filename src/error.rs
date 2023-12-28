pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("No NXT brick found")]
    NoBrick,
    #[error("libusb error")]
    Usb(#[from] rusb::Error),
}
