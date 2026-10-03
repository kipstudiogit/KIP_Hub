export interface LaunchPayload {
  version: string
  loader: string
  loader_version?: string | null
}

export interface LaunchResult {
  success: boolean
  message: string
  pid?: number | null
}

export interface GenericActionResult {
  success: boolean
  msg: string
}

export interface LocalModRecord {
  filename: string
  id: string
  name: string
  version: string
  author: string
  description: string
  loaders: string[]
  disabled: boolean
  icon: string
  size_bytes: number
  date_modified: string
  content_type: string
  dependencies: string[]
}

export interface FriendRecord {
  name: string
  status: string
  avatar: string
  activity?: string | null
  is_favorite?: boolean
}

export interface AutoBuildResult {
  success: boolean
  mods: string[]
  foundation: string[]
}

export interface KeybindResolveResult {
  success: boolean
  changes: number
}

export interface AppInitData {
  appName: string
  appAccent: string
  version: string
  greeting: string
  plugins_js: string[]
}

export interface DashboardStats {
  size: string
  saves: string
  playtime: string
  java: string
}

export interface ServerPingResultDto {
  online: boolean
  ping?: number | null
  motd?: string | null
  players?: string | null
  icon?: string | null
}

export type ServerPingResult = ServerPingResultDto

export interface GraphNodeColor {
  background: string
  border: string
}

export interface GraphNode {
  id: string
  label: string
  shape: string
  size: number
  color: GraphNodeColor
}

export interface GraphEdgeColor {
  color: string
}

export interface GraphEdge {
  from: string
  to: string
  color: GraphEdgeColor
  arrows: string
}

export interface ModGraphDataDto {
  nodes: GraphNode[]
  edges: GraphEdge[]
}

export interface StoreItemFileDto {
  filename: string
  url: string
  primary: boolean
  size: number
}

export type StoreItemFile = StoreItemFileDto

export interface StoreItemDependencyDto {
  project_id: string
  version_id?: string | null
  dependency_type: string
  file_name?: string | null
}

export interface StoreItemVersionDto {
  id: string
  version_number: string
  name: string
  date: string
  changelog: string
  files: StoreItemFileDto[]
  dependencies: StoreItemDependencyDto[]
  game_versions: string[]
  loaders: string[]
}

export type StoreItemVersion = StoreItemVersionDto

export interface StoreItemGalleryDto {
  url: string
  title?: string | null
}

export interface StoreItemDetailsDto {
  body: string
  gallery: StoreItemGalleryDto[]
}

export type StoreItemDetailsData = StoreItemDetailsDto

export interface StoreItemRecordDto {
  project_id: string
  slug: string
  title: string
  author: string
  description: string
  icon_url: string
  downloads: number
  follows: number
  categories: string[]
  provider: 'modrinth' | 'curseforge'
  project_type: string
  is_installed: boolean
  installed_filename?: string | null
  downloading?: boolean
  progress?: number
  status_text?: string
}

export interface StoreSearchResultDto {
  success: boolean
  hits: StoreItemRecordDto[]
  total_hits: number
  msg?: string | null
}

export interface StoreDetailsResponseDto {
  success: boolean
  details: StoreItemDetailsDto
  versions: StoreItemVersionDto[]
  msg?: string | null
}

export type StoreDetailsResponse = StoreDetailsResponseDto

export interface StoreInstallResultDto {
  success: boolean
  filename: string
  installed_dependencies: string[]
  message: string
}

export interface JavaValidationResultDto {
  valid: boolean
  version_str: string
  major: number
  message: string
}

export interface AiTestResultDto {
  success: boolean
  latency_ms: number
  message: string
}

export interface ThreatIndicatorDto {
  category: string
  title: string
  severity: string
  weight: number
  location: string
}

export interface FileSecurityReportDto {
  filepath: string
  filename: string
  sha256: string
  threat_score: number
  threat_level: 'CLEAN' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL'
  is_clean: boolean
  entropy: number
  indicators: ThreatIndicatorDto[]
  file_size: number
}

export interface ShieldScanReportDto {
  total_scanned: number
  clean_count: number
  threat_count: number
  critical_count: number
  threats: FileSecurityReportDto[]
  timestamp: string
}

