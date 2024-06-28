pub mod preferences;

use std::time::Duration;

use crate::components::{
    about::{AboutInput, AboutPageModel},
    header::{HeaderModel, HeaderOutput},
    preferences::{PreferencesPageInput, PreferencesPageModel, PreferencesPageOutput},
};
use adw::prelude::*;
use preferences::AppPreferences;
use relm4::{
    abstractions::Toaster, adw, gtk, Component, ComponentController, ComponentParts,
    ComponentSender, Controller, RelmWidgetExt, SimpleComponent,
};

// App Utils

#[derive(Debug, Copy, Clone)]
pub enum AppMode {
    Blind,
    Visible,
}

impl TryFrom<u8> for AppMode {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(AppMode::Blind),
            1 => Ok(AppMode::Visible),
            _ => Err(()),
        }
    }
}

macro_rules! push_toast {
    ($e:expr, $f:expr, $sender:expr) => {
        $sender.input(AppInput::PushToast((
            $e.to_string(),
            Duration::from_secs($f),
        )))
    };
}

// App Component

pub struct AppModel {
    user_pi: String,
    preferences: AppPreferences,

    // components
    header: Controller<HeaderModel>,
    about_page: Controller<AboutPageModel>,
    preferences_page: Controller<PreferencesPageModel>,
    toaster: Toaster,
}

#[derive(Debug)]
pub enum AppInput {
    AddChar(char),
    Open(HeaderOutput),
    SetPreference(PreferencesPageOutput),
    PushToast((String, Duration)),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = AppPreferences;
    type Input = AppInput;
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

                #[local_ref]
                toast_overlay -> adw::ToastOverlay {
                    set_vexpand: true,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 5,
                        set_margin_all: 5,
                        set_valign: gtk::Align::Center,

                        // Here lie the app UI code
                    },
                }
            }


        }
    }

    // Initialize the UI.
    fn init(
        preferences: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let header: Controller<HeaderModel> = HeaderModel::builder()
            .launch(())
            .forward(sender.input_sender(), AppInput::Open);
        let about_page = AboutPageModel::builder()
            .transient_for(&root)
            .launch(true)
            .detach();
        let preferences_page = PreferencesPageModel::builder()
            .transient_for(&root)
            .launch((true, preferences))
            .forward(sender.input_sender(), AppInput::SetPreference);

        let model = AppModel {
            user_pi: String::new(),
            preferences,

            header,
            about_page,
            preferences_page,
            toaster: Toaster::default(),
        };

        let toast_overlay = model.toaster.overlay_widget();

        // Insert the macro code generation here
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppInput::AddChar(_) => todo!(),
            AppInput::Open(HeaderOutput::About) => {
                if self.about_page.sender().send(AboutInput::Show).is_err() {
                    push_toast!("Failed to open about page", 2, sender);
                }
            }
            AppInput::Open(HeaderOutput::Preferences) => {
                if self
                    .preferences_page
                    .sender()
                    .send(PreferencesPageInput::Show)
                    .is_err()
                {
                    push_toast!("Failed to open preference page", 2, sender);
                }
            }
            AppInput::SetPreference(new_pref) => {
                match new_pref {
                    PreferencesPageOutput::SetMode(mode) => self.preferences.mode = mode,
                    PreferencesPageOutput::SetTimeout(dur) => self.preferences.timeout = dur,
                };
                if AppPreferences::set(self.preferences).is_err() {
                    push_toast!("Failed to save preference", 2, sender);
                }
            }
            AppInput::PushToast((text, timeout)) => {
                let toast = adw::Toast::builder()
                    .title(text)
                    .button_label("Cancel")
                    .timeout(timeout.as_secs() as u32)
                    .build();
                toast.connect_button_clicked(move |this| this.dismiss());
                self.toaster.add_toast(toast);
            }
        };
    }
}
