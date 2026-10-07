import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event'

export type { LaunchResultDto } from './types/launcher'
export type { ContentActionResultDto } from './types/content'
export type {
  ToolExecutionResultDto,
  ToolExecutionResultDto as ToolExecutionResult,
} from './types/tools'
export type {
  StoreDetailsResponseDto,
  StoreItemFileDto,
  StoreItemRecordDto,
} from './types/store'

export interface ServerPingResultDto {
  online: boolean
  ping?: number | null
  motd?: string | null
  players?: string | null
  icon?: string | null
}
export type ServerPingResult = ServerPingResultDto

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

export interface DashboardStats {
  size: string
  saves: string
  playtime: string
  java: string
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

export interface AppUpdateCheckResult {
  has_update: boolean
  version?: string
  body?: string
  error?: string
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

export interface TauriEventCallbackMap {
  onDaemonStatus?: (running: boolean, status: string) => void
  onConsoleLine?: (line: string) => void
  onCrashAlert?: (logData: string) => void
  onLaunchStatus?: (msg: string) => void
  onLaunchProgress?: (progress: number) => void
  onTunnelStatus?: (msg: string) => void
  onToggleOverlay?: () => void
}

export async function invokeSafe<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  return await invoke<T>(command, args)
}

export const bridge = {
  async getInitData(): Promise<any> {
    return await invoke('get_init_data')
  },

  async getSettings(): Promise<Record<string, unknown>> {
    return await invoke<Record<string, unknown>>('get_settings')
  },

  async saveSetting(key: string, value: unknown): Promise<boolean> {
    return await invoke<boolean>('save_setting', { key, value })
  },

  async resetSettingsToDefault(): Promise<Record<string, unknown>> {
    return await invoke<Record<string, unknown>>('reset_app_settings_default')
  },

  async validateJavaExecutable(path: string): Promise<any> {
    return await invoke('validate_java_executable', { path })
  },

  async testAiConnection(provider: string): Promise<any> {
    return await invoke('test_neural_connection', { provider })
  },

  async vacuumDatabase(): Promise<string> {
    return await invoke<string>('vacuum_sqlite_database')
  },

  async openInstanceFolder(): Promise<void> {
    return await invoke<void>('open_instance_directory')
  },

  async getTranslations(lang: string): Promise<Record<string, string>> {
    return await invoke<Record<string, string>>('get_translations', { lang })
  },

  async getDashboardStats(): Promise<DashboardStats> {
    return await invoke<DashboardStats>('get_dashboard_stats')
  },

  async getMcVersions(): Promise<string[]> {
    return await invoke<string[]>('get_mc_versions')
  },

  async getLoaderVersions(loader: string, mcVersion: string): Promise<string[]> {
    return await invoke<string[]>('get_loader_versions', { loader, mcVersion })
  },

  async getLocalMods(contentType?: string): Promise<LocalModRecord[]> {
    return await invoke<LocalModRecord[]>('get_local_mods', { contentType: contentType || 'mods' })
  },

  async toggleMod(filename: string, contentType?: string): Promise<boolean> {
    return await invoke<boolean>('toggle_mod', { filename, contentType: contentType || 'mods' })
  },

  async deleteMod(filename: string, contentType?: string): Promise<GenericActionResult> {
    return await invoke<GenericActionResult>('delete_mod', { filename, contentType: contentType || 'mods' })
  },

  async batchToggleMods(filenames: string[], enable: boolean, contentType?: string): Promise<number> {
    return await invoke<number>('batch_toggle_mods', { filenames, enable, contentType: contentType || 'mods' })
  },

  async batchDeleteMods(filenames: string[], contentType?: string): Promise<number> {
    return await invoke<number>('batch_delete_mods', { filenames, contentType: contentType || 'mods' })
  },

  async openContentFolder(contentType: string): Promise<void> {
    return await invoke<void>('open_content_folder', { contentType })
  },

  async generateAutoBuild(prompt: string, mcVersion: string, loader: string): Promise<AutoBuildResult> {
    return await invoke<AutoBuildResult>('generate_auto_build', { prompt, mcVersion, loader })
  },

  async resolveKeybinds(): Promise<KeybindResolveResult> {
    return await invoke<KeybindResolveResult>('resolve_keybinds')
  },

  async fetchHub(): Promise<HubPreset[]> {
    return await invoke<HubPreset[]>('fetch_hub')
  },

  async publishHub(title: string, author: string, desc: string, mods: string[]): Promise<boolean> {
    return await invoke<boolean>('publish_hub', { title, author, desc, mods })
  },

  async swarmDownload(magnet: string, targetDir: string): Promise<{ success: boolean; msg?: string }> {
    return await invoke('swarm_download', { magnet, targetDir })
  },

  async swarmSeedStatus(): Promise<SwarmStatusDto[]> {
    return await invoke<SwarmStatusDto[]>('swarm_seed_status')
  },

  async importDroppedMods(files: string[]): Promise<import('./types/content').ContentActionResultDto> {
    return await invoke<import('./types/content').ContentActionResultDto>('import_dropped_content', { files })
  },

  async getFriends(): Promise<FriendRecord[]> {
    return await invoke<FriendRecord[]>('get_friends')
  },

  async addFriend(name: string): Promise<GenericActionResult> {
    return await invoke<GenericActionResult>('add_friend', { name })
  },

  async removeFriend(name: string): Promise<GenericActionResult> {
    return await invoke<GenericActionResult>('remove_friend', { name })
  },

  async startTunnel(port: string): Promise<boolean> {
    return await invoke<boolean>('start_tunnel', { port })
  },

  async stopTunnel(): Promise<boolean> {
    return await invoke<boolean>('stop_tunnel')
  },

  async msAuthStart(): Promise<{ verification_uri?: string; user_code?: string; device_code: string }> {
    return await invoke('ms_auth_start')
  },

  async msAuthPoll(deviceCode: string): Promise<boolean> {
    return await invoke<boolean>('ms_auth_poll', { deviceCode })
  },

  async msLogout(): Promise<boolean> {
    return await invoke<boolean>('ms_logout')
  },

  async getMsProfile(): Promise<{ success: boolean; name?: string; uuid?: string; msg?: string }> {
    return await invoke('get_ms_profile')
  },

  async kipLogin(username: string, password: string): Promise<any> {
    return await invoke('kip_login', { username, password })
  },

  async kipRegister(username: string, email: string | null, password: string): Promise<any> {
    return await invoke('kip_register', { username, email, password })
  },

  async getKipProfile(): Promise<any> {
    return await invoke('get_kip_profile')
  },

  async kipLogout(): Promise<boolean> {
    return await invoke<boolean>('kip_logout')
  },

  async partyInvitePrepare(): Promise<{ mods: string[]; tunnel_url: string }> {
    return await invoke('party_invite_prepare')
  },

  async toggleOverlay(): Promise<boolean> {
    return await invoke<boolean>('toggle_overlay')
  },

  async toggleBigPicture(): Promise<boolean> {
    return await invoke<boolean>('toggle_big_picture')
  },

  async setMiniMode(mini: boolean): Promise<boolean> {
    return await invoke<boolean>('set_mini_mode', { mini })
  },

  async pickFile(): Promise<string> {
    return await invoke<string>('pick_file')
  },

  async windowMinimize(): Promise<void> {
    return await invoke<void>('window_minimize')
  },

  async windowMaximize(): Promise<void> {
    return await invoke<void>('window_maximize')
  },

  async windowClose(): Promise<void> {
    return await invoke<void>('window_close')
  },

  async checkAppUpdate(): Promise<AppUpdateCheckResult> {
    return await invoke<AppUpdateCheckResult>('check_app_update')
  },

  async performAppUpdate(): Promise<boolean> {
    return await invoke<boolean>('perform_app_update')
  },

  async executeSystemTool(toolId: string): Promise<import('./types/tools').ToolExecutionResultDto> {
    return await invoke<import('./types/tools').ToolExecutionResultDto>('execute_system_tool', {
      toolId,
      mcVersion: null,
      loader: null,
    })
  },
}

export function setupTauriListeners(callbacks: TauriEventCallbackMap = {}): UnlistenFn[] {
  const unlisteners: UnlistenFn[] = []

  if (callbacks.onDaemonStatus) {
    listen<{ running: boolean; status: string }>(
      'updateDaemonStatus',
      (event: Event<{ running: boolean; status: string }>) => {
        callbacks.onDaemonStatus?.(event.payload.running, event.payload.status)
      }
    ).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onConsoleLine) {
    listen<string>('appendConsoleLine', (event: Event<string>) => {
      callbacks.onConsoleLine?.(event.payload)
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onCrashAlert) {
    listen<string>('showCrashAlert', (event: Event<string>) => {
      callbacks.onCrashAlert?.(event.payload)
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onLaunchStatus) {
    listen<string>('updateLaunchStatus', (event: Event<string>) => {
      callbacks.onLaunchStatus?.(event.payload)
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onLaunchProgress) {
    listen<number>('updateLaunchProgress', (event: Event<number>) => {
      callbacks.onLaunchProgress?.(event.payload)
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onTunnelStatus) {
    listen<string>('updateTunnelStatus', (event: Event<string>) => {
      callbacks.onTunnelStatus?.(event.payload)
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  if (callbacks.onToggleOverlay) {
    listen<void>('toggleOverlay', (_event: Event<void>) => {
      callbacks.onToggleOverlay?.()
    }).then((unlisten: UnlistenFn) => unlisteners.push(unlisten))
  }

  return unlisteners
}