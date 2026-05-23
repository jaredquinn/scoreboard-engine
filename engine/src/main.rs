// Scoreboard Engine
// Copyright 2025-2026, Jared Quinn
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation...
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{sync::{Arc, RwLock}, time::Duration};
use indexmap::IndexMap;

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse, Html },
    routing::{get, post, get_service},
    Json, Router,
};

use axum_extra::response::JavaScript;

use chrono::Local;
use serde::{Deserialize, Serialize};

use tokio::sync::broadcast;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use clap::Parser;
use std::net::SocketAddr;
use std::convert::Infallible;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Renamed to avoid name clashes with evalexpr::Value
type JsonValue = serde_json::Value;
use evalexpr::{eval_with_context, HashMapContext, };

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WidgetValue {
    Counter {
        value: i64,
        increments: Vec<i64>,
        #[serde(rename = "dashboard-ui", default = "default_true")]
        dashboard_ui: bool,
    },
    Timer {
        formatted_time: String,
        seconds: i64,
        running: bool,
        paused: bool,
        initial_seconds: i64,
        paused_time: i64,
        is_down: bool,
        min_value: i64,
        max_value: i64,
        format: String,
        #[serde(rename = "dashboard-ui", default = "default_true")]
        dashboard_ui: bool,
    },
    List {
        index: usize,
        options: Vec<String>,
        #[serde(rename = "dashboard-ui", default = "default_true")]
        dashboard_ui: bool,
    },
    Text {
        content: String,
        #[serde(rename = "dashboard-ui", default = "default_true")]
        dashboard_ui: bool,
    },
    Calculation{
        value: String,
        expression: String,
        #[serde(rename = "dashboard-ui", default = "default_true")]
        dashboard_ui: bool,
    },
}

// Helper function to handle serde defaults to true
fn default_true() -> bool {
    true
}

// Trait defining the shared behaviors for widgets
pub trait Widget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String);
    fn tick(&mut self, flat_context: &IndexMap<String, JsonValue>) -> (bool, String);
    fn to_value(&self) -> WidgetValue;
    fn is_visible(&self) -> bool;
}

// Counter
pub struct CounterWidget {
    pub value: i64,
    pub increments: Vec<i64>,
    pub dashboard_ui: bool,
}

impl Widget for CounterWidget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, amount, .. } => {
                let amt = amount.unwrap_or(1);
                match action.as_str() {
                    "increment" => self.value += amt,
                    "decrement" => self.value -= amt,
                    "reset" => self.value = self.increments.first().cloned().unwrap_or(0),
                    _ => return (false, String::new()),
                }
                (true, self.value.to_string())
            }
            UpdatePayload::Value(v) => {
                if let Some(new_val) = v.as_i64() {
                    self.value = new_val;
                    (true, self.value.to_string())
                } else {
                    (false, String::new())
                }
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, JsonValue>) -> (bool, String) {
        (false, String::new())
    }


    fn is_visible(&self) -> bool {
        self.dashboard_ui
    }

    fn to_value(&self) -> WidgetValue {
        WidgetValue::Counter {
            value: self.value,
            increments: self.increments.clone(),
            dashboard_ui: self.dashboard_ui,
        }
    }
}

// Timer
pub struct TimerWidget {
    pub seconds: i64,
    pub paused_time: i64,
    pub initial_seconds: i64,
    pub formatted_time: String,
    pub running: bool,
    pub paused: bool,
    pub is_down: bool,
    pub min_value: i64,
    pub max_value: i64,
    pub format: String,
    pub dashboard_ui: bool,
}