export interface QuarantineRecordDto {
  id: string
  original_path: string
  filename: string
  isolated_filename: string
  threat_name: string
  threat_score: number
  sha256: string
  quarantined_at: string
  size_bytes: number
}

export interface HubPreset {
  id: string
  title: string
  author: string
  description: string
  preset: string[]
}

export interface SwarmStatusDto {
  name: string
  seeders: number
  peers: number
  upload_rate: number
  total_upload: number
}

export interface ToolExecutionResult {
  success: boolean
  msg: string
  clipboard?: string | null
  doctor_res?: Record<string, unknown> | null
  threats?: Array<Record<string, unknown>> | null
}

export interface ImportDroppedModsResult {
  success: boolean
  count: number
  msg?: string
}

export interface AppUpdateCheckResult {
  has_update: boolean
  version?: string
  body?: string
  error?: string
}

export interface ContentInspectionDto {
  detected_loader: string
  detected_version: string
  mod_count: usizeNumber
  incompatible_mods: string[]
  recommended_ram_gb: number
  is_clean: boolean
}

export type usizeNumber = number

export interface PreflightIssueDto {
  level: string
  title: string
  description: string
  auto_fixable: boolean
}

export interface PreflightReportDto {
  ready_to_launch: boolean
  java_compatible: boolean
  java_version: string
  java_path: string
  issues: PreflightIssueDto[]
  memory_allocated_gb: number
  total_system_memory_gb: number
}

export interface AutoRepairResultDto {
  success: boolean
  fixed_count: number
  message: string
}

export interface ModpackImportResultDto {
  success: boolean
  pack_name: string
  mc_version: string
  loader: string
  mod_count: number
  message: string
}

export interface TauriEventCallbackMap {
  onDaemonStatus?: (running: boolean, status: string) => void
  onConsoleLine?: (line: string) => void
  onCrashAlert?: (logData: string) => void
  onLaunchStatus?: (msg: string) => void
  onLaunchProgress?: (progress: number) => void
  onTunnelStatus?: (msg: string) => void
  onToggleOverlay?: () => void
}

declare global {
  interface Window {
    __TAURI__?: {
      core?: {
        invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>
      }
      event?: {
        listen: <T = unknown>(
          event: string,
          handler: (e: { payload: T }) => void
        ) => Promise<() => void>
      }
    }
    __TAURI_INTERNALS__?: {
      invoke: <T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>
      listen: <T = unknown>(
        event: string,
        handler: (e: { payload: T }) => void
      ) => Promise<() => void>
    }
  }
}

function getInvokeHandler(): (<T = unknown>(cmd: string, args?: Record<string, unknown>) => Promise<T>) | null {
  if (window.__TAURI__?.core?.invoke) {
    return window.__TAURI__.core.invoke
  }
  if (window.__TAURI_INTERNALS__?.invoke) {
    return window.__TAURI_INTERNALS__.invoke
  }
  return null
}

function getListenHandler(): (<T = unknown>(
  event: string,
  handler: (e: { payload: T }) => void
) => Promise<() => void>) | null {
  if (window.__TAURI__?.event?.listen) {
    return window.__TAURI__.event.listen
  }
  if (window.__TAURI_INTERNALS__?.listen) {
    return window.__TAURI_INTERNALS__.listen
  }
  return null
}

export async function invokeSafe<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  const invoker = getInvokeHandler()
  if (!invoker) {
    throw new Error(`Tauri IPC is not initialized. Cannot invoke command: ${command}`)
  }
  return await invoker<T>(command, args)
}

