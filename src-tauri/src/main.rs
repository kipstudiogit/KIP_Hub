#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod api_manager;
mod auth_manager;
mod builder_manager;
mod cartographer_manager;
mod commands;
mod commands_ext;
mod config;
mod crash_service;
mod database;
mod doctor_manager;
mod instance_manager;
mod java_manager;
mod launch;
mod locales;
mod media_manager;
mod mod_manager;
mod monitor_service;
mod overlay_service;
mod rpc_service;
mod settings_manager;
mod shield;
mod signaling_service;
mod store_manager;
mod swarm_manager;
mod system_utils;
mod tool_manager;
mod tunnel_manager;
mod vcs_manager;
mod world_manager;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::{Emitter, RunEvent, WindowEvent};

use api_manager::ApiManager;
use auth_manager::AuthManager;
use builder_manager::BuilderManager;
use cartographer_manager::CartographerManager;
use commands::AppState;
use crash_service::CrashService;
use database::DatabaseManager;
use doctor_manager::DoctorManager;
use instance_manager::InstanceManager;
use java_manager::JavaManager;
use media_manager::MediaManager;
use monitor_service::MonitorService;
use overlay_service::OverlayService;
use rpc_service::RpcService;
use shield::ShieldManager;
use signaling_service::SignalingService;
use store_manager::StoreManager;
use swarm_manager::SwarmManager;
use tunnel_manager::TunnelManager;
use vcs_manager::VCSManager;
use world_manager::WorldManager;

