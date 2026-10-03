import { reactive } from 'vue'
import { bridge } from '../bridge'
import type { AppState } from '../types/app'

export const state = reactive<AppState>({
  appName: 'K.I.P.',
  appAccent: ' Hub',
  greeting: 'Welcome',
  showBoot: true,
  bootProgress: 0,
  bootText: 'INITIALIZING NEURAL CORE...',
  isOverlayActive: false,
  isMiniMode: false,
  isBigPicture: false,
  isNexusOpen: false,
  currentView: 'dashboard',
  partyInvite: null,
  translations: {
    ru: {}, es: {}, de: {}, zh: {}, ja: {}, ko: {}, tr: {}, fr: {}, pt: {}, it: {}, pl: {}, en: {}
  },
  navItems: [
    { id: 'dashboard', label: 'Overview', icon: 'layout-dashboard' },
    { id: 'launcher', label: 'Launcher', icon: 'gamepad-2' },
    { id: 'builder', label: 'Auto-Builder', icon: 'wand-2' },
    { id: 'mods', label: 'Content', icon: 'puzzle' },
    { id: 'store', label: 'Store', icon: 'shopping-cart' },
    { id: 'worlds', label: 'Worlds', icon: 'globe' },
    { id: 'network', label: 'Network', icon: 'wifi' },
    { id: 'tools', label: 'Tools', icon: 'zap' },
    { id: 'media', label: 'Gallery', icon: 'image' },
    { id: 'console', label: 'Console', icon: 'terminal' },
    { id: 'support', label: 'Support', icon: 'life-buoy' },
    { id: 'settings', label: 'Settings', icon: 'settings' }
  ],
  stats: { size: '...', saves: '...', playtime: '...', java: '...' },
  isMcRunning: false,
  mcStatusText: 'Minecraft stopped',
  mcVersions: [],
  launchStatus: 'Ready',
  launchProgress: 0,
  settings: {
    mc_dir: '',
    has_ms_token: false,
    ms_name: '',
    has_kip_token: false,
    kip_username: '',
    auto_backup: false,
    rpc: true,
    ai_provider: 'google',
    ai_model: 'gemini-1.5-flash',
    ai_api_key: '',
    openai_api_key: '',
    anthropic_api_key: '',
    ollama_url: 'http://localhost:11434',
    cf_api_key: '',
    lang: 'en',
    autostart: false,
    safe_mode: false,
    low_graphics: false,
    close_on_launch: false,
    ram_allocation: 0,
    jvm_gc: 'G1GC',
    jvm_preset: 'balanced',
    shield_auto_scan: true,
    voice_noise_suppression: true,
    eula_accepted: false,
    telemetry_opt_in: false,
    instances: [],
    offline_username: 'Player',
    game_resolution: '1920x1080',
    game_fullscreen: false,
    custom_java_path: '',
    custom_jvm_args: '',
    theme_accent: 'indigo'
  },
  friends: [],
  newsText: '',
  consoleHtml: 'Awaiting data stream...',
  _consoleBuffer: [],
  aiInputText: '',
  version: '1.7.0'
})

export async function loadSettings(): Promise<void> {
  try {
    const s = await bridge.getSettings()
    if (s) {
      state.settings.mc_dir = typeof s.mc_dir === 'string' ? s.mc_dir : ''
      state.settings.auto_backup = Boolean(s.auto_backup)
      state.settings.rpc = s.rpc !== false
      state.settings.ai_provider = typeof s.ai_provider === 'string' ? s.ai_provider : 'google'
      state.settings.ai_model = typeof s.ai_model === 'string' ? s.ai_model : 'gemini-1.5-flash'
      state.settings.ai_api_key = typeof s.ai_api_key === 'string' ? s.ai_api_key : ''
      state.settings.openai_api_key = typeof s.openai_api_key === 'string' ? s.openai_api_key : ''
      state.settings.anthropic_api_key = typeof s.anthropic_api_key === 'string' ? s.anthropic_api_key : ''
      state.settings.ollama_url = typeof s.ollama_url === 'string' ? s.ollama_url : 'http://localhost:11434'
      state.settings.cf_api_key = typeof s.cf_api_key === 'string' ? s.cf_api_key : ''
      state.settings.lang = typeof s.lang === 'string' ? s.lang : 'en'
      state.settings.autostart = Boolean(s.autostart)
      state.settings.safe_mode = Boolean(s.safe_mode)
      state.settings.low_graphics = Boolean(s.low_graphics)
      state.settings.close_on_launch = Boolean(s.close_on_launch)
      state.settings.ram_allocation = typeof s.ram_allocation === 'number' ? s.ram_allocation : 0
      state.settings.jvm_gc = typeof s.jvm_gc === 'string' ? s.jvm_gc : 'G1GC'
      state.settings.jvm_preset = typeof s.jvm_preset === 'string' ? s.jvm_preset : 'balanced'
      state.settings.shield_auto_scan = s.shield_auto_scan !== false
      state.settings.voice_noise_suppression = s.voice_noise_suppression !== false
      state.settings.eula_accepted = Boolean(s.eula_accepted)
      state.settings.telemetry_opt_in = Boolean(s.telemetry_opt_in)
      state.settings.instances = Array.isArray(s.instances) ? (s.instances as string[]) : []
      state.settings.offline_username = typeof s.offline_username === 'string' ? s.offline_username : 'Player'
      state.settings.game_resolution = typeof s.game_resolution === 'string' ? s.game_resolution : '1920x1080'
      state.settings.game_fullscreen = Boolean(s.game_fullscreen)
      state.settings.custom_java_path = typeof s.custom_java_path === 'string' ? s.custom_java_path : ''
      state.settings.custom_jvm_args = typeof s.custom_jvm_args === 'string' ? s.custom_jvm_args : ''
      state.settings.theme_accent = typeof s.theme_accent === 'string' ? s.theme_accent : 'indigo'
    }

    const prof = await bridge.getMsProfile()
    if (prof && prof.success) {
      state.settings.has_ms_token = true
      state.settings.ms_name = prof.name || ''
    } else {
      state.settings.has_ms_token = false
      state.settings.ms_name = ''
    }

    const kipProf = await bridge.getKipProfile()
    if (kipProf && kipProf.success) {
      state.settings.has_kip_token = true
      state.settings.kip_username = kipProf.username || ''
    } else {
      state.settings.has_kip_token = false
      state.settings.kip_username = ''
    }
  } catch {
    state.settings.has_ms_token = false
    state.settings.has_kip_token = false
  }
}

export async function loadDashboardStats(): Promise<void> {
  try {
    const s = await bridge.getDashboardStats()
    if (s) {
      state.stats.size = s.size
      state.stats.saves = s.saves
      state.stats.playtime = s.playtime
      state.stats.java = s.java
    }
  } catch {
    state.stats.size = '0 MB'
    state.stats.saves = '0'
    state.stats.playtime = '0h 0m'
    state.stats.java = 'Unknown'
  }
}

export async function saveSetting(key: string, value: unknown): Promise<boolean> {
  try {
    (state.settings as Record<string, unknown>)[key] = value
    const result = await bridge.saveSetting(key, value)
    if (key === 'lang') {
      const { loadTranslations } = await import('../composables/useI18n')
      await loadTranslations()
    }
    return result
  } catch {
    return false
  }
}

export async function acceptEula(): Promise<void> {
  state.settings.eula_accepted = true
  await saveSetting('eula_accepted', true)
}
