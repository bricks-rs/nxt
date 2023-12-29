#[derive(Debug)]
pub struct FileHandle {
    pub(crate) handle: u8,
    pub len: u32,
}

#[derive(Debug)]
pub struct FindFileHandle {
    pub(crate) handle: u8,
    pub name: String,
    pub len: u32,
}

#[derive(Debug)]
pub struct FwVersion {
    pub prot: (u8, u8),
    pub fw: (u8, u8),
}

#[derive(Debug)]
pub struct ModuleHandle {
    pub(crate) handle: u8,
    pub name: String,
    pub id: u32,
    pub len: u32,
    pub iomap_len: u16,
}

#[derive(Debug)]
pub struct DeviceInfo {
    pub name: String,
    pub bt_addr: [u8; 6],
    pub signal_strength: (u8, u8, u8, u8),
    pub flash: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum BufType {
    Usb = 0,
    HighSpeed = 1,
}
