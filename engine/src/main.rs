// Simple Rust Scoreboard Server 
// Copyright 2025, Jared Quinn
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::{collections::HashMap, sync::{Arc, RwLock}, time::Duration};

use axum::{
    extract::{Path, State},
    response::{sse::Event, Sse},
    routing::{get, post},
    Json, Router,
};
use ax_res::Html;

use chrono::Local;
use serde::{Deserialize, Serialize};

use tokio::sync::broadcast;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ax_res {
    pub use axum::response::Html;
}

// --- DATA STRUCTURES ---
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(tag = "type", content = "data")]
pub enum WidgetValue {
    Counter { 
        value: i64, 
        increments: Vec<i64> 
    },
    Timer { 
        seconds: i64, 
        running: bool, 
        initial_seconds: i64,
        is_down: bool,
        min_value: i64,
        max_value: i64,
    },
    MappedList(usize, Vec<String>),
    StaticText(String),
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum UpdatePayload {
    Action { action: String, amount: Option<i64>, value: Option<i64> },
    Value(serde_json::Value),
}

pub struct ScoreboardState {
    pub data: RwLock<HashMap<String, WidgetValue>>,
    pub tx: broadcast::Sender<HashMap<String, WidgetValue>>,
    pub save_path: RwLock<String>,
}


// --- PERSISTENCE & LOGGING ---
async fn log_event(widget_id: String, action: String, value: String) {
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let log_line = format!("[{}] ID: {:<12} | Action: {:<10} | Val: {}\n", timestamp, widget_id, action, value);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("match_log.txt").await {
        eprintln!("{}", log_line);
        let _ = file.write_all(log_line.as_bytes()).await;
    }
}

async fn save_to_disk(data: HashMap<String, WidgetValue>, path: &str) {
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = tokio::fs::write(path, json).await;
    }
}


fn load_config(path: &str) -> (HashMap<String, WidgetValue>, String) {
    let mut data = HashMap::new();

    eprintln!("⚙️ Reading Configration file {}", path);
    let xml_content = std::fs::read_to_string(path).unwrap_or_else(|_| {
        eprintln!("Warning: Could not read {}, using empty config.", path);
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
                    increments: final_increments 
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

                WidgetValue::Timer {
                    seconds: secs,
                    initial_seconds: secs, // Store the reset point
                    running: false,
                    is_down: down,
                    min_value: min,
                    max_value: max
                }
            }
            "MappedList" => {
                let options: Vec<String> = node.descendants()
                    .filter(|n| n.has_tag_name("option"))
                    .filter_map(|n| n.text())
                    .map(|s| s.to_string())
                    .collect();
                WidgetValue::MappedList(0, options)
            }
            "StaticText" => {
                let content = node.children()
                    .find(|n| n.has_tag_name("content"))
                    .and_then(|n| n.text())
                    .unwrap_or("")
                    .to_string();
                WidgetValue::StaticText(content)
            }
            _ => continue, // Skip unknown types
        };

        eprintln!("⚙️ Setting up widget {}:{}", w_type, id);
        data.insert(id, val);
    }

    (data, save_file)
}

// --- HANDLERS ---
async fn serve_index() -> Html<&'static str> {
    Html(include_str!("index.html"))
}

// The handler function
async fn get_all(
    State(state): State<Arc<ScoreboardState>>
) -> Json<HashMap<String, WidgetValue>> {
    let data = state.data.read().unwrap_or_else(|e| e.into_inner());
    Json(data.clone())
}

async fn get_flat(State(state): State<Arc<ScoreboardState>>) -> Json<HashMap<String, serde_json::Value>> {
    let data = state.data.read().unwrap();
    let mut flat = HashMap::new();
    for (id, val) in data.iter() {
        let json_val = match val {
            WidgetValue::Counter { value, .. } => serde_json::Value::from(*value),
            WidgetValue::Timer { seconds, .. } => serde_json::Value::from(*seconds),
            WidgetValue::StaticText(s) => serde_json::Value::String(s.clone()),
            WidgetValue::MappedList(i, opt) => serde_json::Value::String(opt.get(*i).cloned().unwrap_or_default()),
        };
        flat.insert(id.clone(), json_val);
    }
    flat.insert("_last_updated".into(), serde_json::Value::String(Local::now().format("%H:%M:%S").to_string()));
    Json(flat)
}

