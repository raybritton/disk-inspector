use crossterm::{TerminalInput, TerminalCursor, Crossterm, Terminal, ClearType, RawScreen, KeyEvent, InputEvent, ObjectStyle, Attribute};
use std::io::{stdout, Write};
use std::cmp::{max, min};
use crate::terminal_helper::BoxSymbols::{*};

pub struct ListItem {
    pub text: String,
    pub selectable: bool,
}

pub struct TerminalHelper {
    terminal: Terminal,
    cursor: TerminalCursor,
    input: TerminalInput,
}

impl TerminalHelper {
    pub fn new() -> TerminalHelper {
        let crossterm = Crossterm::new();
        return TerminalHelper {
            terminal: crossterm.terminal(),
            cursor: crossterm.cursor(),
            input: crossterm.input(),
        };
    }
}

impl TerminalHelper {
    pub fn setup(&self) {
        self.cursor.hide().unwrap();
        self.terminal.clear(ClearType::All).unwrap();
        stdout().flush().unwrap();
    }

    pub fn teardown(&self) {
        self.cursor.show().unwrap();
    }

    pub fn clear_screen(&self) {
        self.cursor.goto(0, 0).unwrap();
        let (w, h) = self.terminal.terminal_size();
        for _ in 0..=h {
            print!("{:1$}", " ", w as usize);
        }
        self.cursor.goto(0, 0).unwrap();
        stdout().flush().unwrap();
    }

    pub fn show_dialog<S: Into<String>>(&self, title: S) {
        self.clear_screen();

        let message = title.into();
        let width = (message.chars().count() + 3) as u16;
        let (term_width, term_height) = self.terminal.terminal_size();
        let x = (term_width / 2) - (width / 2);
        let y = (term_height / 2) - 3;

        self.draw_box(x, y, width, 4, None);

        self.cursor.goto(x + 2, y + 2).unwrap();
        print!("{}", message);

        stdout().flush().unwrap();
    }

    pub fn draw_progress<S: Into<String>>(&self, title: S, progress: usize) {
        let (term_width, term_height) = self.terminal.terminal_size();

        let percent = progress as f64 / 100_f64;
        let num_of_blocks = (term_width - 6) as f64 * percent;

        let width = term_width - 4;
        let x = 2;
        let y = (term_height / 2) - 3;

        self.draw_box(x, y, width, 4, Some(title.into()));

        for i in 1..(num_of_blocks as i64) {
            self.cursor.goto(x + 2 + i as u16, y + 2).unwrap();
            BoxSymbols::print_os_symbol(ProgressBlock);
        }

        stdout().flush().unwrap();
    }

    fn draw_box(&self, x: u16, y: u16, w: u16, h: u16, message: Option<String>) {
        self.cursor.goto(x, y).unwrap();
        BoxSymbols::print_os_symbol(TopLeft);
        self.cursor.goto(x, y + h).unwrap();
        BoxSymbols::print_os_symbol(BottomLeft);
        self.cursor.goto(x + w, y).unwrap();
        BoxSymbols::print_os_symbol(TopRight);
        self.cursor.goto(x + w, y + h).unwrap();
        BoxSymbols::print_os_symbol(BottomRight);
        for i in 1..h {
            self.cursor.goto(x, y + i).unwrap();
            BoxSymbols::print_os_symbol(Vertical);
            self.cursor.goto(x + w, y + i).unwrap();
            BoxSymbols::print_os_symbol(Vertical);
        }
        for i in 1..(w) {
            self.cursor.goto(x + i, y).unwrap();
            BoxSymbols::print_os_symbol(Horizontal);
            self.cursor.goto(x + i, y + h).unwrap();
            BoxSymbols::print_os_symbol(Horizontal);
        }

        if let Some(title) = message {
            let (term_width, _) = self.terminal.terminal_size();
            let title_x = term_width / 2 - title.chars().count() as u16 / 2;
            self.cursor.goto(title_x, y).unwrap();
            print!("{} {} {}", BoxSymbols::symbol_for_os(TitlePrefix), title, BoxSymbols::symbol_for_os(TitleSuffix));
        }
    }

