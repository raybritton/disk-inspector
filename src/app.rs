use sysinfo::SystemExt;
use crate::inspector::{*};
use std::io::{Error, ErrorKind};
use std::thread;
use cursive::utils::Counter;
use std::thread::{JoinHandle};


pub struct App {}

impl App {
    pub fn new() -> App {
        return App { };
    }
}

impl App {
    pub fn setup(&self) -> Result<Vec<Disk>, std::io::Error> {
        let mut system = sysinfo::System::new();
        system.refresh_all();
        let disks = get_all_disks(system);

        if disks.is_empty() {
            return Err(Error::new(ErrorKind::NotFound, "No hard drives found"));
        }

        return Ok(disks);
    }

    pub fn read_file_sizes(&self, selected_disk: &mut Disk, progress_counter: Counter) -> JoinHandle<()> {
        let thread_counter = progress_counter.clone();
        let available_size = selected_disk.total_space - selected_disk.available_space;
        let disk_path = selected_disk.root.path.clone();
        return thread::spawn(move || {
            let mut inspector = Inspector::new(available_size, |status| {
                match status {
                    Status::Reading { percentage } => {
                        thread_counter.set(percentage)
                    }
                    Status::Done => {
                        thread_counter.set(100)
                    }
                };
            });

            match inspector.populate(disk_path) {
                Ok(_) => {
                    thread_counter.set(100);
                }
                Err(e) => {
                    thread_counter.set(100);
                    eprintln!("{:?}", e);
                }
            }
        });
    }
}
