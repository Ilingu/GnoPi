use std::time::Duration;

use crate::components::{
    about::{AboutInput, AboutPageModel},
    header::{HeaderModel, HeaderOutput},
};
use gtk::prelude::*;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    RelmWidgetExt, SimpleComponent,
};

enum AppMode {
    Blind,
    Visible,
}

pub struct AppModel {
    user_pi: String,
    mode: AppMode,
    timeout: Option<Duration>,

    // components
    header: Controller<HeaderModel>,
    about: Controller<AboutPageModel>,
}

#[derive(Debug)]
pub enum AppMsg {
    AddChar(char),
    Open(HeaderOutput),
    Close(HeaderOutput),
}

#[relm4::component(pub)]
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
            set_icon_name: Some("logo"),

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
        let about_dialog = AboutPageModel::builder()
            .transient_for(&root)
            .launch(true)
            .detach();

        let model = AppModel {
            user_pi: String::new(),
            mode: AppMode::Visible,
            timeout: None,
            header,
            about: about_dialog,
        };

        // Insert the macro code generation here
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppMsg::AddChar(_) => todo!(),
            AppMsg::Open(HeaderOutput::About) => self
                .about
                .sender()
                .send(AboutInput::Show)
                .expect("Failed to open About Page"),
            AppMsg::Open(HeaderOutput::Preferences) => {}
            AppMsg::Close(_) => todo!(),
        };
    }
}
