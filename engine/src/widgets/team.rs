use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TeamData {
    pub short_name: String,
    pub name: String,
    pub primary_color: String,
    pub secondary_color: String,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct TeamWidget {
    data: TeamData,
}

impl TeamWidget {
    pub fn new(data: TeamData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> TeamData {
        let short_name = node.children().find(|n| n.has_tag_name("initial_short_name")).and_then(|n| n.text()).unwrap_or("").to_string();
        let name = node.children().find(|n| n.has_tag_name("initial_name")).and_then(|n| n.text()).unwrap_or("").to_string();
        let primary_color = node.children().find(|n| n.has_tag_name("initial_primary_color")).and_then(|n| n.text()).unwrap_or("").to_string();
        let secondary_color = node.children().find(|n| n.has_tag_name("initial_secondary_color")).and_then(|n| n.text()).unwrap_or("").to_string();
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        TeamData { short_name, name, primary_color, secondary_color, dashboard_ui }
    }
}

impl Widget for TeamWidget {
    fn primary_value(&self) -> serde_json::Value { serde_json::Value::from(self.data.short_name.clone()) }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value, .. } => {
                match action.as_str() {
                    "set" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            self.data.short_name = val_str;
                            return (true, self.data.short_name.clone())
                        }
                    }
                    "set_name" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            self.data.name = val_str;
                            return (true, self.data.name.clone())
                        }
                    }
                    "set_primary" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            self.data.primary_color = val_str;
                            return (true, self.data.primary_color.clone())
                        }
                    }
                    "set_secondary" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            self.data.secondary_color = val_str;
                            return (true, self.data.secondary_color.clone())
                        }
                    }
                    _ => return (false, String::new()),
                }
                (true, self.data.short_name.clone())
            }
            _ => (false, String::new()),
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Team(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> { 
        let mut extras = HashMap::new();
        extras.insert("name".to_string(), serde_json::Value::String(self.data.name.clone()));
        extras.insert("primary_color".to_string(), serde_json::Value::String(self.data.primary_color.clone()));
        extras.insert("secondary_color".to_string(), serde_json::Value::from(self.data.secondary_color.clone()));
        extras
    }
}

