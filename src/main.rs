mod application;
mod canvas;
mod capture;
mod window;

use application::GnomeShotApplication;
use gtk::prelude::*;

fn main() -> anyhow::Result<()> {
    // Initialize GTK and Adwaita
    let app = GnomeShotApplication::new();

    // Run the application
    let exit_code = app.run();
    std::process::exit(exit_code.into());
}
