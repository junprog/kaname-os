# 第4章 カーネルの基礎

## なぜ `no_std` なのか
通常のRustプログラムはOSが提供する仕組みに依存する`std`を使う。OS自体を作る段階ではそれらが未実装なので、`core`を中心に構成する。

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    loop { core::hint::spin_loop(); }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop { core::hint::spin_loop(); }
}
```

- `#![no_main]`: 通常の`main`起動方式を使わない。
- `#[unsafe(no_mangle)]`: 外部から参照するシンボル名を維持する（Rust 2024 Edition）。
- `extern "C"`: C ABIを指定。
- `-> !`: 通常の呼び出し元へ戻らない。
- `#[panic_handler]`: `no_std`環境のpanic処理。

> `_start`という名前だけではブートローダーからの起動は保証されない。ELFの入口とブートプロトコルへの対応が必要。
