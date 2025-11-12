#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod commands;
mod slash_commands;

use chrono::Local;
use orcs_core::session::{AppMode, PLACEHOLDER_WORKSPACE_ID};
use orcs_execution::tracing_layer::OrchestratorEvent;
use orcs_infrastructure::paths::{OrcsPaths, ServiceType};
use tauri::Emitter;
use tracing_subscriber::{filter::LevelFilter, prelude::*};

fn main() {
    let path_type = OrcsPaths::new(None)
        .get_path(ServiceType::Logs)
        .expect("Failed to get logs directory");
    let log_dir = path_type.into_path_buf();

    std::fs::create_dir_all(&log_dir).expect("Failed to create logs directory");

    let today = Local::now().format("%Y-%m-%d").to_string();
    let log_file_name = format!("orcs-desktop-{}.log", today);
    let log_file_path = log_dir.join(&log_file_name);

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file_path)
        .expect("Failed to open log file");

    let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);
    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<OrchestratorEvent>();

    let subscriber = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true)
                .with_filter(LevelFilter::TRACE),
        )
        .with(
            orcs_execution::tracing_layer::OrchestratorEventLayer::new(event_tx.clone())
                .with_filter(
                    tracing_subscriber::filter::EnvFilter::new("off")
                        .add_directive("llm_toolkit=debug".parse().unwrap())
                        .add_directive("orcs_execution=debug".parse().unwrap()),
                ),
        );

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");

    tracing::info!("ORCS Desktop starting...");

    tauri::async_runtime::block_on(async move {
        let bootstrap = app::bootstrap(event_tx.clone()).await;
        let session_usecase_for_setup = bootstrap.app_state.session_usecase.clone();

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init())
            .manage(bootstrap.app_state)
            .invoke_handler(commands::handlers())
            .setup(move |app| {
                let handle_for_events = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    println!("[EventListener] Starting orchestrator event listener");
                    while let Some(event) = event_rx.recv().await {
                        println!(
                            "[EventListener] Received event - target: {}, level: {}, message: {}",
                            event.target, event.level, event.message
                        );
                        if let Err(e) = handle_for_events.emit("task-event", &event) {
                            eprintln!("[EventListener] Failed to emit task event: {}", e);
                        }
                    }
                    println!("[EventListener] Orchestrator event listener stopped");
                });

                let handle = app.handle().clone();
                let session_usecase_for_setup = session_usecase_for_setup.clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                    if let Some(session_mgr) = session_usecase_for_setup.active_session().await {
                        let app_mode_locked = AppMode::Idle;
                        // Get workspace_id from session
                        let workspace_id = PLACEHOLDER_WORKSPACE_ID.to_string();
                        let session = session_mgr.to_session(app_mode_locked, workspace_id).await;
                        if session.workspace_id != PLACEHOLDER_WORKSPACE_ID {
                            let workspace_id = &session.workspace_id;
                            tracing::info!(
                                "[Startup] Emitting workspace-switched event for: {}",
                                workspace_id
                            );
                            let _ = handle.emit("workspace-switched", workspace_id);
                        }
                    }
                });

                Ok(())
            })
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}
