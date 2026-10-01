#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(dead_code)]

mod config;
mod database;
mod system_utils;
mod mod_manager;
mod auth_manager;
mod api_manager;
mod tool_manager;
mod shield_manager;
mod world_manager;
mod media_manager;
mod vcs_manager;
mod builder_manager;
mod cartographer_manager;
mod doctor_manager;
mod instance_manager;
mod locales;
mod monitor_service;
mod crash_service;
mod rpc_service;
mod overlay_service;
mod signaling_service;
mod swarm_manager;
mod tunnel_manager;
mod commands;
mod commands_ext;

use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use parking_lot::Mutex;
use tauri::Emitter;

use commands::AppState;
use database::DatabaseManager;
use auth_manager::AuthManager;
use api_manager::ApiManager;
use instance_manager::InstanceManager;
use doctor_manager::DoctorManager;
use shield_manager::ShieldManager;
use swarm_manager::SwarmManager;
use tunnel_manager::TunnelManager;
use world_manager::WorldManager;
use media_manager::MediaManager;
use vcs_manager::VCSManager;
use cartographer_manager::CartographerManager;
use builder_manager::BuilderManager;
use monitor_service::MonitorService;
use crash_service::CrashService;
use rpc_service::RpcService;
use overlay_service::OverlayService;
use signaling_service::SignalingService;

fn main() {
    let db_path = config::get_app_data_dir().join("kip_data.db");
    let db = Arc::new(DatabaseManager::new(db_path.to_str().unwrap()).expect("Failed to initialize database"));
    let stop_signal = Arc::new(AtomicBool::new(false));

    let auth = Arc::new(AuthManager::new());
    let api = Arc::new(ApiManager::new());
    let instance = Arc::new(InstanceManager::new());
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

    let app_state = AppState {
        db: Arc::clone(&db),
        auth,
        api: Arc::clone(&api),
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
        stop_signal: Arc::clone(&stop_signal),
        cached_kip_profile: Arc::new(Mutex::new(None)),
        cached_ms_profile: Arc::new(Mutex::new(None)),
    };

    let cfg = config::load_app_config();
    let current_inst = cfg.current_instance.clone();
    let log_file_path = std::path::Path::new(&current_inst).join("logs").join("latest.log").to_string_lossy().to_string();
    let crash_dir_path = std::path::Path::new(&current_inst).join("crash-reports").to_string_lossy().to_string();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(app_state)
        .setup(move |app| {
            let handle = app.handle().clone();

            let h1 = handle.clone();
            monitor.start_process_monitor(Arc::clone(&stop_signal), current_inst, move |is_running, status| {
                let _ = h1.emit("updateDaemonStatus", serde_json::json!({ "running": is_running, "status": status }));
            });

            let h2 = handle.clone();
            monitor.start_console_stream(Arc::clone(&stop_signal), log_file_path, move |line| {
                let _ = h2.emit("appendConsoleLine", line);
            });

            let h3 = handle.clone();
            CrashService::start_crash_monitor(crash_dir_path, Arc::clone(&stop_signal), move |crash_snippet| {
                let _ = h3.emit("showCrashAlert", crash_snippet);
            });

            RpcService::start_rpc(
                config::DISCORD_CLIENT_ID,
                Arc::clone(&db),
                Arc::clone(&monitor.is_mc_running),
                Arc::clone(&stop_signal),
            );

            let h4 = handle.clone();
            OverlayService::start_hotkey_listener(Arc::clone(&stop_signal), move || {
                let _ = h4.emit("toggleOverlay", ());
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
            commands::search_store,
            commands::get_store_full_details,
            commands::download_store_item,
            commands::download_specific_file,
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
            commands_ext::get_settings,
            commands_ext::save_setting,
            commands_ext::get_init_data,
            commands_ext::get_translations,
            commands_ext::window_minimize,
            commands_ext::window_maximize,
            commands_ext::window_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}