#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod gdt;
mod interrupts;
mod vga_buffer;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

fn init() {
    interrupts::init_idt();
    gdt::init();
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    init();
    println!("Hello world!");
    loop {}
}
