// Framebufferを使ったピクセル描画を担当するモジュール。
// ハードウェアへのメモリ書き込みは、このモジュール内に閉じ込める。

use crate::boot::limine::FramebufferInfo;
use crate::graphics::font::FONT_A;

// Framebufferの描画処理を管理する構造体
pub struct Framebuffer {
    info: FramebufferInfo,
}

impl Framebuffer {
    /// # Safety
    ///
    /// 呼び出し側は以下を保証すること。
    /// - info.addressが有効なFramebufferメモリを指すこと
    /// - 対象メモリが書き込み可能であること
    /// - 必要なメモリ領域が有効であり続けること
    /// - Framebufferへの競合するアクセスがないこと
    // Limineから取得した情報を保持する。
    pub unsafe fn new(info: FramebufferInfo) -> Option<Self> {
        let framebuffer = Self { info };

        if framebuffer.is_supported() {
            Some(framebuffer)
        } else {
            None
        }
    }

    // 指定した座標のバイトオフセットを計算する。
    fn pixel_offset(&self, x: u64, y: u64) -> Option<usize> {
        if x >= self.info.width || y >= self.info.height {
            return None;
        }

        let bytes_per_pixel = u64::from(self.info.bpp).checked_add(7)? / 8;

        let row_offset = y.checked_mul(self.info.pitch)?;
        let column_offset = x.checked_mul(bytes_per_pixel)?;
        let offset = row_offset.checked_add(column_offset)?;

        usize::try_from(offset).ok()
    }

    // 座標から計算されるオフセットを検証する。
    pub fn verify_offset(&self) -> bool {
        self.pixel_offset(0, 0) == Some(0)
            && self.pixel_offset(1, 0) == Some(4)
            && self.pixel_offset(0, 1) == Some(5120)
            && self.pixel_offset(100, 100) == Some(512400)
            && self.pixel_offset(1280, 0).is_none()
            && self.pixel_offset(0, 800).is_none()
    }

    // 対応するピクセル形式とメモリ配置か確認する。
    pub fn is_supported(&self) -> bool {
        let info = &self.info;

        if info.address.is_null()
            || !(info.address as usize).is_multiple_of(4)
            || info.width == 0
            || info.height == 0
            || info.bpp != 32
            || info.memory_model != 1
            || !info.pitch.is_multiple_of(4)
        {
            return false;
        }

        // RGB各8ビットの配置を確認する。
        if info.red_mask_size != 8
            || info.red_mask_shift != 16
            || info.green_mask_size != 8
            || info.green_mask_shift != 8
            || info.blue_mask_size != 8
            || info.blue_mask_shift != 0
        {
            return false;
        }

        // 1行に必要なバイト数を計算する。
        let Some(row_bytes) = info.width.checked_mul(4) else {
            return false;
        };

        if info.pitch < row_bytes {
            return false;
        }

        // Framebuffer全体のサイズが計算可能か確認する。
        let Some(total_bytes) = row_bytes.checked_mul(info.height) else {
            return false;
        };

        total_bytes <= isize::MAX as u64 && usize::try_from(total_bytes).is_ok()
    }

    // 指定した座標にRGB色を書き込む。
    // 画面外の座標では何もせずfalseを返す。
    pub fn put_pixel(&mut self, x: u64, y: u64, red: u8, green: u8, blue: u8) -> bool {
        let Some(offset) = self.pixel_offset(x, y) else {
            return false;
        };

        // ピクセル値をRGB888形式で組み立てる
        let pixel = ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32);

        // SAFETY:
        // new()の呼び出し側がFramebufferメモリの有効性と
        // 書き込み権限、排他的アクセスを保証する。
        // is_supported()で32bpp、アドレスとpitchの
        // 4バイトアラインメントを検証済み。
        // pixel_offset()は画面内の座標を検証している。
        unsafe {
            let pixel_ptr = self.info.address.add(offset).cast::<u32>();
            core::ptr::write_volatile(pixel_ptr, pixel);
        }

        true
    }

    pub fn fill_rect(
        &mut self,
        x: u64,
        y: u64,
        width: u64,
        height: u64,
        red: u8,
        green: u8,
        blue: u8,
    ) {
        // 加算のオーバーフローを防ぎつつ、描画範囲を画面内に制限する。
        let end_x = x.saturating_add(width).min(self.info.width);
        let end_y = y.saturating_add(height).min(self.info.height);

        for py in y..end_y {
            for px in x..end_x {
                self.put_pixel(px, py, red, green, blue);
            }
        }
    }

    // 8x8ビットマップフォントの文字Aを描画する。
    pub fn draw_char(&mut self, x: u64, y: u64, red: u8, green: u8, blue: u8) {
        for (row, &row_bits) in FONT_A.iter().enumerate() {
            for col in 0..8u8 {
                // 左端から順番にビットを確認する。
                let mask = 0x80u8 >> col;

                if row_bits & mask != 0 {
                    // 画面外や整数オーバーフローを避ける。
                    let Some(px) = x.checked_add(u64::from(col)) else {
                        continue;
                    };

                    let Some(py) = y.checked_add(row as u64) else {
                        continue;
                    };

                    self.put_pixel(px, py, red, green, blue);
                }
            }
        }
    }
}
