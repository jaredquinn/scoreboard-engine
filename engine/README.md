# 𖣘  Scoreboard Engine Development

If you are not interested in building from the rust source code; you can follow the quickstart guide
in the top level [README](/README.md).



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

