use sysinfo::SystemExt;
use crate::inspector::{*};
use std::io::{Error, ErrorKind};
use std::thread;
use std::thread::JoinHandle;
use crate::atomic_counter::AtomicCounter;
use std::sync::Arc;

pub struct App {}

impl App {
    pub fn new() -> App {
        return App {};
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

    pub fn read_file_sizes(&self, selected_disk: &mut Disk, progress_counter: Arc<AtomicCounter>) -> JoinHandle<Option<DiskItem>> {
        let name = selected_disk.name.clone();
        let total_space = selected_disk.total_space;
        let available_space = selected_disk.available_space;
        let available_size = total_space - available_space;
        let disk_path = selected_disk.root.path.clone();
        return thread::spawn(move || {
            let mut inspector = Inspector::new(available_size, |status| {
                match status {
                    Status::Reading { percentage } => {
                        progress_counter.set(percentage)
                    }
                    Status::Done => {
                        progress_counter.set(100);
                        debug!("Disk read done");
                    }
                };
            });

            match inspector.populate(disk_path) {
                Ok(root_item) => {
                    progress_counter.set(100);
                    debug!("Disk read complete");
                    return Some(root_item);
                }
                Err(e) => {
                    progress_counter.set(100);
                    eprintln!("{:?}", e);
                    error!("Disk read failed: {:?}", e);
                    return None;
                }
            }
        });
    }
}

