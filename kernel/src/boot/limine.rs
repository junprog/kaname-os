#[repr(C)]
pub struct Framebuffer {
    pub address: *mut u64,
    pub pitch: u64,
    pub width: u64,
    pub height: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
    pub unused: [u8; 7],
    pub edid_size: u64,
    pub edid: *mut u8,
}

const _: () = assert!(core::mem::size_of::<Framebuffer>() == 64);

#[repr(C)]
pub struct FramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *mut *mut Framebuffer,
}

const _: () = assert!(core::mem::size_of::<FramebufferResponse>() == 24);

use core::cell::UnsafeCell;

#[repr(C)]
pub struct FramebufferRequest {
    id: [u64; 4],
    revision: u64,
    response: UnsafeCell<*mut FramebufferResponse>,
}

// SAFETY:
// Limineはカーネルに制御を渡す前にresponseを書き込む。
// カーネル起動後はresponseを読み取り専用として扱い、
// Limineや他のCPUからの書き込みは発生しない。
// また、responseフィールドは非公開とし、外部からの変更を許可しない。
// 以上の条件により、共有参照を通じたアクセスでデータ競合は発生しない。
unsafe impl Sync for FramebufferRequest {}

const _: () = assert!(core::mem::size_of::<FramebufferRequest>() == 48);

#[used]
#[unsafe(link_section = ".limine_requests")]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: UnsafeCell::new(core::ptr::null_mut()),
};
