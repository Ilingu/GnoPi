use adw::prelude::*;
use relm4::{
    adw, gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender, RelmWidgetExt,
};

pub enum PiDigitState {
    Right,
    Wrong,
    Placeholder,
}

pub struct PiDigitModel {
    digit: u8,
    state: PiDigitState,
}

#[relm4::factory(pub)]
impl FactoryComponent for PiDigitModel {
    type ParentWidget = gtk::Box;
    type Input = ();
    type Output = ();
    type Init = PiDigitModel;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Label {
            set_css_classes: &["card", "title-3",
                match self.state {
                    PiDigitState::Right => "suggested-action",
                    PiDigitState::Wrong => "destructive-action",
                    PiDigitState::Placeholder => "raised",
                }
            ],
            set_label: &self.digit.to_string(),
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        init
    }
}