impl Widget for TimerWidget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, value, .. } => {
                match action.as_str() {
                    "start" => self.running = true,
                    "stop" => self.running = false,
                    "toggle" => self.running = !self.running,
                    "pause" => self.paused = !self.paused,
                    "reset" => {
                        self.seconds = self.initial_seconds;
                        self.formatted_time = format_timer(self.seconds, &self.format);
                        self.paused = false;
                        self.paused_time = 0;
                        self.running = false;
                    }
                    "set" => {
                        if let Some(val_str) = value.and_then(|v| v.as_str().map(String::from)) {
                            if let Some(parsed_secs) = parse_time_string(&val_str) {
                                self.seconds = parsed_secs;
                                self.formatted_time = format_timer(self.seconds, &self.format);
                            }
                        }
                    }
                    _ => return (false, String::new()),
                }
                (true, self.formatted_time.clone())
            }
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    if let Some(parsed_secs) = parse_time_string(val_str) {
                        self.seconds = parsed_secs;
                        self.formatted_time = format_timer(self.seconds, &self.format);
                        return (true, self.formatted_time.clone());
                    }
                }
                (false, String::new())
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, JsonValue>) -> (bool, String) {
        if self.running {
            if self.paused {
                self.paused_time += 1;
            } else {
                if self.is_down {
                    if self.seconds > self.min_value {
                        self.seconds -= 1;
                    } else {
                        self.running = false;
                    }
                } else {
                    if self.seconds < self.max_value {
                        self.seconds += 1;
                    } else {
                        self.running = false;
                    }
                }
                self.formatted_time = format_timer(self.seconds, &self.format);
            }
            (true, self.formatted_time.clone())
        } else {
            (false, String::new())
        }
    }

    fn is_visible(&self) -> bool {
        self.dashboard_ui
    }

    fn to_value(&self) -> WidgetValue {
        WidgetValue::Timer {
            formatted_time: self.formatted_time.clone(),
            seconds: self.seconds,
            running: self.running,
            paused: self.paused,
            paused_time: self.paused_time,
            initial_seconds: self.initial_seconds,
            is_down: self.is_down,
            min_value: self.min_value,
            max_value: self.max_value,
            format: self.format.clone(),
            dashboard_ui: self.dashboard_ui,
        }
    }
}

// List
pub struct ListWidget {
    pub index: usize,
    pub options: Vec<String>,
    pub dashboard_ui: bool,
}

impl Widget for ListWidget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Action { action, .. } => {
                match action.as_str() {
                    "next" => {
                        if !self.options.is_empty() {
                            self.index = (self.index + 1) % self.options.len();
                        }
                    }
                    "prev" => {
                        if !self.options.is_empty() {
                            self.index = if self.index == 0 { self.options.len() - 1 } else { self.index - 1 };
                        }
                    }
                    "reset" => self.index = 0,
                    _ => return (false, String::new()),
                }
                let log_val = self.options.get(self.index).cloned().unwrap_or_default();
                (true, log_val)
            }
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    if let Some(pos) = self.options.iter().position(|s| s == val_str) {
                        self.index = pos;
                        return (true, val_str.to_string());
                    }
                } else if let Some(idx) = v.as_u64() {
                    if (idx as usize) < self.options.len() {
                        self.index = idx as usize;
                        let log_val = self.options.get(self.index).cloned().unwrap_or_default();
                        return (true, log_val);
                    }
                }
                (false, String::new())
            }
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, JsonValue>) -> (bool, String) {
        (false, String::new())
    }

    fn is_visible(&self) -> bool {
        self.dashboard_ui
    }

    fn to_value(&self) -> WidgetValue {
        WidgetValue::List {
            index: self.index,
            options: self.options.clone(),
            dashboard_ui: self.dashboard_ui,
        }
    }
}

// Text
pub struct TextWidget {
    pub content: String,
    pub dashboard_ui: bool,
}

impl Widget for TextWidget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    self.content = val_str.to_string();
                    (true, self.content.clone())
                } else {
                    (false, String::new())
                }
            }
            _ => (false, String::new()),
        }
    }

    fn tick(&mut self, _flat_context: &IndexMap<String, JsonValue>) -> (bool, String) {
        (false, String::new())
    }

    fn is_visible(&self) -> bool {
        self.dashboard_ui
    }

    fn to_value(&self) -> WidgetValue {
        WidgetValue::Text {
            content: self.content.clone(),
            dashboard_ui: self.dashboard_ui,
        }
    }
}

// Calculation
pub struct CalculationWidget {
    pub value: String,
    pub expression: String,
    pub dashboard_ui: bool,
}

impl Widget for CalculationWidget {
    fn update(&mut self, payload: UpdatePayload) -> (bool, String) {
        match payload {
            UpdatePayload::Value(v) => {
                if let Some(val_str) = v.as_str() {
                    self.expression = val_str.to_string();
                    (true, self.expression.clone())
                } else {
                    (false, String::new())
                }
            }
            _ => (false, String::new()),
        }
    }

