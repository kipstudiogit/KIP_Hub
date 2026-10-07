import { ref } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import type {
  AutoBuildRequestDto,
  BuildProgressDto,
  BuildResultDto,
  KeybindResolveReportDto,
} from '../types/builder'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useAutoBuilder() {
  const isBuilding = ref(false)
  const isResolvingKeybinds = ref(false)
  const terminalLogs = ref<string[]>([
    '[INIT] Quantum Modpack Synthesizer primed.',
    '[READY] Select preset or input prompt to ignite synthesis.',
  ])

  const currentProgress = ref<BuildProgressDto>({
    phase: 'AI_SYNTHESIS',
    percent: 0,
    currentStep: 'Standby',
    currentMod: null,
    totalMods: 0,
    downloadedMods: 0,
    logLine: null,
  })

  const lastBuildResult = ref<BuildResultDto | null>(null)
  const keybindReport = ref<KeybindResolveReportDto | null>(null)

  async function startBuild(payload: AutoBuildRequestDto): Promise<BuildResultDto | null> {
    if (isBuilding.value) return null
    isBuilding.value = true
    lastBuildResult.value = null
    terminalLogs.value = [`[START] Synthesis initiated for MC ${payload.mcVersion} (${payload.loader})`]

    const channel = new Channel<BuildProgressDto>()
    channel.onmessage = (progress) => {
      currentProgress.value = progress
      if (progress.logLine) {
        terminalLogs.value.push(progress.logLine)
        if (terminalLogs.value.length > 80) terminalLogs.value.shift()
      }
    }

    try {
      const result = await invoke<BuildResultDto>('execute_auto_build_stream', {
        payload,
        progressChannel: channel,
      })

      lastBuildResult.value = result
      showToast(t('Synthesis Complete'), `Installed ${result.totalInstalled} packages.`, 'success')
      return result
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Synthesis Fault'), error.message || String(err), 'danger')
      terminalLogs.value.push(`[ERROR] ${error.message || String(err)}`)
      return null
    } finally {
      isBuilding.value = false
    }
  }

  async function resolveKeybinds(): Promise<KeybindResolveReportDto | null> {
    isResolvingKeybinds.value = true
    try {
      const report = await invoke<KeybindResolveReportDto>('resolve_keybind_conflicts_detailed')
      keybindReport.value = report
      if (report.conflictsResolved > 0) {
        showToast(t('Resolved'), `Remapped ${report.conflictsResolved} keybind conflicts.`, 'success')
      } else {
        showToast(t('Clean'), 'No keybind conflicts detected in options.txt.', 'info')
      }
      return report
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Keybind Fault'), error.message || String(err), 'danger')
      return null
    } finally {
      isResolvingKeybinds.value = false
    }
  }

  return {
    isBuilding,
    isResolvingKeybinds,
    terminalLogs,
    currentProgress,
    lastBuildResult,
    keybindReport,
    startBuild,
    resolveKeybinds,
  }
}