function getInvoke() {
    if (window.__TAURI__?.core?.invoke) {
        return window.__TAURI__.core.invoke;
    }
    if (window.__TAURI_INTERNALS__?.invoke) {
        return window.__TAURI_INTERNALS__.invoke;
    }
    return null;
}

function getListen() {
    if (window.__TAURI__?.event?.listen) {
        return window.__TAURI__.event.listen;
    }
    if (window.__TAURI_INTERNALS__?.listen) {
        return window.__TAURI_INTERNALS__.listen;
    }
    return null;
}

const bridgeCore = {
    async invoke(command, args = {}) {
        const invoker = getInvoke();
        if (!invoker) {
            return null;
        }
        return await invoker(command, args);
    },

    async getInitData() {
        return await this.invoke('get_init_data');
    },

    async getSettings() {
        return await this.invoke('get_settings');
    },

    async saveSetting(key, value) {
        return await this.invoke('save_setting', { key, value });
    },

    async getTranslations(lang) {
        return await this.invoke('get_translations', { lang });
    },

    async getDashboardStats() {
        return await this.invoke('get_dashboard_stats');
    },

    async getMcVersions() {
        return await this.invoke('get_mc_versions');
    },

    async getLoaderVersions(loader, mcVersion) {
        return await this.invoke('get_loader_versions', { loader, mcVersion });
    },

    async launchGame(version, loader, loaderVersion) {
        return await this.invoke('launch_game', {
            version,
            loader,
            loaderVersion: loaderVersion || null
        });
    },

    async changeInstance(newDir) {
        return await this.invoke('change_instance', { newDir });
    },

    async getLocalMods() {
        return await this.invoke('get_local_mods');
    },

    async toggleMod(filename) {
        return await this.invoke('toggle_mod', { filename });
    },

    async deleteMod(filename) {
        return await this.invoke('delete_mod', { filename });
    },

    async searchStore(provider, query, projectType, loader, gameVersion, category, sortIndex, offset) {
        let l = loader;
        let gv = gameVersion;
        let cat = category;
        let si = sortIndex;
        let off = offset;

        if (typeof gameVersion === 'number') {
            off = gameVersion;
            si = category || 'relevance';
            cat = sortIndex || null;
            gv = loader || null;
            l = projectType || null;
        }

        return await this.invoke('search_store', {
            provider,
            query: query || null,
            projectType: projectType || null,
            loader: l || null,
            gameVersion: gv || null,
            category: cat || null,
            sortIndex: si || null,
            offset: typeof off === 'number' ? off : null
        });
    },

    async getStoreFullDetails(provider, projectId, loader, gameVersion) {
        return await this.invoke('get_store_full_details', {
            provider,
            projectId,
            loader: loader || null,
            gameVersion: gameVersion || null
        });
    },

    async downloadStoreItem(url, filename, projectType) {
        return await this.invoke('download_store_item', { url, filename, projectType });
    },

    async downloadSpecificFile(url, filename, projectType) {
        return await this.invoke('download_specific_file', { url, filename, projectType });
    },

    async generateAutoBuild(prompt, mcVersion, loader) {
        return await this.invoke('generate_auto_build', { prompt, mcVersion, loader });
    },

    async resolveKeybinds() {
        return await this.invoke('resolve_keybinds');
    },

    async checkModUpdates() {
        return await this.invoke('check_mod_updates');
    },

    async applyModUpdates(updates) {
        return await this.invoke('apply_mod_updates', { updates });
    },

    async exportModpack() {
        return await this.invoke('export_modpack');
    },

    async importDroppedMods(files) {
        return await this.invoke('import_dropped_mods', { files });
    },

    async importModsDialog() {
        return await this.invoke('import_mods_dialog');
    },

    async getModGraphData() {
        return await this.invoke('get_mod_graph_data');
    },

    async fetchHub() {
        return await this.invoke('fetch_hub');
    },

    async publishHub(title, author, desc, mods) {
        return await this.invoke('publish_hub', { title, author, desc, mods });
    },

    async swarmDownload(magnet, targetDir) {
        return await this.invoke('swarm_download', { magnet, targetDir });
    },

    async swarmSeedStart(targetFolder) {
        return await this.invoke('swarm_seed_start', { targetFolder });
    },

    async swarmSeedStop(torrentName) {
        return await this.invoke('swarm_seed_stop', { torrentName });
    },

    async swarmSeedStatus() {
        return await this.invoke('swarm_seed_status');
    },

    async getWorlds() {
        return await this.invoke('get_worlds');
    },

    async deleteWorld(worldName) {
        return await this.invoke('delete_world', { worldName });
    },

    async healWorldPlayer(worldName) {
        return await this.invoke('heal_world_player', { worldName });
    },

    async syncCloudWorld(worldName) {
        return await this.invoke('sync_cloud_world', { worldName });
    },

    async getWorldMap(worldName) {
        return await this.invoke('get_world_map', { worldName });
    },

    async vcsCommit(worldName) {
        return await this.invoke('vcs_commit', { worldName });
    },

    async vcsGetHistory(worldName) {
        return await this.invoke('vcs_get_history', { worldName });
    },

    async vcsRestore(worldName, commitId) {
        return await this.invoke('vcs_restore', { worldName, commitId });
    },

    async pingServer(ip) {
        return await this.invoke('ping_server', { ip });
    },

    async startTunnel(port) {
        return await this.invoke('start_tunnel', { port });
    },

    async stopTunnel() {
        return await this.invoke('stop_tunnel');
    },

    async pteroConnect(url, key) {
        return await this.invoke('ptero_connect', { url, key });
    },

    async pteroAction(action, serverId) {
        return await this.invoke('ptero_action', { action, serverId });
    },

    async deployDockerServer(core, version, port) {
        return await this.invoke('deploy_docker_server', { core, version, port });
    },

    async runTool(toolId) {
        return await this.invoke('run_tool', { toolId });
    },

    async applyDoctorFixes(issues, mcVersion, loader) {
        return await this.invoke('apply_doctor_fixes', {
            issues,
            mcVersion: mcVersion || null,
            loader: loader || null
        });
    },

    async getSafeModeState() {
        return await this.invoke('get_safe_mode_state');
    },

    async toggleSafeMode() {
        return await this.invoke('toggle_safe_mode');
    },

    async getMedia(offset = 0, limit = 12) {
        return await this.invoke('get_media', { offset, limit });
    },

    async getMediaFull(filename) {
        return await this.invoke('get_media_full', { filename });
    },

    async compressMedia() {
        return await this.invoke('compress_media');
    },

    async deleteMedia(filename) {
        return await this.invoke('delete_media', { filename });
    },

    async openMediaFolder() {
        return await this.invoke('open_media_folder');
    },

    async toggleConsoleStream(active) {
        return await this.invoke('toggle_console_stream', { active });
    },

    async getConsoleLogs() {
        return await this.invoke('get_console_logs');
    },

    async getSysInfo() {
        return await this.invoke('get_sys_info');
    },

    async analyzeCrashAi(logSnippet) {
        return await this.invoke('analyze_crash_ai', { logSnippet });
    },

    async sendBugReport(reportText) {
        return await this.invoke('send_bug_report', { reportText });
    },

    async saveNote(text) {
        return await this.invoke('save_note', { text });
    },

    async getNote() {
        return await this.invoke('get_note');
    },

    async getFriends() {
        return await this.invoke('get_friends');
    },

    async addFriend(name) {
        return await this.invoke('add_friend', { name });
    },

    async removeFriend(name) {
        return await this.invoke('remove_friend', { name });
    },

    async msAuthStart() {
        return await this.invoke('ms_auth_start');
    },

    async msAuthPoll(deviceCode) {
        return await this.invoke('ms_auth_poll', { deviceCode });
    },

    async msLogout() {
        return await this.invoke('ms_logout');
    },

    async getMsProfile() {
        return await this.invoke('get_ms_profile');
    },

    async kipLogin(username, password) {
        return await this.invoke('kip_login', { username, password });
    },

    async kipRegister(username, email, password) {
        return await this.invoke('kip_register', { username, email: email || null, password });
    },

    async getKipProfile() {
        return await this.invoke('get_kip_profile');
    },

    async kipLogout() {
        return await this.invoke('kip_logout');
    },

    async partyInvitePrepare() {
        return await this.invoke('party_invite_prepare');
    },

    async toggleOverlay() {
        return await this.invoke('toggle_overlay');
    },

    async toggleBigPicture() {
        return await this.invoke('toggle_big_picture');
    },

    async setMiniMode(mini) {
        return await this.invoke('set_mini_mode', { mini });
    },

    async pickFile() {
        return await this.invoke('pick_file');
    },

    async windowMinimize() {
        return await this.invoke('window_minimize');
    },

    async windowMaximize() {
        return await this.invoke('window_maximize');
    },

    async windowClose() {
        return await this.invoke('window_close');
    }
};

