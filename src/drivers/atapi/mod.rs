use crate::serial_println;
use crate::println;

use lazy_static::lazy_static;
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

// DCR bits
const ATA_INTERRUPT_DISABLE: u8 = 1 << 1;
const ATA_SRST: u8 = 1 << 2;

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

lazy_static! {
    static ref DRIVE: Option<ATABus> = {
        ATABus::discover_atapi_drive()
    };
}

pub fn init() {
    println!("Detecting drives");
    match DRIVE.as_ref() {
        None => println!("No drive detected :("),
        Some(drive) => {
            let drive_type = match drive.current_drive {
                ATA_DRIVE_MASTER => "master",
                ATA_DRIVE_SLAVE => "slave",
                _ => "bad"
            };
            let bus = match drive.base_port {
                ATA_BUS_PRIMARY => "primary",
                ATA_BUS_SECONDARY => "secondary",
                _ => "bad"
            };
            println!("Detected {} drive on {} bus", drive_type, bus);
            serial_println!("Detected drive: {:?}", drive);
        }
    }
}

#[derive(Debug)]
struct ATABus {
    base_port: u16,

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
    fn discover_atapi_drive() -> Option<Self> {
        let mut primary_bus = ATABus::new(ATA_BUS_PRIMARY);

        unsafe {
            primary_bus.dcr.write(ATA_SRST);
            primary_bus.dcr.write(ATA_INTERRUPT_DISABLE);
        }

        primary_bus.select_drive(ATA_DRIVE_MASTER);
        if primary_bus.is_atapi() {
            return Some(primary_bus);
        }

        primary_bus.select_drive(ATA_DRIVE_SLAVE);
        if primary_bus.is_atapi() {
            return Some(primary_bus);
        }

        let mut secondary_bus = ATABus::new(ATA_BUS_SECONDARY);
        
        unsafe {
            secondary_bus.dcr.write(ATA_SRST);
            primary_bus.dcr.write(ATA_INTERRUPT_DISABLE);
        }

        secondary_bus.select_drive(ATA_DRIVE_MASTER);
        if secondary_bus.is_atapi() {
            return Some(secondary_bus);
        }

        secondary_bus.select_drive(ATA_DRIVE_SLAVE);
        if secondary_bus.is_atapi() {
            return Some(secondary_bus);
        }
        None
    }

    fn new(port: u16) -> Self {
        ATABus {
            base_port: port,

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

