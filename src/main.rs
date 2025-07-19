mod core;
mod metrics;
mod ui;

use core::{App, run};
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Try to create app with Unix metrics first, fall back to standard metrics if needed
    let app = match App::new_with_metrics(true) {
        Ok(app) => {
            println!("✅ Unix metrics enabled");
            app
        }
        Err(_) => {
            println!("⚠️  Unix metrics not available, using standard metrics");
            App::new()?
        }
    };
    
    run(app)
}
