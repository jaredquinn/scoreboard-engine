
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

Yes, I know these instructions could be better!

These quickstart instructions are for getting up and running quickly using the precompiled binary version of the latest release.


## Engine

Refer to [engine/README.md](/engine/README.md) if you're interested in development/building from source.

* Download the latest version from the [Github repository releases](https://github.com/jaredquinn/scoreboard-engine/releases).  There is precompiled binaries there for Windows, Linux & Mac.
* Extract the archive to any location on your computer.
* Edit/create a config.xml to match your requirements.
* Run `scoreboard-engine` (or `scoreboard-engine.exe` on windows)
* Open your dashboard [http://localhost:3000](http://localhost:3000)

## Companion

* Have Bitfocus Companion already running.
* Download the latest companion module from the [Github repository releases](https://github.com/jaredquinn/scoreboard-engine/releases).  This is the file named companion-module-scoreboard-engine-verison.tgz.
* In Companion under modules select import module package and select the zip file downloaded in the previous step.
* Go to connections and add a connection for "Scoreboard Engine/Jared Quinn (The Scoreboard)"
* Add buttons using the companion UI

There is an example page of companion buttons designed to work with the example config.xml file found in the examples folder, you can import this to a companion page.


**Refer to the README file in each directory for instructions on how to build and run**


