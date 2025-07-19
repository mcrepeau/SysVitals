# SysVitals

SysVitals is a modern terminal-based system monitor written in Rust. It provides real-time system monitoring with a TUI (terminal user interface), displaying CPU, memory, network, and GPU statistics in a clear, interactive format.

## Features

- Real-time system monitoring
- CPU, memory, network, and GPU usage statistics
- Terminal-based UI using `ratatui`
- Configurable refresh rate
- Easy to quit with 'q' or 'Esc'

## Prerequisites

- Rust (latest stable version)
- Cargo (Rust's package manager)

## Installation

1. Clone the repository:

    ```sh
    git clone https://github.com/mcrepeau/sysvitals.git
    cd sysvitals
    ```

2. Build the project:

    ```sh
    cargo build --release
    ```

3. Run the application:

    ```sh
    cargo run --release
    ```

## Configuration

The application uses a configuration file located at:
- `~/.config/sysvitals/config.toml` for Unix systems
- `~/Library/Application Support/sysvitals/config.toml` for MacOS
- `%APPDATA%\sysvitals\config.toml` for Windows

Example `config.toml`:

```toml
refresh_rate = 500
show_cpu = true
show_memory = false
show_gpu = true
show_network = true
selected_network_interface = 'Wi-Fi'
```

## Project Structure

The project is organized into several modules for better separation of concerns:

- `src/core`: Core application logic, configuration, error handling, and the main event loop.
- `src/metrics`: Modules for collecting system metrics (CPU, memory, network, GPU), including historical data tracking.
- `src/ui`: Handles rendering the user interface for each metric and the main UI logic.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [ratatui](https://github.com/tui-rs-revival/ratatui) for the terminal UI library.
- [crossterm](https://github.com/crossterm-rs/crossterm) for terminal manipulation.
- [sysinfo](https://github.com/GuillaumeGomez/sysinfo) for system information.
- [tokio](https://github.com/tokio-rs/tokio) for asynchronous runtime.
- [nvml-wrapper](https://github.com/robmikh/nvml-wrapper) for NVIDIA GPU monitoring.
- [raw-cpuid](https://github.com/gz/rust-cpuid) for CPU information.
