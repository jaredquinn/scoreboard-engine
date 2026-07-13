// Scoreboard Engine
// Copyright 2025-2026, Jared Quinn
// 
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation...
// You should have received a copy of the GNU Affero General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::{sync::{Arc, RwLock}, time::Duration};
use indexmap::IndexMap;

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse, Html },
    routing::{get, post},
    Json, Router,
};

use axum_extra::response::JavaScript;
use chrono::Local;

use tokio::sync::broadcast;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use clap::Parser;
use std::net::SocketAddr;
use std::convert::Infallible;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

// Import decoupled widget system modules
pub mod widgets;
use widgets::{WidgetValue, UpdatePayload, create_widget};

type JsonValue = serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TICK_FREQ: u64 = 100;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TriggerCondition {
    pub widget_id: String,
    pub operator: String,
    pub value: i64,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TriggerAction {
    pub target_id: String,
    pub action: String,
    pub value: Option<serde_json::Value>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
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
    pub automations: RwLock<Vec<AutomationTrigger>>,
}

fn process_automations(
    data: &mut IndexMap<String, WidgetValue>,
    automations: &mut Vec<AutomationTrigger>,
) -> bool {
    let fresh_flat_context = flatten_state(data);
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

                                let act_id = act.target_id.clone();
                                let act_name = act.action.clone();
                                tokio::spawn(log_event(act_id, format!("{}*", act_name), log_val));
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

// --- PERSISTENCE & LOGGING ---
async fn log_event(widget_id: String, action: String, value: String) {
    let ts_ms = time_format::now_ms().unwrap();
    let timestamp = time_format::strftime_ms_local("%Y-%m-%d %H:%M:%S.{ms}", ts_ms).unwrap();
    let con_line = format!("[{}] ID: {:<18} | {:<10} | Val: {}", timestamp, widget_id, action, value);
    eprintln!("{}", con_line);

    let log_line = format!("[{}] ID: {:<18} | {:<10} | Val: {}\n", timestamp, widget_id, action, value);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("match_log.txt").await {
        let _ = file.write_all(log_line.as_bytes()).await;
    }
}

async fn save_to_disk(data: IndexMap<String, WidgetValue>, path: &str) {
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = tokio::fs::write(path, json).await;
    }
}

fn load_config(path: &str) -> (IndexMap<String, WidgetValue>, String, Vec<AutomationTrigger>) {
    let mut data = IndexMap::new();
    let mut automations = Vec::new();

    eprintln!("📁 Reading Configuration file {}", path);
    let xml_content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("⚠️ Warning: Could not read {}, using empty config.", path);
        "<ScoreboardConfig></ScoreboardConfig>".to_string()
    });

    let doc = match roxmltree::Document::parse(&xml_content) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error parsing XML: {}. Returning defaults.", e);
            return (data, "state_persistence.json".to_string(), Vec::new());
        }
    };

    let root = doc.root_element();
    let save_file = root.children()
        .find(|n| n.has_tag_name("persistence_file"))
        .and_then(|n| n.text())
        .unwrap_or("state_persistence.json")
        .to_string();

    for node in root.descendants().filter(|n| n.has_tag_name("widget")) {
        let id = node.children().find(|n| n.has_tag_name("id")).and_then(|n| n.text()).unwrap_or("unknown").to_string();
        let w_type = node.children().find(|n| n.has_tag_name("type")).and_then(|n| n.text()).unwrap_or("");

        let val = match w_type {
            "Counter" => WidgetValue::Counter(widgets::counter::CounterWidget::from_xml(&node)),
            "Timer" => WidgetValue::Timer(widgets::timer::TimerWidget::from_xml(&node)),
            "Switch" => WidgetValue::Switch(widgets::switch::SwitchWidget::from_xml(&node)),
            "List" => WidgetValue::List(widgets::list::ListWidget::from_xml(&node)),
            "Team" => WidgetValue::Team(widgets::team::TeamWidget::from_xml(&node)),
            "Text" => WidgetValue::Text(widgets::text::TextWidget::from_xml(&node)),
            "Calculation" => WidgetValue::Calculation(widgets::calculation::CalculationWidget::from_xml(&node)),
            "PenaltyShots" => WidgetValue::PenaltyShots(widgets::penalty_shots::PenaltyShotsWidget::from_xml(&node)),
            _ => continue,
        };
        data.insert(id, val);
    }

    for node in root.descendants().filter(|n| n.has_tag_name("trigger")) {
        let cond_node = match node.children().find(|n| n.has_tag_name("condition")) {
            Some(c) => c,
            None => continue,
        };

        let widget_id = cond_node.children().find(|n| n.has_tag_name("widget_id")).and_then(|n| n.text()).unwrap_or("unknown").to_string();
        let operator = cond_node.children().find(|n| n.has_tag_name("operator")).and_then(|n| n.text()).unwrap_or("==").to_string();
        let val_sec = cond_node.children().find(|n| n.has_tag_name("value")).and_then(|n| n.text()?.parse::<f64>().ok()).unwrap_or(0.0);
        let value = (val_sec * 1000.0) as i64;

        let mut actions = Vec::new();
        if let Some(actions_node) = node.children().find(|n| n.has_tag_name("actions")) {
            for act_node in actions_node.children().filter(|n| n.has_tag_name("action")) {
                let target_id = match act_node.children().find(|n| n.has_tag_name("target_id")).and_then(|n| n.text()) {
                    Some(t) => t.to_string(),
                    None => continue,
                };
                let command = match act_node.children().find(|n| n.has_tag_name("action")).and_then(|n| n.text()) {
                    Some(c) => c.to_string(),
                    None => continue,
                };

                let val_text = act_node.children().find(|n| n.has_tag_name("value")).and_then(|n| n.text());
                let val_json = val_text.map(|v| {
                    if let Ok(num) = v.parse::<i64>() {
                        serde_json::Value::Number(num.into())
                    } else if let Ok(num_f) = v.parse::<f64>() {
                        if let Some(n) = serde_json::Number::from_f64(num_f) {
                            serde_json::Value::Number(n)
                        } else {
                            serde_json::Value::String(v.to_string())
                        }
                    } else {
                        serde_json::Value::String(v.to_string())
                    }
                });
                actions.push(TriggerAction { target_id, action: command, value: val_json });
            }
        }

        automations.push(AutomationTrigger {
            condition: TriggerCondition { widget_id, operator, value },
            actions,
            active: true,
            last_value: None,
        });
    }

    tokio::spawn(log_event("core".to_string(), "loadconfig".to_string(), path.to_string()));
    (data, save_file, automations)
}

