//! Command-line argument parsing (no external dependencies).

/// Arguments parsed from the command line.
/// All fields are `Option` so unspecified flags leave the saved config intact.
#[derive(Default)]
pub struct CliArgs {
    pub compact:    Option<bool>,
    pub interval_ms: Option<u64>,
    pub show_cpu:   Option<bool>,
    pub show_memory: Option<bool>,
    pub show_gpu:   Option<bool>,
    pub show_disk:  Option<bool>,
    pub show_network: Option<bool>,
}

impl CliArgs {
    pub fn parse() -> Result<Self, String> {
        let mut args = std::env::args().skip(1).peekable();
        let mut out = Self::default();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "-c" | "--compact"     => out.compact       = Some(true),
                "--no-cpu"             => out.show_cpu      = Some(false),
                "--no-memory"          => out.show_memory   = Some(false),
                "--no-gpu"             => out.show_gpu      = Some(false),
                "--no-disk"            => out.show_disk     = Some(false),
                "--no-network"         => out.show_network  = Some(false),
                "-i" | "--interval" => {
                    let val = args.next().ok_or("--interval requires a value in ms")?;
                    let ms: u64 = val.parse().map_err(|_| format!("invalid interval: {val}"))?;
                    if ms < 100 {
                        return Err("interval must be at least 100 ms".into());
                    }
                    out.interval_ms = Some(ms);
                }
                other => return Err(format!("unknown argument: {other}")),
            }
        }
        Ok(out)
    }
}

fn print_help() {
    println!("sysvitals — lightweight terminal system monitor

USAGE:
    sysvitals [OPTIONS]

OPTIONS:
    -c, --compact          Start in compact bars view
    -i, --interval <ms>    Refresh interval in milliseconds (default: 1000, min: 100)
        --no-cpu           Hide CPU panel
        --no-memory        Hide memory panel
        --no-gpu           Hide GPU panel
        --no-disk          Hide disk panel
        --no-network       Hide network panel
    -h, --help             Print this help message

KEYS (while running):
    q / Esc    Quit
    o          Open options menu
    v          Toggle compact / chart view
    Tab        Cycle network interface (options menu)");
}
