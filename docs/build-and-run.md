# kanameOS — ビルド・起動ガイド

## 1. 概要

kanameOSは、Rustで開発しているx86_64向けの自作OSです。

カーネルは `#![no_std]` / `#![no_main]` で実装し、Limineブートローダを使用してUEFI環境から起動します。

本ドキュメントでは、Ubuntu環境でkanameOSをビルドし、QEMU上で起動するまでの手順を説明します。

### 起動構成

```text
QEMU（仮想マシン）
  └── OVMF（UEFIファームウェア）
       └── Limine v12.9.4
            └── kanameOS（Rustカーネル）
                 └── _start()
```

現在の到達点は、カーネルのエントリポイントへの到達と、Limine Base Revision 6の対応確認です。

正常起動時には、QEMUを実行したターミナルに `KB` と表示されます。

## 2. 開発環境

動作確認済みの環境は以下のとおりです。

| 項目 | 環境 |
|---|---|
| ホストOS | Windows + WSL2 |
| Linux | Ubuntu 24.04.5 LTS |
| アーキテクチャ | x86_64 |
| Rust | 1.99.1 |
| QEMU | 8.2.2 |
| ブート方式 | UEFI |
| UEFIファームウェア | OVMF |
| ブートローダ | Limine v12.9.4 |
| Limine Base Revision | 6 |
| Rustターゲット | `x86_64-unknown-none` |

別のLinux環境でも動作する可能性がありますが、上記以外の構成は未検証です。

## 3. リポジトリの取得

```bash
git clone https://github.com/junprog/kaname-os.git
cd kaname-os
```

## 4. 必要なツールのインストール

### 4.1 Ubuntuの依存パッケージ

```bash
sudo apt update

sudo apt install -y \
  build-essential \
  make \
  curl \
  tar \
  qemu-system-x86 \
  ovmf \
  dosfstools \
  mtools
```

主な用途は以下のとおりです。

| ツール | 用途 |
|---|---|
| `make` | ビルド・起動処理の自動化 |
| `curl`, `tar` | Limineのダウンロード・展開 |
| `qemu-system-x86_64` | 仮想マシンの実行 |
| OVMF | UEFIファームウェア |
| `mkfs.vfat` | FAT32ディスクイメージの初期化 |
| `mmd`, `mcopy` | FAT32イメージ内のディレクトリ作成・ファイルコピー |

### 4.2 Rustのインストール

Rustがインストールされていない場合は、公式のrustupを使用します。

https://rustup.rs/

動作確認済みのRustバージョンをインストールします。

```bash
rustup toolchain install 1.99.1
rustup override set 1.99.1
```

`rustup override set` は、現在のディレクトリに適用するRustツールチェーンを設定します。

次に、OS開発用ターゲットを追加します。

```bash
rustup target add x86_64-unknown-none
```

確認します。

```bash
rustc --version
cargo --version
rustup target list --installed
```

`x86_64-unknown-none` が表示されれば、ターゲットの準備は完了です。

## 5. プロジェクト構成

ビルドに関係する主なファイルは次のとおりです。

```text
kaname-os/
├── .cargo/
│   └── config.toml
├── kernel/
│   ├── Cargo.toml
│   └── src/
│       └── main.rs
├── boot/
│   └── image/
│       └── EFI/
│           └── BOOT/
│               └── limine.conf
├── docs/
├── Cargo.toml
├── Cargo.lock
├── linker.ld
├── Makefile
└── .gitignore
```

ビルドや起動時には、以下のファイルが生成されます。

```text
target/
└── x86_64-unknown-none/
    └── debug/
        └── kaname-kernel

boot/
├── dist/
│   ├── limine-binary.tar.gz
│   └── limine/
│       └── BOOTX64.EFI
├── kaname.img
└── OVMF_VARS.fd
```

これらの生成物はGit管理の対象外です。

## 6. ビルドと起動

プロジェクトルートの `Makefile` で、ビルドから起動までの操作を管理します。

### 6.1 カーネルのビルド

```bash
make build
```

内部では次のコマンドを実行します。

```bash
cargo rustc -p kaname-kernel -- \
  -C relocation-model=static \
  -C link-arg=-Tlinker.ld \
  -C link-arg=-zmax-page-size=0x1000
```

各オプションの役割は次のとおりです。

| オプション | 役割 |
|---|---|
| `-C relocation-model=static` | 静的なアドレス配置を使用する |
| `-C link-arg=-Tlinker.ld` | 独自のリンカスクリプトを指定する |
| `-C link-arg=-zmax-page-size=0x1000` | リンカの最大ページサイズを4KiBに指定する |

