use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::{Widget, WidgetValue, UpdatePayload, default_false, default_frequency};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimerData {
    pub seconds: i64,
    pub paused_time: i64,
    pub paused_formatted: String,
    pub total_time: i64,
    pub total_formatted: String,
    pub initial_seconds: i64,
    pub formatted_time: String,
    pub running: bool,
    pub reset_on_start: bool,
    pub paused: bool,
    pub is_down: bool,
    pub min_value: i64,
    pub max_value: i64,
    pub format: String,
    pub dashboard_ui: bool,
    #[serde(default = "default_false")]
    pub allow_additional: bool, 
    pub additional_active: bool,
    pub additional_time: i64,
    pub additional_formatted: String,
    pub additional_total_formatted: String,
    #[serde(default = "default_frequency")]
    pub frequency: i64,
    pub last_system_time: Option<i64>,
    #[serde(skip)]
    pub start_time: Option<chrono::DateTime<chrono::Local>>,
}

pub struct TimerWidget {
    data: TimerData,
}

impl TimerWidget {
    pub fn new(data: TimerData) -> Self {
        Self { data }
    }

    pub fn from_xml(node: &roxmltree::Node) -> TimerData {
        let secs = node.children().find(|n| n.has_tag_name("initial_seconds")).and_then(|n| n.text()?.parse::<f64>().ok()).unwrap_or(0.0);
        let down = node.children().find(|n| n.has_tag_name("is_down")).and_then(|n| n.text()?.parse().ok()).unwrap_or(true);
        let min = node.children().find(|n| n.has_tag_name("min_value")).and_then(|n| n.text()?.parse::<f64>().ok()).unwrap_or(0.0);
        let max = node.children().find(|n| n.has_tag_name("max_value")).and_then(|n| n.text()?.parse::<f64>().ok()).unwrap_or(3600.0);
        let fmt = node.children().find(|n| n.has_tag_name("format")).and_then(|n| n.text()).unwrap_or("mm:ss").to_string();
        let ros = node.children().find(|n| n.has_tag_name("reset_on_start")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() == "true").unwrap_or(false);
        let frequency = node.children().find(|n| n.has_tag_name("frequency")).and_then(|n| n.text()?.parse::<i64>().ok()).unwrap_or(100);
        let allow_additional = node.children().find(|n| n.has_tag_name("allow_additional")).and_then(|n| n.text()).and_then(|t| t.parse::<bool>().ok()).unwrap_or(false);
        let dashboard_ui = node.children().find(|n| n.has_tag_name("dashboard-ui")).and_then(|n| n.text()).map(|t| t.trim().to_lowercase() != "false").unwrap_or(true);

        let secs_ms = (secs * 1000.0) as i64;
        let min_ms = (min * 1000.0) as i64;
        let max_ms = (max * 1000.0) as i64;

        TimerData {
            seconds: secs_ms,
            initial_seconds: secs_ms,
            paused_time: 0,
            paused_formatted: format_timer(0, &fmt),
            total_time: 0,
            total_formatted: format_timer(0, &fmt),
            formatted_time: format_timer(secs_ms, &fmt),
            reset_on_start: ros,
            running: false,
            paused: false,
            is_down: down,
            min_value: min_ms,
            max_value: max_ms,
            dashboard_ui,
            allow_additional,
            additional_active: false,
            additional_time: 0,
            additional_formatted: format_timer(0, &fmt),
            additional_total_formatted: format_timer(0, &fmt),
            format: fmt,
            frequency,
            last_system_time: None,
            start_time: None,
        }
    }
}

