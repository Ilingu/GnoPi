use adw::prelude::*;
use relm4::{
    adw,
    factory::{positions::GridPosition, Position},
    gtk,
    prelude::{DynamicIndex, FactoryComponent},
    FactorySender,
};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum PiDigitState {
    Right,
    Wrong,
    Placeholder,
}

#[derive(Clone)]
pub struct PiDigitModel {
    pub digit: u8,
    pub state: PiDigitState,
    pub digits_per_row: u8,
}

#[derive(Debug, Clone)]
pub enum PiDigitInput {
    UpdateDigitState((u8, PiDigitState)),
}

impl Position<GridPosition, DynamicIndex> for PiDigitModel {
    fn position(&self, index: &DynamicIndex) -> GridPosition {
        let index = index.current_index();
        let x = index % self.digits_per_row as usize;
        let y = index / self.digits_per_row as usize;
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
    type Input = PiDigitInput;
    type Output = ();
    type Init = (u8, PiDigitState, u8);
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_css_classes: &["pill", "title-3",
                match self.state {
                    PiDigitState::Right => "suggested-action",
                    PiDigitState::Wrong => "destructive-action",
                    PiDigitState::Placeholder => "raised",
                }
            ],
            #[watch]
            set_label: &self.digit.to_string(),
        }
    }

    fn init_model(
        (digit, state, dpr): Self::Init,
        _index: &DynamicIndex,
        _sender: FactorySender<Self>,
    ) -> Self {
        Self {
            digit,
            state,
            digits_per_row: dpr,
        }
    }

    fn update(&mut self, message: Self::Input, _s: FactorySender<Self>) {
        match message {
            PiDigitInput::UpdateDigitState((digit, state)) => {
                self.digit = digit;
                self.state = state;
            }
        };
    }
}
