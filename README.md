
# 🎯 Scoreboard Engine 

* 𖣘  `engine` - A high-performance, real-time scoreboard backend built with **Rust**, **Axum**, and **Tokio** with integrated web dashboard.
* 🦾 `companion-module-scoreboard-engine` - A module for BitFocus Companion supporting all features.

# Key Features

* **Single-Binary**: Single executable, built in rust for speed and robustness.
* **Integrated Dashboard**: A HTML dashboard is baked directly into the executable.
* **Disaster Recovery**: Live state is persisted on every change and tick.
* **SSE Support**: Get every tick and every change in realtime via HTTP Server Sent Events.
* **Live Audit**: Every action is time-stamped in `match_log.txt` and mirrored to the stdout console.
* **JSON Data Source**: Link to vMix titles using Data Sources
* **Javascript Helper**: Included Javascript helper library for easy HTML title creation.
* **Static Server**: Serve custom HTML titles from a folder for use in OBS and as a web source/input.
    
<img alt="Dashboard Screenshot" src="https://github.com/user-attachments/assets/eb497cdd-4557-4086-bfb1-62f20e4448d7" />

If you find this useful - buy me a [Ko-Fi](https://ko-fi.com/vk2way).

# Available Widgets

* Timers (up/down,  min, max, start time, reset, set)
* Counters (increment, decrement, set, shortcut list)
* Text (set)
* List (set, next, prev)
* Calculation (using evalexpr)

Example configurations are provided for Rugby League & AFL.

# Quickstart

Yes, I know these instructions could be better!

These quickstart instructions are for getting up and running quickly using the precompiled binary version of the latest release.

**Refer to the README file in each directory for instructions on how to build from source**

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

<img alt="image" src="https://github.com/user-attachments/assets/ed380278-96f4-4808-abd3-fce3fcddd136" />

## vMix

Scoreboard Engine provides a flatten list of data designed for consumption in vMix Data Sources;  this can be found at the `/widgets/flat` endpoint.

### Using in a vMix Title

Add the vMix Data Source, refer to the vMix documentation for more information on adding data sources:

<img alt="image" src="https://github.com/user-attachments/assets/7cd63eb2-ae3a-44a2-ac2c-52238872609c" />

Create a title for your production, ensuring you label any fields appropriately for use in Title Editor when linking Data Sources.

<img alt="image" src="https://github.com/user-attachments/assets/2876ab56-8a93-4c2c-b1a8-656bad099586" />


## OBS 

For OBS and other systems using a web input; Scoreboard Engine provides a simple Javascript library and hosting for your HTML title.

### Create the HTML Title

Refer to the HTML pages in the `examples` directory for demonstrations.

Copy this example to your `pages` directory and update it according to your needs.

To handle the live updates from the scoreboard engine, ensure the JS library is included in your page:

```
<script src="/scoreboard.js"></script>
```

To update an element's content with a value from the scoreboard engine simply add the attribute `data-bind=widget_id` to the HTML element; e.g.

```
<div class="score homescore">
   <span data-bind="home_score"></span>
</div>
```

Pages in the /pages folder are served at http://localhost:3000/pages/...

## Add to OBS

Add a "Browser" input to your OBS Scene:

<img alt="image" src="https://github.com/user-attachments/assets/1e8180d1-2be9-4876-af95-d13821563446" />

Set the URL to your filename in the `pages` folder.

