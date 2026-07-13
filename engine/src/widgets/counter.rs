use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CounterData {
    pub value: i64,
    pub initial_value: i64,
    pub increments: Vec<i64>,
    pub min_value: i64,
    pub max_value: i64,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct CounterWidget {
    data: CounterData,
}

impl CounterWidget {
    pub fn new(data: CounterData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> CounterData {
        let initial = node.children().find(|n| n.has_tag_name("initial_value")).and_then(|n| n.text()?.parse().ok()).unwrap_or(0);
        let max = node.children().find(|n| n.has_tag_name("max_value")).and_then(|n| n.text()?.parse().ok()).unwrap_or(65535);
        let min = node.children().find(|n| n.has_tag_name("min_value")).and_then(|n| n.text()?.parse().ok()).unwrap_or(0);
        let increments: Vec<i64> = node.descendants().filter(|n| n.has_tag_name("value")).filter_map(|n| n.text()?.parse().ok()).collect();
        let final_increments = if increments.is_empty() { vec![1] } else { increments };
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        CounterData { value: initial, initial_value: initial, increments: final_increments, min_value: min, max_value: max, dashboard_ui }
    }
}

impl Widget for CounterWidget {
    fn primary_value(&self) -> serde_json::Value { serde_json::Value::from(self.data.value) }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value, .. } => {
                let amt = value.and_then(|v| v.as_i64()).unwrap_or(1);
                match action.as_str() {
                    "increment" => {
                        if self.data.value + amt > self.data.max_value { return (false, String::new()); }
                        self.data.value += amt
                    },
                    "decrement" => {
                        if self.data.value - amt < self.data.min_value { return (false, String::new()); }
                        self.data.value -= amt
                    },
                    "set" => {
                        if amt < self.data.min_value || amt > self.data.max_value { return (false, String::new()); }
                        self.data.value = amt
                    },
                    "reset" => self.data.value = self.data.initial_value,
                    _ => return (false, String::new()),
                }
                (true, self.data.value.to_string())
            }
            UpdatePayload::Value(v) => {
                if let Some(new_val) = v.as_i64() {
                    self.data.value = new_val;
                    (true, self.data.value.to_string())
                } else {
                    (false, String::new())
                }
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Counter(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> {
        let mut extras = HashMap::new();
        extras.insert("min".to_string(), serde_json::Value::from(self.data.min_value));
        extras.insert("max".to_string(), serde_json::Value::from(self.data.max_value));
        extras
    }
}

