mod core;
mod metrics;
mod ui;

use core::{App, run};
use core::args::CliArgs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = CliArgs::parse().unwrap_or_else(|e| {
        eprintln!("error: {e}\nRun with --help for usage.");
        std::process::exit(1);
    });
    let app = App::new(&args)?;
    run(app)
}
