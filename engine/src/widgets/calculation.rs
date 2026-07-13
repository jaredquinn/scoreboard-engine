use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use evalexpr::{eval_with_context, HashMapContext};
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CalculationData {
    pub value: String,
    pub expression: String,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct CalculationWidget {
    data: CalculationData,
}

impl CalculationWidget {
    pub fn new(data: CalculationData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> CalculationData {
        let initial = node.children().find(|n| n.has_tag_name("initial_value")).and_then(|n| n.text()).unwrap_or("").to_string();
        let expression = node.children().find(|n| n.has_tag_name("expression")).and_then(|n| n.text()).unwrap_or("").to_string();
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        CalculationData { value: initial, expression, dashboard_ui }
    }
}

impl Widget for CalculationWidget {
    fn primary_value(&self) -> serde_json::Value { serde_json::Value::from(self.data.value.clone()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Calculation(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> { HashMap::new() }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    self.data.expression = val_str.to_string();
                    (true, self.data.expression.clone())
                } else {
                    (false, String::new())
                }
            },
            _ => (false, String::new()),
        }
    }

    fn tick(&mut self, flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) {
        if self.data.expression.is_empty() { return (false, String::new()); }

        let mut context = HashMapContext::<evalexpr::DefaultNumericTypes>::new();
        for (key, val) in flat_context.iter() {
            let var_name: String = key.clone();
            if let Some(i) = val.as_i64() {
                let _ = evalexpr::ContextWithMutableVariables::set_value(&mut context, var_name, evalexpr::Value::Int(i));
            } else if let Some(f) = val.as_f64() {
                let _ = evalexpr::ContextWithMutableVariables::set_value(&mut context, var_name, evalexpr::Value::Float(f));
            } else if let Some(b) = val.as_bool() {
                let _ = evalexpr::ContextWithMutableVariables::set_value(&mut context, var_name, evalexpr::Value::Boolean(b));
            } else if let Some(s) = val.as_str() {
                let _ = evalexpr::ContextWithMutableVariables::set_value(&mut context, var_name, evalexpr::Value::String(s.to_string()));
            }
        }

        match eval_with_context(&self.data.expression, &context) {
            Ok(eval_val) => {
                let new_value: String = match eval_val {
                    evalexpr::Value::String(s) => s,
                    evalexpr::Value::Float(f) => f.to_string(),
                    evalexpr::Value::Int(i) => i.to_string(),
                    evalexpr::Value::Boolean(b) => b.to_string(),
                    _ => return (false, String::new()),
                };

                if new_value != self.data.value {
                    self.data.value = new_value;
                    (true, self.data.value.clone())
                } else {
                    (false, String::new())
                }
            }
            Err(e) => {
                eprintln!("Calculation error evaluating '{}': {:?}", self.data.expression, e);
                (false, String::new())
            }
        }
    }
}

