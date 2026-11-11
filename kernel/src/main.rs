#![no_std]
#![no_main]

mod boot;
mod graphics;

use core::panic::PanicInfo;
use graphics::framebuffer::Framebuffer;

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

fn debug_hex(value: u64) {
    for i in (0..16).rev() {
        let digit = ((value >> (i * 4)) & 0xF) as u8;

        let c = if digit < 10 {
            b'0' + digit
        } else {
            b'A' + (digit - 10)
        };

        debug_putchar(c);
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

    match boot::limine::framebuffer_info() {
        Some(info) => {
            debug_putchar(b'\n');

            debug_putchar(b'W');
            debug_hex(info.width);

            debug_putchar(b'\n');

            debug_putchar(b'H');
            debug_hex(info.height);

            debug_putchar(b'\n');

            debug_putchar(b'P');
            debug_hex(info.pitch);

            debug_putchar(b'\n');

            debug_putchar(b'B');
            debug_hex(info.bpp as u64);

            debug_putchar(b'\n');

            debug_putchar(b'A');
            debug_hex(info.address as u64);

            // ピクセルのメモリモデル
            debug_putchar(b'\n');
            debug_putchar(b'M');
            debug_hex(info.memory_model as u64);

            // 赤のビット数とシフト量
            debug_putchar(b'\n');
            debug_putchar(b'R');
            debug_hex(info.red_mask_size as u64);
            debug_putchar(b':');
            debug_hex(info.red_mask_shift as u64);

            // 緑のビット数とシフト量
            debug_putchar(b'\n');
            debug_putchar(b'G');
            debug_hex(info.green_mask_size as u64);
            debug_putchar(b':');
            debug_hex(info.green_mask_shift as u64);

            // 青のビット数とシフト量
            debug_putchar(b'\n');
            debug_putchar(b'U');
            debug_hex(info.blue_mask_size as u64);
            debug_putchar(b':');
            debug_hex(info.blue_mask_shift as u64);

            // SAFETY:
            // Limineが提供したFramebufferのメモリを使用する。
            // カーネルはこのFramebufferを単独で管理し、
            // 他のコードから競合するアクセスを行わない。
            let Some(mut framebuffer) = (unsafe { Framebuffer::new(info) }) else {
                debug_putchar(b'E');

                loop {
                    core::hint::spin_loop();
                }
            };

            debug_putchar(b'\n');

            if framebuffer.verify_offset() {
                debug_putchar(b'O');
            } else {
                debug_putchar(b'E');
            }

            debug_putchar(b'\n');

            if framebuffer.is_supported() {
                debug_putchar(b'S');
            } else {
                debug_putchar(b'E');
            }

            // 画面中央付近に赤いピクセルを描画する。
            if framebuffer.put_pixel(640, 400, 255, 0, 0) {
                debug_putchar(b'P');
            } else {
                debug_putchar(b'E');
            }

            // 画面中央付近に赤い矩形を描画する。
            framebuffer.fill_rect(540, 350, 200, 100, 255, 0, 0);
            debug_putchar(b'R');

            // 実験
            // 先に青い矩形を描く。
            framebuffer.fill_rect(540, 350, 200, 100, 0, 0, 255);

            // その後、中央に赤いピクセルを描く。
            framebuffer.put_pixel(640, 400, 255, 0, 0);

            // 画面に白い文字Aを描画する。
            framebuffer.draw_char(640, 400, 255, 255, 255);
            debug_putchar(b'T');
        }
        None => debug_putchar(b'N'),
    }

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
