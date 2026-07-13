use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SwitchData {
    pub value: bool,
    pub initial_value: bool,
    pub display_true: String,
    pub display_false: String,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct SwitchWidget {
    data: SwitchData,
}

impl SwitchWidget {
    pub fn new(data: SwitchData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> SwitchData {
        let iv = node.children().find(|n| n.has_tag_name("initial_value")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);
        let dt = node.children().find(|n| n.has_tag_name("display_true")).and_then(|n| n.text()).unwrap_or("ON").to_string();
        let df = node.children().find(|n| n.has_tag_name("display_false")).and_then(|n| n.text()).unwrap_or("OFF").to_string();
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        SwitchData { value: iv, initial_value: iv, display_true: dt, display_false: df, dashboard_ui }
    }
}

impl Widget for SwitchWidget {
    fn primary_value(&self) -> serde_json::Value {
        serde_json::Value::from(if self.data.value { self.data.display_true.clone() } else { self.data.display_false.clone() })
    }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value, .. } => {
                match action.as_str() {
                    "on" => self.data.value = true,
                    "off" => self.data.value = false,
                    "toggle" => self.data.value = !self.data.value,
                    "reset" => self.data.value = self.data.initial_value,
                    "set" => self.data.value = {
                        if let Some(new_val) = value.and_then(|v| v.as_bool()) {
                            new_val
                        } else {
                            return (false, String::new())
                        }
                    },
                    _ => return (false, String::new()),
                }
                (true, if self.data.value { self.data.display_true.to_string() } else { self.data.display_false.to_string() })
            }
            UpdatePayload::Value(v) => {
                if let Some(new_val) = v.as_bool() {
                    self.data.value = new_val;
                    (true, if self.data.value { self.data.display_true.to_string() } else { self.data.display_false.to_string() } )
                } else {
                    (false, String::new())
                }
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Switch(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> { 
        let mut extras = HashMap::new();
        extras.insert("value".to_string(), serde_json::Value::from(self.data.value));
        extras
    }
}