impl Widget for TimerWidget {
    fn primary_value(&self) -> serde_json::Value {
        serde_json::Value::from((self.data.seconds as f64) / 1000.0)
    }

    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value, .. } => {
                match action.as_str() {
                    "start" => {
                        if self.data.reset_on_start {
                            self.data.seconds = self.data.initial_seconds;
                            self.data.formatted_time = format_timer(self.data.seconds, &self.data.format);
                            self.data.paused_time = 0;
                            self.data.paused_formatted = format_timer(self.data.paused_time, &self.data.format);
                            self.data.additional_time = 0;
                            self.data.additional_formatted = format_timer(0, &self.data.format);
                            self.data.additional_total_formatted = format_timer(0, &self.data.format);
                        }
                        self.data.paused = false;
                        self.data.running = true;
                        self.data.start_time = Some(chrono::Local::now());
                        self.data.last_system_time = Some(chrono::Local::now().timestamp_millis());
                    },
                    "set_direction" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            self.data.is_down = val_str == "DOWN";
                        }
                    },
                    "set_max" => { if let Some(parsed_ms) = parse_value_to_ms(&value) { self.data.max_value = parsed_ms; } },
                    "set_min" => { if let Some(parsed_ms) = parse_value_to_ms(&value) { self.data.min_value = parsed_ms; } },
                    "set_initial" => { if let Some(parsed_ms) = parse_value_to_ms(&value) { self.data.initial_seconds = parsed_ms; } },
                    "toggle" => {
                        self.data.running = !self.data.running;
                        self.data.start_time = if self.data.running { Some(chrono::Local::now()) } else { None };
                        self.data.last_system_time = Some(chrono::Local::now().timestamp_millis());
                    },
                    "pause" => {
                        self.data.paused = !self.data.paused;
                        self.data.start_time = Some(chrono::Local::now());
                        self.data.last_system_time = Some(chrono::Local::now().timestamp_millis());
                    },
                    "reset" => {
                        self.data.seconds = self.data.initial_seconds;
                        self.data.formatted_time = format_timer(self.data.seconds, &self.data.format);
                        self.data.paused = false;
                        self.data.paused_time = 0;
                        self.data.paused_formatted = format_timer(self.data.paused_time, &self.data.format);
                        self.data.additional_time = 0;
                        self.data.additional_formatted = format_timer(self.data.additional_time, &self.data.format);
                        self.data.additional_total_formatted = format_timer(self.data.additional_time, &self.data.format);
                        self.data.running = false;
                        self.data.start_time = None;
                        self.data.last_system_time = None;
                    },
                    "set" | "set_time" => {
                        if let Some(parsed_ms) = parse_value_to_ms(&value) {
                            self.data.seconds = parsed_ms;
                            self.data.formatted_time = format_timer(self.data.seconds, &self.data.format);
                        }
                    },
                    "set_stoppage" => {
                        if let Some(parsed_ms) = parse_value_to_ms(&value) {
                            self.data.paused_time = parsed_ms;
                            self.data.paused_formatted = format_timer(self.data.paused_time, &self.data.format);
                        }
                    },
                    "set_additional" => {
                        if let Some(parsed_ms) = parse_value_to_ms(&value) {
                            self.data.additional_time = parsed_ms;
                            self.data.additional_formatted = format_timer(self.data.additional_time, &self.data.format);
                            self.data.additional_total_formatted = format_timer(self.data.additional_time + self.data.seconds, &self.data.format);
                        }
                    },
                    _ => return (false, String::new()),
                }
                (true, self.data.formatted_time.clone())
            }
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    if let Some(parsed_ms) = parse_time_string(val_str) {
                        self.data.seconds = parsed_ms;
                        self.data.formatted_time = format_timer(self.data.seconds, &self.data.format);
                        return (true, self.data.formatted_time.clone());
                    }
                }
                (false, String::new())
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, serde_json::Value>) -> (bool, String) {
        if !self.data.running {
            self.data.start_time = None;
            self.data.last_system_time = None;
            return (false, self.data.formatted_time.clone());
        }

        let now = chrono::Local::now();
        let now_ms = now.timestamp_millis();

        let delta_ms = match self.data.start_time {
            Some(last_tick) => (now - last_tick).num_milliseconds(),
            None => {
                let computed_delta = match self.data.last_system_time {
                    Some(persisted_ms) if now_ms > persisted_ms => now_ms - persisted_ms,
                    _ => 0,
                };
                self.data.start_time = Some(now);
                self.data.last_system_time = Some(now_ms);
                if computed_delta > 0 { computed_delta } else { return (true, self.data.formatted_time.clone()); }
            }
        };

        let prev_seconds_bucket = self.data.seconds / self.data.frequency;
        let prev_additional_bucket = self.data.additional_time / self.data.frequency;
        let prev_paused_bucket = self.data.paused_time / self.data.frequency;

        self.data.start_time = Some(now);
        self.data.last_system_time = Some(now_ms);

        if delta_ms <= 0 { return (false, self.data.formatted_time.clone()); }

        if self.data.paused {
            self.data.paused_time += delta_ms;
            self.data.total_time += delta_ms;

            if self.data.paused_time / self.data.frequency == prev_paused_bucket {
                return (false, self.data.formatted_time.clone());
            }

            self.data.paused_formatted = format_timer(self.data.paused_time, &self.data.format);
            self.data.total_formatted = format_timer(self.data.total_time, &self.data.format);
            return (true, format!("PAUSED: {} [Stoppage: {}] [Additional: {}] [Total: {}]", self.data.formatted_time.clone(), self.data.paused_formatted.clone(), self.data.additional_formatted.clone(), self.data.total_formatted.clone()))
        }

        if self.data.is_down {
            if self.data.seconds - delta_ms >= self.data.min_value {
                self.data.seconds -= delta_ms;
            } else {
                self.data.seconds = self.data.min_value;
                self.data.running = false;
                self.data.start_time = None;
                self.data.last_system_time = None;
            }
        } else {
            if self.data.seconds < self.data.max_value {
                self.data.seconds += delta_ms;
                self.data.additional_active = false;

                if self.data.seconds > self.data.max_value {
                    let overflow = self.data.seconds - self.data.max_value;
                    self.data.seconds = self.data.max_value;
                    if self.data.allow_additional {
                        self.data.additional_time += overflow;
                        self.data.additional_active = true;
                    } else {
                        self.data.running = false;
                        self.data.start_time = None;
                        self.data.last_system_time = None;
                    }
                }
            } else if self.data.allow_additional {
                self.data.additional_time += delta_ms;
                self.data.additional_active = true;
            } else {
                self.data.seconds = self.data.max_value;
                self.data.running = false;
                self.data.additional_active = false;
                self.data.start_time = None;
                self.data.last_system_time = None;
            }
        }

        self.data.total_time = self.data.seconds + self.data.paused_time + self.data.additional_time;

        let core_changed = (self.data.seconds / self.data.frequency) != prev_seconds_bucket;
        let additional_changed = (self.data.additional_time / self.data.frequency) != prev_additional_bucket;
        let engine_halted = !self.data.running;

        if !core_changed && !additional_changed && !engine_halted {
            return (false, self.data.formatted_time.clone());
        }

        self.data.formatted_time = format_timer(self.data.seconds, &self.data.format);
        self.data.additional_formatted = format_timer(self.data.additional_time, &self.data.format);
        self.data.additional_total_formatted = format_timer(self.data.additional_time + self.data.seconds, &self.data.format);
        self.data.total_formatted = format_timer(self.data.total_time, &self.data.format);

        (true, format!("RUNNING: {} [Stoppage: {}] [Additional: {}] [Total: {}]", self.data.formatted_time.clone(), self.data.paused_formatted.clone(), self.data.additional_formatted.clone(), self.data.total_formatted.clone()))
    }

    fn is_visible(&self) -> bool { self.data.dashboard_ui }
    fn to_value(&self) -> WidgetValue { WidgetValue::Timer(self.data.clone()) }

    fn extra_values(&self) -> HashMap<String, serde_json::Value> {
        let mut extras = HashMap::new();
        extras.insert("formatted".to_string(), serde_json::Value::String(self.data.formatted_time.clone()));
        extras.insert("additional_time".to_string(), serde_json::Value::from((self.data.additional_time as f64) / 1000.0));
        extras.insert("additional_formatted".to_string(), serde_json::Value::String(self.data.additional_formatted.clone()));
        extras.insert("additional_total_formatted".to_string(), serde_json::Value::String(self.data.additional_total_formatted.clone()));
        extras.insert("additional_active".to_string(), serde_json::Value::from(self.data.additional_active));
        extras.insert("paused_time".to_string(), serde_json::Value::from((self.data.paused_time as f64) / 1000.0));
        extras.insert("paused_formatted".to_string(), serde_json::Value::String(self.data.paused_formatted.clone()));
        extras.insert("total_time".to_string(), serde_json::Value::from((self.data.total_time as f64) / 1000.0));
        extras.insert("total_formatted".to_string(), serde_json::Value::String(self.data.total_formatted.clone()));
        extras.insert("paused".to_string(), serde_json::Value::from(self.data.paused));
        extras.insert("running".to_string(), serde_json::Value::from(self.data.running));
        extras
    }
}

