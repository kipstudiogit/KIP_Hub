import type { DashboardStats, FriendRecord } from '../bridge'

export interface NavItem {
  id: string
  label: string
  icon: string
}

export interface SettingsState {
  mc_dir: string
  has_ms_token: boolean
  ms_name: string
  has_kip_token: boolean
  kip_username: string
  auto_backup: boolean
  rpc: boolean
  ai_provider: string
  ai_model: string
  ai_api_key: string
  openai_api_key: string
  anthropic_api_key: string
  ollama_url: string
  cf_api_key: string
  lang: string
  autostart: boolean
  safe_mode: boolean
  low_graphics: boolean
  mica: boolean
  close_on_launch: boolean
  ram_allocation: number
  jvm_gc: string
  jvm_preset: string
  shield_auto_scan: boolean
  voice_noise_suppression: boolean
  eula_accepted: boolean
  telemetry_opt_in: boolean
  instances: string[]
  offline_username: string
  game_resolution: string
  game_fullscreen: boolean
  custom_java_path: string
  custom_jvm_args: string
  theme_accent: string
}

export interface PartyInviteData {
  senderId: string
  senderName: string
  mods: string[]
  tunnelUrl: string
}

export interface AppState {
  appName: string
  appAccent: string
  greeting: string
  showBoot: boolean
  bootProgress: number
  bootText: string
  isOverlayActive: boolean
  isMiniMode: boolean
  isBigPicture: boolean
  isNexusOpen: boolean
  currentView: string
  partyInvite: PartyInviteData | null
  translations: Record<string, Record<string, string>>
  navItems: NavItem[]
  stats: DashboardStats
  isMcRunning: boolean
  mcStatusText: string
  mcVersions: string[]
  launchStatus: string
  launchProgress: number
  settings: SettingsState
  friends: FriendRecord[]
  newsText: string
  consoleHtml: string
  _consoleBuffer: string[]
  aiInputText: string
  version: string
}

export interface ToastItem {
  id: number
  title: string
  message: string
  type: 'info' | 'success' | 'danger'
  icon: string
}