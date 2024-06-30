use adw::prelude::*;
use relm4::{
    adw,
    factory::{positions::GridPosition, Position},
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};

use crate::config;

#[derive(Debug, PartialEq)]
pub enum PiDigitState {
    Right,
    Wrong,
    Placeholder,
}

pub struct PiDigitModel {
    digit: u8,
    state: PiDigitState,
}

impl Position<GridPosition, DynamicIndex> for PiDigitModel {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();
        let x = index % config::PRELOADED_DIGITS;
        let y = index / config::PRELOADED_DIGITS;
        GridPosition {
            column: x as i32,
            row: y as i32,
            width: 1,
            height: 1,
        }
    }
}

#[relm4::factory(pub)]
impl FactoryComponent for PiDigitModel {
    type ParentWidget = gtk::Grid;
    type Input = ();
    type Output = ();
    type Init = (u8, PiDigitState);
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Button {
            set_css_classes: &["pill", "title-3",
                match self.state {
                    PiDigitState::Right => "suggested-action",
                    PiDigitState::Wrong => "destructive-action",
                    PiDigitState::Placeholder => "raised",
                }
            ],
            set_label: &self.digit.to_string(),
        }
    }

    fn init_model(
        (digit, state): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self { digit, state }
    }
}