    fn tick(&mut self, flat_context: &IndexMap<String, JsonValue>) -> (bool, String) {
        if self.expression.is_empty() {
            return (false, String::new());
        }

        // Build execution context cleanly without trait ambiguity
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

        // Evaluate the mathematical/logical expression
        match eval_with_context(&self.expression, &context) {
            Ok(eval_val) => {
                let new_value: String = match eval_val {
                    evalexpr::Value::String(s) => s,
                    evalexpr::Value::Float(f) => {
                        let f: f64 = f;
                        f.to_string()
                    },
                    evalexpr::Value::Int(i) => {
                        let i: i64 = i;
                        i.to_string()
                    },
                    evalexpr::Value::Boolean(b) => b.to_string(),
                    _ => return (false, String::new()),
                };

                if new_value != self.value {
                    self.value = new_value;
                    (true, self.value.clone())
                } else {
                    (false, String::new())
                }
            }
            Err(e) => {
                eprintln!("Calculation error evaluating '{}': {:?}", self.expression, e);
                (false, String::new())
            }
        }
    }

    fn is_visible(&self) -> bool {
        self.dashboard_ui
    }

    fn to_value(&self) -> WidgetValue {
        WidgetValue::Calculation {
            value: self.value.clone(),
            expression: self.expression.clone(),
            dashboard_ui: self.dashboard_ui,
        }
    }
}

// Helper factory to dynamically instantiate widget from its data representation
fn create_widget(value: &WidgetValue) -> Box<dyn Widget> {
    match value {
        WidgetValue::Counter { value, increments, dashboard_ui } => Box::new(CounterWidget {
            value: *value,
            increments: increments.clone(),
            dashboard_ui: *dashboard_ui,
        }),
        WidgetValue::Timer {
            seconds,
            initial_seconds,
            formatted_time,
            paused_time,
            running,
            paused,
            is_down,
            min_value,
            max_value,
            format,
            dashboard_ui,
        } => Box::new(TimerWidget {
            seconds: *seconds,
            initial_seconds: *initial_seconds,
            formatted_time: formatted_time.clone(),
            paused_time: *paused_time,
            running: *running,
            paused: *paused,
            is_down: *is_down,
            min_value: *min_value,
            max_value: *max_value,
            format: format.clone(),
            dashboard_ui: *dashboard_ui,
        }),
        WidgetValue::List { index, options, dashboard_ui } => Box::new(ListWidget {
            index: *index,
            options: options.clone(),
            dashboard_ui: *dashboard_ui,
        }),
        WidgetValue::Text { content, dashboard_ui } => Box::new(TextWidget {
            content: content.clone(),
            dashboard_ui: *dashboard_ui,
        }),
        WidgetValue::Calculation { value, expression , dashboard_ui } => Box::new(CalculationWidget {
            value: value.clone(),
            expression: expression.clone(),
            dashboard_ui: *dashboard_ui,
        })
    }
}

// --- DATA STRUCTURES ---

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdatePayload {
    Action {
        action: String,
        amount: Option<i64>,
        value: Option<serde_json::Value>,
    },
    Value(serde_json::Value),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.xml")]
    config: String,
    #[arg(short, long, default_value_t = 3000)]
    port: u16,
}

pub struct ScoreboardState {
    pub data: RwLock<IndexMap<String, WidgetValue>>,
    pub tx: broadcast::Sender<IndexMap<String, WidgetValue>>,
    pub save_path: RwLock<String>,
    pub config_path: String,
}

// --- PERSISTENCE & LOGGING ---

async fn log_event(widget_id: String, action: String, value: String) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let con_line = format!("[{}] ID: {:<12} | Action: {:<10} | Val: {}", timestamp, widget_id, action, value);
    eprintln!("{}", con_line);

    let log_line = format!("[{}] ID: {:<12} | Action: {:<10} | Val: {}\n", timestamp, widget_id, action, value);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("match_log.txt").await {
        let _ = file.write_all(log_line.as_bytes()).await;
    }
}

async fn save_to_disk(data: IndexMap<String, WidgetValue>, path: &str) {
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = tokio::fs::write(path, json).await;
    }
}