// Keep parsing isolation local to the compilation target unit
fn parse_value_to_ms(value: &Option<serde_json::Value>) -> Option<i64> {
    let val_ref = value.as_ref()?;
    if let Some(s) = val_ref.as_str() {
        let parts: Vec<&str> = s.trim().split('.').collect();
        if parts.is_empty() || parts.len() > 2 { return None; }
        let ms: i64 = if parts.len() == 2 {
            let padded = format!("{:0<3}", parts[1]);
            padded[..3].parse().ok()?
        } else { 0 };

        let time_blocks: Vec<&str> = parts[0].split(':').collect();
        match time_blocks.len() {
            2 => {
                let mm: i64 = time_blocks[0].parse().ok()?;
                let ss: i64 = time_blocks[1].parse().ok()?;
                Some((mm * 60 * 1000) + (ss * 1000) + ms)
            }
            3 => {
                let hh: i64 = time_blocks[0].parse().ok()?;
                let mm: i64 = time_blocks[1].parse().ok()?;
                let ss: i64 = time_blocks[2].parse().ok()?;
                Some((hh * 3600 * 1000) + (mm * 60 * 1000) + (ss * 1000) + ms)
            }
            _ => None,
        }
    } else {
        val_ref.as_f64().map(|val_num| (val_num * 1000.0) as i64)
    }
}

