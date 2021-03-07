/*
  <標準ライブラリの無効化>
    デフォルトでは、全ての Rust クレートは標準ライブラリにリンクされています。
    標準ライブラリはスレッドやファイル、ネットワークのような OS の機能に依存しています。
    また OS と密接な関係にある C の標準ライブラリ(libc)にも依存しています。
    私達の目的は OS を書くことなので、 OS 依存のライブラリを使うことはできません。
    そのため、 no_std attribute を使って標準ライブラリが自動的にリンクされるのを無効にします。
 */

#![no_std]

/*
  <start attribute>
    自作OSではmain関数よりも先に呼び出されるエントリーポイントを自分で実装する必要がある
    そのためattributeに#![no_main]を追加し、通常のエントリーポイントで動かないように明示的に指定する
 */

#![no_main]

use core::panic::PanicInfo;

/** no_mainの為、不要 */
// fn main() {}

/*
  <Panic の実装>
    panic_handler attribute はパニックが発生したときにコンパイラが呼び出す関数を定義します。
    標準ライブラリには独自のパニックハンドラー関数がありますが、
    no_std 環境では私達の手でそれを実装する必要があります。
    ！はnever型という意味
*/
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}


/**
  <エントリポイントの上書き>
    Rust コンパイラが _start という名前の関数を実際に出力するように、
    #[no_mangle] attributeを用いて名前修飾を無効にします。
    この attribute がないと、コンパイラはすべての関数にユニークな名前をつけるために、
    _ZN3blog_os4_start7hb173fedf945531caE のようなシンボルを生成します。

    この関数に C の呼び出し規約を使用するようコンパイラに伝えるために、
    関数を extern "C" として定義する必要があります。
    _startという名前をつける理由は、これがほとんどのシステムのデフォルトのエントリポイント名だからです。
*/
#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 0xb8000という整数を生ポインタにキャストします。これはVRAMの領域を指しています。
    // この辺の処理はVGAのテキストモードとGoogle検索すれば何をやっているか理解できると思います。
    let vga_text_buffer = 0xb8000 as *mut u8;

    // enumerate:イテレーターが返す要素と現在のインデックスのペアを生成して返します。
    // ちなみにインデックスは0,1,2のように0始まり
    for (i, &byte) in HELLO.iter().enumerate() {

        // 演習の為にunsafeを使用していますが、本来は使用すべきではない
        unsafe {
            // キャラクタコード＋色とかのコードの2バイトで構成される
            *vga_text_buffer.offset(i as isize * 2) = byte; // offset:生ポインタへの整数加減算
            *vga_text_buffer.offset(i as isize * 2 + 1) = 0xb; // 0xbは明るいシアン色
        }
    }
    loop {}
}

static HELLO: &[u8] = b"Hello World!";





