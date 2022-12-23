use postcard::to_vec;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, Eq, PartialEq)]
#[repr(C, packed)]
pub struct SCSIPacket {
    pub op_code: u8,
    flags_lo: u8,
    lba_hi: u8,
    lba_mihi: u8,
    lba_milo: u8,
    lba_lo: u8,
    transfer_length_hi: u8,
    transfer_length_mihi: u8,
    transfer_length_milo: u8,
    transfer_length_lo: u8,
    flags_hi: u8,
    control: u8,
}

impl SCSIPacket {
    pub fn new() -> Self {
        SCSIPacket::default()
    }

    pub fn serialize(&self) -> heapless::Vec<u8, 12> {
        to_vec(&self).unwrap()
    }

    pub fn set_lba(&mut self, lba: u32) {
        self.lba_lo = (lba & 0xff) as u8;
        self.lba_milo = ((lba >> 0x8) & 0xff) as u8;
        self.lba_mihi = ((lba >> 0x10) & 0xff) as u8;
        self.lba_hi = ((lba >> 0x18) & 0xff) as u8;
    }

    pub fn set_transfer_length(&mut self, l: u32) {
        self.transfer_length_lo = (l & 0xff) as u8;
        self.transfer_length_milo = ((l >> 0x8) & 0xff) as u8;
        self.transfer_length_mihi = ((l >> 0x10) & 0xff) as u8;
        self.transfer_length_hi = ((l >> 0x18) & 0xff) as u8;
    }
}