export const bridge = {
  async importModpackOrArchive(filePath: string): Promise<ModpackImportResultDto> {
    return await invokeSafe<ModpackImportResultDto>('import_modpack_or_archive', { filePath })
  },

  async autoTuneRam(ramGb: number): Promise<boolean> {
    return await invokeSafe<boolean>('auto_tune_ram', { ramGb })
  },

  async inspectInstalledContent(): Promise<ContentInspectionDto> {
    return await invokeSafe<ContentInspectionDto>('inspect_installed_content')
  },

  async preflightInspection(version: string, loader: string): Promise<PreflightReportDto> {
    return await invokeSafe<PreflightReportDto>('preflight_inspection', { version, loader })
  },

  async autoRepairInstance(version: string, loader: string): Promise<AutoRepairResultDto> {
    return await invokeSafe<AutoRepairResultDto>('auto_repair_instance', { version, loader })
  },

  async getInitData(): Promise<AppInitData> {
    return await invokeSafe<AppInitData>('get_init_data')
  },

  async getSettings(): Promise<Record<string, unknown>> {
    return await invokeSafe<Record<string, unknown>>('get_settings')
  },

  async saveSetting(key: string, value: unknown): Promise<boolean> {
    return await invokeSafe<boolean>('save_setting', { key, value })
  },

  async resetSettingsToDefault(): Promise<Record<string, unknown>> {
    return await invokeSafe<Record<string, unknown>>('reset_settings_to_default')
  },

  async validateJavaBinary(path: string): Promise<JavaValidationResultDto> {
    return await invokeSafe<JavaValidationResultDto>('validate_java_binary', { path })
  },

  async testAiConnection(provider: string, model?: string | null, endpoint?: string | null): Promise<AiTestResultDto> {
    return await invokeSafe<AiTestResultDto>('test_ai_connection', { provider, model: model || null, endpoint: endpoint || null })
  },

  async vacuumDatabase(): Promise<string> {
    return await invokeSafe<string>('vacuum_database')
  },

  async openInstanceFolder(): Promise<void> {
    return await invokeSafe<void>('open_instance_folder')
  },

  async shieldScanFull(targetDir?: string | null): Promise<ShieldScanReportDto> {
    return await invokeSafe<ShieldScanReportDto>('shield_scan_full', { targetDir: targetDir || null })
  },

  async shieldScanFile(filepath: string): Promise<FileSecurityReportDto> {
    return await invokeSafe<FileSecurityReportDto>('shield_scan_file', { filepath })
  },

  async shieldQuarantineThreat(filepath: string): Promise<QuarantineRecordDto> {
    return await invokeSafe<QuarantineRecordDto>('shield_quarantine_threat', { filepath })
  },

  async shieldRestoreThreat(quarantineId: string): Promise<boolean> {
    return await invokeSafe<boolean>('shield_restore_threat', { quarantineId })
  },

  async shieldShredThreat(quarantineId: string): Promise<boolean> {
    return await invokeSafe<boolean>('shield_shred_threat', { quarantineId })
  },

  async shieldGetVault(): Promise<QuarantineRecordDto[]> {
    return await invokeSafe<QuarantineRecordDto[]>('shield_get_vault')
  },

  async getTranslations(lang: string): Promise<Record<string, string>> {
    return await invokeSafe<Record<string, string>>('get_translations', { lang })
  },

  async getDashboardStats(): Promise<DashboardStats> {
    return await invokeSafe<DashboardStats>('get_dashboard_stats')
  },

  async getMcVersions(): Promise<string[]> {
    return await invokeSafe<string[]>('get_mc_versions')
  },

  async getLoaderVersions(loader: string, mcVersion: string): Promise<string[]> {
    return await invokeSafe<string[]>('get_loader_versions', { loader, mcVersion })
  },

  async launchGame(version: string, loader: string, loaderVersion?: string | null): Promise<LaunchResult> {
    return await invokeSafe<LaunchResult>('launch_game', {
      version,
      loader,
      loaderVersion: loaderVersion || null,
    })
  },

  async changeInstance(newDir: string): Promise<boolean> {
    return await invokeSafe<boolean>('change_instance', { newDir })
  },

  async getLocalMods(contentType?: string): Promise<LocalModRecord[]> {
    return await invokeSafe<LocalModRecord[]>('get_local_mods', { contentType: contentType || 'mods' })
  },

  async toggleMod(filename: string, contentType?: string): Promise<boolean> {
    return await invokeSafe<boolean>('toggle_mod', { filename, contentType: contentType || 'mods' })
  },

  async deleteMod(filename: string, contentType?: string): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('delete_mod', { filename, contentType: contentType || 'mods' })
  },

  async batchToggleMods(filenames: string[], enable: boolean, contentType?: string): Promise<number> {
    return await invokeSafe<number>('batch_toggle_mods', { filenames, enable, contentType: contentType || 'mods' })
  },

  async batchDeleteMods(filenames: string[], contentType?: string): Promise<number> {
    return await invokeSafe<number>('batch_delete_mods', { filenames, contentType: contentType || 'mods' })
  },

  async openContentFolder(contentType: string): Promise<void> {
    return await invokeSafe<void>('open_content_folder', { contentType })
  },

  async searchStore(
    provider: string,
    query?: string | null,
    projectType?: string | null,
    loader?: string | null,
    gameVersion?: string | null,
    category?: string | null,
    sortIndex?: string | null,
    offset?: number | null
  ): Promise<StoreSearchResultDto> {
    return await invokeSafe<StoreSearchResultDto>('search_store', {
      provider,
      query: query || null,
      projectType: projectType || null,
      loader: loader || null,
      gameVersion: gameVersion || null,
      category: category || null,
      sortIndex: sortIndex || null,
      offset: offset || 0,
    })
  },

  async getStoreFullDetails(
    provider: string,
    projectId: string,
    loader?: string | null,
    gameVersion?: string | null
  ): Promise<StoreDetailsResponseDto> {
    return await invokeSafe<StoreDetailsResponseDto>('get_store_full_details', {
      provider,
      projectId,
      loader: loader || null,
      gameVersion: gameVersion || null,
    })
  },

  async downloadStoreItem(
    provider: string,
    projectId: string,
    versionId: string | null,
    url: string,
    filename: string,
    projectType: string,
    loader?: string | null,
    gameVersion?: string | null
  ): Promise<StoreInstallResultDto> {
    return await invokeSafe<StoreInstallResultDto>('download_store_item', {
      provider,
      projectId,
      versionId,
      url,
      filename,
      projectType,
      loader: loader || null,
      gameVersion: gameVersion || null,
    })
  },

  async downloadSpecificFile(
    provider: string | null,
    projectId: string | null,
    versionId: string | null,
    url: string,
    filename: string,
    projectType: string,
    loader?: string | null,
    gameVersion?: string | null
  ): Promise<StoreInstallResultDto> {
    return await invokeSafe<StoreInstallResultDto>('download_specific_file', {
      provider,
      projectId,
      versionId,
      url,
      filename,
      projectType,
      loader: loader || null,
      gameVersion: gameVersion || null,
    })
  },

  async uninstallStoreItem(projectType: string, filename: string): Promise<boolean> {
    return await invokeSafe<boolean>('uninstall_store_item', {
      projectType,
      filename,
    })
  },

  async generateAutoBuild(prompt: string, mcVersion: string, loader: string): Promise<AutoBuildResult> {
    return await invokeSafe<AutoBuildResult>('generate_auto_build', { prompt, mcVersion, loader })
  },

  async resolveKeybinds(): Promise<KeybindResolveResult> {
    return await invokeSafe<KeybindResolveResult>('resolve_keybinds')
  },

  async checkModUpdates(): Promise<{ success: boolean; updates: Array<{ filename: string; project_id: string; name: string }> }> {
    return await invokeSafe('check_mod_updates')
  },

  async applyModUpdates(updates: Array<{ filename: string; project_id: string; name: string }>): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('apply_mod_updates', { updates })
  },

  async exportModpack(): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('export_modpack')
  },

  async fetchHub(): Promise<HubPreset[]> {
    return await invokeSafe<HubPreset[]>('fetch_hub')
  },

  async publishHub(title: string, author: string, desc: string, mods: string[]): Promise<boolean> {
    return await invokeSafe<boolean>('publish_hub', { title, author, desc, mods })
  },

  async swarmDownload(magnet: string, targetDir: string): Promise<{ success: boolean; msg?: string }> {
    return await invokeSafe<{ success: boolean; msg?: string }>('swarm_download', { magnet, targetDir })
  },

  async swarmSeedStatus(): Promise<SwarmStatusDto[]> {
    return await invokeSafe<SwarmStatusDto[]>('swarm_seed_status')
  },

  async importDroppedMods(files: string[]): Promise<ImportDroppedModsResult> {
    return await invokeSafe<ImportDroppedModsResult>('import_dropped_mods', { files })
  },

  async importModsDialog(): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('import_mods_dialog')
  },

  async getFriends(): Promise<FriendRecord[]> {
    return await invokeSafe<FriendRecord[]>('get_friends')
  },

  async addFriend(name: string): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('add_friend', { name })
  },

  async removeFriend(name: string): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('remove_friend', { name })
  },

  async startTunnel(port: string): Promise<boolean> {
    return await invokeSafe<boolean>('start_tunnel', { port })
  },

  async stopTunnel(): Promise<boolean> {
    return await invokeSafe<boolean>('stop_tunnel')
  },

  async msAuthStart(): Promise<{ verification_uri?: string; user_code?: string; device_code: string }> {
    return await invokeSafe('ms_auth_start')
  },

  async msAuthPoll(deviceCode: string): Promise<boolean> {
    return await invokeSafe<boolean>('ms_auth_poll', { deviceCode })
  },

  async msLogout(): Promise<boolean> {
    return await invokeSafe<boolean>('ms_logout')
  },

  async getMsProfile(): Promise<{ success: boolean; name?: string; uuid?: string; msg?: string }> {
    return await invokeSafe('get_ms_profile')
  },

  async kipLogin(username: string, password: string): Promise<{ success: boolean; token?: string; username?: string; msg?: string }> {
    return await invokeSafe('kip_login', { username, password })
  },

  async kipRegister(username: string, email: string | null, password: string): Promise<{ success: boolean; token?: string; username?: string; msg?: string }> {
    return await invokeSafe('kip_register', { username, email, password })
  },

  async getKipProfile(): Promise<{ success: boolean; username?: string; token?: string }> {
    return await invokeSafe('get_kip_profile')
  },

  async kipLogout(): Promise<boolean> {
    return await invokeSafe<boolean>('kip_logout')
  },

  async partyInvitePrepare(): Promise<{ mods: string[]; tunnel_url: string }> {
    return await invokeSafe('party_invite_prepare')
  },

  async toggleOverlay(): Promise<boolean> {
    return await invokeSafe<boolean>('toggle_overlay')
  },

  async toggleBigPicture(): Promise<boolean> {
    return await invokeSafe<boolean>('toggle_big_picture')
  },

  async setMiniMode(mini: boolean): Promise<boolean> {
    return await invokeSafe<boolean>('set_mini_mode', { mini })
  },

  async pickFile(): Promise<string> {
    return await invokeSafe<string>('pick_file')
  },

  async windowMinimize(): Promise<void> {
    return await invokeSafe<void>('window_minimize')
  },

  async windowMaximize(): Promise<void> {
    return await invokeSafe<void>('window_maximize')
  },

  async windowClose(): Promise<void> {
    return await invokeSafe<void>('window_close')
  },

  async checkAppUpdate(): Promise<AppUpdateCheckResult> {
    return await invokeSafe<AppUpdateCheckResult>('check_app_update')
  },

  async performAppUpdate(): Promise<boolean> {
    return await invokeSafe<boolean>('perform_app_update')
  }
}

