# 🏟️ Scoreboard Engine v0.2

A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio**. Designed for live broadcast environments where reliability is critical.

---

## 🚀 Key Features

* **Single-Binary**: The HTML dashboard is baked directly into the executable`.
* **Disaster Recovery**: Live state is persisted on every change and tick.
* **Live Audit**: Every action is time-stamped in `match_log.txt` and mirrored to the stdout console.


---
## 🛠️ Build from Source

**Dependencies**:

* rustc
* cargo


```bash
cargo build --release
```

## Starting a Scoreboard 

Copy the `config.xml` file and create a configuration for your match.

Starting the scoreboard from the compiled source code:

```bash
./target/release/scoreboard-engine --config config.xml
```

## Access the Cockpit

Open [http://localhost:3000](http://localhost:3000) in any browser.

---

