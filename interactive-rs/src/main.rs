extern crate termion;

use std::io::{stdin, stdout, Write};
use termion::cursor::DetectCursorPos;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

fn main() {
    // Get the standard input stream.
    let stdin = stdin();
    // Get the standard output stream and go to raw mode.
    let mut stdout = stdout().into_raw_mode().unwrap();

    write!(
        stdout,
        "{}{}q to exit. Type stuff, use alt, and so on.{}",
        // Clear the screen.
        termion::clear::All,
        // Goto (1,1).
        termion::cursor::Goto(1, 1),
        // Hide the cursor.
        termion::cursor::Hide
    )
    .unwrap();
    // Flush stdout (i.e. make the output appear).
    stdout.flush().unwrap();

    for c in stdin.keys() {
        let val = c.unwrap();

        if let Key::Char('q') = val {
            break;
        }
        if let Key::Char('\n') = val {
            let (x, y) = stdout.cursor_pos().unwrap();
            write!(stdout, "{}{}", " ", termion::cursor::Goto(0, y + 1)).unwrap();
            continue;
        }
        if let Key::Char(x) = val {
            //stdout.write_all(x.to_string().as_bytes()).unwrap();
            //print!("{:}", x);
            write!(stdout, "{}", x).unwrap();
        }

        // Flush again.
        stdout.flush().unwrap();
    }

    // Show the cursor again before we exit.
    write!(stdout, "{}", termion::cursor::Show).unwrap();
}
