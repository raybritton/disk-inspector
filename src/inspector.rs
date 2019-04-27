use sysinfo::{SystemExt, DiskExt, System};
use std::path::PathBuf;
use std::fs::DirEntry;

#[derive(Debug)]
pub struct Disk {
    pub name: String,
    pub size: u64,
    pub available_space: u64,
    pub total_space: u64,
    pub root: DiskItem,
}

#[derive(Debug)]
pub struct DiskItem {
    path: PathBuf,
    pub children: Vec<DiskItem>,
    size: u64,
    files_size: u64,
    is_dir: bool,
    is_symlink: bool,
    bad_file: bool,
}

impl DiskItem {
    fn new_root(path: PathBuf) -> DiskItem {
        return DiskItem {
            path,
            children: vec![],
            size: 0,
            files_size: 0,
            is_dir: true,
            is_symlink: false,
            bad_file: false,
        };
    }

    fn new(entry: &DirEntry) -> Result<DiskItem, std::io::Error> {
        let path = entry.path();
        let mut is_dir = false;
        let mut is_symlink = false;
        let mut size = 0;
        let mut bad_file = false;
        match entry.metadata() {
            Ok(metadata) => {
                is_symlink = metadata.file_type().is_symlink();
                is_dir = metadata.file_type().is_dir();
                size = metadata.len();
            }
            Err(_) => {
                bad_file = true;
            }
        }
        return Ok(DiskItem { path, children: vec![], size, files_size: 0, is_dir, is_symlink, bad_file });
    }

    fn populate(&mut self, mut observer: impl FnMut(u64)) -> Result<(), std::io::Error> {
        self.populate_actual(&mut observer)
    }

    fn populate_actual(&mut self, observer: &mut impl FnMut(u64)) -> Result<(), std::io::Error> {
        match self.path.read_dir() {
            Ok(entries) => {
                for entry in entries {
                    let mut disk_item = DiskItem::new(&entry?)?;
                    if disk_item.is_dir && !disk_item.is_symlink {
                        disk_item.populate_actual(observer)?;
                    } else {
                        if disk_item.is_symlink {}
                    }
                    self.children.push(disk_item);
                }
            }
            Err(_) => { /*don't care, nothing can be done*/ }
        }
        self.size += self.children
            .iter()
            .fold(0, |acc, child| acc + child.size);
        self.files_size = self.children
            .iter()
            .filter(|child| !child.is_dir && !child.is_symlink)
            .fold(0, |acc, child| acc + child.size);
        observer(self.files_size);
        Ok(())
    }
}

pub fn get_all_disks(system: System) -> Vec<Disk> {
    return system.get_disks()
        .iter()
        .map(|disk| Disk {
            name: disk.get_name().to_string_lossy().into_owned(),
            size: disk.get_total_space(),
            available_space: disk.get_available_space(),
            total_space: disk.get_total_space(),
            root: DiskItem::new_root(disk.get_mount_point().to_owned()),
        })
        .collect();
}

pub enum Status {
    Reading { percentage: f64 },
    Done,
}

pub struct Inspector<'a, F: FnMut(Status)> {
    disk: Disk,
    total_used_space: u64,
    bytes_counted: u64,
    status_observer: F,
    mem_used: u128,
}

impl<'a, F> Inspector<'a, F> where F: FnMut(Status) -> () {
    pub fn new(mut disk: Disk, observer: F) -> Inspector<F> {
        let total_used_space = disk.size - disk.available_space;
        Inspector {
            disk,
            total_used_space,
            bytes_counted: 0,
            status_observer: observer,
            mem_used: 0,
        }
    }

    pub fn populate(&mut self) -> Result<(), std::io::Error> {
        let bytes_counted = &mut (self.bytes_counted as f64);
        let total_used_space = self.total_used_space as f64;
        let observer = &mut self.status_observer;
        self.disk.root.populate(|bytes| {
            *bytes_counted += bytes as f64;
            observer(Status::Reading { percentage: (*bytes_counted / total_used_space) });
        })?;
        println!("Last file read");
        (self.status_observer)(Status::Done);
        Ok(())
    }
}