#[axum::debug_handler]
async fn universal_update(
    Path(id): Path<String>,
    State(state): State<Arc<ScoreboardState>>,
    Json(payload): Json<UpdatePayload>,
) -> Json<bool> {
    let (success, log_val, current_data) = {
        let mut data = state.data.write().unwrap();
        let mut success = false;
        let mut log_val = String::new();

        if let Some(widget) = data.get_mut(&id) {
            match (widget, payload) {
                (WidgetValue::Counter { value, .. }, UpdatePayload::Action { action, amount, .. }) => {
                    let d = amount.unwrap_or(1);
                    if action == "increment" { *value += d; success = true; }
                    else if action == "decrement" { *value -= d; success = true; }
                    log_val = value.to_string();
                }
                (WidgetValue::Timer { seconds, running, min_value, max_value, initial_seconds, .. },
                 UpdatePayload::Action { action, amount, value }) => {

                    let opt = value.or(amount).unwrap_or(0);
                    match action.as_str() {
                            "start" => { *running = true; success = true; }
                            "stop" => { *running = false; success = true; }
                            "reset" => { *seconds = *initial_seconds; *running = false; success = true; }
                            "set_initial" => { *initial_seconds = opt; *seconds = opt; success = true; }
                            "set_min" => { *min_value = opt; success = true; }
                            "set_max" => { *max_value = opt; success = true; }
                            "increment" => { *seconds += amount.unwrap_or(1); success = true; }
                            "decrement" => { *seconds -= amount.unwrap_or(1); success = true; }
                            _ => {}
                    }
                    log_val = format!("Action: {}, Current: {}", action, seconds);
                }
                (WidgetValue::StaticText(s), UpdatePayload::Value(serde_json::Value::String(new_val))) => {
                    *s = new_val.clone(); success = true; log_val = s.clone();
                }
                (WidgetValue::MappedList(idx, options), UpdatePayload::Action { action, .. }) => {
                    match action.as_str() {
                        "next" => { *idx = (*idx + 1) % options.len(); success = true; }
                        "prev" => { *idx = if *idx == 0 { options.len() - 1 } else { *idx - 1 }; success = true; }
                        "reset" => { *idx = 0; success = true; } // THE NEW LINE
                        _ => {}
                    }
                    log_val = options.get(*idx).cloned().unwrap_or_default();
                }
                _ => {}
            }
        }
        (success, log_val, data.clone())
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
    let (new_widgets, new_path) = load_config("config.xml");
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



async fn sse_handler(
    State(state): State<Arc<ScoreboardState>>
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let mut rx = state.tx.subscribe();
    let stream = async_stream::stream! {
        while let Ok(data) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&data) {
                yield Ok(Event::default().data(json));
            }
        }
    };
    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::new())
}

// --- MAIN ---
#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "tower_http=debug,axum::rejection=trace".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let (xml_widgets, persistence_path) = load_config("config.xml");

    let initial_data = if let Ok(content) = std::fs::read_to_string(&persistence_path) {
        serde_json::from_str(&content).unwrap_or(xml_widgets)
    } else {
        xml_widgets
    };

    let (tx, _rx) = broadcast::channel(16);

    let state = Arc::new(ScoreboardState { 
        data: RwLock::new(initial_data), 
        tx,
        save_path: RwLock::new(persistence_path) 
    });


    let timer_state = Arc::clone(&state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            let mut changed = false;
            let mut snapshot = HashMap::new();
            
            {
                let mut data = timer_state.data.write().unwrap();
                for val in data.values_mut() {
                    //if let WidgetValue::Timer { seconds, running, initial_seconds, is_down, min_value, max_value } = val {
                    if let WidgetValue::Timer { seconds, running, is_down, min_value, max_value, .. } = val {
                        if *running {
                            if *is_down {
                                if *seconds > *min_value { *seconds -= 1; } else { *running = false; }
                            } else {
                                if *seconds < *max_value { *seconds += 1; } else { *running = false; }
                            }
                            changed = true;
                        }
                    }
                }
                if changed { snapshot = data.clone(); }
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
        .route("/widgets", get(get_all))
        .route("/widgets/flat", get(get_flat))
        .route("/reset", post(reset_all))
        .route("/widgets/:id/update", post(universal_update))
        .route("/events", get(sse_handler))
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive())
        .with_state(state);

    eprintln!("🏃 running server...");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

