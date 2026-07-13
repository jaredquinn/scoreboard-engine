use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TextData {
    pub content: String,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct TextWidget {
    data: TextData,
}

impl TextWidget {
    pub fn new(data: TextData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> TextData {
        let content = node.children().find(|n| n.has_tag_name("content")).and_then(|n| n.text()).unwrap_or("").to_string();
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        TextData { content, dashboard_ui }
    }
}

impl Widget for TextWidget {
    fn primary_value(&self) -> serde_json::Value { serde_json::Value::from(self.data.content.clone()) }
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    self.data.content = val_str.to_string();
                    (true, self.data.content.clone())
                } else {
                    (false, String::new())
                }
            }
            _ => (false, String::new()),
        }
    }
    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Text(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> { HashMap::new() }
}