const bridgeHandler = {
    get(target, prop) {
        if (prop in target) {
            return typeof target[prop] === 'function' ? target[prop].bind(target) : target[prop];
        }

        const camel = prop.replace(/_([a-z])/g, (_, c) => c.toUpperCase());
        if (camel in target) {
            return target[camel].bind(target);
        }

        return async (...args) => {
            const payload = args[0] && typeof args[0] === 'object' ? args[0] : {};
            return await target.invoke(prop, payload);
        };
    }
};

export const bridge = new Proxy(bridgeCore, bridgeHandler);

export function setupTauriListeners(callbacks = {}) {
    const listen = getListen();
    if (!listen) {
        return;
    }

    if (callbacks.onDaemonStatus) {
        listen('updateDaemonStatus', (event) => {
            callbacks.onDaemonStatus(event.payload.running, event.payload.status);
        });
    }

    if (callbacks.onConsoleLine) {
        listen('appendConsoleLine', (event) => {
            callbacks.onConsoleLine(event.payload);
        });
    }

    if (callbacks.onCrashAlert) {
        listen('showCrashAlert', (event) => {
            callbacks.onCrashAlert(event.payload);
        });
    }

    if (callbacks.onLaunchStatus) {
        listen('updateLaunchStatus', (event) => {
            callbacks.onLaunchStatus(event.payload);
        });
    }

    if (callbacks.onLaunchProgress) {
        listen('updateLaunchProgress', (event) => {
            callbacks.onLaunchProgress(event.payload);
        });
    }

    if (callbacks.onTunnelStatus) {
        listen('updateTunnelStatus', (event) => {
            callbacks.onTunnelStatus(event.payload);
        });
    }

    if (callbacks.onToggleOverlay) {
        listen('toggleOverlay', () => {
            callbacks.onToggleOverlay();
        });
    }
}