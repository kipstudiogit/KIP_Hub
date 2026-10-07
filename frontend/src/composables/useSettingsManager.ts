import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  AppSettingsDto,
  JavaValidationResultDto,
  DetectedJavaRuntimeDto,
  AiTestResultDto,
} from '../types/settings'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useSettingsManager() {
  const settings = ref<AppSettingsDto>({
    lang: 'en',
    autostart: false,
    theme: 'Indigo',
    customColor: '',
    appearance: 'Dark',
    mica: true,
    rpc: true,
    scale: 1.0,
    instances: [],
    currentInstance: '',
    autoBackup: false,
    pteroUrl: '',
    lowGraphics: false,
    aiProvider: 'google',
    aiModel: 'gemini-1.5-flash',
    ollamaUrl: 'http://localhost:11434',
    closeOnLaunch: false,
    ramAllocation: 0,
    jvmGc: 'G1GC',
    jvmPreset: 'balanced',
    shieldAutoScan: true,
    voiceNoiseSuppression: true,
    eulaAccepted: false,
    telemetryOptIn: false,
    kipUsername: '',
    offlineUsername: 'Player',
    gameResolution: '1920x1080',
    gameFullscreen: false,
    customJavaPath: '',
    customJvmArgs: '',
    themeAccent: 'indigo',
    aiApiKey: '',
    openaiApiKey: '',
    anthropicApiKey: '',
    cfApiKey: '',
    safeMode: false,
  })

  const isLoading = ref(false)
  const isValidatingJava = ref(false)
  const isDetectingJava = ref(false)
  const isTestingAi = ref(false)
  const isVacuuming = ref(false)

  const javaValidationData = ref<JavaValidationResultDto | null>(null)
  const detectedRuntimes = ref<DetectedJavaRuntimeDto[]>([])

  const compiledJvmPreview = computed(() => {
    const ram = settings.value.ramAllocation > 0 ? settings.value.ramAllocation : 4
    let gcFlag = '-XX:+UseG1GC -XX:+ParallelRefProcEnabled'

    if (settings.value.jvmGc === 'ZGC') {
      gcFlag = '-XX:+UseZGC -XX:+ZGenerational'
    } else if (settings.value.jvmGc === 'Shenandoah') {
      gcFlag = '-XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu'
    } else if (settings.value.jvmGc === 'Parallel') {
      gcFlag = '-XX:+UseParallelGC'
    }

    const custom = settings.value.customJvmArgs.trim()
    return `-Xmx${ram}G -Xms256M ${gcFlag} ${custom}`.trim()
  })

  function applyRuntimeVisuals(cfg: AppSettingsDto): void {
    state.settings.low_graphics = cfg.lowGraphics
    state.settings.mica = cfg.mica
    state.settings.theme_accent = cfg.themeAccent
    state.settings.lang = cfg.lang

    if (cfg.lowGraphics) {
      document.documentElement.classList.add('low-graphics-mode')
      document.body.classList.add('low-graphics-mode')
    } else {
      document.documentElement.classList.remove('low-graphics-mode')
      document.body.classList.remove('low-graphics-mode')
    }

    if (!cfg.mica) {
      document.documentElement.classList.add('no-blur-mode')
      document.body.classList.add('no-blur-mode')
    } else {
      document.documentElement.classList.remove('no-blur-mode')
      document.body.classList.remove('no-blur-mode')
    }

    if (cfg.scale && cfg.scale !== 1.0) {
      document.documentElement.style.zoom = String(cfg.scale)
      document.body.style.zoom = String(cfg.scale)
    }

    const accent = cfg.themeAccent || 'indigo'
    document.documentElement.setAttribute('data-accent', accent)
    document.body.setAttribute('data-accent', accent)
  }

  async function loadSettings(): Promise<void> {
    isLoading.value = true
    try {
      const data = await invoke<AppSettingsDto>('get_app_settings')
      settings.value = data
      applyRuntimeVisuals(data)
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isLoading.value = false
    }
  }

  async function commitSettings(): Promise<boolean> {
    try {
      state.settings.lang = settings.value.lang
      applyRuntimeVisuals(settings.value)
      await invoke<boolean>('update_app_settings', { settings: settings.value })
      showToast(t('Saved'), t('Settings synchronized.'), 'success')
      return true
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
      return false
    }
  }

  async function validateJava(): Promise<void> {
    if (!settings.value.customJavaPath.trim()) return
    isValidatingJava.value = true
    try {
      const res = await invoke<JavaValidationResultDto>('validate_java_executable', {
        path: settings.value.customJavaPath.trim(),
      })
      javaValidationData.value = res
      if (res.valid) {
        showToast(t('Java Verified'), res.message, 'success')
      } else {
        showToast(t('Validation Error'), res.message, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isValidatingJava.value = false
    }
  }

  async function detectSystemJava(): Promise<void> {
    isDetectingJava.value = true
    try {
      detectedRuntimes.value = await invoke<DetectedJavaRuntimeDto[]>('detect_system_java_runtimes')
      showToast(t('Scan Complete'), `Found ${detectedRuntimes.value.length} compatible OpenJDK runtimes.`, 'success')
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isDetectingJava.value = false
    }
  }

  async function pickCustomInstancePath(): Promise<void> {
    try {
      const chosen = await invoke<string>('pick_instance_directory')
      if (chosen && chosen.trim()) {
        settings.value.currentInstance = chosen.trim()
        await commitSettings()
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  async function testAiConnection(): Promise<void> {
    isTestingAi.value = true
    try {
      const res = await invoke<AiTestResultDto>('test_neural_connection', {
        provider: settings.value.aiProvider,
      })
      if (res.success) {
        showToast(t('Neural Link Online'), `${res.message} (${res.latencyMs} ms)`, 'success')
      } else {
        showToast(t('Connection Failed'), res.message, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isTestingAi.value = false
    }
  }

  async function runVacuum(): Promise<void> {
    isVacuuming.value = true
    try {
      const msg = await invoke<string>('vacuum_sqlite_database')
      showToast(t('Database Optimized'), msg, 'success')
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isVacuuming.value = false
    }
  }

  async function restoreFactoryDefaults(): Promise<void> {
    try {
      const defaults = await invoke<AppSettingsDto>('reset_app_settings_default')
      settings.value = defaults
      applyRuntimeVisuals(defaults)
      showToast(t('Factory Reset'), 'Engine settings restored.', 'info')
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  async function openActiveDir(): Promise<void> {
    try {
      await invoke('open_instance_directory')
    } catch {
      showToast(t('Error'), 'Could not open native file explorer.', 'danger')
    }
  }

  return {
    settings,
    isLoading,
    isValidatingJava,
    isDetectingJava,
    isTestingAi,
    isVacuuming,
    javaValidationData,
    detectedRuntimes,
    compiledJvmPreview,
    applyRuntimeVisuals,
    loadSettings,
    commitSettings,
    validateJava,
    detectSystemJava,
    pickCustomInstancePath,
    testAiConnection,
    runVacuum,
    restoreFactoryDefaults,
    openActiveDir,
  }
}