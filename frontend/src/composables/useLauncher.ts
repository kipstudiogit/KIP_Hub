import { ref, computed } from 'vue'
import {
  apiInspectPreflight,
  apiAutoRepairEnvironment,
  apiLaunchInstanceStream,
} from '../api/launcher'
import type {
  LoaderType,
  PreflightReportDto,
  LaunchProgressDto,
  LaunchResultDto,
  BackendAppError,
} from '../types/launcher'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'

export function useLauncher() {
  const isLaunching = ref(false)
  const isRepairing = ref(false)
  const isPreflightRunning = ref(false)
  const isDoctorGuardActive = ref(localStorage.getItem('kip_auto_doctor_guard') !== 'false')

  const preflightReport = ref<PreflightReportDto | null>(null)
  const currentProgress = ref<LaunchProgressDto>({
    phase: 'STANDBY',
    percent: 0,
    message: 'System Ready',
  })

  const hasCriticalConflicts = computed(() => {
    return Boolean(preflightReport.value?.hasCriticalConflicts)
  })

  async function checkPreflight(version: string, loader: LoaderType): Promise<void> {
    isPreflightRunning.value = true
    try {
      preflightReport.value = await apiInspectPreflight(
        version,
        loader,
        isDoctorGuardActive.value
      )
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Preflight Notice'), error.message || String(err), 'info')
      preflightReport.value = null
    } finally {
      isPreflightRunning.value = false
    }
  }

  async function repairEnvironment(version: string, loader: LoaderType): Promise<boolean> {
    isRepairing.value = true
    try {
      const result = await apiAutoRepairEnvironment(version, loader)
      if (result.success) {
        showToast(t('Environment Repaired'), result.message, 'success')
        await checkPreflight(version, loader)
        return true
      }
      return false
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Repair Fault'), error.message || String(err), 'danger')
      return false
    } finally {
      isRepairing.value = false
    }
  }

  async function ignite(
    version: string,
    loader: LoaderType,
    loaderVersion: string | null,
    ramAllocation: number,
    jvmArgs: string,
    resolution: string,
    fullscreen: boolean,
    bypassChecks = false
  ): Promise<LaunchResultDto | null> {
    if (isLaunching.value) return null

    if (isDoctorGuardActive.value && !bypassChecks && hasCriticalConflicts.value) {
      showToast(
        t('Conflicts Detected'),
        t('Resolve compatibility collisions or use Force Ignition.'),
        'danger'
      )
      return null
    }

    isLaunching.value = true
    currentProgress.value = {
      phase: 'INITIALIZATION',
      percent: 5,
      message: 'Priming launch subsystem...',
    }

    try {
      const result = await apiLaunchInstanceStream(
        {
          version,
          loader,
          loaderVersion: loaderVersion || null,
          ramAllocation,
          jvmArgs: jvmArgs || null,
          resolution: resolution || null,
          fullscreen,
          bypassChecks,
        },
        (progress) => {
          currentProgress.value = progress
          state.launchStatus = progress.message
          state.launchProgress = progress.percent
        }
      )

      state.isMcRunning = true
      state.mcStatusText = 'Minecraft running'
      showToast(t('Engine Ignited'), result.message, 'success')
      return result
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Launch Fault'), error.message || String(err), 'danger')
      return null
    } finally {
      setTimeout(() => {
        isLaunching.value = false
        currentProgress.value = {
          phase: 'STANDBY',
          percent: 0,
          message: 'System Ready',
        }
      }, 1500)
    }
  }

  function toggleDoctorGuard(version: string, loader: LoaderType): void {
    isDoctorGuardActive.value = !isDoctorGuardActive.value
    localStorage.setItem('kip_auto_doctor_guard', String(isDoctorGuardActive.value))
    showToast(
      t('Doctor Guard'),
      isDoctorGuardActive.value ? t('Doctor Guard Active') : t('Doctor Guard Bypassed'),
      'info'
    )
    checkPreflight(version, loader)
  }

  return {
    isLaunching,
    isRepairing,
    isPreflightRunning,
    isDoctorGuardActive,
    preflightReport,
    currentProgress,
    hasCriticalConflicts,
    checkPreflight,
    repairEnvironment,
    ignite,
    toggleDoctorGuard,
  }
}