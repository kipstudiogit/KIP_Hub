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
    mica: true,
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
  version: '2.0.0'
})

export async function loadSettings(): Promise<void> {
  try {
    const s = await bridge.getSettings()
    if (s) {
      const activeInstance = (s.current_instance || s.currentInstance || s.mc_dir || '') as string
      state.settings.mc_dir = activeInstance
      state.settings.auto_backup = Boolean(s.auto_backup ?? s.autoBackup)
      state.settings.rpc = (s.rpc !== false)
      state.settings.ai_provider = (s.ai_provider || s.aiProvider || 'google') as string
      state.settings.ai_model = (s.ai_model || s.aiModel || 'gemini-1.5-flash') as string
      state.settings.ai_api_key = (s.ai_api_key || s.aiApiKey || '') as string
      state.settings.openai_api_key = (s.openai_api_key || s.openaiApiKey || '') as string
      state.settings.anthropic_api_key = (s.anthropic_api_key || s.anthropicApiKey || '') as string
      state.settings.ollama_url = (s.ollama_url || s.ollamaUrl || 'http://localhost:11434') as string
      state.settings.cf_api_key = (s.cf_api_key || s.cfApiKey || '') as string
      state.settings.lang = (s.lang || 'en') as string
      state.settings.autostart = Boolean(s.autostart)
      state.settings.safe_mode = Boolean(s.safe_mode ?? s.safeMode)
      state.settings.low_graphics = Boolean(s.low_graphics ?? s.lowGraphics)
      state.settings.mica = Boolean(s.mica ?? true)
      state.settings.close_on_launch = Boolean(s.close_on_launch ?? s.closeOnLaunch)
      state.settings.ram_allocation = typeof s.ram_allocation === 'number' ? s.ram_allocation : (typeof s.ramAllocation === 'number' ? s.ramAllocation : 0)
      state.settings.jvm_gc = (s.jvm_gc || s.jvmGc || 'G1GC') as string
      state.settings.jvm_preset = (s.jvm_preset || s.jvmPreset || 'balanced') as string
      state.settings.shield_auto_scan = (s.shield_auto_scan ?? s.shieldAutoScan) !== false
      state.settings.voice_noise_suppression = (s.voice_noise_suppression ?? s.voiceNoiseSuppression) !== false
      state.settings.eula_accepted = Boolean(s.eula_accepted ?? s.eulaAccepted)
      state.settings.telemetry_opt_in = Boolean(s.telemetry_opt_in ?? s.telemetryOptIn)
      state.settings.instances = Array.isArray(s.instances) ? (s.instances as string[]) : (activeInstance ? [activeInstance] : [])
      state.settings.offline_username = (s.offline_username || s.offlineUsername || 'Player') as string
      state.settings.game_resolution = (s.game_resolution || s.gameResolution || '1920x1080') as string
      state.settings.game_fullscreen = Boolean(s.game_fullscreen ?? s.gameFullscreen)
      state.settings.custom_java_path = (s.custom_java_path || s.customJavaPath || '') as string
      state.settings.custom_jvm_args = (s.custom_jvm_args || s.customJvmArgs || '') as string
      state.settings.theme_accent = (s.theme_accent || s.themeAccent || 'indigo') as string
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