    pub fn show_list<S: Into<String>>(&self, title: S, list: Vec<ListItem>) -> Option<usize> {
        let mut cursor_idx = 0_usize;

        let box_x;
        let box_y;
        let box_w;
        let box_h;
        let text_w;
        let text_h;

        let (term_width, term_height) = self.terminal.terminal_size();

        let longest_line = list.iter()
            .max_by(|lhs, rhs| lhs.text.chars().count().cmp(&rhs.text.chars().count()))
            .unwrap()
            .text
            .len();

        box_w = min(term_width, longest_line as u16 + 4);
        box_h = min(term_height, list.len() as u16 + 1);
        box_x = max(0, term_width / 2 - box_w / 2);
        box_y = max(0, term_height / 2 - box_h / 2);
        text_w = box_w as usize - 4;
        text_h = box_h - 1;

        let text_y = box_y + 1;
        let col_text = box_x + 2;

        let mut list_start_idx = 0;

        self.clear_screen();

        self.draw_box(box_x, box_y, box_w, box_h, Some(title.into()));

        let list_text: Vec<String> = list.iter()
            .enumerate()
            .map(|(idx, item)| {
                self.cursor.goto(col_text, text_y + idx as u16).unwrap();
                let mut name;
                if item.text.chars().count() > text_w {
                    let (truncated, _) = item.text.split_at(text_w - 1);
                    name = truncated.to_string();
                    name.push_str("…");
                } else {
                    name = item.text.clone();
                }
                format!("{:1$}", name, text_w)
            })
            .collect();


        loop {
            for i in 0..text_h as usize {
                self.cursor.goto(col_text, text_y + i as u16).unwrap();
                if cursor_idx == i + list_start_idx {
                    print!("{} {}", BoxSymbols::symbol_for_os(ListCursor), list_text[i + list_start_idx]);
                } else {
                    print!("  {}", list_text[i + list_start_idx]);
                }
            }
            stdout().flush().unwrap();
            match self.wait_for_key(vec![KeyEvent::Esc, KeyEvent::Up, KeyEvent::Down, KeyEvent::Char('\n')]) {
                KeyEvent::Esc => {
                    return None;
                }
                KeyEvent::Up => {
                    if cursor_idx > 0 {
                        cursor_idx -= 1;
                        if list_start_idx > 0 {
                            list_start_idx -= 1;
                        }
                    }
                }
                KeyEvent::Down => {
                    if cursor_idx < list.len() - 1 {
                        cursor_idx += 1;
                        if list_start_idx < list.len() - text_h as usize {
                            list_start_idx += 1;
                        }
                    }
                }
                KeyEvent::Char(c) => {
                    if c == '\n' {
                        if list[cursor_idx].selectable {
                            return Some(cursor_idx);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn wait_for_input(&self) {
        let mut stdin = self.input.read_sync();
        loop {
            if let Some(_) = stdin.next() {
                return;
            }
        }
    }

    fn wait_for_key(&self, allowed_keys: Vec<KeyEvent>) -> KeyEvent {
        let mut stdin = self.input.read_sync();
        loop {
            if let Some(event) = stdin.next() {
                match event {
                    InputEvent::Keyboard(key_event) => {
                        if allowed_keys.contains(&key_event) {
                            return key_event;
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

enum BoxSymbols {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Vertical,
    Horizontal,
    ListCursor,
    TitlePrefix,
    TitleSuffix,
    ProgressBlock
}

impl BoxSymbols {
    fn simple(&self) -> &str {
        match self {
            TopLeft => return "/",
            TopRight => return "\\",
            BottomLeft => return "\\",
            BottomRight => return "/",
            Vertical => return "|",
            Horizontal => return "-",
            ListCursor => return ">",
            TitlePrefix => return "[",
            TitleSuffix => return "]",
            ProgressBlock => return "█",
        }
    }

    fn fancy(&self) -> &str {
        match self {
            TopLeft => return "┏",
            TopRight => return "┓",
            BottomLeft => return "┗",
            BottomRight => return "┛",
            Vertical => return "┃",
            Horizontal => return "━",
            ListCursor => return "▶",
            TitlePrefix => return "┫",
            TitleSuffix => return "┣",
            ProgressBlock => return "█",
        }
    }

    pub fn symbol_for_os(symbol: BoxSymbols) -> String {
        if cfg!(target_os = "windows") {
            return symbol.simple().to_string();
        } else {
            return symbol.fancy().to_string();
        }
    }

    pub fn print_os_symbol(symbol: BoxSymbols) {
        print!("{}", BoxSymbols::symbol_for_os(symbol));
    }
}