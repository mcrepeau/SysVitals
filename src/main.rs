mod core;
mod metrics;
mod ui;

use core::{App, run};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    run(app)
}
