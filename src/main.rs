extern crate sysinfo;
extern crate cursive;
#[macro_use]
extern crate log;
extern crate simplelog;

mod app;
mod inspector;
mod view;
mod index_of;

use simplelog::*;

use std::fs::File;

use cursive::traits::*;
use crate::app::App;
use cursive::Cursive;
use crate::inspector::{DiskItem, navigate_directory};
use crate::view::*;
use cursive::views::{Dialog, ProgressBar};
use std::error::Error;
use core::borrow::BorrowMut;
use cursive::utils::Counter;
use crate::index_of::IndexOf;

fn main() -> Result<(), std::io::Error> {
    WriteLogger::init(LevelFilter::Debug, Config::default(), File::create("di.log").unwrap()).unwrap();

    debug!("Starting up");

    let mut siv = Cursive::default();
    siv.add_global_callback('q', |s| s.quit());

    let app = App::new();

    siv.add_layer(Dialog::new().title("Gathering system info"));

    debug!("Getting disk info");

    match app.setup() {
        Ok(disk_list) => {
            debug!("{} disks found", disk_list.len());
            let disks = disk_list;
            let disk_info_list = disks.iter().map(|disk| (disk.name.clone(), disk.available_space, disk.total_space)).collect();

            show_disk_list(&mut siv, disk_info_list, move |lambda_siv, disk_name| {
                let idx = disks.index_of(|disk| disk.name == disk_name).unwrap();
                let mut disk = disks.get(idx).unwrap().clone();

                debug!("Disk is starting with {} children", disk.root.children.len());

                let callback = lambda_siv.borrow_mut().cb_sink().clone();
                let counter = Counter::new(0);

                lambda_siv.pop_layer();
                lambda_siv.add_layer(Dialog::around(ProgressBar::new()
                    .range(0, 100)
                    .with_value(counter.clone())
                    .fixed_width(100))
                    .title("Reading file sizes"));

                debug!("Reading all files");

                let child = app.read_file_sizes(&mut disk, counter.clone());

                loop {
                    lambda_siv.refresh();
                    if counter.get() >= 100 {
                        debug!("Counter above 100");
                        break;
                    }
                }

                debug!("Disk read reported complete");

                let filled_disk = child.join().unwrap().unwrap();

                debug!("Disk read thread complete");

                debug!("Disk is finishing with {} children", filled_disk.root.children.len());

                lambda_siv.set_user_data(filled_disk.root);
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

    debug!("Beginning nav");
    siv.add_layer(Dialog::new().title("Loading"));

    navigate_directory(siv, root, vec![]);
}


pub fn human_readable_bytes(bytes: f64) -> String {
    let unit = 1024.0;
    if bytes < unit { return format!("{}B", bytes); }
    let exp = (bytes.ln() / unit.ln()) as usize;
    let pre = ['k', 'M', 'G', 'T', 'P'][(exp as usize) - 1];
    return format!("{:.1}{}B", bytes / unit.powf(exp as f64), pre);
}