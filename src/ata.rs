use x86_64::instructions::port::Port;

use crate::println;

const IO_BASE: u16 = 0x1F0;
const DEVICE_CONTROL: u16 = 0x3F6;

struct Ports {
    data: Port<u16>,
    sector_count: Port<u8>,
    lbalo: Port<u8>,
    lbami: Port<u8>,
    lbahi: Port<u8>,
    drive: Port<u8>,
    command_status: Port<u8>,
    device_control: Port<u8>,
}

impl Ports {
    fn new() -> Self {
        Self {
            data: Port::<u16>::new(IO_BASE),
            sector_count: Port::<u8>::new(IO_BASE + 2),
            lbalo: Port::<u8>::new(IO_BASE + 3),
            lbami: Port::<u8>::new(IO_BASE + 4),
            lbahi: Port::<u8>::new(IO_BASE + 5),
            drive: Port::<u8>::new(IO_BASE + 6),
            command_status: Port::<u8>::new(IO_BASE + 7),
            device_control: Port::<u8>::new(DEVICE_CONTROL),
        }
    }
}

unsafe fn wait() {
    let mut command_status_port = Port::<u8>::new(IO_BASE + 7);
    for _ in 0..4 {
        command_status_port.read();
    }
}

unsafe fn disable_interrupts() {
    let mut device_control_port = Port::<u8>::new(DEVICE_CONTROL);
    device_control_port.write(0b00000010);
}

unsafe fn init_drive(drive_0: bool) {
    let mut ports = Ports::new();

    ports.drive.write(if drive_0 { 0xa0 } else { 0xb0 });

    ports.device_control.write(0);
    disable_interrupts();

    ports.sector_count.write(0);
    ports.lbalo.write(0);
    ports.lbami.write(0);
    ports.lbahi.write(0);

    ports.command_status.write(0xEC);

    let mut status = ports.command_status.read();
    if status == 0 {
        panic!("ATA drive does not exist");
    }
    if ports.lbami.read() != 0 || ports.lbahi.read() != 0 {
        panic!("Only ATA drives supported.")
    }

    loop {
        if status & 0b00000001 != 0 {
            panic!("ATA drive had error.")
        }
        if status & 0b00001000 != 0 {
            break;
        }
        status = ports.command_status.read();
    }

    // get resulting identity data
    let mut buffer = [0u8; 512];
    let mut j = 0;
    for _ in 0..256 {
        let d = ports.data.read();
        buffer[j] = d as u8;
        buffer[j + 1] = (d >> 8) as u8;
        j += 2;
    }

    println!("log: ATA drive identified: {}", status);
}

unsafe fn read_sector(drive_0: bool, lba: u32) -> [u8; 512] {
    let mut ports = Ports::new();

    ports
        .drive
        .write((if drive_0 { 0xe0 } else { 0xf0 } | ((lba >> 24) & 0x0F)) as u8);
    ports.sector_count.write(1);
    ports.lbalo.write(lba as u8);
    ports.lbami.write((lba >> 8) as u8);
    ports.lbahi.write((lba >> 16) as u8);

    ports.command_status.write(0x20);
    wait();

    let mut status = ports.command_status.read();
    loop {
        if status & 0b00000001 != 0 {
            panic!("Had error while initializing drive read.")
        }
        if status & 0b00001000 != 0 {
            break;
        }
        status = ports.command_status.read();
    }

    let mut buffer = [0u8; 512];
    let mut j = 0;
    for _ in 0..256 {
        let d = ports.data.read();
        buffer[j] = d as u8;
        buffer[j + 1] = (d >> 8) as u8;
        j += 2;
    }

    buffer
}

unsafe fn cache_flush() {
    let mut ports = Ports::new();
    ports.command_status.write(0xE7);
    let mut status = ports.command_status.read();
    while status & 0b10000000 != 0 {
        status = ports.command_status.read();
    }
}

unsafe fn write_sector(drive_0: bool, lba: u32, data: [u8; 512]) {
    let mut ports = Ports::new();

    ports
        .drive
        .write((if drive_0 { 0xe0 } else { 0xf0 } | ((lba >> 24) & 0x0F)) as u8);
    ports.sector_count.write(1);
    ports.lbalo.write(lba as u8);
    ports.lbami.write((lba >> 8) as u8);
    ports.lbahi.write((lba >> 16) as u8);

    ports.command_status.write(0x30);
    wait();

    let mut status = ports.command_status.read();
    loop {
        if status & 0b00000001 != 0 {
            panic!("Had error while initializing drive write.")
        }
        if status & 0b00001000 != 0 {
            break;
        }
        status = ports.command_status.read();
    }

    for chunk in data.chunks(2) {
        ports.data.write(chunk[0] as u16 | (chunk[1] as u16) << 8);
    }

    cache_flush();
}

pub unsafe fn test() {
    let drive_0 = false;
    unsafe { init_drive(drive_0) }

    // let mut drive_port = Port::<u8>::new(IO_BASE + 6);
    // drive_port.write(if drive_0 { 0xa0 } else { 0xb0 });
    // disable_interrupts();
    let lba = 4;
    write_sector(drive_0, lba, [1u8; 512]);
    let sector = read_sector(drive_0, lba);
    println!("read: {:?} from lba {}", sector, lba);
}
