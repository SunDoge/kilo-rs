use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use termion::event::Key;
use termion::input::{Keys, MouseTerminal, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use termion::{clear, cursor, terminal_size};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const KILO_TAB_STOP: usize = 8;

struct Row {
    chars: Vec<char>,
    render: String,
}

impl Row {
    pub fn new(line: String) -> Row {
        Row {
            chars: line.chars().collect(),
            render: String::new(),
        }
    }
}

struct Config {
    cx: usize,
    cy: usize,
    rx: usize,
    rowoff: usize,
    coloff: usize,
    screencols: usize,
    screenrows: usize,
    rows: Vec<Row>,
}

impl Config {
    pub fn new() -> Config {
        let (w, h) = terminal_size().expect("unable to get terminal size");

        Config {
            cx: 0,
            cy: 0,
            rx: 0,
            rowoff: 0,
            coloff: 0,
            screencols: w as usize,
            screenrows: h as usize,
            rows: Vec::new(),
        }
    }
}

struct Editor<R, W> {
    stdin: R,
    stdout: W,
    config: Config,
    buffer: String,
}

impl<R: Iterator<Item = Result<Key, std::io::Error>>, W: Write> Editor<R, W> {
    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        let f = File::open(filename)?;

        for line in io::BufReader::new(f).lines() {
            self.append_row(line?);
        }

        Ok(())
    }

    pub fn refresh_screen(&mut self) {
        self.scroll();

        self.buffer
            .push_str(&format!("{}{}", cursor::Hide, cursor::Goto::default()));
        self.draw_rows();
        self.buffer.push_str(&format!(
            "{}{}",
            cursor::Goto(
                (self.config.rx - self.config.coloff + 1) as u16,
                (self.config.cy - self.config.rowoff + 1) as u16
            ),
            cursor::Show
        ));
        write!(self.stdout, "{}", self.buffer);
        self.stdout.flush().unwrap();
        self.buffer.clear();
    }

    pub fn process_key_press(&mut self) -> bool {
        let c = self.read_key().unwrap();

        match c {
            Key::Ctrl(k) => match k {
                'q' => {
                    write!(self.stdout, "{}{}", clear::All, cursor::Goto::default());
                    return false;
                }
                _ => {}
            },
            Key::Home => self.config.cx = 0,
            Key::End => {
                if self.config.cy < self.config.rows.len() {
                    self.config.cx = self.config.rows[self.config.cy].chars.len();
                }
            }
            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(c),
            Key::PageUp | Key::PageDown => {}
            _ => {}
        }

        true
    }

    pub fn set_status_message(&mut self, msg: &str) {}

    pub fn run(&mut self) {
        self.refresh_screen();
        while self.process_key_press() {
            self.refresh_screen();
        }
    }
}

impl<R: Iterator<Item = Result<Key, std::io::Error>>, W: Write> Editor<R, W> {
    fn read_key(&mut self) -> Result<Key, io::Error> {
        self.stdin.next().unwrap()
    }

    fn draw_rows(&mut self) {
        for y in 0..self.config.screenrows {
            let filerow = y + self.config.rowoff;

            if filerow >= self.config.rows.len() {
                if self.config.rows.len() == 0 && y == self.config.screenrows / 3 {
                    let welcome = format!("Kilo editor -- version {}", VERSION);

                    let mut welcomelen = welcome.len();

                    if welcomelen > self.config.screencols {
                        welcomelen = self.config.screencols;
                    }

                    let mut padding = (self.config.screencols - welcomelen) / 2;

                    if padding > 0 {
                        self.buffer.push('~');
                        padding -= 1;
                    }

                    self.buffer.push_str(&" ".repeat(padding));
                    self.buffer.push_str(&welcome);
                } else {
                    self.buffer.push('~');
                }
            } else {
                let mut len =
                    self.config.rows[filerow].chars.len() as isize - self.config.coloff as isize;
                if len < 0 {
                    len = 0;
                }

                if len > self.config.screencols as isize {
                    len = self.config.screencols as isize;
                }

                let coloff = self.config.coloff;
                self.buffer.push_str(
                    std::str::from_utf8(
                        &self.config.rows[filerow].render.as_bytes()[coloff..coloff + len as usize],
                    )
                    .unwrap(),
                );
            }

            self.buffer.push_str(&format!("{}", clear::UntilNewline));
            if y < self.config.screenrows - 1 {
                self.buffer.push_str("\r\n");
            }
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let row = if self.config.cy >= self.config.rows.len() {
            None
        } else {
            Some(&self.config.rows[self.config.cy])
        };

        match key {
            Key::Left => {
                if self.config.cx != 0 {
                    self.config.cx -= 1;
                } else if self.config.cy > 0 {
                    self.config.cy -= 1;
                    self.config.cx = self.config.rows[self.config.cy].chars.len();
                }
            }
            Key::Right => {
                if let Some(row) = row {
                    if self.config.cx < row.chars.len() {
                        self.config.cx += 1;
                    } else if self.config.cx == row.chars.len() {
                        self.config.cy += 1;
                        self.config.cx = 0;
                    }
                }
            }
            Key::Up => {
                if self.config.cy != 0 {
                    self.config.cy -= 1;
                }
            }
            Key::Down => {
                if self.config.cy < self.config.rows.len() {
                    self.config.cy += 1;
                }
            }
            _ => {}
        }
    }

    fn row_cx_to_rx(&self) -> usize {
        let mut rx = 0;
        for j in 0..self.config.cx {
            if self.config.rows[self.config.cy].chars[j] == '\t' {
                rx += (KILO_TAB_STOP - 1) - (rx % KILO_TAB_STOP);
            }
            rx += 1;
        }
        rx
    }

    fn update_row(&mut self) {
        let row = self.config.rows.last_mut().unwrap();

        row.render.clear();

        for &c in &row.chars {
            if c == '\t' {
                row.render.push_str(&" ".repeat(KILO_TAB_STOP));
            } else {
                row.render.push(c);
            }
        }
    }

    fn append_row(&mut self, s: String) {
        self.config.rows.push(Row::new(s));
        self.update_row();
    }

    fn scroll(&mut self) {
        self.config.rx = 0;

        if self.config.cy < self.config.rows.len() {
            self.config.rx = self.row_cx_to_rx();
        }

        if self.config.cy < self.config.rowoff {
            self.config.rowoff = self.config.cy;
        }

        if self.config.cy > self.config.rowoff + self.config.screenrows {
            self.config.rowoff = self.config.cy - self.config.screenrows + 1;
        }

        if self.config.rx < self.config.coloff {
            self.config.coloff = self.config.rx;
        }

        if self.config.rx >= self.config.coloff + self.config.screencols {
            self.config.coloff = self.config.rx - self.config.screencols + 1;
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let stdout = io::stdout();

    let mut editor = Editor {
        stdin: stdin.keys(),
        stdout: MouseTerminal::from(AlternateScreen::from(stdout.into_raw_mode().unwrap())),
        config: Config::new(),
        buffer: String::new(),
    };

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        editor.open(&args[2]).expect("fail to open");
    }

    editor.set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");

    editor.run();
}
