#![windows_subsystem = "windows"]

use std::fs::File;
use tracing_subscriber::prelude::*;

fn main() {
    // Set up file logger in AppData/Roaming/CloaKey/cloakey.log
    if let Some(config_dir) = dirs::config_dir().map(|base| base.join("CloaKey")) {
        let _ = std::fs::create_dir_all(&config_dir);
        let log_file_path = config_dir.join("cloakey.log");
        
        if let Ok(file) = File::create(log_file_path) {
            let file_layer = tracing_subscriber::fmt::layer()
                .with_writer(file)
                .with_ansi(false);
            
            let filter = tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("cloakey=info".parse().unwrap())
                .add_directive("cloakey_tray=info".parse().unwrap());
            
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(file_layer)
                .try_init();
        }
    }

    tracing::info!("CloaKey background daemon starting up...");

    if let Err(e) = cloakey_tray::app_loop::run_daemon() {
        tracing::error!("Background daemon execution failed: {}", e);
        std::process::exit(1);
    }
}
