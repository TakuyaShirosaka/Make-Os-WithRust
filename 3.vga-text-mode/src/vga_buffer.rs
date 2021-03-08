use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// [allow(dead_code)]属性を使うことでColor enumに対するそれらの警告を消すことができます。
// Copy、Clone、Debug、PartialEq、および Eqをderiveすることによって、
// この型のコピーセマンティクスを有効化し、この型を出力することと比較することを可能にします。
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// ColorCodeがu8と全く同じデータ構造を持つようにするために、
// repr(transparent)属性（訳注：翻訳当時、リンク先未訳）を使います。
// derive:継承の様なもの、継承している内容はこれが参考になった。https://qiita.com/apollo_program/items/2495dda519ae160971ed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

impl ColorCode {
    fn new(background: Color, foreground: Color) -> ColorCode {
        // 左シフトとOR演算 例えばbackgroundがBlack:0なら
        // 2進数で 00000000(シフト演算後) 、foregroundがYellow:14なら00001110、OR演算して00001110、
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    // 文字1バイト
    // カラー1バイト
    ascii_character: u8,
    color_code: ColorCode,
}

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// 'staticライフタイムは、
// その参照がプログラムの実行中ずっと有効であることを指定しています
pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });


                self.column_position += 1;
            }
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // 出力可能なASCIIバイトか、改行コード
                0x20..=0x7e | b'\n' => self.write_byte(byte),

                // 出力可能なASCIIバイトではない
                // 文字■を出力します（これはVGAハードウェアにおいて16進コード0xfeを持っています）。
                _ => self.write_byte(0xfe),
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    // このメソッドはすべての文字を空白文字で書き換えることによって行をクリアしてくれます。
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }
}

// フォーマットマクロの作成
// 整数や浮動小数点数といった様々な型を簡単に出力できます。
// それらをサポートするためには、core::fmt::Writeトレイトを実装する必要があります。
// このトレイトに必要なメソッドはwrite_strだけです。
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

// 初期化を実行時に行うように遅延させる
lazy_static! {
    // スピンロックを使うと、ブロックする代わりに、
    // スレッドは単純にリソースを何度も何度もロックしようとすることで、mutexが開放されるまでの間CPU時間を使い尽くします。
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Black, Color::Yellow),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// <printlnマクロ>
// マクロは1つ以上のルールを使って定義されます（matchアームと似ていますね）。
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

// printlnには2つのルールがあります：1つ目は引数なし呼び出し（例えば println!()）のためのもので、これはprint!("\n")に展開され、よってただ改行を出力するだけになります。
// 2つ目のルールはパラメータ付きの呼び出し（例えばprintln!("Hello")や println!("Number: {}", 4)）のためのものです。
// これもprint!マクロの呼び出しへと展開され、すべての引数に加え、改行\nを最後に追加して渡します。
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

// write_fmt : Writeトレイト、WRITERは左記のトレイトを実装しているのでこの関数を使用できる
// write_fmt → write_strと処理が動く
#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}
