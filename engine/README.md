# 𖣘  Scoreboard Engine

A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio**. 

Note: This was my 'learn rust in a weekend' project, and while solid and reliable should not be considered production ready.


If you find this useful - buy me a [Ko-Fi](https://ko-fi.com/vk2way).

---

## 🔑 Key Features

* **Single-Binary**: The HTML dashboard is baked directly into the executable`.
* **Disaster Recovery**: Live state is persisted on every change and tick.
* **Live Audit**: Every action is time-stamped in `match_log.txt` and mirrored to the stdout console.
* **SSE Support**: Get every tick and every change in realtime via HTTP Server Sent Events.

If you are not interested in building from the rust source code; you can follow the quickstart guide
in the top level [README](/README.md).

## 🚀 Future Plans

* Calculated fields
* Event Actions (on timer stop, perform any action)

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

---

## Config Example

See the examples directory for example configurations


## Access the Cockpit

Open [http://localhost:3000](http://localhost:3000) in any browser.

---

