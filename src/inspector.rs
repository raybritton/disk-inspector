use sysinfo::{SystemExt, DiskExt, System};
use std::path::PathBuf;
use std::fs::DirEntry;
use crate::view::draw_dir_items;
use crate::index_of::IndexOf;
use std::ffi::OsStr;
use crate::terminal_helper::TerminalHelper;
use std::process::exit;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Disk {
    pub name: String,
    pub available_space: u64,
    pub total_space: u64,
    pub root: Arc<DiskItem>,
}

#[derive(Debug, Clone)]
pub struct DiskItem {
    pub path: PathBuf,
    pub children: Vec<Arc<DiskItem>>,
    size: u64,
    pub files_size: u64,
    pub is_dir: bool,
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

    pub fn name(&self) -> String {
        return self.path.file_name().unwrap_or(OsStr::new("<Root>")).to_string_lossy().to_string();
    }

    fn populate(&mut self, observer: &mut impl FnMut(u64)) -> Result<(), std::io::Error> {
        match self.path.read_dir() {
            Ok(entries) => {
                for entry in entries {
                    let mut disk_item = DiskItem::new(&entry?)?;
                    if disk_item.is_dir && !disk_item.is_symlink {
                        disk_item.populate(observer)?;
                    } else {
                        if disk_item.is_symlink {}
                    }
                    self.children.push(Arc::new(disk_item));
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
        self.children.sort_by(|lhs, rhs| {
            let lhs_file = lhs.path.file_name().unwrap_or(OsStr::new("")).to_string_lossy().to_string().to_lowercase();
            let rhs_file = rhs.path.file_name().unwrap_or(OsStr::new("")).to_string_lossy().to_string().to_lowercase();
            lhs.is_dir.cmp(&rhs.is_dir).reverse().then(lhs_file.cmp(&rhs_file))
        });
        observer(self.files_size);
        Ok(())
    }
}

pub fn get_all_disks(system: System) -> Vec<Disk> {
    return system.get_disks()
        .iter()
        .map(|disk| Disk {
            name: disk.get_name().to_string_lossy().into_owned(),
            available_space: disk.get_available_space(),
            total_space: disk.get_total_space(),
            root: Arc::new(DiskItem::new_root(disk.get_mount_point().to_owned())),
        })
        .collect();
}

pub enum Status {
    Reading { percentage: usize },
    Done,
}

pub struct Inspector<F: FnMut(Status)> {
    total_used_space: u64,
    bytes_counted: u64,
    status_observer: F,
}

impl<F> Inspector<F> where F: FnMut(Status) -> () {
    pub fn new(total_used_space: u64, observer: F) -> Inspector<F> {
        Inspector {
            total_used_space,
            bytes_counted: 0,
            status_observer: observer,
        }
    }

    pub fn populate(&mut self, disk_root_path: PathBuf) -> Result<DiskItem, std::io::Error> {
        let bytes_counted = &mut (self.bytes_counted as f64);
        let total_used_space = self.total_used_space as f64;
        let observer = &mut self.status_observer;
        let mut disk_item = DiskItem::new_root(disk_root_path);
        disk_item.populate(&mut |bytes| {
            *bytes_counted += bytes as f64;
            observer(Status::Reading { percentage: ((*bytes_counted / total_used_space) * 100_f64) as usize });
        })?;
        (self.status_observer)(Status::Done);
        Ok(disk_item)
    }
}

pub struct DirNav {
    disk: Box<Disk>,
}

impl DirNav {
    pub fn new(disk: Disk) -> DirNav {
        return DirNav {
            disk: Box::new(disk),
        };
    }
}

impl DirNav {
    //This needs to be fixed, it definitely will cause a stack overflow eventually
    pub fn navigate_directory(&self, terminal_helper: &TerminalHelper, current_dir: Arc<DiskItem>, mut parents: Vec<Arc<DiskItem>>) {
        terminal_helper.clear_screen();

        let parent_names: Vec<String> = parents.iter().map(|item| format!("'{}'", item.name())).collect();
        debug!("Path: {}", parent_names.join("/"));

        let item_names: Vec<(String, u64, bool)> = current_dir.children
            .iter()
            .map(|item| (item.name(), item.size, item.is_dir))
            .collect();
        let title = current_dir.path.to_string_lossy().to_string();
        let show_go_up = !parents.is_empty();

        debug!("Navigating {} with {} children", title, current_dir.children.len());

        match draw_dir_items(terminal_helper, title, show_go_up, item_names) {
            None => {
                terminal_helper.teardown();
                exit(0);
            }
            Some(selected) => {
                if selected == ".." {
                    let new_dir = parents.pop().unwrap();
                    debug!("Removed {}", new_dir.name());
                    self.navigate_directory(terminal_helper, new_dir, parents);
                } else {
                    let idx = current_dir.children.index_of(|item| item.name() == selected).unwrap();
                    let new_dir = &current_dir.children[idx];
                    debug!("Adding {}", current_dir.name());
                    parents.push(current_dir.clone());
                    self.navigate_directory(terminal_helper, new_dir.clone(), parents);
                }
            }
        }
    }
}
