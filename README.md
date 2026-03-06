# SysVitals

SysVitals is a lightweight terminal system monitor written in Rust. It displays live CPU, memory, disk, network, and GPU metrics in a clean TUI with two views: a scrolling chart history and a compact live-values bar view.

## Features

- CPU usage, temperature, and load average (1/5/15 min)
- Memory and swap usage
- Disk I/O read/write rates
- Network RX/TX rates, with per-interface selection
- GPU compute and VRAM usage (NVIDIA only)
- Two views: **chart** (scrolling history) and **compact bars** (live values)
- Configurable refresh rate and per-panel visibility
- Preferences saved automatically across sessions
- CLI flags for quick one-off sessions

## Installation

### Linux/macOS

```sh
curl -fsSL https://github.com/mcrepeau/SysVitals/raw/refs/heads/main/install.sh | bash
```

### Windows

```powershell
irm https://github.com/mcrepeau/SysVitals/raw/refs/heads/main/install.ps1 | iex
```

## Usage

```
sysvitals [OPTIONS]

OPTIONS:
    -c, --compact          Start in compact bars view
    -i, --interval <ms>    Refresh interval in ms (default: 1000, min: 100)
        --no-cpu           Hide CPU panel
        --no-memory        Hide memory panel
        --no-gpu           Hide GPU panel
        --no-disk          Hide disk panel
        --no-network       Hide network panel
    -h, --help             Print help
```

## Compile from source

**Prerequisites:** Rust (latest stable) and Cargo.

```sh
git clone https://github.com/mcrepeau/sysvitals.git
cd sysvitals
cargo build --release
./target/release/sysvitals
```

## Configuration

Preferences are saved automatically when changed via the options menu. The config file lives at:

- Linux: `~/.config/sysvitals/config.toml`
- macOS: `~/Library/Application Support/sysvitals/config.toml`
- Windows: `%APPDATA%\sysvitals\config.toml`

Example `config.toml`:

```toml
refresh_rate = 1000
show_cpu = true
show_memory = true
show_gpu = true
show_network = true
show_disk = true
compact_view = false
selected_network_interface = "eth0"
```

## Project structure

- `src/core` — app state, event loop, config, CLI args, error handling
- `src/metrics` — per-subsystem collectors (CPU, memory, disk, network, GPU) with rolling history
- `src/ui` — chart and bar renderers, shared utilities in `chart_utils`

## License

MIT — see [LICENSE](LICENSE).
