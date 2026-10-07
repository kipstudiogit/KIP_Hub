import { ref } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import type {
  ToolExecutionResultDto,
  DoctorAnalysisReportDto,
  DoctorIssueDto,
  ShieldScanReportDto,
  ShieldScanProgressDto,
  FileSecurityReportDto,
  QuarantineRecordDto,
  SafeModeStatusDto,
  CrashInvestigationDto,
} from '../types/tools'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useToolsManager() {
  const isExecutingTool = ref(false)
  const isSafeModeActive = ref(false)

  const crashReport = ref<CrashInvestigationDto | null>(null)
  const isInvestigatingCrash = ref(false)

  const isDoctorModalOpen = ref(false)
  const isDoctorAnalyzing = ref(false)
  const isDoctorRemediating = ref(false)
  const doctorReport = ref<DoctorAnalysisReportDto | null>(null)

  const isShieldModalOpen = ref(false)
  const isShieldScanning = ref(false)
  const shieldReport = ref<ShieldScanReportDto | null>(null)
  const shieldProgress = ref<ShieldScanProgressDto>({
    currentFile: '',
    scannedCount: 0,
    totalFiles: 0,
    percent: 0,
    threatsFound: 0,
  })
  const vaultRecords = ref<QuarantineRecordDto[]>([])

  async function executeTool(toolId: string): Promise<ToolExecutionResultDto | null> {
    isExecutingTool.value = true
    try {
      const res = await invoke<ToolExecutionResultDto>('execute_system_tool', {
        toolId,
        mcVersion: null,
        loader: null,
      })

      if (res.success) {
        showToast(t('Success'), res.msg, 'success')
        if (res.clipboard) {
          await navigator.clipboard.writeText(res.clipboard)
          showToast(t('Copied'), 'Output copied to clipboard.', 'info')
        }
      } else {
        showToast(t('Action Failed'), res.msg, 'danger')
      }
      return res
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
      return null
    } finally {
      isExecutingTool.value = false
    }
  }

  async function checkLatestCrash(): Promise<void> {
    isInvestigatingCrash.value = true
    try {
      crashReport.value = await invoke<CrashInvestigationDto>('investigate_latest_crash')
    } catch {
      crashReport.value = null
    } finally {
      isInvestigatingCrash.value = false
    }
  }

  async function fetchSafeModeStatus(): Promise<void> {
    try {
      const res = await invoke<SafeModeStatusDto>('get_safe_mode_status')
      isSafeModeActive.value = res.enabled
    } catch {
      isSafeModeActive.value = false
    }
  }

  async function toggleSafeMode(): Promise<void> {
    try {
      const res = await invoke<SafeModeStatusDto>('toggle_safe_mode_status')
      isSafeModeActive.value = res.enabled
      showToast(
        t('Safe Mode'),
        res.enabled ? t('Safe Mode Activated (Mods Isolated)') : t('Safe Mode Deactivated'),
        'success'
      )
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Safe Mode Error'), error.message || String(err), 'danger')
    }
  }

  async function analyzeDoctor(version: string, loader: string): Promise<void> {
    isDoctorAnalyzing.value = true
    try {
      const report = await invoke<DoctorAnalysisReportDto>('analyze_mod_doctor', {
        mcVersion: version,
        loader,
      })
      doctorReport.value = {
        ...report,
        issues: report.issues.map((i) => ({
          ...i,
          selected: i.action !== 'NONE',
        })),
      }
      showToast(
        t('Diagnostics Complete'),
        `Health index: ${report.health_score}%. Audited ${report.total_checked} packages.`,
        'success'
      )
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Doctor Fault'), error.message || String(err), 'danger')
    } finally {
      isDoctorAnalyzing.value = false
    }
  }

  async function remediateDoctorIssues(
    issues: DoctorIssueDto[],
    version: string,
    loader: string
  ): Promise<void> {
    if (issues.length === 0) return
    isDoctorRemediating.value = true

    try {
      await invoke('apply_doctor_remediation', {
        issues,
        mcVersion: version,
        loader,
      })
      showToast(t('Remediation Applied'), 'Conflict fixes executed.', 'success')
      await analyzeDoctor(version, loader)
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Remediation Fault'), error.message || String(err), 'danger')
    } finally {
      isDoctorRemediating.value = false
    }
  }

  async function scanShieldStream(targetDir?: string): Promise<void> {
    isShieldScanning.value = true
    shieldProgress.value = {
      currentFile: 'Starting bytecode inspection...',
      scannedCount: 0,
      totalFiles: 0,
      percent: 0,
      threatsFound: 0,
    }

    const channel = new Channel<ShieldScanProgressDto>()
    channel.onmessage = (progress) => {
      shieldProgress.value = progress
    }

    try {
      shieldReport.value = await invoke<ShieldScanReportDto>('scan_shield_security_stream', {
        targetDir: targetDir || null,
        progressChannel: channel,
      })
      showToast(
        t('Audit Complete'),
        `Inspected ${shieldReport.value.total_scanned} archives. Threats: ${shieldReport.value.threat_count}`,
        'info'
      )
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Shield Fault'), error.message || String(err), 'danger')
    } finally {
      isShieldScanning.value = false
    }
  }

  async function quarantineThreat(threat: FileSecurityReportDto): Promise<void> {
    try {
      await invoke<QuarantineRecordDto>('quarantine_shield_file', {
        filepath: threat.filepath,
      })
      showToast(t('Threat Isolated'), `${threat.filename} placed into quarantine vault.`, 'success')
      await scanShieldStream()
      await loadVaultRecords()
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Quarantine Error'), error.message || String(err), 'danger')
    }
  }

  async function loadVaultRecords(): Promise<void> {
    try {
      vaultRecords.value = await invoke<QuarantineRecordDto[]>('get_shield_vault_records')
    } catch {
      vaultRecords.value = []
    }
  }

  async function restoreVaultItem(id: string): Promise<void> {
    try {
      await invoke('restore_shield_record', { quarantineId: id })
      showToast(t('Restored'), 'File restored to original directory.', 'success')
      await loadVaultRecords()
      await scanShieldStream()
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Restore Error'), error.message || String(err), 'danger')
    }
  }

  async function shredVaultItem(id: string): Promise<void> {
    try {
      await invoke('shred_shield_record', { quarantineId: id })
      showToast(t('DoD Shred Complete'), 'Archive zero-filled and erased from storage.', 'success')
      await loadVaultRecords()
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Shred Error'), error.message || String(err), 'danger')
    }
  }

  return {
    isExecutingTool,
    isSafeModeActive,
    crashReport,
    isInvestigatingCrash,
    isDoctorModalOpen,
    isDoctorAnalyzing,
    isDoctorRemediating,
    doctorReport,
    isShieldModalOpen,
    isShieldScanning,
    shieldReport,
    shieldProgress,
    vaultRecords,
    executeTool,
    checkLatestCrash,
    fetchSafeModeStatus,
    toggleSafeMode,
    analyzeDoctor,
    remediateDoctorIssues,
    scanShieldStream,
    quarantineThreat,
    loadVaultRecords,
    restoreVaultItem,
    shredVaultItem,
  }
}