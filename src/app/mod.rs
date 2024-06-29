pub mod preferences;

use std::time::Duration;

use crate::components::{
    about::{AboutInput, AboutPageModel},
    header::{HeaderModel, HeaderOutput},
    pi_digit::PiDigitModel,
    preferences::{PreferencesPageInput, PreferencesPageModel, PreferencesPageOutput},
};
use adw::prelude::*;
use preferences::AppPreferences;
use relm4::{
    abstractions::Toaster,
    adw,
    factory::FactoryVecDeque,
    gtk::{self, gdk::Key, EventControllerKey},
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent,
};

// include pi digits into the binary (1 million digits)
const PI_DIGITS: &[u8; 1_000_000] = include_bytes!("../../data/app/1m");

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

#[derive(Debug)]
pub enum AppPages {
    Placeholder,
    Memoriser,
}

macro_rules! push_toast {
    ($e:expr, $f:expr, $sender:expr) => {
        $sender.input(AppInput::PushToast((
            $e.to_string(),
            Duration::from_secs($f),
        )))
    };
}

/// Simple macro to return when None (tor = try or return)
macro_rules! tor {
    ($expr:expr) => {
        match $expr {
            Some(val) => val,
            None => return,
        }
    };
}

// App Component

pub struct AppModel {
    curr_pi_index: usize,
    preferences: AppPreferences,

    // components
    header: Controller<HeaderModel>,
    about_page: Controller<AboutPageModel>,
    preferences_page: Controller<PreferencesPageModel>,
    toaster: Toaster,
    current_page: AppPages,

    // factories
    pi_digits: FactoryVecDeque<PiDigitModel>,
}

#[derive(Debug)]
pub enum AppInput {
    KeyPressed(Key),
    AddDigit(char),
    RemoveLastDigit,

    Open(HeaderOutput),
    SetPreference(PreferencesPageOutput),
    PushToast((String, Duration)),
    SwitchPage(AppPages),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = AppPreferences;
    type Input = AppInput;
    type Output = ();

    view! {
        main_window = gtk::Window {
            set_title: Some("GnoPi"),
            set_default_width: 700,
            set_default_height: 500,
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
                        gtk::Stack {
                            set_transition_type: gtk::StackTransitionType::SlideLeftRight,
                            set_transition_duration: 500,

                            #[watch]
                            set_visible_child_name: &format!("{:?}", model.current_page).to_lowercase(),

                            add_named[Some("placeholder")] = &adw::StatusPage {
                                set_icon_name: Some("logo"),
                                set_title: "Welcome to GnoPi!",
                                set_description: Some("A π memorization trainer"),

                                gtk::Button {
                                    set_css_classes: &["suggested-action", "pill"],
                                    set_label: "Launch!",
                                    set_use_underline: true,
                                    set_halign: gtk::Align::Center,
                                    connect_clicked => AppInput::SwitchPage(AppPages::Memoriser)
                                },
                            } ,

                            // #[name = "memoriser"] -> to get the component in init
                            add_named[Some("memoriser")]  = &gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_valign: gtk::Align::Center,

                                add_controller: {
                                    let key_event = EventControllerKey::new();
                                    let key_sender = sender.clone();
                                    key_event.connect_key_pressed(move |_, key, _, _| {
                                        key_sender.input(AppInput::KeyPressed(key));
                                        gtk::glib::Propagation::Proceed
                                    });
                                    key_event
                                },

                                gtk::Label {
                                    set_label: "Memorize!",
                                    set_css_classes: &["title-2"],
                                },
                                gtk::ScrolledWindow {
                                    set_css_classes: &["undershoot-start", "undershoot-end"],
                                    set_hexpand: true,
                                    set_hscrollbar_policy: gtk::PolicyType::Automatic,
                                    set_vscrollbar_policy: gtk::PolicyType::Never,

                                    #[local_ref]
                                    pi_digits_box -> gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 5,
                                    }
                                }
                            }
                        },
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
        // components init
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

        // factories
        let pi_digits = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .detach();

        // define default model
        let model = AppModel {
            curr_pi_index: 0,
            preferences,
            current_page: AppPages::Placeholder,

            header,
            about_page,
            preferences_page,
            toaster: Toaster::default(),
            pi_digits,
        };

        // inject to view!
        let pi_digits_box = model.pi_digits.widget();
        let toast_overlay = model.toaster.overlay_widget();

        // Insert the macro code generation here
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AppInput::KeyPressed(key) => {
                let key_name = tor!(key.name());
                match key_name.as_str() {
                    "BackSpace" => sender.input(AppInput::RemoveLastDigit),
                    _ => {
                        let character = tor!(key.to_unicode());
                        sender.input(AppInput::AddDigit(character));
                    }
                }
            }
            AppInput::AddDigit(character) => {
                if !character.is_numeric() {
                    return; // only numeric value are
                }
                let digit = tor!(character.to_digit(10)) as u8;

                // check if ch is digit
                // add char with right state and if in visible mode add next placeholder
                self.curr_pi_index += 1
            }
            AppInput::RemoveLastDigit => match self.pi_digits.guard().pop_back() {
                Some(_) => self.curr_pi_index -= 1,
                None => push_toast!("Failed to remove last digit ʕノ•ᴥ•ʔノ ︵ ┻━┻", 2, sender),
            },
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
            AppInput::SwitchPage(page) => self.current_page = page,
        };
    }
}
