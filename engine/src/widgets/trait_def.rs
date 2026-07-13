use indexmap::IndexMap;
use std::collections::HashMap;

type JsonValue = serde_json::Value;

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum UpdatePayload {
    Action { action: String, value: Option<serde_json::Value> },
    Value(serde_json::Value),
}

pub trait Widget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String);
    fn tick(&mut self, flat_context: &IndexMap<String, JsonValue>) -> (bool, String);
    fn to_value(&self) -> super::WidgetValue;
    fn is_visible(&self) -> bool;
    fn extra_values(&self) -> HashMap<String, serde_json::Value>;
    fn primary_value(&self) -> serde_json::Value;
}

