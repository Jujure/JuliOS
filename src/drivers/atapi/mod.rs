use x86_64::instructions::port::Port;

const CD_SECTOR_SIZE: usize = 2048;

// Data buses
const ATA_BUS_PRIMARY: u16= 0x1f0;
const ATA_BUS_SECONDARY: u16 = 0x170;

// Drives
const ATA_DRIVE_MASTER: u8 = 0xa0;
const ATA_DRIVE_SLAVE: u8 = 0xb0;

// Status bits
const ATA_ERR: u8 = 1 << 0;
const ATA_DRQ: u8 = 1 << 3;
const ATA_SRV: u8 = 1 << 4;
const ATA_DF: u8 = 1 << 5;
const ATA_RDY: u8 = 1 << 6;
const ATA_BSY: u8 = 1 << 7;

// ATAPI signature
const ATAPI_SIG_SC: u8 = 0x01;
const ATAPI_SIG_LBA_LO: u8 = 0x01;
const ATAPI_SIG_LBA_MI: u8 = 0x14;
const ATAPI_SIG_LBA_HI: u8 = 0xeb;

static ATAPI_SIG: [u8; 4] = [
    ATAPI_SIG_SC,
    ATAPI_SIG_LBA_LO,
    ATAPI_SIG_LBA_MI,
    ATAPI_SIG_LBA_HI
];

struct ATABus {
    // IO ports
    data: Port<u8>,
    features: Port<u8>, // write
    error: Port<u8>, // read
    sector_count: Port<u8>,
    address1: Port<u8>,
    address2: Port<u8>,
    address3: Port<u8>,
    drive_select: Port<u8>,
    command: Port<u8>, // write
    status: Port<u8>, // read
    dcr: Port<u8>,

    current_drive: u8,
}

impl ATABus {
    pub fn new(port: u16) -> Self {
        ATABus {
            data: Port::new(port),
            features: Port::new(port + 1), // write
            error: Port::new(port + 1), // read
            sector_count: Port::new(port + 2),
            address1: Port::new(port + 3),
            address2: Port::new(port + 4),
            address3: Port::new(port + 5),
            drive_select: Port::new(port + 6),
            command: Port::new(port + 7), // write
            status: Port::new(port + 7), // read
            dcr: Port::new(port + 0x206),

            current_drive: 0,
        }
    }

    fn select_drive(&mut self, drive: u8) {
        unsafe {
            self.drive_select.write(drive);
        }
        self.select_delay();
        self.current_drive = drive;
    }

    fn is_atapi(&mut self) -> bool {

        let mut sig: [u8; 4] = [0, 0, 0, 0];
        unsafe {
            sig[0] = self.sector_count.read();
            sig[1] = self.address1.read();
            sig[2] = self.address2.read();
            sig[3] = self.address3.read();
        }

        ATAPI_SIG == sig
    }

    fn wait_busy(&mut self) {
        let mut status = ATA_BSY;
        while (status & ATA_BSY) != 0 {
            unsafe {
                status = self.status.read();
            }
        }
    }

    fn select_delay(&mut self) {
        unsafe {
            self.dcr.read();
            self.dcr.read();
            self.dcr.read();
            self.dcr.read();
        }
    }

    fn wait_packet_request(&mut self) {
        let mut status = ATA_BSY;
        while (status & ATA_BSY) != 0 && (status & ATA_DRQ) == 0 {
            unsafe {
                status = self.status.read();
            }
        }
    }
}