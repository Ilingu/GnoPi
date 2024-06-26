use std::time::Duration;

use gtk::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    adw, gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmApp,
    RelmWidgetExt, SimpleComponent,
};

// include pi digits into the binary (1 million digits)
const PI_DIGITS: &[u8; 1_000_000] = include_bytes!("../assets/1m");

struct HeaderModel;

#[derive(Debug)]
enum HeaderOutput {
    Preferences,
    About,
}

relm4::new_action_group!(HeaderMenuActionGroup, "win");
relm4::new_stateless_action!(OpenPreference, HeaderMenuActionGroup, "preferences");
relm4::new_stateless_action!(OpenAbout, HeaderMenuActionGroup, "about");

#[relm4::component]
impl SimpleComponent for HeaderModel {
    type Init = ();
    type Input = ();
    type Output = HeaderOutput;

    view! {
        #[root]
        header = adw::HeaderBar {
            pack_end = &gtk::MenuButton {
                set_icon_name: "open-menu-symbolic",
                #[wrap(Some)]
                    set_popover = &gtk::PopoverMenu::from_model(Some(&main_menu)) {}
            },
        }
    }

    menu! {
        main_menu: {
            "Preferences" => OpenPreference,
            "About GnoPi" => OpenAbout,
        }
    }

    fn init(
        _params: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HeaderModel;
        let widgets = view_output!();

        let senderp = sender.clone();
        let action_preference: RelmAction<OpenPreference> = {
            RelmAction::new_stateless(move |_| {
                senderp
                    .output(HeaderOutput::Preferences)
                    .expect("Failed to open preference");
            })
        };

        let sendera = sender.clone();
        let action_about: RelmAction<OpenAbout> = {
            RelmAction::new_stateless(move |_| {
                sendera
                    .output(HeaderOutput::About)
                    .expect("Failed to open about");
            })
        };

        let mut group = RelmActionGroup::<HeaderMenuActionGroup>::new();
        group.add_action(action_preference);
        group.add_action(action_about);
        group.register_for_widget(&widgets.header);

        ComponentParts { model, widgets }
    }
}

enum AppMode {
    Blind,
    Visible,
}

struct AppModel {
    user_pi: String,
    mode: AppMode,
    timeout: Option<Duration>,

    header: Controller<HeaderModel>,
}

#[derive(Debug)]
enum AppMsg {
    AddChar(char),
    Open(HeaderOutput),
    Close(HeaderOutput),
}

#[relm4::component]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppMsg;
    type Output = ();

    view! {
        main_window = gtk::Window {
            set_title: Some("GnoPi"),
            set_default_width: 500,
            set_default_height: 400,
            set_titlebar: Some(model.header.widget()),


            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_margin_all: 5,
            }
        }
    }

    // Initialize the UI.
    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header: Controller<HeaderModel> = HeaderModel::builder()
            .launch(())
            .forward(sender.input_sender(), AppMsg::Open);

        let model = AppModel {
            user_pi: String::new(),
            mode: AppMode::Visible,
            timeout: None,
            header,
        };

        // Insert the macro code generation here
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::AddChar(_) => todo!(),
            AppMsg::Open(page) => println!("Opening: {page:?}"),
            AppMsg::Close(_) => todo!(),
        };
    }
}

const APP_ID: &str = "com.ilingu.pitrainer";

fn main() {
    relm4_icons::initialize_icons();
    let app = RelmApp::new(APP_ID);
    app.run::<AppModel>(());
}
