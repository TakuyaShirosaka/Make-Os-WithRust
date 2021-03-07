### (重要)

ここからは以下のチャンネルを追加する必要がある

    nightlyチャンネルのインストール
	rustup toolchain install nightly-2021-03-01
	rustup default nightly-2021-03-01-x86_64-pc-windows-msvc
	※2021/03/07時点の最新だとbootloaderがコンパイル失敗した。

    追加する必要のあるコンポーネント
	rustup component add rust-src
	rustup component add llvm-tools-preview


### ビルド

	cargo build

### ディスクイメージの作成

	cargo install bootimage
	cargo bootimage

	target/x86_64-blog_os/debug　
	上記の配下にbootimage-blog_os.binというファイルが出来ます。

### QEMUで起動する

	取得先
	https://www.qemu.org/

	手動での実行イメージ
	cd {qemuのインストール先}
	./qemu-system-x86_64 -drive format=raw,file={ディスクイメージファイルのパス}
	
	cargo runでQMENUと連動する場合はWindowsだと難しそうだったので諦めました。
	
	
	