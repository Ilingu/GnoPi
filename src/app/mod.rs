pub mod preferences;

use std::time::Duration;

use crate::{
    components::{
        about::{AboutInput, AboutPageModel},
        header::{HeaderModel, HeaderOutput},
        pi_digit::{PiDigitInput, PiDigitModel, PiDigitState},
        preferences::{PreferencesPageInput, PreferencesPageModel, PreferencesPageOutput},
    },
    config,
};
use adw::prelude::*;
use preferences::AppPreferences;
use relm4::{
    abstractions::Toaster,
    adw,
    factory::FactoryVecDeque,
    gtk::{self, gdk::Key, glib::SourceId, EventControllerKey},
    Component, ComponentController, ComponentParts, ComponentSender, Controller, RelmWidgetExt,
    SimpleComponent,
};

// include pi digits into the binary (1 million digits)
const PI_DIGITS: &[u8; 1_000_000] = include_bytes!("../../data/app/1m");

// App Utils

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AppMode {
    Blind,
    Learn,
    InstantDeath,
}

impl TryFrom<u8> for AppMode {
    type Error = ();

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(AppMode::Blind),
            1 => Ok(AppMode::Learn),
            2 => Ok(AppMode::InstantDeath),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq)]
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
    timeout_progress: f64,

    // components
    header: Controller<HeaderModel>,
    about_page: Controller<AboutPageModel>,
    preferences_page: Controller<PreferencesPageModel>,
    toaster: Toaster,
    current_page: AppPages,

    // factories
    pi_digits: FactoryVecDeque<PiDigitModel>,

    // other
    timeout_ticker: Option<SourceId>,
}

impl AppModel {
    fn reset_digits(&mut self) {
        self.pi_digits.guard().clear();
        self.curr_pi_index = 0;
    }

    fn spawn_ticker(&mut self, tick_sender: ComponentSender<Self>) {
        if self.preferences.timeout.is_none() {
            return;
        }
        let ticker_id = gtk::glib::timeout_add_local(
            Duration::from_millis(config::PROGRESSBAR_UPDATE_FREQUENCY as u64),
            move || {
                tick_sender.input(AppInput::TickTimeout);
                gtk::glib::ControlFlow::Continue
            },
        );
        self.timeout_ticker = Some(ticker_id);
    }

    fn clear_ticker(&mut self) {
        if let Some(ticker) = self.timeout_ticker.take() {
            ticker.remove();
        }
        self.timeout_progress = 0.0; // after ticker kill to prevent a last minute trigger
    }
}

#[derive(Debug)]
pub enum AppInput {
    KeyPressed(Key),
    AddDigit(char),
    RemoveLastDigit,
    TickTimeout,

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

            add_controller: {
                let key_event = EventControllerKey::new();
                let key_sender = sender.clone();
                key_event.connect_key_pressed(move |_, key, _, _| {
                    key_sender.input(AppInput::KeyPressed(key));
                    gtk::glib::Propagation::Proceed
                });
                key_event
            },

