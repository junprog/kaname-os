#![no_std]
#![no_main]

use core::panic::PanicInfo;

static mut BUFFER: [u8; 4096] = [0; 4096];

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    unsafe {
        let ptr = core::ptr::addr_of_mut!(BUFFER) as *mut u8;
        core::ptr::write_volatile(ptr, 42);
    }

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
