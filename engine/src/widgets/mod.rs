
use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

pub mod trait_def;
pub mod counter;
pub mod timer;
pub mod switch;
pub mod list;
pub mod team;
pub mod text;
pub mod calculation;
pub mod penalty_shots;

use crate::automations::{AutomationTrigger, TriggerAction, TriggerCondition};

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

pub fn load_config<F>(path: &str, log_callback: F) -> (IndexMap<String, WidgetValue>, String, Vec<AutomationTrigger>)
where
    F: Fn(String, String, String) + Send + 'static,
{
    let mut data = IndexMap::new();
    let mut automations = Vec::new();

    eprintln!("📁 Reading Configuration file {}", path);
    let xml_content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("⚠️ Warning: Could not read {}, using empty config.", path);
        "<ScoreboardConfig></ScoreboardConfig>".to_string()
    });

    let doc = match roxmltree::Document::parse(&xml_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing XML: {}. Returning defaults.", e);
            return (data, "state_persistence.json".to_string(), Vec::new());
        }
    };

    let root = doc.root_element();
    let save_file = root.children()
        .find(|n| n.has_tag_name("persistence_file"))
        .and_then(|n| n.text())
        .unwrap_or("state_persistence.json")
        .to_string();

    for node in root.descendants().filter(|n| n.has_tag_name("widget")) {
        let id = node.children().find(|n| n.has_tag_name("id")).and_then(|n| n.text()).unwrap_or("unknown").to_string();
        let w_type = node.children().find(|n| n.has_tag_name("type")).and_then(|n| n.text()).unwrap_or("");

        let val = match w_type {
            "Counter" => WidgetValue::Counter(counter::CounterWidget::from_xml(&node)),
            "Timer" => WidgetValue::Timer(timer::TimerWidget::from_xml(&node)),
            "Switch" => WidgetValue::Switch(switch::SwitchWidget::from_xml(&node)),
            "List" => WidgetValue::List(list::ListWidget::from_xml(&node)),
            "Team" => WidgetValue::Team(team::TeamWidget::from_xml(&node)),
            "Text" => WidgetValue::Text(text::TextWidget::from_xml(&node)),
            "Calculation" => WidgetValue::Calculation(calculation::CalculationWidget::from_xml(&node)),
            "PenaltyShots" => WidgetValue::PenaltyShots(penalty_shots::PenaltyShotsWidget::from_xml(&node)),
            _ => continue,
        };
        data.insert(id, val);
    }


    for node in root.descendants().filter(|n| n.has_tag_name("trigger")) {
        let cond_node = match node.children().find(|n| n.has_tag_name("condition")) {
            Some(c) => c,
            None => continue,
        };

        let widget_id = cond_node.children().find(|n| n.has_tag_name("widget_id")).and_then(|n| n.text()).unwrap_or("unknown").to_string();
        let operator = cond_node.children().find(|n| n.has_tag_name("operator")).and_then(|n| n.text()).unwrap_or("==").to_string();
        let val_sec = cond_node.children().find(|n| n.has_tag_name("value")).and_then(|n| n.text()?.parse::<f64>().ok()).unwrap_or(0.0);
        let value = (val_sec * 1000.0) as i64;

        let mut actions = Vec::new();
        if let Some(actions_node) = node.children().find(|n| n.has_tag_name("actions")) {
            for act_node in actions_node.children().filter(|n| n.has_tag_name("action")) {
                let target_id = match act_node.children().find(|n| n.has_tag_name("target_id")).and_then(|n| n.text()) {
                    Some(t) => t.to_string(),
                    None => continue,
                };
                let command = match act_node.children().find(|n| n.has_tag_name("action")).and_then(|n| n.text()) {
                    Some(c) => c.to_string(),
                    None => continue,
                };

                let val_text = act_node.children().find(|n| n.has_tag_name("value")).and_then(|n| n.text());
                let val_json = val_text.map(|v| {
                    if let Ok(num) = v.parse::<i64>() {
                        serde_json::Value::Number(num.into())
                    } else if let Ok(num_f) = v.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(num_f) {
                            serde_json::Value::Number(n)
                        } else {
                            serde_json::Value::String(v.to_string())
                        }
                    } else {
                        serde_json::Value::String(v.to_string())
                    }
                });
                actions.push(TriggerAction { target_id, action: command, value: val_json });
            }
        }

        automations.push(AutomationTrigger {
            condition: TriggerCondition { widget_id, operator, value },
            actions,
            active: true,
            last_value: None,
        });
    }

    log_callback("core".to_string(), "loadconfig".to_string(), path.to_string());
    (data, save_file, automations)
}

