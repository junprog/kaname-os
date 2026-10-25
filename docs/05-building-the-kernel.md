# 第5章 カーネルのビルドとELF

## 環境（確認済み）
- Windows + WSL2、Ubuntu 24.04.5 LTS、x86-64
- rustc 1.99.1、Cargo 1.99.1
- ターゲット: `x86_64-unknown-none`
- リポジトリ: `junprog/kaname-os`

## Cargo Workspace
`Cargo.toml`に`[workspace]`と`members = ["kernel"]`を設定し、`kernel/Cargo.toml`で`kaname-kernel`パッケージを定義する。

## ビルド
```bash
cargo build
file target/x86_64-unknown-none/debug/kaname-kernel
```

実際の結果: `ELF 64-bit LSB pie executable, x86-64, static-pie linked`。

## ELFヘッダーとエントリーポイント
```bash
readelf -h target/x86_64-unknown-none/debug/kaname-kernel
nm -n target/x86_64-unknown-none/debug/kaname-kernel | grep _start
```

確認した結果:
```text
Type: DYN (Position-Independent Executable file)
Entry point address: 0x1278
0000000000001278 T _start
```

`DYN`/PIE形式のため、`0x1278`は実行時の固定物理アドレスではない。ロード基準アドレスに応じて実行時の仮想アドレスが決まる。

## プログラムヘッダー
```bash
readelf -l target/x86_64-unknown-none/debug/kaname-kernel
```

確認したLOAD領域:

| Offset | VirtAddr | FileSiz | MemSiz | Flags |
|---|---|---|---|---|
| `0x0` | `0x0` | `0x274` | `0x274` | R |
| `0x278` | `0x1278` | `0x6` | `0x6` | R E |
| `0x280` | `0x2280` | `0xa0` | `0xd80` | RW |

- `Offset`: ELFファイル内の位置。
- `VirtAddr`: ロード時の仮想アドレスの基準となる値。
- `FileSiz`: ファイルから読み込むバイト数。
- `MemSiz`: メモリ上で確保するバイト数。
- `MemSiz > FileSiz`なら差分はゼロ初期化する。今回の第3LOADでは`0xd80 - 0xa0 = 0xce0`（3296バイト）。

## 理解度チェック
あるLOAD領域が`Offset=0x500`, `VirtAddr=0x2000`, `FileSiz=0x100`, `MemSiz=0x300`のとき、ファイルの`0x500`から256バイト読み、残り512バイトをゼロ初期化する。

## 次の学習
リンカースクリプトの`ENTRY`と`SECTIONS`、Limineとの接続、QEMUでの起動。
