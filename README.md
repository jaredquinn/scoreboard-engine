
# 🎯 Scoreboard Engine 

## What is Scoreboard Engine?

* 𖣘  `engine` - A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio** with integrated web dashboard.
* 🦾 `companion-module-scoreboard-engine` - A module for BitFocus Companion supporting all features.

<img alt="Dashboard Screenshot" src="https://raw.github.com/jaredquinn/scoreboard-engine/pages/screenshots/readme-dashboard.png" />

# Quick Links

* [Latest Release](https://github.com/jaredquinn/scoreboard-engine/releases/latest)
* [Documentation Wiki](https://github.com/jaredquinn/scoreboard-engine/wiki)
* [Build Instructions](engine/README.md)
  
# Key Features

* **Single-Binary**: Single executable, built in rust for speed and robustness.
* **Integrated Dashboard**: A HTML dashboard is baked directly into the executable.
* **Disaster Recovery**: Live state is persisted on every change and tick.
* **SSE Support**: Get every tick and every change in realtime via HTTP Server Sent Events.
* **Live Audit**: Every action is time-stamped in `match_log.txt` and mirrored to the stdout console.
* **JSON Data Source**: Link to vMix titles using Data Sources
* **Javascript Helper**: Included Javascript helper library for easy HTML title creation.
* **Static Server**: Serve custom HTML titles from a folder for use in OBS and as a web source/input.
    
If you find this useful - buy me a [Ko-Fi](https://ko-fi.com/vk2way).

# Available Widgets

* [Timer](https://github.com/jaredquinn/scoreboard-engine/wiki/Widgets#timer) (up/down,  min, max, start time, reset, set)
* [Counter](https://github.com/jaredquinn/scoreboard-engine/wiki/Widgets#counter) (increment, decrement, set, shortcut list)
* [Text](https://github.com/jaredquinn/scoreboard-engine/wiki/Widgets/#text) (set)
* List (set, next, prev)
* Calculation (using evalexpr)

Example configurations are provided for Rugby League & AFL.

# Quickstart

Yes, I know these instructions could be better!

These quickstart instructions are for getting up and running quickly using the precompiled binary version of the latest release.

## Engine

* Download the latest version from the [Github repository releases](https://github.com/jaredquinn/scoreboard-engine/releases).  There is precompiled binaries there for Windows, Linux & Mac.
* Extract the archive to any location on your computer.
* Copy/edit/create a config.xml to match your requirements.
* Run `scoreboard-engine` (or `scoreboard-engine.exe` on windows)
* Open your dashboard [http://localhost:3000](http://localhost:3000)

To install the **Bitfocus Companion module** see the [Bitfocus Companion](https://github.com/jaredquinn/scoreboard-engine/wiki/Bitfocus-Companion) page on our Wiki

## Quickstart Integration Guides

_The quickstart guides have moved to the wiki_

* [Quickstart OBS](https://github.com/jaredquinn/scoreboard-engine/wiki/Quickstart-OBS)
* [Quickstart vMix](https://github.com/jaredquinn/scoreboard-engine/wiki/Quickstart-vMix)
  