ビルド成果物は `target/x86_64-unknown-none/debug/kaname-kernel` に生成されます。

### 6.2 起動ディスクの生成

```bash
make image
```

このコマンドは、以下を順番に実行します。

1. 必要なツールの確認
2. Rustカーネルのビルド
3. Limine v12.9.4の取得（未取得の場合）
4. 64MiBのFAT32ディスクイメージの作成
5. Limine、設定ファイル、カーネルELFの配置

生成される `boot/kaname.img` の内部構造は次のとおりです。

```text
/
├── EFI/
│   └── BOOT/
│       ├── BOOTX64.EFI
│       └── limine.conf
└── boot/
    └── kaname-kernel
```

このディスクイメージにはパーティションテーブルを設けず、ディスク全体をFAT32ファイルシステムとして使用します。

### 6.3 QEMUで起動

```bash
make run
```

このコマンドは、カーネルのビルド、ディスクイメージの生成、QEMUの起動を実行します。

QEMUには以下の構成を使用します。

- マシン：`q35`
- メモリ：512MiB
- ファームウェア：OVMF
- 起動ディスク：`boot/kaname.img`
- デバッグ出力：I/Oポート `0xE9`

初回起動時には、OVMFの変数保存用ファイル `boot/OVMF_VARS.fd` も生成されます。

## 7. 正常起動の確認

LimineからkanameOSを起動すると、QEMUを実行したUbuntuターミナルに次の文字が表示されます。

```text
KB
```

各文字には次の意味があります。

| 文字 | 意味 |
|---|---|
| `K` | カーネルの `_start()` に到達した |
| `B` | Limine Base Revision 6の対応確認に成功した |
| `F` | Base Revisionの確認に失敗した |

現時点では画面描画機能が未実装のため、QEMUのウィンドウが黒いままでも異常とは限りません。

**正常起動の判断には、QEMUの画面ではなくUbuntuターミナルの出力を使用してください。**

## 8. よく使うコマンド

| コマンド | 動作 |
|---|---|
| `make` | Makefileのデフォルトターゲットを実行 |
| `make build` | カーネルをビルド |
| `make image` | 起動ディスクを再生成 |
| `make run` | QEMUで起動 |
| `make clean` | カーネルのビルド成果物とディスクイメージを削除 |
| `make -n run` | 実行予定のコマンドを表示 |

`make` 単体の動作は、Makefileの `.DEFAULT_GOAL` の設定に従います。

## 9. トラブルシューティング

### `make: command not found`

`make` がインストールされていません。

```bash
sudo apt install make
```

### `can't find crate for core`

Rustのターゲットがインストールされていない可能性があります。

```bash
rustup target add x86_64-unknown-none
```

### OVMFファイルが見つからない

以下を確認してください。

```bash
ls /usr/share/OVMF/
```

このプロジェクトでは、`OVMF_CODE_4M.fd` と `OVMF_VARS_4M.fd` を使用します。

ディストリビューションによってファイル名や配置が異なる場合があります。

### LimineがELFのページ権限エラーを表示する

ELFのロードセグメントに、ページ境界やアクセス権限の問題がある可能性があります。

```bash
readelf -lW target/x86_64-unknown-none/debug/kaname-kernel
```

`.text`、`.rodata`、`.got`、`.limine_requests` のロードセグメントが適切に配置されているか確認してください。

詳細は `linker.ld` を参照してください。

### QEMUが黒い画面のままになる

現在のkanameOSでは、フレームバッファへの描画を実装していません。

Ubuntuターミナルに `KB` が出力されている場合、カーネルの起動とBase Revisionの確認は成功しています。

何も出力されない場合は、カーネルELFのビルド、ディスクへの配置、Limineの設定を確認してください。

## 10. 今後の改善予定

- Rustツールチェーンを `rust-toolchain.toml` で固定する
- ビルド・起動処理のエラーメッセージを改善する
- Limineのダウンロードファイルの整合性検証を追加する
- GitHub Actionsによるカーネルのビルド検証を導入する
- Debug / Releaseビルドを切り替えられるようにする

---

## 参考資料

- [Rust公式ドキュメント](https://doc.rust-lang.org/)
- [Rust x86_64-unknown-none Target](https://doc.rust-lang.org/rustc/platform-support/x86_64-unknown-none.html)
- [Limine公式リポジトリ](https://github.com/limine-bootloader/limine)
- [QEMU公式ドキュメント](https://www.qemu.org/docs/master/)
- [GNU Make公式マニュアル](https://www.gnu.org/software/make/manual/)
