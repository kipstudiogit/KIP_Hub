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
  name: string
  version: string
  author: string
  loaders: string[]
  disabled: boolean
  icon: string
}

export interface FriendRecord {
  name: string
  status: string
  avatar: string
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

export interface StoreItemFile {
  filename: string
  url: string
  primary?: boolean
}

export interface StoreItemVersion {
  id: string
  version_number: string
  name: string
  date: string
  changelog?: string
  files: StoreItemFile[]
  expanded?: boolean
  downloading?: boolean
  downloaded?: boolean
  progress?: number
  downloadTarget?: string
}

export interface StoreItemDetailsData {
  body?: string
  gallery?: Array<{ url: string; title?: string }>
}

export interface StoreDetailsResponse {
  success: boolean
  details: StoreItemDetailsData
  versions: StoreItemVersion[]
  msg?: string
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
  async getInitData(): Promise<AppInitData> {
    return await invokeSafe<AppInitData>('get_init_data')
  },

  async getSettings(): Promise<Record<string, unknown>> {
    return await invokeSafe<Record<string, unknown>>('get_settings')
  },

  async saveSetting(key: string, value: unknown): Promise<boolean> {
    return await invokeSafe<boolean>('save_setting', { key, value })
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

  async getLocalMods(): Promise<LocalModRecord[]> {
    return await invokeSafe<LocalModRecord[]>('get_local_mods')
  },

  async toggleMod(filename: string): Promise<boolean> {
    return await invokeSafe<boolean>('toggle_mod', { filename })
  },

  async deleteMod(filename: string): Promise<GenericActionResult> {
    return await invokeSafe<GenericActionResult>('delete_mod', { filename })
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