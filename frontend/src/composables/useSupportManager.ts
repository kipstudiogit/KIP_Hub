import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { marked } from 'marked'
import type {
  SystemTelemetryDto,
  AiDiagnosticResponseDto,
  BugReportResultDto,
} from '../types/support'
import { state } from '../stores/appState'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useSupportManager() {
  const telemetry = ref<SystemTelemetryDto>({
    osName: 'Auditing...',
    osVersion: '',
    cpuBrand: 'Detecting...',
    cpuCores: 0,
    totalRamGb: 0,
    availableRamGb: 0,
    javaVersion: 'Probing...',
    appVersion: '1.7.0',
    activeInstance: 'Default',
  })

  const isFetchingTelemetry = ref(false)
  const isAiDiagnosing = ref(false)
  const isSubmittingBug = ref(false)
  const isLoadingLog = ref(false)

  const rawLogInput = ref(state.aiInputText || '')
  const aiResult = ref<AiDiagnosticResponseDto | null>(null)
  const formattedAnalysisHtml = ref('')

  const bugReportText = ref('')
  const includeTelemetryInReport = ref(true)
  const lastSubmittedReportId = ref<string | null>(null)

  async function loadTelemetry(): Promise<void> {
    isFetchingTelemetry.value = true
    try {
      telemetry.value = await invoke<SystemTelemetryDto>('get_system_telemetry_dossier')
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Telemetry Notice'), error.message || String(err), 'info')
    } finally {
      isFetchingTelemetry.value = false
    }
  }

  async function loadLatestLogSnippet(): Promise<void> {
    isLoadingLog.value = true
    try {
      const snippet = await invoke<string>('load_latest_crash_or_log')
      rawLogInput.value = snippet
      state.aiInputText = snippet
      showToast(t('Log Ingested'), 'Ingested recent crash or log tail.', 'success')
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isLoadingLog.value = false
    }
  }

  async function runAiDiagnosis(): Promise<void> {
    const clean = rawLogInput.value.trim()
    if (!clean) return

    if (
      !state.settings.ai_api_key &&
      !state.settings.openai_api_key &&
      !state.settings.anthropic_api_key &&
      state.settings.ai_provider !== 'ollama'
    ) {
      showToast(t('Error'), 'AI Provider API Key must be set in Settings.', 'danger')
      return
    }

    isAiDiagnosing.value = true
    aiResult.value = null
    formattedAnalysisHtml.value = ''

    try {
      const res = await invoke<AiDiagnosticResponseDto>('diagnose_crash_with_neural_core', {
        logSnippet: clean,
      })
      aiResult.value = res

      if (res.analysis) {
        formattedAnalysisHtml.value = (await marked.parse(res.analysis)) as string
      }

      if (res.success) {
        showToast(t('Analysis Finalized'), `Oracle synthesized findings via ${res.provider.toUpperCase()}.`, 'success')
      } else {
        showToast(t('Notice'), res.analysis || 'Diagnosis incomplete.', 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Diagnostic Fault'), error.message || String(err), 'danger')
    } finally {
      isAiDiagnosing.value = false
    }
  }

  async function submitBugReport(): Promise<void> {
    const text = bugReportText.value.trim()
    if (!text) return
    isSubmittingBug.value = true

    try {
      const res = await invoke<BugReportResultDto>('submit_encrypted_bug_report', {
        payload: {
          reportText: text,
          includeTelemetry: includeTelemetryInReport.value,
        },
      })

      if (res.success) {
        lastSubmittedReportId.value = res.reportId
        bugReportText.value = ''
        showToast(t('Dispatched'), `Submitted report ID: ${res.reportId}`, 'success')
      } else {
        showToast(t('Dispatch Failed'), res.message, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Transmission Error'), error.message || String(err), 'danger')
    } finally {
      isSubmittingBug.value = false
    }
  }

  async function copyTelemetryDossier(): Promise<void> {
    const dossierText = `--- K.I.P. SYSTEM TELEMETRY DOSSIER ---
OS: ${telemetry.value.osName} ${telemetry.value.osVersion}
CPU: ${telemetry.value.cpuBrand} (${telemetry.value.cpuCores} Cores)
RAM: ${telemetry.value.totalRamGb} GB Total (${telemetry.value.availableRamGb} GB Free)
Java Runtime: ${telemetry.value.javaVersion}
Engine Core: v${telemetry.value.appVersion}
Instance: ${telemetry.value.activeInstance}`

    await navigator.clipboard.writeText(dossierText)
    showToast(t('Copied'), 'Hardware telemetry dossier copied to clipboard.', 'success')
  }

  return {
    telemetry,
    isFetchingTelemetry,
    isAiDiagnosing,
    isSubmittingBug,
    isLoadingLog,
    rawLogInput,
    aiResult,
    formattedAnalysisHtml,
    bugReportText,
    includeTelemetryInReport,
    lastSubmittedReportId,
    loadTelemetry,
    loadLatestLogSnippet,
    runAiDiagnosis,
    submitBugReport,
    copyTelemetryDossier,
  }
}