// //! Bluetooth protocol support

// use super::Socket;
// use crate::{Error, Result};
// use bluer::{
//     Adapter, AdapterEvent, Address, DeviceEvent, DiscoveryFilter,
//     DiscoveryTransport,
// };
// use futures::{pin_mut, stream::SelectAll, StreamExt};
// use std::{collections::HashSet, sync::OnceLock};
// use tokio::sync::{mpsc, oneshot};

// type BtMsg = (BtMsgType, oneshot::Sender<BtMsgType>);

// enum BtMsgType {
//     ListDiscovered,
//     DiscoveredDevices { discovered: Vec<Address> },
//     Connect { addr: Address },
//     ConnectStatus { addr: Address },
//     SendReq { addr: Address, pkt: Vec<u8> },
//     SendResp { len: usize },
//     RecvReq,
//     RecvResp { pkt: Vec<u8> },
// }

// impl BtMsgType {
//     fn send_resp(self) -> Result<usize> {
//         let BtMsgType::SendResp(len) = self else {
//             return Err(Error::Parse("Unexpected message type"));
//         };
//         Ok(len)
//     }
//     fn recv_resp(self) -> Result<Vec<u8>> {
//         let BtMsgType::RecvResp { pkt } = self else {
//             return Err(Error::Parse("Unexpected message type"));
//         };
//         Ok(pkt)
//     }
// }

// static BT_TX: OnceLock<mpsc::Sender<BtMsg>> = OnceLock::new();

// /// Observed device class advertised by NXT brick
// const NXT_DEVICE_CLASS: u32 = 0x804;

// fn init_bt() -> mpsc::Sender<BtMsg> {
//     let (tx, rx) = mpsc::channel(10);

//     // spawn a tokio runtime in a background thread
//     std::thread::spawn(move || {
//         let rt = tokio::runtime::Builder::new_current_thread()
//             .build()
//             .unwrap();
//         rt.block_on(bluetooth_background_task(rx));
//     });

//     tx
// }

// async fn bluetooth_background_task(rx: mpsc::Receiver<BtMsg>) {
//     let session = bluer::Session::new().await.unwrap();
//     let adapter = session.default_adapter().await.unwrap();
//     adapter.set_powered(true).await.unwrap();
//     let device_events = adapter.discover_devices().await.unwrap();
//     pin_mut!(device_events);

//     let mut discovered_devices = HashSet::new();
//     loop {
//         tokio::select! {
//             Some(device_event) = device_events.next() => {
//                 handle_device_event(
//                     &adapter,
//                     &mut discovered_devices,
//                     device_event,
//                 ).await;
//             }
//         }
//     }
// }

// async fn handle_device_event(
//     adapter: &Adapter,
//     discovered_devices: &mut HashSet<Address>,
//     device_event: AdapterEvent,
// ) {
//     match device_event {
//         AdapterEvent::DeviceAdded(addr) => {
//             println!("Device added: {addr:?}");
//             // check whether it looks like an NXT
//             let device = adapter.device(addr).unwrap();
//             if device.class().await.unwrap_or_default()
//                 == Some(NXT_DEVICE_CLASS)
//             {
//                 discovered_devices.insert(addr);
//             }
//         }
//         AdapterEvent::DeviceRemoved(addr) => {
//             println!("Device removed: {addr:?}");
//             discovered_devices.remove(&addr);
//         }
//         AdapterEvent::PropertyChanged(_) => {}
//     }
// }

pub struct Bluetooth {
    // tx: mpsc::Sender<BtMsg>,
    // addr: Option<Address>,
}

// impl Default for Bluetooth {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl Bluetooth {
//     pub fn new() -> Self {
//         let tx = BT_TX.get_or_init(init_bt).clone();
//         Self { tx, addr: None }
//     }

//     pub fn connect(&self, addr: Address) -> Result<()> {
//         let (tx, rx) = oneshot::channel();
//         self.tx.blocking_send(())
//     }
// }

// impl Socket for Bluetooth {
//     fn send(&self, data: &[u8]) -> Result<usize> {
//         let (tx, rx) = oneshot::channel();
//         let Some(addr) = self.addr else {
//             return Err(Error::NoBrick);
//         };
//         self.tx
//             .blocking_send((
//                 BtMsgType::SendReq {
//                     addr,
//                     pkt: data.to_vec(),
//                 },
//                 tx,
//             ))
//             .unwrap();
//         Ok(rx.blocking_recv().unwrap().send_resp().unwrap())
//     }

//     fn recv<'buf>(&self, buf: &'buf mut [u8]) -> Result<&'buf [u8]> {
//         let (tx, rx) = oneshot::channel();
//         self.tx.blocking_send((BtMsgType::RecvReq, tx)).unwrap();
//         let recv = rx.blocking_recv().unwrap().recv_resp().unwrap();
//         if recv.len() > buf.len() {
//             Err(Error::Parse("Message longer than buffer"))
//         } else {
//             buf[..recv.len()].copy_from_slice(&recv);
//             Ok(&buf[..recv.len()])
//         }
//     }
// }