export function setupTauriListeners(callbacks: TauriEventCallbackMap = {}): void {
  const listen = getListenHandler()
  if (!listen) {
    return
  }

  if (callbacks.onDaemonStatus) {
    listen<{ running: boolean; status: string }>('updateDaemonStatus', (event) => {
      callbacks.onDaemonStatus?.(event.payload.running, event.payload.status)
    })
  }

  if (callbacks.onConsoleLine) {
    listen<string>('appendConsoleLine', (event) => {
      callbacks.onConsoleLine?.(event.payload)
    })
  }

  if (callbacks.onCrashAlert) {
    listen<string>('showCrashAlert', (event) => {
      callbacks.onCrashAlert?.(event.payload)
    })
  }

  if (callbacks.onLaunchStatus) {
    listen<string>('updateLaunchStatus', (event) => {
      callbacks.onLaunchStatus?.(event.payload)
    })
  }

  if (callbacks.onLaunchProgress) {
    listen<number>('updateLaunchProgress', (event) => {
      callbacks.onLaunchProgress?.(event.payload)
    })
  }

  if (callbacks.onTunnelStatus) {
    listen<string>('updateTunnelStatus', (event) => {
      callbacks.onTunnelStatus?.(event.payload)
    })
  }

  if (callbacks.onToggleOverlay) {
    listen<void>('toggleOverlay', () => {
      callbacks.onToggleOverlay?.()
    })
  }
}