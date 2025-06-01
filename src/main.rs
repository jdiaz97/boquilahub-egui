#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use boquilahub::{api::rest::run_api, cli::{self, run_cli}};
// hide console window on Windows in release
use clap::{Arg, Command};

// When compiling natively:
#[tokio::main]
async fn main() -> eframe::Result {
    run_cli().await;

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                // NOTE: Adding an icon is optional
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "BoquilaHUB",
        native_options,
        Box::new(|_cc| Ok(Box::new(boquilahub::MainApp::new()))),
    )
}
