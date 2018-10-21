use std::env;
use std::fs::File;
use std::io::{self, BufRead, Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::{clear, cursor, terminal_size};

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct Config {
    cx: u16,
    cy: u16,
    screencols: u16,
    screenrows: u16,
    rows: Vec<String>,
}

impl Config {
    pub fn new() -> Config {
        let (w, h) = terminal_size().expect("unable to get terminal size");

        Config {
            cx: 0,
            cy: 0,
            screencols: w,
            screenrows: h,
            rows: Vec::new(),
        }
    }
}

struct Editor {
    stdin: Keys<Stdin>,
    stdout: RawTerminal<Stdout>,
    config: Config,
    buffer: String,
}

impl Editor {
    pub fn new() -> Editor {
        let stdin = io::stdin();
        let stdout = io::stdout()
            .into_raw_mode()
            .expect("unable to enable raw mode");

        Editor {
            stdin: stdin.keys(),
            stdout: stdout,
            config: Config::new(),
            buffer: String::new(),
        }
    }

    pub fn open(&mut self, filename: &str) -> io::Result<()> {
        let f = File::open(filename)?;

        for line in io::BufReader::new(f).lines() {
            self.config.rows.push(line?);
        }

        Ok(())
    }

    pub fn refresh_screen(&mut self) {
        self.buffer
            .push_str(&format!("{}{}", cursor::Hide, cursor::Goto::default()));
        self.draw_rows();
        self.buffer.push_str(&format!(
            "{}{}",
            cursor::Goto(self.config.cy + 1, self.config.cx + 1),
            cursor::Show
        ));
        write!(self.stdout, "{}", self.buffer);
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
            Key::Up | Key::Down | Key::Left | Key::Right => self.move_cursor(c),
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

impl Editor {
    fn read_key(&mut self) -> Result<Key, io::Error> {
        self.stdin.next().unwrap()
    }

    fn draw_rows(&mut self) {
        for y in 0..self.config.screenrows {
            if y as usize > self.config.rows.len() {
                if self.config.rows.len() == 0 && y == self.config.screenrows / 3 {
                    let welcome = format!("Kilo editor -- version {}", VERSION);

                    let mut welcomelen = welcome.len() as u16;

                    if welcomelen > self.config.screencols {
                        welcomelen = self.config.screencols;
                    }

                    let mut padding = (self.config.screencols - welcomelen) / 2;

                    if padding > 0 {
                        self.buffer.push('~');
                        padding -= 1;
                    }

                    self.buffer.push_str(&" ".repeat(padding as usize));
                    self.buffer.push_str(&welcome);
                } else {
                    self.buffer.push('~');
                }
            } else {

            }

            self.buffer.push_str(&format!("{}", clear::UntilNewline));
            if y < self.config.screenrows - 1 {
                self.buffer.push_str("\r\n");
            }
        }
    }

    fn move_cursor(&mut self, key: Key) {
        match key {
            Key::Left => {
                if self.config.cx != 0 {
                    self.config.cx -= 1;
                }
            }
            Key::Right => {
                if self.config.cx != self.config.screencols - 1 {
                    self.config.cx += 1;
                }
            }
            Key::Up => {
                if self.config.cy != 0 {
                    self.config.cy -= 1;
                }
            }
            Key::Down => {
                if self.config.cy != self.config.screenrows {
                    self.config.cy += 1;
                }
            }
            _ => {}
        }
    }
}

fn main() {
    let mut editor = Editor::new();

    let args: Vec<String> = env::args().collect();

    if args.len() > 2 {
        editor.open(&args[2]).expect("fail to open");
    }

    editor.set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");

    editor.run();
}
