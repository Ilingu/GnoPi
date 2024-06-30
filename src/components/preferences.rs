use std::time::Duration;

use adw::prelude::*;
use relm4::{adw, gtk, ComponentParts, ComponentSender, SimpleComponent};

use crate::app::{preferences::AppPreferences, AppMode};

pub struct PreferencesPageModel {
    hidden: bool,
    mode: AppMode,
    timeout: Option<Duration>,
}

#[derive(Debug)]
pub enum PreferencesPageInput {
    Show,
    Hide,
    SelectMode(AppMode),
    SelectTimeout(f32),
}

#[derive(Debug)]
pub enum PreferencesPageOutput {
    SetMode(AppMode),
    SetTimeout(Option<Duration>),
    // SetNb(Option<Duration>),
}

#[relm4::component(pub)]
impl SimpleComponent for PreferencesPageModel {
    type Input = PreferencesPageInput;
    type Output = PreferencesPageOutput;
    type Init = (bool, AppPreferences);

    view! {
        #[root]
        adw::PreferencesWindow {
            set_modal: true,
            set_search_enabled: false,

            #[watch]
            set_visible: !model.hidden,
            connect_close_request[sender] => move |_| {
                sender.input(PreferencesPageInput::Hide);
                  gtk::glib::Propagation::Stop
            },

            // add nb of digits per row
            add = &adw::PreferencesPage {
                add = &adw::PreferencesGroup {
                    set_title: "App settings",
                    adw::ComboRow {
                        set_title: "App Mode",
                        set_model: Some(&gtk::StringList::new(&["Blind", "Visible", "InstantDeath"])),

                        #[watch]
                        set_selected: model.mode as u32,
                        connect_selected_notify[sender] => move |combo_row| {
                            if let Some(selected) = combo_row.selected_item() {
                                if let Some(text) = selected.downcast_ref::<gtk::StringObject>() {
                                    let selected_text = text.string().to_string();
                                    let selected_mode = match selected_text.as_str() {
                                        "Blind" => AppMode::Blind,
                                        "Visible" => AppMode::Visible,
                                        "InstantDeath" => AppMode::InstantDeath,
                                        _ => AppMode::Visible // should be unreachable
                                    };
                                    sender.input(PreferencesPageInput::SelectMode(selected_mode));
                                }
                            }
                        }
                    },
                    adw::SpinRow {
                        set_title: "Timeout",
                        set_subtitle: "in seconds (0 to disable)",
                        set_numeric: true,
                        set_digits: 1,
                        set_adjustment: Some(&gtk::Adjustment::new(0.0,0.0,15.0,0.5,0.0,0.0)), // set range and step increment
                        #[watch]
                        set_value: model.timeout.unwrap_or_default().as_secs_f64(),
                        connect_value_notify[sender] => move |spin_row| {
                            sender.input(PreferencesPageInput::SelectTimeout(spin_row.value() as f32));
                        }
                    },
                    adw::SpinRow {
                        set_title: "Digits per row",
                        set_subtitle: "Number of pi digits in one row",
                        set_numeric: true,
                        set_digits: 0,
                        set_adjustment: Some(&gtk::Adjustment::new(0.0,0.0,15.0,0.5,0.0,0.0)), // set range and step increment
                        #[watch]
                        set_value: 10.0,
                        connect_value_notify[sender] => move |spin_row| {
                            sender.input(PreferencesPageInput::);
                        }
                    }
                }
            }
        }
    }

    fn init(
        (hidden, preferences): Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PreferencesPageModel {
            hidden,
            mode: preferences.mode,
            timeout: preferences.timeout,
        };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            PreferencesPageInput::Show => self.hidden = false,
            PreferencesPageInput::Hide => self.hidden = true,
            PreferencesPageInput::SelectMode(mode) => {
                self.mode = mode;
                let _ = sender.output(PreferencesPageOutput::SetMode(mode)); // todo: if failed send toast to main app
            }
            PreferencesPageInput::SelectTimeout(durf32) => {
                let dur = match durf32 == 0.0 {
                    true => None,
                    false => Some(Duration::from_secs_f32(durf32)),
                };
                self.timeout = dur;
                let _ = sender.output(PreferencesPageOutput::SetTimeout(dur));
            }
        }
    }
}
