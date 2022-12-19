mod scsi;
pub mod interrupt;

use crate::{println, serial_println};
use scsi::{SCSIPacket};
use interrupt::{INTERRUPT_FUTURE};

use core::convert::TryInto;

use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::instructions::port::Port;

const CD_SECTOR_SIZE: usize = 2048;

// Data buses
const ATA_BUS_PRIMARY: u16= 0x1f0;
const ATA_BUS_SECONDARY: u16 = 0x170;

// Drives
const ATA_DRIVE_MASTER: u8 = 0xa0;
const ATA_DRIVE_SLAVE: u8 = 0xb0;

// ATA Commands
const ATA_CMD_PACKET: u8 = 0xa0;

// SCSI commands
const SCSI_READ_12: u8 = 0xa8;

// Status bits
#[allow(dead_code)]
const ATA_ERR: u8 = 1 << 0;
const ATA_DRQ: u8 = 1 << 3;
#[allow(dead_code)]
const ATA_SRV: u8 = 1 << 4;
#[allow(dead_code)]
const ATA_DF: u8 = 1 << 5;
#[allow(dead_code)]
const ATA_RDY: u8 = 1 << 6;
const ATA_BSY: u8 = 1 << 7;

// DCR bits
#[allow(dead_code)]
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
    pub static ref DRIVE: Mutex<Option<ATABus>> = {
        Mutex::new(ATABus::discover_atapi_drive())
    };
}


pub fn init() {
    println!("Detecting drives");
    match DRIVE.lock().as_ref() {
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
        }
    };
    INTERRUPT_FUTURE.pop();
}

#[derive(Debug)]
pub struct ATABus {
    base_port: u16,

    // IO ports
    data: Port<u16>,
    features: Port<u8>, // write
    #[allow(dead_code)]
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

    pub block: [u8; CD_SECTOR_SIZE],
}

impl ATABus {
    fn discover_atapi_drive() -> Option<Self> {
        let mut primary_bus = ATABus::new(ATA_BUS_PRIMARY);

        primary_bus.software_reset();

        primary_bus.select_drive(ATA_DRIVE_MASTER);
        if primary_bus.is_atapi() {
            return Some(primary_bus);
        }

        primary_bus.select_drive(ATA_DRIVE_SLAVE);
        if primary_bus.is_atapi() {
            return Some(primary_bus);
        }

        let mut secondary_bus = ATABus::new(ATA_BUS_SECONDARY);

        secondary_bus.software_reset();

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

    fn software_reset(&mut self) {
        unsafe {
            self.dcr.write(ATA_SRST);
            self.dcr.write(0);
        }
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

            block: [0; CD_SECTOR_SIZE],
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

    fn send_packet(&mut self, packet: SCSIPacket) {
        let raw_packet = packet.serialize();
        self.wait_busy();

        unsafe {
            self.features.write(0);
            self.sector_count.write(0);
            self.address2.write((CD_SECTOR_SIZE & 0xff) as u8);
            self.address3.write(((CD_SECTOR_SIZE >> 8) & 0xff) as u8);
            self.command.write(ATA_CMD_PACKET);
        }

        self.wait_packet_request();

        for i in (0..raw_packet.len()).step_by(2) {
            let word = u16::from_le_bytes(raw_packet[i..i+2].try_into().unwrap());
            unsafe {
                self.data.write(word);
            }
        }
    }

    pub fn sync_read_block(&mut self, lba: u32) {
        let mut packet = SCSIPacket::new();

        packet.op_code = SCSI_READ_12;
        packet.set_lba(lba);
        packet.set_transfer_length(1);

        self.send_packet(packet);

        // Wait packet is transmitted
        let mut transmit: u8 = 0;
        // 0x2 is PACKET_DATA_TRANSMIT
        while transmit != 0x2 {
            unsafe {
                transmit = self.sector_count.read();
            }
        }

        for i in (0..CD_SECTOR_SIZE).step_by(2) {
            unsafe {
                let bytes: [u8; 2] = self.data.read().to_le_bytes();
                self.block[i] = bytes[0];
                self.block[i + 1] = bytes[1];
            }
        }

        // Wait command end
        let mut complete: u8 = 0;
        // 0x3 is PACKET_COMMAND_COMPLETE
        while complete != 0x3 {
            unsafe {
                complete = self.sector_count.read();
            }
        }
        self.wait_command_end();
    }

    pub async fn read_block(&mut self, lba: u32) -> [u8; CD_SECTOR_SIZE] {
        let mut packet = SCSIPacket::new();

        packet.op_code = SCSI_READ_12;
        packet.set_lba(lba);
        packet.set_transfer_length(1);

        self.send_packet(packet);

        // Wait packet is transmitted
        (*INTERRUPT_FUTURE).await;

        let mut _size: usize = 0;
        unsafe {
            _size = ((self.address3.read() as usize) << 8) | self.address2.read() as usize;
        }

        for i in (0..CD_SECTOR_SIZE).step_by(2) {
            unsafe {
                let bytes: [u8; 2] = self.data.read().to_le_bytes();
                self.block[i] = bytes[0];
                self.block[i + 1] = bytes[1];
            }
        }

        // Wait command end
        //(*INTERRUPT_FUTURE).await;

        self.wait_command_end();

        self.block
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
            for _ in 0..100 {
                self.dcr.read();
                self.dcr.read();
                self.dcr.read();
                self.dcr.read();
            }
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

    fn wait_command_end(&mut self) {
        let mut status = ATA_BSY;
        while (status & ATA_BSY) != 0 && (status & ATA_DRQ) != 0 {
            unsafe {
                status = self.status.read();
            }
        }
    }
}


pub async fn print_block() {
    DRIVE.lock().as_mut().unwrap().read_block(500).await;
    serial_println!("{:x?}", DRIVE.lock().as_mut().unwrap().block);
}