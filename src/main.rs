mod app;
mod components;
mod config;

use app::{preferences::AppPreferences, AppModel};
use config::APP_ID;
use relm4::{
    gtk::{self, gdk, gio, glib},
    RelmApp,
};

// include pi digits into the binary (1 million digits)
const PI_DIGITS: &[u8; 1_000_000] = include_bytes!("../data/app/1m");

fn main() {
    glib::set_application_name("GnoPi");

    // create app
    let app = RelmApp::new(APP_ID);

    // init icons
    initialize_custom_icons();
    // relm4_icons::initialize_icons();

    // launch app
    app.run::<AppModel>(AppPreferences::load());
}

fn initialize_custom_icons() {
    gio::resources_register_include!("../../../../../icons.gresource").unwrap();

    let display = gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/com/ilingu/icons/");
}
