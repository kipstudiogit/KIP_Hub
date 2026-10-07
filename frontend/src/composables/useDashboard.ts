import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { DashboardOverviewDto } from '../types/dashboard'
import type { ToolExecutionResultDto } from '../types/tools'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useDashboard() {
  const isRefreshing = ref(false)
  const isActionExecuting = ref(false)
  const selectedPreset = ref<'fabric' | 'vanilla' | 'forge' | 'neoforge'>('fabric')

  const overview = ref<DashboardOverviewDto>({
    size: '0 MB',
    saves: '0',
    playtime: '0h 0m',
    java: 'OpenJDK 25',
    modsCount: 0,
    ramUsagePercent: 0,
    totalRamGb: 16,
    usedRamGb: 4,
    cpuUsagePercent: 0,
    cpuBrand: 'System Host CPU',
    lastWorld: null,
    latestScreenshotThumbnail: null,
    storage: {
      totalSizeMb: 0,
      modsSizeMb: 0,
      savesSizeMb: 0,
      screenshotsSizeMb: 0,
      logsSizeMb: 0,
    },
    isGameRunning: false,
    runningGamePid: null,
    currentInstanceName: 'Default (.minecraft)',
  })

  const cpuStatusColor = computed(() => {
    const val = overview.value.cpuUsagePercent
    if (val < 45) return 'text-emerald-400 border-emerald-500/30 bg-emerald-500/10'
    if (val < 75) return 'text-amber-400 border-amber-500/30 bg-amber-500/10'
    return 'text-rose-400 border-rose-500/30 bg-rose-500/10'
  })

  const ramStatusColor = computed(() => {
    const val = overview.value.ramUsagePercent
    if (val < 60) return 'text-cyan-400 border-cyan-500/30 bg-cyan-500/10'
    if (val < 85) return 'text-amber-400 border-amber-500/30 bg-amber-500/10'
    return 'text-rose-400 border-rose-500/30 bg-rose-500/10'
  })

  async function fetchOverview(): Promise<void> {
    isRefreshing.value = true
    try {
      const data = await invoke<DashboardOverviewDto>('get_dashboard_overview')
      overview.value = data
      state.isMcRunning = data.isGameRunning
      state.mcStatusText = data.isGameRunning ? 'Minecraft running' : 'Minecraft stopped'
      state.stats.size = data.size
      state.stats.saves = data.saves
      state.stats.playtime = data.playtime
      state.stats.java = data.java
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Telemetry Warning'), error.message || String(err), 'danger')
    } finally {
      isRefreshing.value = false
    }
  }

  async function executeQuickTool(toolId: string): Promise<void> {
    isActionExecuting.value = true
    try {
      const res = await invoke<ToolExecutionResultDto>('execute_system_tool', {
        toolId,
        mcVersion: null,
        loader: null,
      })
      if (res.success) {
        showToast(t('Action Complete'), res.msg, 'success')
        await fetchOverview()
      } else {
        showToast(t('Action Failed'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Fault'), error.message || String(err), 'danger')
    } finally {
      isActionExecuting.value = false
    }
  }

  async function instantIgnite(): Promise<void> {
    state.currentView = 'launcher'
  }

  async function terminateProcess(): Promise<void> {
    await executeQuickTool('kill_java')
    state.isMcRunning = false
    state.mcStatusText = 'Minecraft stopped'
  }

  return {
    overview,
    isRefreshing,
    isActionExecuting,
    selectedPreset,
    cpuStatusColor,
    ramStatusColor,
    fetchOverview,
    executeQuickTool,
    instantIgnite,
    terminateProcess,
  }
}