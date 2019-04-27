use sysinfo::SystemExt;
use crate::inspector::{*};
use std::io::{Stdout};
use termion::raw::{RawTerminal};
use std::thread;
use termion::input::TermRead;
use termion::event::Key;
use std::sync::{Mutex, Arc};
use tui::{*};
use tui::layout::{*};
use tui::widgets::{*};
use tui::backend::TermionBackend;
use tui::style::{*};
use core::borrow::BorrowMut;

pub struct App {
    terminal: Terminal<TermionBackend<RawTerminal<Stdout>>>,
    selected_disk: Option<Disk>,
}

impl App {
    pub fn new(stdout: RawTerminal<Stdout>) -> App {
        let backend = TermionBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();
        return App { terminal, selected_disk: None };
    }
}

impl App {
    pub fn setup(&mut self) -> Result<(), std::io::Error> {
        println!("Gathering system info");

        let mut system = sysinfo::System::new();
        system.refresh_all();
        let disks = get_all_disks(system);
        self.terminal.hide_cursor()?;

        self.terminal.clear()?;

        self.selected_disk = self.select_disk(disks)?;

        self.terminal.clear()?;

        return Ok(());
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        if self.selected_disk.is_none() {
            eprintln!("Something is wrong");
            return Ok(());
        }

        self.read_disk_files()?;

        return Ok(());
    }

    fn read_disk_files(&mut self) -> Result<Disk, std::io::Error> {
        let percent = Arc::new(Mutex::new(0.0));
        let display_percent = percent.clone();
        let mut disk = &mut self.selected_disk.unwrap();

        let child = thread::spawn(move || {
            let mut inspector = Inspector::new(*disk, |status| {
                match status {
                    Status::Reading { percentage } => {
                        *percent.lock().unwrap() = percentage;
                    }
                    Status::Done => {
                        *percent.lock().unwrap() = 2.0;
                    }
                };
            });

            match inspector.populate() {
                Ok(_) => {
                    *percent.lock().unwrap() = 2.0;
                }
                Err(e) => {
                    *percent.lock().unwrap() = 2.0;
                    eprintln!("{:?}", e);
                }
            }
        });

        loop {

        }
    }

    fn select_disk(&mut self, mut disks: Vec<Disk>) -> Result<Option<Disk>, std::io::Error> {
        let mut selected: usize = 0;
        let mut keys = termion::async_stdin().keys();

        let items: Vec<String> = disks.iter()
            .map(|disk| format!("{:<10}   {} used of {}", disk.name, human_readable_bytes(disk.available_space as f64), human_readable_bytes(disk.total_space as f64)))
            .collect();

        loop {
            self.terminal.draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.size());

                let style = Style::default().fg(Color::Black).bg(Color::White);
                SelectableList::default()
                    .block(Block::default().borders(Borders::ALL).title("Hard drives"))
                    .items(&items)
                    .select(Some(selected))
                    .style(style)
                    .highlight_style(style.fg(Color::LightGreen).modifier(Modifier::BOLD))
                    .highlight_symbol(">")
                    .render(&mut f, chunks[0]);
            })?;

            if let Some(input) = &keys.next() {
                if let Ok(key) = input {
                    match key {
                        Key::Esc | Key::Char('q') => return Ok(None),
                        Key::Up => {
                            if selected == 0 {
                                selected = disks.len() - 1;
                            } else {
                                selected -= 1;
                            }
                        }
                        Key::Down => {
                            selected += 1;
                            if selected > disks.len() {
                                selected = 0;
                            }
                        }
                        Key::Char('\n') => {
                            return Ok(Some(disks.remove(selected)));
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

fn human_readable_bytes(bytes: f64) -> String {
    let unit = 1024.0;
    if bytes < unit { return format!("{}B", bytes); }
    let exp = bytes.ln() as usize / unit.ln() as usize;
    let pre = ['k', 'M', 'G', 'T', 'P'][(exp as usize) - 1];
    return format!("{:.1}{}B", bytes / unit.powf(exp as f64), pre);
}