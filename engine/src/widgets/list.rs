use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListData {
    pub index: usize,
    pub options: Vec<String>,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct ListWidget {
    data: ListData,
}

impl ListWidget {
    pub fn new(data: ListData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> ListData {
        let options: Vec<String> = node.descendants().filter(|n| n.has_tag_name("option")).filter_map(|n| n.text()).map(|s| s.to_string()).collect();
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        ListData { index: 0, options, dashboard_ui }
    }
}

impl Widget for ListWidget {
    fn primary_value(&self) -> serde_json::Value {
        let s = self.data.options.get(self.data.index).cloned().unwrap_or_default();
        serde_json::Value::from(s)
    }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, .. } => {
                match action.as_str() {
                    "next" => {
                        if !self.data.options.is_empty() { self.data.index = (self.data.index + 1) % self.data.options.len(); }
                    }
                    "prev" => {
                        if !self.data.options.is_empty() { self.data.index = if self.data.index == 0 { self.data.options.len() - 1 } else { self.data.index - 1 }; }
                    }
                    "reset" => self.data.index = 0,
                    _ => return (false, String::new()),
                }
                let log_val = self.data.options.get(self.data.index).cloned().unwrap_or_default();
                (true, log_val)
            }
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    if let Some(pos) = self.data.options.iter().position(|s| s == val_str) {
                        self.data.index = pos;
                        return (true, val_str.to_string());
                    }
                } else if let Some(idx) = v.as_u64() {
                    if (idx as usize) < self.data.options.len() {
                        self.data.index = idx as usize;
                        let log_val = self.data.options.get(self.data.index).cloned().unwrap_or_default();
                        return (true, log_val);
                    }
                }
                (false, String::new())
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::List(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> { 
        let mut extras = HashMap::new();
        extras.insert("index".to_string(), serde_json::Value::from(self.data.index));
        extras
    }
}

