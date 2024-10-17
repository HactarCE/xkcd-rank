# Building xkcd-rank

**xkcd-rank requires Rust v1.81.0 or later.**

## Building on Linux or macOS

1. Download/install Cargo.
2. On Linux, install build-time dependencies: `sudo apt install cmake libglib2.0-dev libatk1.0-dev libgtk-3-dev libxkbcommon-x11-dev`
3. Clone this project and build/run:

```sh
git clone https://github.com/HactarCE/xkcd-rank
cd xkcd-rank
cargo run --release
```

## Building on Windows

1. Download/install [Rustup](https://www.rust-lang.org/tools/install).
2. Download this project and extract it somewhere.
3. Open a terminal in the folder where you extracted xkcd-rank (it should have `Cargo.toml` in it) and build it using `cargo build --release` or run it using `cargo run --release`.
