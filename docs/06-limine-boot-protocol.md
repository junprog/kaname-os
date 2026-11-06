# LimineによるkanameOSの初回起動

## 1. 目的

Rustで作成した `no_std` / `no_main` カーネルを、Limineブートローダ経由で起動する。

今回の到達目標は、以下の2点。

- Limineからカーネルの `_start()` に制御が渡ること
- Limine Base Revision 6への対応をカーネル側で確認できること

最終的に、QEMUのデバッグコンソールへ `KB` が出力されることを確認した。

## 2. 起動の全体構成

```text
QEMU（仮想PC）
  └── OVMF（UEFIファームウェア）
       └── BOOTX64.EFI（Limine）
            ├── limine.conf
            └── kaname-kernel（ELF）
                 └── _start()
```

各コンポーネントの責務は次のとおり。

| コンポーネント | 責務 |
|---|---|
| QEMU | CPU、メモリ、ディスクなどのハードウェアをエミュレートする |
| OVMF | UEFIファームウェアとしてブートローダを起動する |
| Limine | ELFをメモリにロードし、カーネルの実行を開始する |
| kanameOS | ブートローダから受け取った環境でOSの処理を開始する |

Limineを使用することで、カーネル自身がUEFIのファイル読み込みやELFロードを実装する必要がなくなる。

## 3. ELFとリンカスクリプト

### 3.1 SectionとSegment

ELFには、異なる目的を持つ2つの構造がある。

- **Section**：コードやデータを種類ごとに分類する
- **Segment**：実行時にメモリへロードする単位を定義する

代表的なSectionには、`.text`、`.rodata`、`.data`、`.bss` がある。

`readelf -SW` でSectionを、`readelf -lW` でSegmentを確認できる。

### 3.2 Higher Half Kernel

カーネルの配置開始アドレスとして、次を指定した。

```ld
. = 0xffffffff80000000;
```

これはx86_64の高位仮想アドレス領域にカーネルを配置する設計である。

今回のELFでは、エントリポイントが以下になった。

```text
Entry point 0xffffffff80000010
```

### 3.3 ページ権限の問題

最初のQEMU起動では、Limineが次のエラーを報告した。

```text
PANIC: elf: Attempted to load ELF file with PHDRs
with different permissions sharing the same memory page.
```

原因は、読み取り専用の `.rodata` と書き込み可能な `.got` が、同じ4KiBページ内に配置されていたこと。

ページテーブルのアクセス権限はページ単位で管理されるため、異なる権限のロードセグメントを同じページに配置すると問題になる。

そこで、`linker.ld` に `.got` を明示的に配置し、4KiB境界で分離した。

修正後の主な配置は次のとおり。

| 仮想アドレス | 内容 | 権限 |
|---|---|---|
| `0xffffffff80000000` | `.text` | R E |
| `0xffffffff80001000` | `.rodata` | R |
| `0xffffffff80002000` | `.got` | RW |
| `0xffffffff80003000` | `.limine_requests` | RW |

これにより、LimineによるELFロード時の権限衝突が解消された。

## 4. Limine Base Revision 6

### 4.1 Base Revisionとは

LimineのBase Revisionは、ブートローダとカーネル間のプロトコル仕様を識別する仕組みである。

Limine本体のリリースバージョンとは異なる。

カーネルは特定のマジックナンバーと要求するBase RevisionをELF内に埋め込む。

```rust
#[used]
#[unsafe(link_section = ".limine_requests")]
static mut BASE_REVISION: [u64; 3] = [
    0xf9562b2d5c95a6c8,
    0x6a7b384944536bdc,
    6,
];
```

対応するBase Revisionを使用できる場合、Limineは3番目の値を `0` に書き換える。

カーネルはその値を確認して、要求したプロトコルを利用できるか判定する。

### 4.2 リクエスト領域

開始・終了マーカーを定義した。

```rust
#[used]
#[unsafe(link_section = ".limine_requests_start")]
static REQUESTS_START: [u64; 2] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
];

#[used]
#[unsafe(link_section = ".limine_requests_end")]
static REQUESTS_END: [u64; 2] = [
    0xadc0e0531bb10d03,
    0x9572709f31764c62,
];
```

リンカスクリプトでは `KEEP` を使用し、開始マーカー、Base Revision、終了マーカーの順に配置した。

```ld
.limine_requests : {
    KEEP(*(.limine_requests_start))
    KEEP(*(.limine_requests))
    KEEP(*(.limine_requests_end))
}
```

`readelf -x .limine_requests` により、合計56バイトが期待した順序で配置されていることを確認した。

## 5. RustからBase Revisionを確認する

```rust
let base_revision = unsafe {
    let ptr = core::ptr::addr_of!(BASE_REVISION) as *const u64;
    core::ptr::read_volatile(ptr.add(2))
};
```