#[axum::debug_handler]
async fn serve_index() -> Html<&'static str> { Html(include_str!("index.html")) }
#[axum::debug_handler]
async fn serve_js() -> JavaScript<&'static str> { JavaScript(include_str!("scoreboard.js")) }

async fn get_all(State(state): State<Arc<ScoreboardState>>) -> Json<IndexMap<String, WidgetValue>> {
    let data = state.data.read().unwrap();
    let parsed_data: IndexMap<String, WidgetValue> = data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    Json(parsed_data)
}

fn flatten_state(data: &IndexMap<String, WidgetValue>) -> IndexMap<String, JsonValue> {
    let mut flat = IndexMap::new();
    for (id, val) in data.iter() {
        let widget_obj = create_widget(val);
        let v = widget_obj.primary_value();
        flat.insert(id.clone(), v);

        let extra_vals = widget_obj.extra_values();
        for (key, xvalue) in extra_vals {
            let prefixed_key = format!("{}_{}", id.clone(), key);
            flat.insert(prefixed_key, xvalue);
        }
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
            let (success, log_val) = widget_obj.update(payload.clone()); 

            if success {
                *val = widget_obj.to_value();
                let mut automations = state.automations.write().unwrap();
                process_automations(&mut data, &mut automations);
                let final_data_snapshot = data.clone();
                (true, log_val, final_data_snapshot)
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

        let is_reset = match &payload {
            UpdatePayload::Action { action, .. } => action == "reset",
            _ => false,
        };

        if is_reset {
            let mut automations = state.automations.write().unwrap();
            for trigger in automations.iter_mut() {
                if trigger.condition.widget_id.starts_with(&id_c) {
                    trigger.last_value = None;
                }
            }
        }

        tokio::spawn(async move {
            log_event(id_c, "UPDATE".into(), lv_c).await;
            save_to_disk(dt_c, &path_clone).await;
        });
        let _ = state.tx.send(current_data); 
    }
    Json(success)
}

async fn reset_all(State(state): State<Arc<ScoreboardState>>) -> Json<bool> {
    let (new_widgets, new_path, new_automations) = load_config(&state.config_path);
    {
        let mut data = state.data.write().unwrap_or_else(|e| e.into_inner());
        *data = new_widgets.clone();
        let mut path = state.save_path.write().unwrap_or_else(|e| e.into_inner());
        *path = new_path.clone();
        let mut automations = state.automations.write().unwrap_or_else(|e| e.into_inner());
        *automations = new_automations;
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
            let filtered_data: IndexMap<String, WidgetValue> = data.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
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

#[tokio::main]
async fn main() {
    eprintln!("⭐ Scoreboard Engine {}", VERSION);
    println!("");

    let args = Args::parse();
    let (xml_widgets, persistence_path, xml_automations) = load_config(&args.config);

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
        automations: RwLock::new(xml_automations),
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "tower_http=debug,axum::rejection=trace".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let timer_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_millis(TICK_FREQ));
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

                let mut automations = timer_state.automations.write().unwrap();
                if process_automations(&mut data, &mut automations) {
                    changed = true;
                }

                if changed { snapshot = data.clone(); }
            }

            if changed {
                let _ = timer_state.tx.send(snapshot.clone());
                let current_path = timer_state.save_path.read().unwrap().clone();
                tokio::spawn(async move { save_to_disk(snapshot, &current_path).await; });
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
        .nest_service("/pages", axum::routing::get_service(ServeDir::new("pages")))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🏃 Running HTTP Server. Press Ctrl-C to shutdown.");
    println!("📂 Serving static content from ./pages folder");
    print_listening_urls(args.port);
    tokio::spawn(log_event("core".to_string(), "startup".to_string(), "".to_string()));
    axum::serve(listener, app).await.unwrap();
}