fn parse_time_string(input: &str) -> Option<i64> {
    if let Ok(raw_secs) = input.parse::<f64>() { return Some((raw_secs * 1000.0) as i64); }
    let parts: Vec<&str> = input.split(':').collect();
    match parts.len() {
        2 => {
            let m = parts[0].parse::<i64>().ok()?;
            let s = parts[1].parse::<i64>().ok()?;
            Some(((m * 60) + s) * 1000)
        }
        3 => {
            let h = parts[0].parse::<i64>().ok()?;
            let m = parts[1].parse::<i64>().ok()?;
            let s = parts[2].parse::<i64>().ok()?;
            Some(((h * 3600) + (m * 60) + s) * 1000)
        }
        _ => None,
    }
}

fn format_timer(total_ms: i64, format: &str) -> String {
    let sign = if total_ms < 0 { "-" } else { "" };
    let abs_ms = total_ms.abs();
    let total_secs = abs_ms / 1000;
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    let tenths = (abs_ms % 1000) / 100;

    match format {
        "hh:mm:ss" => format!("{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds),
        "m:ss" => format!("{}{}:{:02}", sign, (hours * 60) + minutes, seconds),
        "s.auto" => if total_secs < 5 { format!("{}{}.{}", sign, total_secs, tenths) } else { format!("{}{}", sign, total_secs) },
        "s" => format!("{}{}", sign, total_secs),
        "s.ms" => format!("{}{}.{:01}", sign, total_secs, tenths),
        _ => format!("{}{:02}:{:02}", sign, (hours * 60) + minutes, seconds),
    }
}
