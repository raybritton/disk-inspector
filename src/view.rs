use cursive::Cursive;
use cursive::traits::*;
use cursive::views::{Dialog, SelectView, LinearLayout};
use crate::human_readable_bytes;
use cursive::direction::Orientation;

pub fn show_disk_list<F: 'static>(siv: &mut Cursive, disk_info_list: Vec<(String, u64, u64)>, callback: F) where F: Fn(&mut Cursive, String) {
    siv.pop_layer();

    let mut select = SelectView::new()
        .autojump();

    for disk in disk_info_list {
        let remaining = disk.2 - disk.1;
        let percent = 1f64 - (disk.1 as f64 / disk.2 as f64);
        let remaining_text = format!("{}/{}", human_readable_bytes(remaining as f64), human_readable_bytes(disk.2 as f64));
        select.add_item(format!("{:<20} {:>14} ({:.2}%)", disk.0, remaining_text, percent), disk.0);
    }

    select.set_on_submit(move |s, disk_name: &String| {
        callback(s, disk_name.clone());
    });

    siv.add_layer(
        Dialog::around(select.scrollable().fixed_size((45, 15)))
            .title("Select disk to inspect")
    );
}

pub fn draw_dir_items<F: 'static>(siv: &mut Cursive, title: String, show_go_up: bool, mut contents: Vec<(String, u64, bool)>, callback: F) where F: Fn(&mut Cursive, String) {
    siv.pop_layer();

    let mut select = SelectView::new()
        .autojump();

    if show_go_up {
        contents.insert(0, ("..".to_string(), 0, true));
    }

    for item in contents {
        let dir = if item.2 { "D" } else { "" };
        select.add_item(format!("{:<26} {:<1} {:>7}", item.0, dir, human_readable_bytes(item.1 as f64)), item.0);
    }

    select.set_on_submit(move |s, disk_name: &String| {
        callback(s, disk_name.clone());
    });

    &siv.add_layer(Dialog::around(
        LinearLayout::new(Orientation::Vertical)
            .child(select)).title(title));
}
