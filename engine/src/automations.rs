use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use crate::widgets::{WidgetValue, create_widget, UpdatePayload};

// Helper macro or type alias inherited from core layers
type JsonValue = serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub widget_id: String,
    pub operator: String,
    pub value: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TriggerAction {
    pub target_id: String,
    pub action: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AutomationTrigger {
    pub condition: TriggerCondition,
    pub actions: Vec<TriggerAction>,
    #[serde(default = "default_true")]
    pub active: bool,
    #[serde(skip, default = "default_none_i64")]
    pub last_value: Option<i64>,
}

fn default_true() -> bool { true }
fn default_none_i64() -> Option<i64> { None }

pub fn process_automations(
    data: &mut IndexMap<String, WidgetValue>,
    automations: &mut Vec<AutomationTrigger>,
    flatten_state_fn: fn(&IndexMap<String, WidgetValue>) -> IndexMap<String, JsonValue>,
    log_event_fn: fn(String, String, String),
) -> bool {
    let fresh_flat_context = flatten_state_fn(data);
    let mut automation_triggered = false;
    let mut needs_period_rearm = false;

    for trigger in automations.iter_mut() {
        if let Some(num) = fresh_flat_context.get(&trigger.condition.widget_id).and_then(|v| v.as_f64()) {
            let current_val = (num * 1000.0).round() as i64;
            let target_val = trigger.condition.value;
            let is_first_sample = trigger.last_value.is_none();
            let last_val_unwrapped = trigger.last_value.unwrap_or(0);

            let condition_met = match trigger.condition.operator.as_str() {
                "==" => current_val == target_val,
                ">=" => current_val >= target_val,
                "<=" => current_val <= target_val,
                "++" => !is_first_sample && current_val == target_val && current_val > last_val_unwrapped,
                "--" => !is_first_sample && current_val == target_val && current_val < last_val_unwrapped,
                _ => false,
            };

            let value_changed = is_first_sample || current_val != last_val_unwrapped;

            if condition_met && value_changed {
                if !is_first_sample {
                    let mut target_widgets_to_sync = std::collections::HashSet::new();

                    for act in &trigger.actions {
                        if let Some(val) = data.get_mut(&act.target_id) {
                            let mut widget_obj = create_widget(val);
                            let payload = UpdatePayload::Action {
                                action: act.action.clone(),
                                value: act.value.clone(),
                            };
                            let (success, log_val) = widget_obj.update(payload);
                            if success {
                                *val = widget_obj.to_value();
                                automation_triggered = true;
                                target_widgets_to_sync.insert(act.target_id.clone());

                                // Fire off tracking log through the core function pointer pass
                                log_event_fn(act.target_id.clone(), format!("{}*", act.action), log_val);
                            }
                        }
                    }

                    for target_id in target_widgets_to_sync {
                        if let Some(val) = data.get_mut(&target_id) {
                            let mut widget_obj = create_widget(val);
                            let _ = widget_obj.tick(&fresh_flat_context);
                            *val = widget_obj.to_value();
                        }
                    }

                    if trigger.condition.widget_id == "match_period_index" {
                        needs_period_rearm = true;
                    }
                }
            }
            trigger.last_value = Some(current_val);
        }
    }

    if needs_period_rearm {
        for t in automations.iter_mut() {
            if t.condition.widget_id == "match_clock" {
                t.last_value = None; 
            }
        }
    }

    automation_triggered
}

