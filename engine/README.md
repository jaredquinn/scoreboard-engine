# 𖣘  Scoreboard Engine v0.2

A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio**. 

Note: This was my 'learn rust in a weekend' project, and while solid and reliable should not be considered production ready.
---

## 🚀 Key Features

* **Single-Binary**: The HTML dashboard is baked directly into the executable`.
* **Disaster Recovery**: Live state is persisted on every change and tick.
* **Live Audit**: Every action is time-stamped in `match_log.txt` and mirrored to the stdout console.
* **SSE Support**: Get every tick and every change in realtime via HTTP Server Sent Events.

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
## API Functions

| Endpoint | Method | Payload Example | Functionality |
| :--- | :--- | :--- | :--- |
| `/events` | `GET` | *None* | Opens the **SSE (Server-Sent Events)** stream for real-time state broadcasting. |
| `/reset` | `POST` | *None* | Reloads specified `config.xml`, clears persistence, and broadcasts the **fresh state**. |
| `/widgets` | `GET` | *None* | Returns **all current widget states** as a JSON object.  Contains all widgets, values and metadata.|
| `/widgets/flat` | `GET` | *None* | Returns **key and value only** as a JSON object for each widget.|
| `/widgets/:id/update` | `POST` | `{"action": "increment", "amount": 1}` | **Counter**: Increases/decreases value by `amount` (use negative numbers for dec). |
| `/widgets/:id/update` | `POST` | `{"action": "start"}` / `{"action": "stop"}` | **Timer**: Toggles the background ticker for the specified ID. |
| `/widgets/:id/update` | `POST` | `{"action": "reset"}` | **Timer**: Reverts to `initial_seconds`. **List**: Resets index to `0`. |
| `/widgets/:id/update` | `POST` | `{"action": "set_initial", "value": 600}` | **Timer**: Updates both current `seconds` and the `initial_seconds` reset point. |
| `/widgets/:id/update` | `POST` | `{"action": "set_min", "value": 0}` | **Timer**: Updates the minimum bound (Buzzer floor). |
| `/widgets/:id/update` | `POST` | `{"action": "set_max", "value": 3600}` | **Timer**: Updates the maximum bound (Ceiling). |
| `/widgets/:id/update` | `POST` | `{"action": "next"}` / `{"action": "prev"}` | **MappedList**: Navigates the options array. |
| `/widgets/:id/update` | `POST` | `"New Text Content"` | **StaticText**: Directly replaces the string value (Raw JSON string). |

## Config Example

Here is an example configuration file showing all available widgets and options

```xml
<ScoreboardConfig>
    <persistence_file>match_data_final.json</persistence_file>
    <widget>
        <id>home_score</id>
        <type>Counter</type>
        <initial_value>0</initial_value>
        <increments>
            <value>4</value>
            <value>2</value>
            <value>1</value>
        </increments>
    </widget>

    <widget>
        <id>away_score</id>
        <type>Counter</type>
        <initial_value>0</initial_value>
        <increments>
            <value>4</value>
            <value>2</value>
            <value>1</value>
        </increments>
    </widget>

    <widget>
        <id>match_clock</id>
        <type>Timer</type>
        <initial_seconds>0</initial_seconds>
        <is_down>false</is_down>
        <min_value>0</min_value>
        <max_value>3600</max_value>
    </widget>

    <widget>
        <id>period</id>
        <type>MappedList</type>
        <initial_index>0</initial_index>
        <options>
            <option>1ST HALF</option>
            <option>HALFTIME</option>
            <option>2ND HALF</option>
            <option>FULLTIME</option>
        </options>
    </widget>

    <widget>
        <id>tackle_count</id>
        <type>MappedList</type>
        <initial_index>0</initial_index>
        <options>
            <option>ZERO</option>
            <option>1ST</option>
            <option>2ND</option>
            <option>3RD</option>
            <option>4TH</option>
            <option>5TH</option>
        </options>
    </widget>

    <widget>
        <id>home_team_name</id>
        <type>StaticText</type>
        <content>HOME TEAM</content>
    </widget>

    <widget>
        <id>away_team_name</id>
        <type>StaticText</type>
        <content>AWAY TEAM</content>
    </widget>
</ScoreboardConfig>
```

## Access the Cockpit

Open [http://localhost:3000](http://localhost:3000) in any browser.

---