`addr_of!` はRustの参照を生成せず、生ポインタを取得する。

`ptr.add(2)` は `u64` 2要素分、すなわち16バイト先を指す。

`read_volatile` を使用するのは、ブートローダがカーネルの実行開始前に変更したメモリ上の値を、実際に読み取るためである。

なお、`volatile` はスレッド間の同期やメモリ安全性を保証するものではない。

## 6. QEMUでの起動

### 6.1 必要なツール

Ubuntu上で以下を使用した。

- `qemu-system-x86_64`
- OVMF
- `dosfstools`
- `mtools`

### 6.2 ブート設定

`boot/image/EFI/BOOT/limine.conf`

```ini
timeout: 5

/kanameOS
    protocol: limine
    path: boot():/boot/kaname-kernel
```

### 6.3 FAT32ディスクイメージの作成

```bash
truncate -s 64M boot/kaname.img
mkfs.vfat -F 32 boot/kaname.img

mmd -i boot/kaname.img ::/EFI
mmd -i boot/kaname.img ::/EFI/BOOT
mmd -i boot/kaname.img ::/boot

mcopy -i boot/kaname.img \
  boot/image/EFI/BOOT/BOOTX64.EFI ::/EFI/BOOT/

mcopy -i boot/kaname.img \
  boot/image/EFI/BOOT/limine.conf ::/EFI/BOOT/

mcopy -i boot/kaname.img \
  boot/image/boot/kaname-kernel ::/boot/
```

このディスクイメージはパーティションテーブルを持たない、単純なFAT32イメージである。

### 6.4 QEMUの起動

初回にOVMFの変数保存領域をコピーする。

```bash
cp /usr/share/OVMF/OVMF_VARS_4M.fd boot/OVMF_VARS.fd
```

QEMUを起動する。

```bash
qemu-system-x86_64 \
  -machine q35 \
  -m 512M \
  -drive if=pflash,format=raw,readonly=on,file=/usr/share/OVMF/OVMF_CODE_4M.fd \
  -drive if=pflash,format=raw,file=boot/OVMF_VARS.fd \
  -drive format=raw,file=boot/kaname.img,if=virtio \
  -boot order=c \
  -debugcon stdio \
  -global isa-debugcon.iobase=0xe9
```

## 7. 最初のデバッグ出力

カーネルにはまだ画面描画機能がない。

そこで、QEMUのデバッグ用I/Oポート `0xE9` を使用した。

```rust
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
```

`out dx, al` はx86のI/O命令である。

`dx` にポート番号、`al` に出力する1バイトを指定する。

`_start()` では、処理の到達地点を確認するために文字を出力した。

```rust
debug_putchar(b'K');

if base_revision != 0 {
    debug_putchar(b'F');
    loop {
        core::hint::spin_loop();
    }
}

debug_putchar(b'B');
```

出力の意味は以下のとおり。

| 出力 | 意味 |
|---|---|
| `K` | `_start()` に到達した |
| `KF` | Base Revisionの判定に失敗した |
| `KB` | Base Revisionの判定に成功した |

## 8. 実行結果

QEMUを起動し、LimineのメニューからkanameOSを選択した。

WSLのターミナルに次の出力を確認した。

```text
KB
```

これにより、次の動作が確認できた。

1. UEFIからLimineが起動した。
2. LimineがカーネルELFをロードした。
3. `_start()` に制御が渡った。
4. カーネルがI/Oポートへ文字を出力した。
5. Base Revision 6の対応確認が成功した。

QEMUの画面は黒いままだが、現時点ではフレームバッファへの描画を実装していないため、これは想定内の挙動である。

## 9. 今回学んだこと

- OSを起動するには、ファームウェア、ブートローダ、カーネルがそれぞれ異なる責務を担う。
- ELFのSectionはデータの分類、Segmentはロード時のメモリ配置を表す。
- リンカスクリプトによってカーネルの仮想アドレスやSectionの配置を制御できる。
- ページ権限の整合性は、カーネルをロードする段階でも重要になる。
- Limineのプロトコルを通じて、カーネルとブートローダがデータを受け渡せる。
- 画面描画を実装する前でも、I/Oポートを使って実行状況を確認できる。

## 10. 次の課題

次の目標は、Limineからフレームバッファ情報を取得し、kanameOS自身で画面に描画すること。

予定する手順は次のとおり。

1. Framebuffer Requestのデータ構造を理解する。
2. カーネルからフレームバッファを要求する。
3. 返されたアドレス、解像度、ピッチ、ピクセル形式を確認する。
4. 最初のピクセルを描画する。
5. 画面全体の塗りつぶしを実装する。

将来的には、ブート用ディスクイメージの生成をスクリプト化し、開発環境の再現性を高める。
