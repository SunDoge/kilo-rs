use std::env;
use std::io::{self, Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::{Keys, TermRead};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

struct Config {
    cx: u16,
    cy: u16,
    screencols: u16,
    screenrows: u16,
    numrows: u16,
}

impl Config {
    pub fn new() -> Config {
        let (w, h) = terminal_size().expect("unable to get terminal size");

        Config {
            cx: 0,
            cy: 0,
            screencols: w,
            screenrows: h,
            numrows: 0,
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
        let mut stdout = io::stdout()
            .into_raw_mode()
            .expect("unable to enable raw mode");

        let args: Vec<String> = env::args().collect();

        if args.len() > 2 {
            // self.open(&args[2]);
        }

        Editor {
            stdin: stdin.keys(),
            stdout: stdout,
            config: Config::new(),
            buffer: String::new(),
        }
    }

    pub fn open(&mut self, filename: &str) {}

    pub fn refresh_screen(&self) {}

    pub fn process_key_press(&mut self) -> bool {
        let c = self.read_key();

        match c.unwrap() {
            Key::Ctrl(k) => match k {
                'q' => return false,
                _ => {}
            },
            _ => {}
        }

        true
    }

    pub fn set_status_message(&mut self, msg: &str) {}

    pub fn run(&mut self) {
        loop {
            self.refresh_screen();
            if self.process_key_press() == false {
                break;
            }
        }
    }
}

impl Editor {
    fn read_key(&mut self) -> Result<Key, io::Error> {
        self.stdin.next().unwrap()
    }

    fn draw_rows(&mut self) {
        for y in 0..self.config.screenrows {
            if y > self.config.numrows {
                if self.config.numrows == 0 && y == self.config.screenrows / 3 {
                    let welcome = format!("Kilo editor -- version {}", VERSION);

                    let mut welcomelen = welcome.len() as u16;

                    if (welcomelen > self.config.screencols) {
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
        }
    }
}

fn main() {
    let mut editor = Editor::new();

    editor.set_status_message("HELP: Ctrl-S = save | Ctrl-Q = quit | Ctrl-F = find");

    editor.run();
}
