const CD_SECTOR_SIZE: usize = 2048;


const ATA_BUS_PRIMARY: u16= 0x1f0;
const ATA_BUS_SECONDARY: u16 = 0x170;

const ATA_DRIVE_MASTER: u16 = 0xa0;
const ATA_DRIVE_SLAVE: u16 = 0xb0;

macro_rules! ATA_DATA {
    ($reg:expr) => (reg);
}

macro_rules! ATA_ERROR {
    ($reg:expr) => (reg + 1); // read
}

macro_rules! ATA_FEATURES {
    ($reg:expr) => (reg + 1); // write
}

macro_rules! ATA_SECTOR_COUNT {
    ($reg:expr) => (reg + 2);
}

macro_rules! ATA_ADDRESS1 {
    ($reg:expr) => (reg + 3);
}

macro_rules! ATA_ADDRESS2 {
    ($reg:expr) => (reg + 4);
}

macro_rules! ATA_ADDRESS3 {
    ($reg:expr) => (reg + 5);
}

macro_rules! ATA_DRIVE_SELECT {
    ($reg:expr) => (reg + 6);
}

macro_rules! ATA_STATUS {
    ($reg:expr) => (reg + 7); // read
}

macro_rules! ATA_COMMAND {
    ($reg:expr) => (reg + 7); // write
}

macro_rules! ATA_DCR {
    ($reg:expr) => (reg + 0x206);
}