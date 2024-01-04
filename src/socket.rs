//! Abstraction over various socket types (namely USB and Bluetooth) to
//! allow the base NXT struct to transparently use any supported backend

use crate::Result;

#[cfg(feature = "usb")]
pub mod usb;

/// Abstraction over various socket types (namely USB and Bluetooth) to
/// allow the base NXT struct to transparently use any supported backend
pub trait Socket {
    /// Send the provided data over the socket, returning the number of
    /// bytes sent
    fn send(&self, data: &[u8]) -> Result<usize>;

    /// Receive data from the socket into the provided buffer, returning
    /// the subslice that was read into
    fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]>;
}