fn main() {
    let db_path = config::get_app_data_dir().join("kip_data.db");
    let db = Arc::new(
        DatabaseManager::new(db_path.to_str().expect("Failed to build database path string"))
            .expect("Failed to initialize SQLite persistence layer"),
    );

    let stop_signal = Arc::new(AtomicBool::new(false));
    let auth = Arc::new(AuthManager::new());
    let api = Arc::new(ApiManager::new());
    let java = Arc::new(JavaManager::new());
    let instance = Arc::new(InstanceManager::new(Arc::clone(&java)));
    let doctor = Arc::new(DoctorManager::new());
    let shield = Arc::new(ShieldManager::new());
    let swarm = Arc::new(SwarmManager::new());
    let tunnel = Arc::new(TunnelManager::new());
    let world = Arc::new(WorldManager);
    let media = Arc::new(MediaManager);
    let vcs = Arc::new(VCSManager);
    let cartographer = Arc::new(CartographerManager::new());
    let builder = Arc::new(BuilderManager::new());
    let monitor = Arc::new(MonitorService::new(Arc::clone(&db)));
    let store = Arc::new(StoreManager::new());

    let app_state = AppState {
        db: Arc::clone(&db),
        auth,
        api: Arc::clone(&api),
        java: Arc::clone(&java),
        instance,
        doctor,
        shield,
        swarm,
        tunnel,
        world,
        media,
        vcs,
        cartographer,
        builder,
        monitor: Arc::clone(&monitor),
        store,
        stop_signal: Arc::clone(&stop_signal),
        cached_kip_profile: Arc::new(Mutex::new(None)),
        cached_ms_profile: Arc::new(Mutex::new(None)),
    };

    let cfg = config::load_app_config();
    let current_inst = cfg.current_instance.clone();
    let log_file_path = std::path::Path::new(&current_inst)
        .join("logs")
        .join("latest.log")
        .to_string_lossy()
        .to_string();
    let crash_dir_path = std::path::Path::new(&current_inst)
        .join("crash-reports")
        .to_string_lossy()
        .to_string();

    let stop_signal_monitor = Arc::clone(&stop_signal);
    let stop_signal_console = Arc::clone(&stop_signal);
    let stop_signal_crash = Arc::clone(&stop_signal);
    let stop_signal_rpc = Arc::clone(&stop_signal);
    let stop_signal_overlay = Arc::clone(&stop_signal);
    let stop_signal_shutdown = Arc::clone(&stop_signal);

    let monitor_ref = Arc::clone(&monitor);
    let is_mc_running_ref = Arc::clone(&monitor.is_mc_running);

    let app = tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let handle = app.handle().clone();

            let h_daemon = handle.clone();
            monitor_ref.start_process_monitor(
                stop_signal_monitor,
                current_inst,
                move |is_running, status| {
                    let _ = h_daemon.emit(
                        "updateDaemonStatus",
                        serde_json::json!({ "running": is_running, "status": status }),
                    );
                },
            );

            let h_console = handle.clone();
            monitor_ref.start_console_stream(
                stop_signal_console,
                log_file_path,
                move |line| {
                    let _ = h_console.emit("appendConsoleLine", line);
                },
            );

            let h_crash = handle.clone();
            CrashService::start_crash_monitor(
                crash_dir_path,
                stop_signal_crash,
                move |crash_snippet| {
                    let _ = h_crash.emit("showCrashAlert", crash_snippet);
                },
            );

            RpcService::start_rpc(
                config::DISCORD_CLIENT_ID,
                db,
                is_mc_running_ref,
                stop_signal_rpc,
            );

            let h_overlay = handle.clone();
            OverlayService::start_hotkey_listener(stop_signal_overlay, move || {
                let _ = h_overlay.emit("toggleOverlay", ());
            });

            tauri::async_runtime::spawn(async move {
                let signaling = SignalingService::new();
                let _ = signaling.run_server("0.0.0.0:8765").await;
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::kip_login,
            commands::kip_register,
            commands::get_kip_profile,
            commands::kip_logout,
            commands::ms_auth_start,
            commands::ms_auth_poll,
            commands::ms_logout,
            commands::get_ms_profile,
            commands::get_mc_versions,
            commands::get_loader_versions,
            commands::launch_game,
            commands::change_instance,
            commands::get_local_mods,
            commands::toggle_mod,
            commands::delete_mod,
            commands::batch_toggle_mods,
            commands::batch_delete_mods,
            commands::open_content_folder,
            commands::search_store,
            commands::get_store_full_details,
            commands::download_store_item,
            commands::download_specific_file,
            commands::uninstall_store_item,
            commands::generate_auto_build,
            commands::resolve_keybinds,
            commands::swarm_seed_start,
            commands::swarm_seed_stop,
            commands::swarm_seed_status,
            commands::get_friends,
            commands::add_friend,
            commands::remove_friend,
            commands::start_tunnel,
            commands::stop_tunnel,
            commands::deploy_docker_server,
            commands::ptero_connect,
            commands::ptero_action,
            settings_manager::get_settings,
            settings_manager::save_setting,
            settings_manager::reset_settings_to_default,
            settings_manager::validate_java_binary,
            settings_manager::test_ai_connection,
            settings_manager::vacuum_database,
            settings_manager::open_instance_folder,
            shield::shield_scan_full,
            shield::shield_scan_file,
            shield::shield_quarantine_threat,
            shield::shield_restore_threat,
            shield::shield_shred_threat,
            shield::shield_get_vault,
            commands_ext::import_modpack_or_archive,
            commands_ext::auto_tune_ram,
            commands_ext::inspect_installed_content,
            commands_ext::preflight_inspection,
            commands_ext::auto_repair_instance,
            commands_ext::get_dashboard_stats,
            commands_ext::toggle_console_stream,
            commands_ext::ping_server,
            commands_ext::get_mod_graph_data,
            commands_ext::import_dropped_mods,
            commands_ext::import_mods_dialog,
            commands_ext::check_mod_updates,
            commands_ext::apply_mod_updates,
            commands_ext::export_modpack,
            commands_ext::fetch_hub,
            commands_ext::publish_hub,
            commands_ext::swarm_download,
            commands_ext::party_invite_prepare,
            commands_ext::toggle_overlay,
            commands_ext::toggle_big_picture,
            commands_ext::set_mini_mode,
            commands_ext::send_bug_report,
            commands_ext::sync_cloud_world,
            commands_ext::pick_file,
            commands_ext::run_tool,
            commands_ext::apply_doctor_fixes,
            commands_ext::get_media,
            commands_ext::get_media_full,
            commands_ext::compress_media,
            commands_ext::delete_media,
            commands_ext::open_media_folder,
            commands_ext::get_console_logs,
            commands_ext::get_sys_info,
            commands_ext::analyze_crash_ai,
            commands_ext::get_worlds,
            commands_ext::delete_world,
            commands_ext::heal_world_player,
            commands_ext::vcs_commit,
            commands_ext::vcs_get_history,
            commands_ext::vcs_restore,
            commands_ext::get_world_map,
            commands_ext::get_safe_mode_state,
            commands_ext::toggle_safe_mode,
            commands_ext::save_note,
            commands_ext::get_note,
            commands_ext::get_init_data,
            commands_ext::get_translations,
            commands_ext::window_minimize,
            commands_ext::window_maximize,
            commands_ext::window_close,
            commands_ext::check_app_update,
            commands_ext::perform_app_update
        ])
        .build(tauri::generate_context!())
        .expect("Failed to build Tauri execution context");

    app.run(move |_app_handle, event| match event {
        RunEvent::WindowEvent {
            event: WindowEvent::CloseRequested { .. },
            ..
        }
        | RunEvent::ExitRequested { .. } => {
            stop_signal_shutdown.store(true, Ordering::SeqCst);
        }
        _ => {}
    });
}