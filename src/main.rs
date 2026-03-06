#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
use anyhow::Result;
use app::App;
use egui::{IconData, ViewportBuilder};
use std::sync::Arc;
use tracing::{error, info};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, fmt};

fn main() -> Result<()> {
    init_logging()?;

    info!("Запуск приложения Handiva");

    let viewport = ViewportBuilder::default()
        .with_inner_size([500.0, 274.0])
        .with_min_inner_size([500.0, 274.0])
        .with_icon(load_icon());

    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };

    let result = eframe::run_native(
        "Handiva",
        options,
        Box::new(|cc| match App::new(cc) {
            Ok(app) => {
                info!("Приложение успешно инициализировано");
                Ok(Box::new(app))
            }
            Err(e) => {
                error!("Ошибка инициализации приложения: {}", e);
                Err(e.into())
            }
        }),
    );

    if let Err(e) = result {
        error!("Ошибка выполнения приложения: {}", e);
    }

    info!("Приложение завершено");
    Ok(())
}

fn init_logging() -> Result<()> {
    let log_dir = directories::ProjectDirs::from("com", "handiva", "Handiva")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());

    std::fs::create_dir_all(&log_dir)?;

    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "handiva.log");

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        #[cfg(debug_assertions)]
        let default_level = "debug";
        #[cfg(not(debug_assertions))]
        let default_level = "info";

        EnvFilter::new(default_level)
    });

    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true);

    let console_layer = fmt::layer()
        .with_writer(std::io::stdout)
        .with_ansi(true)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false);

    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(file_layer)
        .with(console_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

fn load_icon() -> Arc<IconData> {
    const ICON_DATA: &[u8] = include_bytes!("../assets/icon.png");

    let image = image::load_from_memory(ICON_DATA).expect("Failed to decode embedded icon");

    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();

    Arc::new(IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    })
}