fn load_config(path: &str) -> (IndexMap<String, WidgetValue>, String) {
    let mut data = IndexMap::new();


    eprintln!("📁 Reading Configuration file {}", path);
    let xml_content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("⚠️ Warning: Could not read {}, using empty config.", path);
        "<ScoreboardConfig></ScoreboardConfig>".to_string()
    });

    let doc = match roxmltree::Document::parse(&xml_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing XML: {}. Returning defaults.", e);
            return (data, "state_persistence.json".to_string());
        }
    };

    let root = doc.root_element();
    let save_file = root.children()
        .find(|n| n.has_tag_name("persistence_file"))
        .and_then(|n| n.text())
        .unwrap_or("state_persistence.json")
        .to_string();

    for node in root.descendants().filter(|n| n.has_tag_name("widget")) {
        let id = node.children()
            .find(|n| n.has_tag_name("id"))
            .and_then(|n| n.text())
            .unwrap_or("unknown")
            .to_string();

        let w_type = node.children()
            .find(|n| n.has_tag_name("type"))
            .and_then(|n| n.text())
            .unwrap_or("");

        let dashboard_ui = node.children()
            .find(|n| n.has_tag_name("dashboard-ui"))
            .and_then(|n| n.text())
            .map(|t| t.trim().to_lowercase() != "false")
            .unwrap_or(true);

        let val = match w_type {
            "Counter" => {
                let initial = node.children()
                    .find(|n| n.has_tag_name("initial_value"))
                    .and_then(|n| n.text()?.parse().ok())
                    .unwrap_or(0);

                let increments: Vec<i64> = node.descendants()
                    .filter(|n| n.has_tag_name("value"))
                    .filter_map(|n| n.text()?.parse().ok())
                    .collect();

                let final_increments = if increments.is_empty() { vec![1] } else { increments };

                WidgetValue::Counter {
                    value: initial,
                    increments: final_increments,
                    dashboard_ui,
                }
            }
            "Timer" => {
                let secs = node.children()
                    .find(|n| n.has_tag_name("initial_seconds"))
                    .and_then(|n| n.text()?.parse().ok())
                    .unwrap_or(0);
                let down = node.children()
                    .find(|n| n.has_tag_name("is_down"))
                    .and_then(|n| n.text()?.parse().ok())
                    .unwrap_or(true);
                let min = node.children()
                    .find(|n| n.has_tag_name("min_value"))
                    .and_then(|n| n.text()?.parse().ok())
                    .unwrap_or(0);
                let max = node.children()
                    .find(|n| n.has_tag_name("max_value"))
                    .and_then(|n| n.text()?.parse().ok())
                    .unwrap_or(3600);

                let fmt = node.children()
                    .find(|n| n.has_tag_name("format"))
                    .and_then(|n| n.text())
                    .unwrap_or("mm:ss")
                    .to_string();

                WidgetValue::Timer {
                    seconds: secs,
                    paused_time: 0,
                    initial_seconds: secs,
                    formatted_time: format_timer(secs, &fmt),
                    running: false,
                    paused: false,
                    is_down: down,
                    min_value: min,
                    max_value: max,
                    format: fmt,
                    dashboard_ui,
                }
            }
            "List" => {
                let options: Vec<String> = node.descendants()
                    .filter(|n| n.has_tag_name("option"))
                    .filter_map(|n| n.text())
                    .map(|s| s.to_string())
                    .collect();
                WidgetValue::List { index: 0, options, dashboard_ui }
            }
            "Text" => {
                let content = node.children()
                    .find(|n| n.has_tag_name("content"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();
                WidgetValue::Text { content, dashboard_ui }
            }
            "Calculation" => {

                let initial = node.children()
                    .find(|n| n.has_tag_name("initial_value"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();

                let expression = node.children()
                    .find(|n| n.has_tag_name("expression"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();

                WidgetValue::Calculation { value: initial, expression, dashboard_ui }
            }
            _ => continue,
        };

        eprintln!("🥅 Setting up widget {}:{} (UI Visible: {})", w_type, id, dashboard_ui);
        data.insert(id, val);
    }
    tokio::spawn(log_event("core".to_string(), "loadconfig".to_string(), path.to_string()));

    (data, save_file)
}

fn parse_time_string(input: &str) -> Option<i64> {
    if let Ok(raw_seconds) = input.parse::<i64>() {
        return Some(raw_seconds);
    }

    let parts: Vec<&str> = input.split(':').collect();
    match parts.len() {
        2 => {
            let m = parts[0].parse::<i64>().ok()?;
            let s = parts[1].parse::<i64>().ok()?;
            Some((m * 60) + s)
        }
        3 => {
            let h = parts[0].parse::<i64>().ok()?;
            let m = parts[1].parse::<i64>().ok()?;
            let s = parts[2].parse::<i64>().ok()?;
            Some((h * 3600) + (m * 60) + s)
        }
        _ => None,
    }
}

fn format_timer(total_seconds: i64, format: &str) -> String {
    let abs_secs = total_seconds.abs();
    let sign = if total_seconds < 0 { "-" } else { "" };

    let hours = abs_secs / 3600;
    let minutes = (abs_secs % 3600) / 60;
    let seconds = abs_secs % 60;

    match format {
        "hh:mm:ss" => format!("{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds),
        "m:ss" => format!("{}{}:{:02}", sign, (hours * 60) + minutes, seconds),
        "s" => format!("{}{}", sign, total_seconds),
        _ => format!("{}{:02}:{:02}", sign, (hours * 60) + minutes, seconds),
    }
}

#[axum::debug_handler]
async fn serve_index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

#[axum::debug_handler]
async fn serve_js() -> JavaScript<&'static str> {
    JavaScript(include_str!("scoreboard.js"))
}

pub async fn get_script() -> JavaScript<&'static str> {
    JavaScript("console.log('Hello from Axum!');")
}

async fn get_all(State(state): State<Arc<ScoreboardState>>) -> Json<IndexMap<String, WidgetValue>> {
    let data = state.data.read().unwrap();
    let parsed_data: IndexMap<String, WidgetValue> = data.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    Json(parsed_data)
}

// Flatten the state
fn flatten_state(data: &IndexMap<String, WidgetValue>) -> IndexMap<String, JsonValue> {
    let mut flat = IndexMap::new();
    for (id, val) in data.iter() {
        let v = match val {
            WidgetValue::Counter { value, .. } => serde_json::Value::from(*value),
            WidgetValue::Timer { seconds, .. } => serde_json::Value::from(*seconds),
            WidgetValue::List { index, options, .. } => {
                let s = options.get(*index).cloned().unwrap_or_default();
                serde_json::Value::from(s)
            }
            WidgetValue::Text { content, .. } => serde_json::Value::from(content.clone()),
            WidgetValue::Calculation { value, .. } => serde_json::Value::from(value.clone()),
        };
        flat.insert(id.clone(), v);


        // These should be moved to a function in the traits for Widgets to return additional
        // values
        let display_val = match val {
            WidgetValue::Timer { formatted_time, .. } => serde_json::Value::String(formatted_time.clone()),
            _ => serde_json::Value::String(String::new()),
        };
        if display_val != serde_json::Value::String(String::new()) {
            flat.insert(format!("{}_formatted", id.clone()), display_val);
        };


        let state_val = match val {
            WidgetValue::Timer { running, .. } => serde_json::Value::from(*running),
            _ => serde_json::Value::String(String::new()),
        };
        if state_val != serde_json::Value::String(String::new()) {
            flat.insert(format!("{}_running", id.clone()), state_val);
        };


    }
    flat.insert("_last_updated".into(), serde_json::Value::String(Local::now().format("%H:%M:%S").to_string()));
    flat
}

fn get_flattened_snapshot(state: &Arc<ScoreboardState>) -> String {
    let data = state.data.read().unwrap();
    let flat = flatten_state(&*data);
    serde_json::to_string(&vec![flat]).unwrap_or_default()
}

async fn get_flat(State(state): State<Arc<ScoreboardState>>) -> Json<Vec<IndexMap<String, serde_json::Value>>> {
    let data = state.data.read().unwrap();
    let flat = flatten_state(&*data);
    Json(vec![flat])
}

async fn universal_update(
    Path(id): Path<String>,
    State(state): State<Arc<ScoreboardState>>,
    Json(payload): Json<UpdatePayload>,
) -> Json<bool> {
    let (success, log_val, current_data) = {
        let mut data = state.data.write().unwrap();
        if let Some(val) = data.get_mut(&id) {
            let mut widget_obj = create_widget(val);
            let (success, log_val) = widget_obj.update(payload);

            if success {
                *val = widget_obj.to_value();
                (true, log_val, data.clone())
            } else {
                (false, String::new(), data.clone())
            }
        } else {
            (false, String::new(), data.clone())
        }
    };

    if success {
        let id_c = id.clone();
        let lv_c = log_val.clone();
        let dt_c = current_data.clone();
        let path_clone = state.save_path.read().unwrap().clone();

        tokio::spawn(async move {
            log_event(id_c, "UPDATE".into(), lv_c).await;
            save_to_disk(dt_c, &path_clone).await;
        });
        let _ = state.tx.send(current_data);
    }
    Json(success)
}

async fn reset_all(State(state): State<Arc<ScoreboardState>>) -> Json<bool> {
    let (new_widgets, new_path) = load_config(&state.config_path);
    {
        let mut data = state.data.write().unwrap_or_else(|e| e.into_inner());
        *data = new_widgets.clone();

        let mut path = state.save_path.write().unwrap_or_else(|e| e.into_inner());
        *path = new_path.clone();
    }
    let _ = state.tx.send(new_widgets.clone());
    let path_to_save = state.save_path.read().unwrap_or_else(|e| e.into_inner()).clone();
    save_to_disk(new_widgets, &path_to_save).await;

    Json(true)
}

#[axum::debug_handler]
async fn web_sse_handler(
    State(state): State<Arc<ScoreboardState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.tx.subscribe();
    let stream_state = Arc::clone(&state);

    let stream = async_stream::stream! {
        let initial_json = get_flattened_snapshot(&stream_state);
        yield Ok::<Event, Infallible>(Event::default().data(initial_json));

        while let Ok(_notification) = rx.recv().await {
            let json = get_flattened_snapshot(&stream_state);
            yield Ok::<Event, Infallible>(Event::default().data(json));
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

#[axum::debug_handler]
async fn full_sse_handler(
    State(state): State<Arc<ScoreboardState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.tx.subscribe();
    let stream = async_stream::stream! {
        while let Ok(data) = rx.recv().await {
            let filtered_data: IndexMap<String, WidgetValue> = data.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            if let Ok(json) = serde_json::to_string(&filtered_data) {
                yield Ok::<Event, Infallible>(Event::default().data(json));
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}

fn print_listening_urls(port: u16) {
    println!("🎯 Scoreboard Engine is live!");
    println!("---------------------------------------");
    println!("Local: http://localhost:{}", port);

    if let Ok(interfaces) = get_if_addrs::get_if_addrs() {
        for interface in interfaces {
            if !interface.is_loopback() {
                if let std::net::IpAddr::V4(ipv4) = interface.ip() {
                    println!("On your network: http://{}:{}", ipv4, port);
                }
            }
        }
    }
    println!("---------------------------------------");
}

// --- MAIN ---
#[tokio::main]
async fn main() {
    eprintln!("⭐ Scoreboard Engine {}", VERSION);
    println!("");

    let args = Args::parse();
    let (xml_widgets, persistence_path) = load_config(&args.config);

    let initial_data = if let Ok(content) = std::fs::read_to_string(&persistence_path) {
        eprintln!("📁 Restoring persistence data from {}", persistence_path);
        serde_json::from_str(&content).unwrap_or(xml_widgets)
    } else {
        xml_widgets
    };
    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));

    let (tx, _rx) = broadcast::channel(16);
    let state = Arc::new(ScoreboardState {
        data: RwLock::new(initial_data),
        tx,
        save_path: RwLock::new(persistence_path),
        config_path: args.config.clone(),
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "tower_http=debug,axum::rejection=trace".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let timer_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;

            let mut changed = false;
            let mut snapshot = IndexMap::new();
            {
                let current_flat_context = {
                    let data_read = timer_state.data.read().unwrap();
                    flatten_state(&*data_read)
                };

                let mut data = timer_state.data.write().unwrap();
                for (id, val) in data.iter_mut() {
                    let mut widget_obj = create_widget(val);
                    let (ticked, display_val) = widget_obj.tick(&current_flat_context);

                    if ticked {
                        *val = widget_obj.to_value();
                        changed = true;

                        let id_clone = id.clone();
                        tokio::spawn(log_event(id_clone, "TICK".to_string(), display_val));
                    }
                }
                if changed {
                    snapshot = data.clone();
                }
            }

            if changed {
                let _ = timer_state.tx.send(snapshot.clone());
                let current_path = timer_state.save_path.read().unwrap().clone();
                tokio::spawn(async move {
                    save_to_disk(snapshot, &current_path).await;
                });
            }
        }
    });

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/scoreboard.js", get(serve_js))
        .route("/widgets", get(get_all))
        .route("/widgets/flat", get(get_flat))
        .route("/reset", post(reset_all))
        .route("/widgets/:id/update", post(universal_update))
        .route("/sse", get(web_sse_handler))
        .route("/events", get(full_sse_handler))
        .nest_service("/pages", get_service(ServeDir::new("pages")))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🏃 Running HTTP Server. Press Ctrl-C to shutdown.");
    println!("📂 Serving static content from ./pages folder");
    print_listening_urls(args.port);
    tokio::spawn(log_event("core".to_string(), "startup".to_string(), "".to_string()));
    axum::serve(listener, app).await.unwrap();
}
