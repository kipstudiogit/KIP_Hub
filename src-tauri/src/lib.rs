mod api_manager;
mod asset_transcoder;
mod auth_manager;
mod builder_manager;
mod cartographer_manager;
mod chunk_engine;
pub mod commands;
mod config;
mod crash_service;
mod database;
mod doctor_manager;
pub mod error;
mod hardware_booster;
mod instance_manager;
mod java_manager;
mod launch;
mod locales;
mod media_manager;
mod memory_matrix;
mod mod_manager;
mod monitor_service;
mod network_optimizer;
mod overlay_service;
mod rpc_service;
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
use chunk_engine::ChunkAcceleratorManager;
use commands::AppState;
use crash_service::CrashService;
use database::DatabaseManager;
use doctor_manager::DoctorManager;
use hardware_booster::HardwareBoosterManager;
use instance_manager::InstanceManager;
use java_manager::JavaManager;
use media_manager::MediaManager;
use memory_matrix::MemoryMatrixManager;
use monitor_service::MonitorService;
use network_optimizer::NetworkOptimizer;
use overlay_service::OverlayService;
use rpc_service::RpcService;
use shield::ShieldManager;
use signaling_service::SignalingService;
use store_manager::StoreManager;
use swarm_manager::SwarmManager;
use tunnel_manager::TunnelManager;
use vcs_manager::VCSManager;
use world_manager::WorldManager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let db_path = config::get_app_data_dir().join("kip_data.db");
    let db = Arc::new(
        DatabaseManager::new(db_path.to_str().expect("Valid SQLite path required"))
            .expect("Persistence layer initialization failed"),
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
    let memory_matrix = Arc::new(MemoryMatrixManager::new());
    let chunk_engine = Arc::new(ChunkAcceleratorManager::new());
    let net_optimizer = Arc::new(NetworkOptimizer::new());
    let hardware_booster = Arc::new(HardwareBoosterManager::new());

    let app_state = AppState {
        db: Arc::clone(&db),
        auth,
        api,
        java,
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
        memory_matrix,
        chunk_engine,
        net_optimizer,
        hardware_booster: Arc::clone(&hardware_booster),
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
    let booster_cleanup_ref = Arc::clone(&hardware_booster);

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
            commands::booster::booster_chunk_accelerator_status,
            commands::booster::booster_chunk_accelerator_toggle,
            commands::booster::booster_chunk_accelerator_prebake_stream,
            commands::booster::booster_chunk_accelerator_flush_ring,
            commands::booster::booster_memory_matrix_status,
            commands::booster::booster_memory_matrix_toggle_large_pages,
            commands::booster::booster_memory_matrix_toggle_compact_headers,
            commands::booster::booster_memory_matrix_grant_privilege,
            commands::booster::booster_memory_matrix_prefault_cache,
            commands::booster::booster_ping_master_get_config,
            commands::booster::booster_ping_master_set_config,
            commands::booster::booster_ping_master_detect_vpn,
            commands::booster::booster_ping_master_start,
            commands::booster::booster_ping_master_stop,
            commands::booster::get_hardware_booster_config,
            commands::booster::set_hardware_booster_config,
            commands::booster::get_hardware_booster_status,
            commands::booster::execute_asset_transcode_stream,
            commands::booster::trigger_manual_defender_bypass,
            commands::booster::trigger_manual_working_set_trim,
            commands::booster::trigger_page_cache_warmup,
            commands::booster::trigger_app_cds_dump,
            commands::window::window_get_status,
            commands::window::window_minimize,
            commands::window::window_toggle_maximize,
            commands::window::window_maximize,
            commands::window::window_is_maximized,
            commands::window::window_close,
            commands::window::window_toggle_pin,
            commands::window::window_set_mini_mode,
            commands::window::set_mini_mode,
            commands::window::window_toggle_fullscreen,
            commands::window::toggle_big_picture,
            commands::overlay::overlay_set_active,
            commands::overlay::overlay_save_layout,
            commands::overlay::overlay_load_layout,
            commands::overlay::overlay_save_waypoints,
            commands::overlay::overlay_load_waypoints,
            commands::overlay::toggle_overlay,
            commands::nexus::nexus_get_friends,
            commands::nexus::nexus_add_friend,
            commands::nexus::nexus_remove_friend,
            commands::nexus::nexus_toggle_favorite,
            commands::nexus::nexus_update_note,
            commands::nexus::nexus_get_party_state,
            commands::settings::get_app_settings,
            commands::settings::get_settings,
            commands::settings::save_setting,
            commands::settings::update_app_settings,
            commands::settings::reset_app_settings_default,
            commands::settings::validate_java_executable,
            commands::settings::detect_system_java_runtimes,
            commands::settings::pick_instance_directory,
            commands::settings::test_neural_connection,
            commands::settings::vacuum_sqlite_database,
            commands::settings::open_instance_directory,
            commands::support::get_system_telemetry_dossier,
            commands::support::diagnose_crash_with_neural_core,
            commands::support::submit_encrypted_bug_report,
            commands::support::load_latest_crash_or_log,
            commands::console::get_recent_console_entries,
            commands::console::upload_active_console_log,
            commands::console::export_console_log_to_file,
            commands::console::toggle_console_streaming,
            commands::media::get_media_catalog,
            commands::media::get_media_full_image,
            commands::media::compress_media_stream,
            commands::media::delete_media_file,
            commands::media::batch_delete_media_files,
            commands::media::open_screenshots_directory,
            commands::worlds::get_worlds_catalog,
            commands::worlds::heal_world_entity_player,
            commands::worlds::toggle_world_gamemode,
            commands::worlds::clone_saved_world,
            commands::worlds::delete_saved_world,
            commands::worlds::capture_world_vcs_commit,
            commands::worlds::get_world_vcs_history,
            commands::worlds::restore_world_vcs_commit,
            commands::worlds::generate_world_satellite_map,
            commands::launch::download_minecraft_version,
            commands::launch::inspect_preflight,
            commands::launch::auto_repair_environment,
            commands::launch::launch_instance_stream,
            commands::content::get_mod_graph_data,
            commands::content::import_dropped_content,
            commands::content::import_modpack_or_archive,
            commands::content::check_content_updates,
            commands::content::apply_content_updates_stream,
            commands::content::get_installed_modpacks,
            commands::content::mount_modpack_archive,
            commands::content::export_active_modpack,
            commands::store::search_store_catalog,
            commands::store::get_store_project_details,
            commands::store::install_store_item_stream,
            commands::store::uninstall_store_item,
            commands::tools::execute_system_tool,
            commands::tools::run_tool,
            commands::tools::investigate_latest_crash,
            commands::tools::analyze_mod_doctor,
            commands::tools::apply_doctor_remediation,
            commands::tools::scan_shield_security_stream,
            commands::tools::quarantine_shield_file,
            commands::tools::restore_shield_record,
            commands::tools::shred_shield_record,
            commands::tools::get_shield_vault_records,
            commands::tools::get_safe_mode_status,
            commands::tools::toggle_safe_mode_status,
            commands::system::get_dashboard_overview,
            commands::system::get_dashboard_stats,
            commands::system::toggle_console_stream,
            commands::system::fetch_hub,
            commands::system::publish_hub,
            commands::system::swarm_download,
            commands::system::send_bug_report,
            commands::system::get_console_logs,
            commands::system::get_sys_info,
            commands::system::analyze_crash_ai,
            commands::system::save_note,
            commands::system::get_note,
            commands::system::get_init_data,
            commands::system::get_translations,
            commands::system::check_app_update,
            commands::system::perform_app_update,
            commands::system::pick_file,
            commands::network::ping_server,
            commands::network::start_tunnel,
            commands::network::stop_tunnel,
            commands::network::deploy_docker_server,
            commands::network::ptero_connect,
            commands::network::ptero_action,
            commands::network::party_invite_prepare,
            commands::builder::execute_auto_build_stream,
            commands::builder::resolve_keybind_conflicts_detailed,
            commands::get_all_minecraft_versions,
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
            commands::change_instance,
            commands::get_local_mods,
            commands::toggle_mod,
            commands::delete_mod,
            commands::batch_toggle_mods,
            commands::batch_delete_mods,
            commands::open_content_folder,
            commands::generate_auto_build,
            commands::resolve_keybinds,
            commands::swarm_seed_start,
            commands::swarm_seed_stop,
            commands::swarm_seed_status,
            commands::get_friends,
            commands::add_friend,
            commands::remove_friend
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
            booster_cleanup_ref.disable_high_resolution_timer();
            booster_cleanup_ref.remove_defender_exclusions();
        }
        _ => {}
    });
}