extern crate sysinfo;
#[macro_use]
extern crate log;
extern crate simplelog;

mod app;
mod inspector;
mod view;
mod index_of;
mod terminal_helper;
mod atomic_counter;

use simplelog::*;

use std::fs::File;

use crate::app::App;
use crate::inspector::{DirNav, Disk};
use crate::view::*;
use std::error::Error;
use crate::terminal_helper::TerminalHelper;
use std::process::exit;
use crate::atomic_counter::AtomicCounter;
use std::sync::Arc;
use crossterm::RawScreen;

fn main() -> Result<(), std::io::Error> {
    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("di.log").unwrap()).unwrap();

    debug!("Starting up");

    let app = App::new();

    let raw = RawScreen::into_raw_mode();
    let terminal_helper = TerminalHelper::new();

    terminal_helper.setup();
//    terminal_helper.on_esc(|| {
//        exit(0);
//    });

    terminal_helper.clear_screen();

    terminal_helper.show_dialog("Gathering system info");

    debug!("Getting disk info");

    match app.setup() {
        Ok(disk_list) => {
            debug!("{} disks found", disk_list.len());
            let disks = disk_list;
            let disk_info_list = disks.iter().map(|disk| (disk.name.clone(), disk.available_space, disk.total_space)).collect();

            match show_disk_list(&terminal_helper, disk_info_list) {
                None => {
                    debug!("Exiting at disk list");
                    exit(0);
                }
                Some(selected) => {
                    debug!("Selected {}", &selected);
                    let mut disk = disks.get(selected).unwrap().clone();

                    terminal_helper.clear_screen();
                    terminal_helper.show_dialog("Getting all file sizes");

                    let progress = Arc::new(AtomicCounter::new());

                    let child = app.read_file_sizes(&mut disk, progress.clone());

                    let mut last_printed = 0;

                    terminal_helper.clear_screen();

                    loop {
                        let progress_value = progress.get();
                        if last_printed != progress_value {
                            last_printed = progress_value;
                            if last_printed ^ 10 == 0 {
                                debug!("{}% read", last_printed);
                            }
                            terminal_helper.draw_progress("Reading files", last_printed);
                        }
                        if progress_value >= 100 {
                            debug!("Read complete");
                            break;
                        }
                    }

                    let x = child.join().expect("join failed");
                    let filled_root = Arc::new(x.expect(""));

                    debug!("Thread joined");

                    let new_disk = Disk {
                        name: disk.name.clone(),
                        available_space: disk.available_space,
                        total_space: disk.total_space,
                        root: filled_root.clone(),
                    };

                    let nav_dir: DirNav = DirNav::new(new_disk);

                    nav_dir.navigate_directory(&terminal_helper, filled_root.clone(), vec![filled_root.clone()]);
                }
            }
        }
        Err(err) => {
            eprintln!("{}", err.description());
            eprintln!("{}", err.raw_os_error().unwrap_or(-1));
            eprintln!("{:?}", err.kind());
        }
    }

    terminal_helper.teardown();

    Ok(())
}


pub fn human_readable_bytes(bytes: f64) -> String {
    let unit = 1024.0;
    if bytes < unit { return format!("{}B", bytes); }
    let exp = (bytes.ln() / unit.ln()) as usize;
    let pre = ['k', 'M', 'G', 'T', 'P'][(exp as usize) - 1];
    return format!("{:.1}{}B", bytes / unit.powf(exp as f64), pre);
}