
use serde::{Deserialize, Serialize};

pub mod trait_def;
pub mod counter;
pub mod timer;
pub mod switch;
pub mod list;
pub mod team;
pub mod text;
pub mod calculation;
pub mod penalty_shots;

// Re-export trait and payload structures
pub use trait_def::{Widget, UpdatePayload};
pub use counter::CounterWidget;
pub use timer::TimerWidget;
pub use switch::SwitchWidget;
pub use list::ListWidget;
pub use team::TeamWidget;
pub use text::TextWidget;
pub use calculation::CalculationWidget;
pub use penalty_shots::{PenaltyShotsWidget, ShotResult};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WidgetValue {
    Counter(counter::CounterData),
    Timer(timer::TimerData),
    List(list::ListData),
    Text(text::TextData),
    Calculation(calculation::CalculationData),
    Switch(switch::SwitchData),
    Team(team::TeamData),
    PenaltyShots(penalty_shots::PenaltyShotsData),
}

// Global defaults for serde instantiation
pub fn default_true() -> bool { true }
pub fn default_false() -> bool { false }
pub fn default_frequency() -> i64 { 100 }

pub fn create_widget(value: &WidgetValue) -> Box<dyn Widget> {
    match value {
        WidgetValue::Counter(data) => Box::new(CounterWidget::new(data.clone())),
        WidgetValue::Timer(data) => Box::new(TimerWidget::new(data.clone())),
        WidgetValue::List(data) => Box::new(ListWidget::new(data.clone())),
        WidgetValue::Text(data) => Box::new(TextWidget::new(data.clone())),
        WidgetValue::Calculation(data) => Box::new(CalculationWidget::new(data.clone())),
        WidgetValue::Switch(data) => Box::new(SwitchWidget::new(data.clone())),
        WidgetValue::Team(data) => Box::new(TeamWidget::new(data.clone())),
        WidgetValue::PenaltyShots(data) => Box::new(PenaltyShotsWidget::new(data.clone())),
    }
}

