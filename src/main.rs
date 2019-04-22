extern crate sysinfo;
extern crate tui;
extern crate termion;

use sysinfo::SystemExt;
use crate::inspector::{*};
use std::io::{Write, stdout};
use termion::raw::IntoRawMode;
use termion::{clear, cursor};
use std::thread;
use termion::input::TermRead;
use termion::event::Key;
use std::sync::{Mutex, Arc};


mod inspector;

fn main() -> Result<(), std::io::Error> {
    let stdin = termion::async_stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();
    let mut system = sysinfo::System::new();
    system.refresh_all();
    let mut disks = get_all_disks(system);

    write!(stdout, "{}", clear::All)?;

    for disk in &disks {
        println!("{:?}", disk);
    }

    let percent = Arc::new(Mutex::new(0.0));
    let display_percent = percent.clone();

    let child = thread::spawn(move || {
        let mut inspector = Inspector::new(&mut disks[0], |status| {
            match status {
                Status::Reading { percentage } => {
                    *percent.lock().unwrap() = percentage;
//                    println!("{}", percent);
                }
                Status::Done => {
                    *percent.lock().unwrap() = 2.0;
                }
            };
        });

        match inspector.populate() {
            Ok(_) => {
                *percent.lock().unwrap() = 2.0;
            },
            Err(e) => {
                *percent.lock().unwrap() = 2.0;
                eprintln!("{:?}", e);
            },
        }
    });


    let mut keys = stdin.keys();
    write!(stdout, "{}", cursor::Hide)?;
    loop {
        if let Some(input) = keys.next() {
            if let Ok(key) = input {
                if key == Key::Char('q') {
                    break;
                }
            }
        }
        if *display_percent.lock().unwrap() < 1.0 {
            write!(stdout, "\r")?;
            write!(stdout, "{:.2}", *display_percent.lock().unwrap())?;
        } else {
            write!(stdout, "\r")?;
            println!("Done");
            child.join().expect("Failed to join");
            break;
        }
//        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
