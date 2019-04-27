extern crate sysinfo;
extern crate tui;
extern crate termion;

mod app;
mod inspector;

use termion::raw::{IntoRawMode};
use crate::app::App;

fn main() -> Result<(), std::io::Error> {
    let stdout = std::io::stdout().into_raw_mode()?;
    let mut app = App::new(stdout);

    app.setup()?;

    app.run()?;

    Ok(())
}
