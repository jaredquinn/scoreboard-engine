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

// Decoupled architecture modules
pub mod widgets;
pub mod automations;

use widgets::{WidgetValue, UpdatePayload, create_widget, load_config};
use automations::{AutomationTrigger, process_automations};


type JsonValue = serde_json::Value;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const TICK_FREQ: u64 = 100;

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

// --- PERSISTENCE & LOGGING ---
async fn log_event(widget_id: String, action: String, value: String) {
    let ts_ms = time_format::now_ms().unwrap();
    let timestamp = time_format::strftime_ms_local("%Y-%m-%d %H:%M:%S.{ms}", ts_ms).unwrap();
    let con_line = format!("[{}] {:<18}|{:<14}| {}", timestamp, widget_id, action, value);
    eprintln!("{}", con_line);

    let log_line = format!("[{}] {:<18}|{:<14}| {}\n", timestamp, widget_id, action, value);
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("match_log.txt").await {
        let _ = file.write_all(log_line.as_bytes()).await;
    }
}

async fn save_to_disk(data: IndexMap<String, WidgetValue>, path: &str) {
    if let Ok(json) = serde_json::to_string_pretty(&data) {
        let _ = tokio::fs::write(path, json).await;
    }
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
    let (action_label, target_value) = match &payload {
        UpdatePayload::Action { action, value } => {
            let val_str = value
                .as_ref()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .unwrap_or_else(|| "None".to_string());
            (action.clone(), val_str)
        }
        UpdatePayload::Value(value) => {
            let val_str = match value {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            ("value_update".to_string(), val_str)
        }
    };

    let (success, current_data) = {
        let mut data = state.data.write().unwrap();
        if let Some(val) = data.get_mut(&id) {
            let mut widget_obj = create_widget(val);
            let (success, _log_val) = widget_obj.update(payload.clone()); 

            if success {
                *val = widget_obj.to_value();
                let mut automations = state.automations.write().unwrap();
                process_automations(
                    &mut data, 
                    &mut automations, 
                    flatten_state, 
                    |act_id, act_name, lv| { tokio::spawn(log_event(act_id, format!("{}*", act_name), lv)); }
                );
                let final_data_snapshot = data.clone();
                (true, final_data_snapshot)
            } else {
                (false, data.clone())
            }
        } else {
            (false, data.clone())
        }
    };

    if success {
        let id_c = id.clone();
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
            log_event(id_c, action_label, target_value).await;
            save_to_disk(dt_c, &path_clone).await;
        });
        let _ = state.tx.send(current_data); 
    }
    Json(success)
}

async fn reset_all(State(state): State<Arc<ScoreboardState>>) -> Json<bool> {
    let (new_widgets, new_path, new_automations) = load_config(&state.config_path, |id, act, val| {
        tokio::spawn(log_event(id, act, val));
    });
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
    let (xml_widgets, persistence_path, xml_automations) = load_config(&args.config, |id, act, val| { tokio::spawn(log_event(id, act, val)); });

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
                        tokio::spawn(log_event(id_clone, "tick".to_string(), display_val));
                    }
                }

                let mut automations = timer_state.automations.write().unwrap();
                if process_automations(
                    &mut data, 
                    &mut automations, 
                    flatten_state, 
                    |act_id, act_name, lv| { tokio::spawn(log_event(act_id, format!("{}*", act_name), lv)); }
                ) {
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
