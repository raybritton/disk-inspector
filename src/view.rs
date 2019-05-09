use crate::human_readable_bytes;
use crate::terminal_helper::{TerminalHelper, ListItem};

pub fn show_disk_list(terminal_helper: &TerminalHelper, disk_info_list: Vec<(String, u64, u64)>) -> Option<usize> {
    terminal_helper.clear_screen();

    let items = disk_info_list.iter()
        .map(|disk| {
            let remaining = disk.2 - disk.1;
            let percent = 1_f64 - (disk.1 as f64 / disk.2 as f64);
            let remaining_text = format!("{}/{}", human_readable_bytes(remaining as f64), human_readable_bytes(disk.2 as f64));
            let text = format!("{:<20} {:>14} ({:.2}%)", disk.0, remaining_text, percent);

            ListItem {
                text,
                selectable: true
            }
        })
        .collect();

    return terminal_helper.show_list("Select a hard drive", items);
}

pub fn draw_dir_items(terminal_helper: &TerminalHelper, title: String, show_go_up: bool, mut contents: Vec<(String, u64, bool)>) -> Option<String> {
    
    if show_go_up {
        contents.insert(0, ("..".to_string(), 0, true));
    }

    let items = contents.iter()
        .map(|item| {
            let dir = if item.2 { "D" } else { "" };
            let original_name = item.0.clone();
            let mut name;
            if original_name.len() > 75 {
                let (truncated_name, _) = original_name.split_at(74);
                name = truncated_name.to_string();
                name.push('…');
            } else {
                name = item.0.clone();
            }
            let text = format!("{:<75} {:<1} {:>8}", name, dir, human_readable_bytes(item.1 as f64));
            ListItem {
                text,
                selectable: item.2
            }
        })
        .collect();

    return terminal_helper.show_list(title,items).and_then(|idx| Some(contents[idx].0.clone()));
}