
extern crate rusted_ruins_common as common;

extern crate gio;
extern crate gtk;
extern crate gdk_pixbuf;
extern crate cairo;

mod edit_map;
mod ui;
mod pixbuf_holder;
mod draw_map;

use gio::prelude::*;
use std::env;
use std::path::PathBuf;

pub fn main() {
    let application = gtk::Application::new("com.github.rusted-ruins-map-editor",
                                            gio::ApplicationFlags::empty())
        .expect("Initialization failed.");

    let mut app_dir = get_app_dir().expect("Could not found application directory");
    app_dir.push("paks");
    common::gobj::init(vec![app_dir]);
    application.connect_startup(move |app| {
        ui::build_ui(app);
    });
    application.connect_activate(|_| {});
    application.run(&env::args().collect::<Vec<_>>());
}

/// Get application directory
fn get_app_dir() -> Option<PathBuf> {
    if let Some(e) = env::var_os("RUSTED_RUINS_APP_DIR") {
        return Some(PathBuf::from(e));
    }

    if let Ok(mut exe_file) = env::current_exe() {
        exe_file.pop();
        exe_file.push("data");
        return Some(exe_file);
    }

    if let Ok(mut cdir) = env::current_dir() {
        cdir.push("data");
        return Some(cdir);
    }
    None
}