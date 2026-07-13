use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_true};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ShotResult {
    Untaken,
    Scored,
    Missed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PenaltyShotsData {
    pub shots: Vec<ShotResult>,
    pub current_round: usize,
    #[serde(rename = "dashboard-ui", default = "default_true")]
    pub dashboard_ui: bool,
}

pub struct PenaltyShotsWidget {
    data: PenaltyShotsData,
}

impl PenaltyShotsWidget {
    pub fn new(data: PenaltyShotsData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> PenaltyShotsData {
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);
        PenaltyShotsData { shots: vec![ShotResult::Untaken; 5], current_round: 0, dashboard_ui }
    }
}

impl Widget for PenaltyShotsWidget {
    fn primary_value(&self) -> serde_json::Value {
        let score = self.data.shots.iter().filter(|&s| *s == ShotResult::Scored).count();
        serde_json::Value::from(score)
    }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value } => {
                match action.as_str() {
                    "record_shot" => {
                        let result_str = value
                            .and_then(|v| v.as_str().map(String::from))
                            .unwrap_or_else(|| "missed".to_string());

                        let result = match result_str.as_str() {
                            "scored" => ShotResult::Scored,
                            _ => ShotResult::Missed,
                        };

                        if self.data.current_round >= self.data.shots.len() {
                            self.data.shots.push(ShotResult::Untaken);
                        }

                        self.data.shots[self.data.current_round] = result;
                        self.data.current_round += 1;
                        
                        let current_score = self.data.shots.iter().filter(|&&s| s == ShotResult::Scored).count();
                        (true, format!("Shot recorded: {}. Total Score: {}", result_str, current_score))
                    }
                    "clear_last" => {
                        if self.data.current_round > 0 {
                            self.data.current_round -= 1;
                            self.data.shots[self.data.current_round] = ShotResult::Untaken;

                            if self.data.shots.len() > 5 && self.data.current_round < self.data.shots.len() - 1 {
                                self.data.shots.pop();
                            }
                            (true, format!("Reverted to shot index {}", self.data.current_round))
                        } else {
                            (false, "No shots taken to clear".to_string())
                        }
                    }
                    "reset" => {
                        self.data.shots = vec![ShotResult::Untaken; 5];
                        self.data.current_round = 0;
                        (true, "Reset".to_string())
                    }
                    _ => (false, String::new()),
                }
            }
            UpdatePayload::Value(v) => {
                if let Ok(new_shots) = serde_json::from_value::<Vec<ShotResult>>(v) {
                    self.data.shots = new_shots;
                    self.data.current_round = self.data.shots.iter().position(|s| *s == ShotResult::Untaken).unwrap_or(self.data.shots.len());
                    (true, "Shots array directly updated".to_string())
                } else {
                    (false, String::new())
                }
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) { (false, String::new()) }
    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::PenaltyShots(self.data.clone()) }
    fn extra_values(&self) -> HashMap<String, serde_json::Value> {
        let mut extras = HashMap::new();
        let score = self.data.shots.iter().filter(|&s| *s == ShotResult::Scored).count();
        extras.insert("score".to_string(), serde_json::Value::from(score));
        extras.insert("current_round".to_string(), serde_json::Value::from(self.data.current_round));
        extras
    }
}

