
# 🎯 Scoreboard Engine 

* 𖣘  `engine` - A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio** with integrated monitoring dashboard.
* 🦾 `companion module` - A module for BitFocus Companion supporting all features.


If you find this useful - buy me a [Ko-Fi](https://ko-fi.com/vk2way).

# Currently supported features

* Timers (up/down,  min, max, start time, reset, set)
* Counters (increment, decrement, set, shortcut list)
* Text (set)
* List (set, next, prev)

# Quickstart

## Engine

Yes, I know these instructions could be better!

* Download the latest version from the [Github repository releases](https://github.com/jaredquinn/scoreboard-engine/releases).
* Extract the archive
* Edit the config.xml to match your requirements.
* Start scoreboard-engine (or scoreboard-engine.exe on windows)
* Open your dashboard [http://localhost:3000](http://localhost:3000)

## Companion

* Have Bitfocus Companion already running.
* Download the latest companion module from the [Github repository releases](https://github.com/jaredquinn/scoreboard-engine/releases).
* In Companion under modules select import module package and select the zip file downloaded in the previous step.
* Go to connections and add a connection for "Scoreboard Engine/Jared Quinn (The Scoreboard)"
* Add buttons using the companion UI

There is an example page of companion buttons designed to work with the example config.xml file found in the examples folder.


**Refer to the README file in each directory for instructions on how to build and run**


