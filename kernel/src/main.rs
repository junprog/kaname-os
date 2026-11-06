#![no_std]
#![no_main]

use core::panic::PanicInfo;

static mut BUFFER: [u8; 4096] = [0; 4096];

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 6];

#[used]
#[unsafe(link_section = ".limine_requests_start")]
static REQUESTS_START: [u64; 2] = [0xf6b8f4b39de7d1ae, 0xfab91a6940fcb9cf];

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

fn debug_putchar(c: u8) {
    unsafe {
        core::arch::asm!(
            "out dx, al",
            in("dx") 0xe9u16,
            in("al") c,
            options(nomem, nostack, preserves_flags)
        );
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    debug_putchar(b'K');

    let base_revision = unsafe {
        let ptr = core::ptr::addr_of!(BASE_REVISION) as *const u64;
        core::ptr::read_volatile(ptr.add(2))
    };

    if base_revision != 0 {
        debug_putchar(b'F');
        loop {
            core::hint::spin_loop();
        }
    }

    debug_putchar(b'B');

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
