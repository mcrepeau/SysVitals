mod core;
mod metrics;
mod ui;

use core::{App, run};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    run(app)
}