            gtk::Overlay {
                add_overlay =  &gtk::ProgressBar {
                    set_valign: gtk::Align::Start,
                    set_hexpand: true,
                    add_css_class: "osd",
                    #[watch]
                    set_fraction: model.timeout_progress,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[local_ref]
                    toast_overlay -> adw::ToastOverlay {
                        set_vexpand: true,
                        set_hexpand: true,

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
                                    set_spacing:5,

                                    gtk::Label {
                                        #[watch]
                                        set_label: match model.preferences.mode {
                                            AppMode::Blind => "Blind PI 😵‍💫",
                                            AppMode::Learn => "Learn PI! 👨‍🎓",
                                            AppMode::InstantDeath => "Instant Death PI ☠️",
                                        },
                                        set_css_classes: &["title-1"],
                                        #[watch]
                                        set_margin_bottom: match model.preferences.mode {
                                            AppMode::Blind | AppMode::InstantDeath => 5,
                                            AppMode::Learn => 15,
                                        },
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_visible: match model.preferences.mode {
                                            AppMode::Blind | AppMode::InstantDeath => true,
                                            AppMode::Learn => false,
                                        },
                                        set_label: "(Start typing the digits 🖮)",
                                        #[watch]
                                        set_margin_bottom: match model.preferences.mode {
                                            AppMode::Blind | AppMode::InstantDeath => 10,
                                            AppMode::Learn => 0,
                                        },
                                    },
                                    #[name = "scrolled_window"]
                                    gtk::ScrolledWindow {
                                        set_css_classes: &["undershoot-top", "undershoot-bottom"],

                                        set_hexpand: true,
                                        set_vexpand: true,
                                        set_valign: gtk::Align::Fill,
                                        set_halign: gtk::Align::Fill,

                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                        set_vscrollbar_policy: gtk::PolicyType::Automatic,

                                        #[watch]
                                        set_vadjustment: Some(&{
                                            let scroll_pos = 100.0*(model.curr_pi_index as f64)/(model.preferences.digits_per_row as f64);
                                            gtk::Adjustment::new(scroll_pos, 0.0, scroll_pos, 30.0, 0.0, 0.0)
                                        }),

                                        #[local_ref]
                                        pi_digits_box -> gtk::Grid {
                                            set_orientation: gtk::Orientation::Horizontal,
                                            set_column_spacing: 5,
                                            set_row_spacing: 10,
                                        }
                                    }
                                }
                            },
                        },
                    }
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
            .launch(preferences)
            .forward(sender.input_sender(), AppInput::SetPreference);

        // factories
        let pi_digits = FactoryVecDeque::builder()
            .launch(gtk::Grid::default())
            .detach();

        // define default model
        let model = AppModel {
            curr_pi_index: 0,
            preferences,
            current_page: AppPages::Placeholder,
            timeout_progress: 0.0,

            header,
            about_page,
            preferences_page,
            toaster: Toaster::default(),
            pi_digits,

            timeout_ticker: None,
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
            AppInput::TickTimeout => {
                if let Some(timeout) = self.preferences.timeout {
                    let incremental_step =
                        config::PROGRESSBAR_UPDATE_FREQUENCY as f64 / timeout.as_millis() as f64;
                    self.timeout_progress += incremental_step;
                    if self.timeout_progress >= 1.0 {
                        // game over, reset game
                        sender.input(AppInput::SwitchPage(AppPages::Placeholder));
                    }
                }
            }
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
                if self.current_page != AppPages::Memoriser || !character.is_numeric() {
                    return;
                }
                let digit = tor!(character.to_digit(10)) as u8;
                let state = if digit == PI_DIGITS[self.curr_pi_index] {
                    PiDigitState::Right
                } else {
                    PiDigitState::Wrong
                };

                if self.preferences.mode == AppMode::InstantDeath && state == PiDigitState::Wrong {
                    // game over, reset game
                    return sender.input(AppInput::SwitchPage(AppPages::Placeholder));
                }
                if state == PiDigitState::Right {
                    self.clear_ticker(); // restart timer
                    self.spawn_ticker(sender.clone())
                }

                match self.preferences.mode {
                    AppMode::Blind | AppMode::InstantDeath => {
                        self.pi_digits.guard().push_back((
                            digit,
                            state,
                            self.preferences.digits_per_row,
                        ));
                    }
                    AppMode::Learn => {
                        // update digit
                        self.pi_digits.guard().send(
                            self.curr_pi_index,
                            PiDigitInput::UpdateDigitState((digit, state)),
                        );

                        // add next visible digit
                        if self
                            .pi_digits
                            .guard()
                            .get(self.curr_pi_index + config::PRELOADED_DIGITS)
                            .is_none()
                        {
                            self.pi_digits.guard().push_back((
                                PI_DIGITS[self.curr_pi_index + config::PRELOADED_DIGITS],
                                PiDigitState::Placeholder,
                                self.preferences.digits_per_row,
                            ));
                        }
                    }
                };

                self.curr_pi_index += 1
            }
            AppInput::RemoveLastDigit => {
                if self.current_page != AppPages::Memoriser {
                    return;
                }
                match self.preferences.mode {
                    AppMode::Blind => {
                        tor!(self.pi_digits.guard().pop_back());
                        self.curr_pi_index = self.curr_pi_index.saturating_sub(1);
                    }
                    AppMode::Learn => {
                        // remove last digit of the user
                        self.curr_pi_index = self.curr_pi_index.saturating_sub(1);
                        // and add right one
                        self.pi_digits.guard().send(
                            self.curr_pi_index,
                            PiDigitInput::UpdateDigitState((
                                PI_DIGITS[self.curr_pi_index],
                                PiDigitState::Placeholder,
                            )),
                        );
                    }
                    _ => {}
                };
            }
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
                    PreferencesPageOutput::SetMode(mode) => {
                        self.preferences.mode = mode;

                        self.reset_digits(); // reset game state
                        if mode == AppMode::Learn {
                            PI_DIGITS
                                .iter()
                                .take(config::PRELOADED_DIGITS)
                                .for_each(|d| {
                                    self.pi_digits.guard().push_back((
                                        *d,
                                        PiDigitState::Placeholder,
                                        self.preferences.digits_per_row,
                                    ));
                                })
                        }
                    }
                    PreferencesPageOutput::SetTimeout(dur) => self.preferences.timeout = dur,
                    PreferencesPageOutput::SetDigitsPerRow(digits_per_row) => {
                        self.preferences.digits_per_row = digits_per_row;

                        let mut guard = self.pi_digits.guard();
                        let cloned_digits = guard.iter().cloned().collect::<Vec<_>>();
                        guard.clear();

                        for d in cloned_digits {
                            guard.push_back((d.digit, d.state, digits_per_row));
                        }
                    }
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
            AppInput::SwitchPage(page) => {
                self.current_page = page;
                match self.current_page {
                    AppPages::Placeholder => {
                        self.reset_digits();
                        self.clear_ticker();
                    }
                    AppPages::Memoriser if self.preferences.mode == AppMode::Learn => PI_DIGITS
                        .iter()
                        .take(config::PRELOADED_DIGITS)
                        .for_each(|d| {
                            self.pi_digits.guard().push_back((
                                *d,
                                PiDigitState::Placeholder,
                                self.preferences.digits_per_row,
                            ));
                        }),
                    _ => {}
                }
            }
        };
    }
}
