extern crate sysinfo;
extern crate cursive;

mod app;
mod inspector;
mod view;

use cursive::traits::*;
use crate::app::App;
use cursive::Cursive;
use crate::inspector::{Disk, DiskItem, DirectoryNavigator};
use crate::view::*;
use cursive::views::{Dialog, ProgressBar, LinearLayout};
use std::error::Error;
use core::borrow::BorrowMut;
use cursive::utils::Counter;
use cursive::direction::Orientation;
use std::borrow::Cow;

trait IndexOf<T> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&T) -> bool;
}

impl IndexOf<Disk> for Vec<Disk> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&Disk) -> bool {
        let mut idx = 0usize;
        for disk in self {
            if predicate(disk) {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }
}

impl IndexOf<DiskItem> for Vec<DiskItem> {
    fn index_of<F>(&self, predicate: F) -> Option<usize> where F: Fn(&DiskItem) -> bool {
        let mut idx = 0usize;
        for disk in self {
            if predicate(disk) {
                return Some(idx);
            }
            idx += 1;
        }
        return None;
    }
}

fn main() -> Result<(), std::io::Error> {
    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    let app = App::new();

    siv.add_layer(Dialog::new().title("Gathering system info"));

    match app.setup() {
        Ok(disk_list) => {
            let disks = disk_list;
            let disk_info_list = disks.iter().map(|disk| (disk.name.clone(), disk.available_space, disk.total_space)).collect();

            show_disk_list(&mut siv, disk_info_list, move |lambda_siv, disk_name| {
                let idx = disks.index_of(|disk| disk.name == disk_name).unwrap();
                let mut disk = disks.get(idx).unwrap().clone();

                let callback = lambda_siv.borrow_mut().cb_sink().clone();
                let counter = Counter::new(0);

                lambda_siv.pop_layer();
                lambda_siv.add_layer(Dialog::around(ProgressBar::new()
                    .range(0, 100)
                    .with_value(counter.clone())
                    .fixed_width(100))
                    .title("Reading file sizes"));


                let child = app.read_file_sizes(&mut disk, counter.clone());

                loop {
                    lambda_siv.refresh();
                    if counter.get() >= 100 {
                        break;
                    }
                }

                child.join().expect_err("Panic during disk read");

                lambda_siv.set_user_data(disk.root);
                callback.send(Box::new(navigate_disk)).unwrap();
            });
        }
        Err(err) => {
            eprintln!("{}", err.description());
            eprintln!("{}", err.raw_os_error().unwrap_or(-1));
            eprintln!("{:?}", err.kind());
            &siv.add_layer(Dialog::new().title(err.description()).button("Ok", |s| s.quit()));
        }
    }

    &siv.run();

    Ok(())
}

fn navigate_disk(siv: &mut Cursive) {
    let root = siv.user_data::<DiskItem>().unwrap().to_owned();
    let mut dir_nav = DirectoryNavigator::new(root);
    dir_nav.navigate_directory(siv);
}


pub fn human_readable_bytes(bytes: f64) -> String {
    let unit = 1024.0;
    if bytes < unit { return format!("{}B", bytes); }
    let exp = (bytes.ln() / unit.ln()) as usize;
    let pre = ['k', 'M', 'G', 'T', 'P'][(exp as usize) - 1];
    return format!("{:.1}{}B", bytes / unit.powf(exp as f64), pre);
}