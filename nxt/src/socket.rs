//! Abstraction over various socket types (namely USB and Bluetooth) to
//! allow the base NXT struct to transparently use any supported backend

use crate::Result;

#[cfg(feature = "usb")]
pub mod usb;

#[cfg(feature = "bluetooth")]
pub mod bluetooth;

/// Abstraction over various socket types (namely USB and Bluetooth) to
/// allow the base NXT struct to transparently use any supported backend
#[async_trait::async_trait]
pub trait Socket {
    /// Send the provided data over the socket, returning the number of
    /// bytes sent
    async fn send(&self, data: &[u8]) -> Result<usize>;

    /// Receive data from the socket into the provided buffer, returning
    /// the subslice that was read into
    async fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]>;
}
