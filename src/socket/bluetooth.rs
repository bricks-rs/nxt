//! Bluetooth protocol support

use std::sync::OnceLock;

use super::Socket;
use crate::{Error, Nxt, Result};
use bluer::{
    rfcomm::{self, SocketAddr},
    Adapter, AdapterEvent, Address,
};
use futures::StreamExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::RwLock,
};

/// Observed device class advertised by NXT brick
const NXT_DEVICE_CLASS: u32 = 0x804;
/// RFCOMM channel used by the NXT brick
const NXT_BT_CHAN: u8 = 1;
/// Stores a shared adapter object for performing device scans
static ADAPTER: OnceLock<Adapter> = OnceLock::new();

/// Get a handle to the default Bluetooth adapter, returning the cached
/// one if present
async fn init_adapter() -> Result<&'static Adapter> {
    if let Some(adapter) = ADAPTER.get() {
        Ok(adapter)
    } else {
        let session = bluer::Session::new().await?;
        let adapter = session.default_adapter().await?;
        adapter.set_powered(true).await?;
        Ok(ADAPTER.get_or_init(|| adapter))
    }
}

/// A handle to an NXT brick over Bluetooth
pub struct Bluetooth {
    /// Connected RFCOMM stream
    stream: RwLock<rfcomm::Stream>,
}

impl From<rfcomm::Stream> for Bluetooth {
    fn from(stream: rfcomm::Stream) -> Self {
        Self {
            stream: RwLock::new(stream),
        }
    }
}

impl Bluetooth {
    /// Connect to an NXT brick at the specified address
    pub async fn connect(addr: Address) -> Result<Nxt> {
        let socket = rfcomm::Socket::new()?;
        let stream = socket
            .connect(SocketAddr::new(addr, NXT_BT_CHAN))
            .await?
            .into();
        let conn = Self { stream };
        println!("Connected!");
        crate::Nxt::init(conn).await
    }

    /// Listen for bluetooth connections and try to connec to the first
    /// NXT device discovered
    pub async fn wait_for_nxt() -> Result<crate::Nxt> {
        let adapter = init_adapter().await?;
        let mut device_events = adapter.discover_devices().await?;

        while let Some(evt) = device_events.next().await {
            if let AdapterEvent::DeviceAdded(addr) = evt {
                println!("Device added: {addr:?}");
                // check whether it looks like an NXT
                let device = adapter.device(addr).unwrap();
                if device.class().await.unwrap_or_default()
                    == Some(NXT_DEVICE_CLASS)
                {
                    // device.connect().await?;
                    println!("Device looks like an NXT; connecting");

                    return Self::connect(addr).await;
                }
            }
        }

        Err(Error::NoBrick)
    }
}

#[async_trait::async_trait]
impl Socket for Bluetooth {
    #[tracing::instrument(skip_all)]
    async fn send(&self, data: &[u8]) -> Result<usize> {
        debug!("Writing {} bytes", data.len());
        let len_prefix: u16 = data.len().try_into()?;
        let mut lock = self.stream.write().await;
        let written = lock.write(&len_prefix.to_le_bytes()).await?;
        debug_assert_eq!(written, 2);
        Ok(lock.write(data).await?)
    }

    #[tracing::instrument(skip_all)]
    async fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]> {
        let mut len_prefix = [0; 2];
        let mut lock = self.stream.write().await;
        let len = lock.read_exact(&mut len_prefix).await?;
        debug_assert_eq!(len, len_prefix.len());
        debug!("Expecting {len} bytes from prefix");

        let len_prefix = u16::from_le_bytes(len_prefix).into();
        if buf.len() < len_prefix {
            return Err(Error::Parse("Data too long for buffer"));
        }
        let len = lock.read_exact(&mut buf[..len_prefix]).await?;
        drop(lock);
        debug_assert_eq!(len, len_prefix);

        Ok(&buf[..len])
    }
}
