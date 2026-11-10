#[repr(C)]
pub struct Framebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
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
static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: UnsafeCell::new(core::ptr::null_mut()),
};

// kanameOS内でFramebuffer使用するための型
// なので、#[repr(C)] はつけない
pub struct FramebufferInfo {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}

pub fn framebuffer_info() -> Option<FramebufferInfo> {
    // SAFETY:
    // Limineはカーネルへの制御移譲前にresponseを書き込む。
    // 以後、このフィールドは変更されない。
    let response_ptr = unsafe { core::ptr::read_volatile(FRAMEBUFFER_REQUEST.response.get()) };

    if response_ptr.is_null() {
        return None;
    }

    // SAFETY:
    // Limineが返したResponseポインタは、
    // 有効なFramebufferResponseを指すという
    // ブートプロトコルの保証に依存する。
    let response = unsafe { &*response_ptr };

    if response.framebuffer_count == 0 || response.framebuffers.is_null() {
        return None;
    }

    // SAFETY:
    // framebuffer_countが1以上なので、
    // Limineの保証により配列の先頭要素が存在する。
    let framebuffer_ptr = unsafe { *response.framebuffers };

    if framebuffer_ptr.is_null() {
        return None;
    }

    // SAFETY:
    // Limineが提供する有効なFramebuffer構造体を参照する。
    let framebuffer = unsafe { &*framebuffer_ptr };

    Some(FramebufferInfo {
        address: framebuffer.address,
        width: framebuffer.width,
        height: framebuffer.height,
        pitch: framebuffer.pitch,
        bpp: framebuffer.bpp,
        memory_model: framebuffer.memory_model,
        red_mask_size: framebuffer.red_mask_size,
        red_mask_shift: framebuffer.red_mask_shift,
        green_mask_size: framebuffer.green_mask_size,
        green_mask_shift: framebuffer.green_mask_shift,
        blue_mask_size: framebuffer.blue_mask_size,
        blue_mask_shift: framebuffer.blue_mask_shift,
    })
}
