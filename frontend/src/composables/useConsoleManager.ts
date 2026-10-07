import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { listen, type Event, type UnlistenFn } from '@tauri-apps/api/event'
import type { ConsoleLineDto, ConsoleExportResultDto, LogLevelFilter } from '../types/console'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useConsoleManager() {
  const logEntries = ref<ConsoleLineDto[]>([])
  const isAutoScroll = ref(true)
  const isFrozen = ref(false)
  const isUploading = ref(false)
  const isExporting = ref(false)
  const searchQuery = ref('')
  const activeLevelFilter = ref<LogLevelFilter>('ALL')
  const maxBufferSize = ref<number>(1000)

  let unlistenLine: UnlistenFn | null = null
  let nextLineId = 1

  function parseLine(raw: string): ConsoleLineDto {
    const trimmed = raw.trim()
    let timestamp: string | null = null
    let thread: string | null = null
    let level: ConsoleLineDto['level'] = 'INFO'
    let message = trimmed

    if (trimmed.startsWith('[')) {
      const closeFirst = trimmed.indexOf(']')
      if (closeFirst > 0) {
        const first = trimmed.slice(1, closeFirst)
        if (first.includes(':') && first.length <= 12) {
          timestamp = first
          const rest = trimmed.slice(closeFirst + 1).trim()
          if (rest.startsWith('[')) {
            const closeSecond = rest.indexOf(']')
            if (closeSecond > 0) {
              const second = rest.slice(1, closeSecond)
              if (second.includes('/')) {
                const [tName, lName] = second.split('/')
                thread = (tName || '').trim()
                const upperL = (lName || '').trim().toUpperCase()
                if (upperL.includes('ERROR') || upperL.includes('FATAL')) level = 'ERROR'
                else if (upperL.includes('WARN')) level = 'WARN'
                else if (upperL.includes('DEBUG')) level = 'DEBUG'
                else level = 'INFO'
              } else {
                thread = second.trim()
              }

              const colonIdx = rest.indexOf(':', closeSecond)
              if (colonIdx > 0) {
                message = rest.slice(colonIdx + 1).trim()
              } else {
                message = rest.slice(closeSecond + 1).trim()
              }
            }
          }
        }
      }
    }

    if (level === 'INFO') {
      const upper = message.toUpperCase()
      if (upper.includes('ERROR') || upper.includes('EXCEPTION') || upper.includes('CRITICAL')) {
        level = 'ERROR'
      } else if (upper.includes('WARN')) {
        level = 'WARN'
      }
    }

    return {
      id: nextLineId++,
      raw: trimmed,
      timestamp,
      thread,
      level,
      message,
    }
  }

  const filteredEntries = computed(() => {
    let list = logEntries.value

    if (activeLevelFilter.value !== 'ALL') {
      list = list.filter((e) => e.level === activeLevelFilter.value)
    }

    const q = searchQuery.value.trim().toLowerCase()
    if (q) {
      list = list.filter(
        (e) =>
          e.message.toLowerCase().includes(q) ||
          e.raw.toLowerCase().includes(q) ||
          (e.thread && e.thread.toLowerCase().includes(q))
      )
    }

    return list
  })

  const stats = computed(() => {
    const total = logEntries.value.length
    const errors = logEntries.value.filter((e) => e.level === 'ERROR').length
    const warns = logEntries.value.filter((e) => e.level === 'WARN').length
    const infos = logEntries.value.filter((e) => e.level === 'INFO').length
    return { total, errors, warns, infos }
  })

  async function initStreaming(): Promise<void> {
    try {
      await invoke('toggle_console_streaming', { active: true })
      const initial = await invoke<ConsoleLineDto[]>('get_recent_console_entries', {
        limit: maxBufferSize.value,
      })
      if (initial && initial.length > 0) {
        logEntries.value = initial
        nextLineId = initial[initial.length - 1]?.id ? initial[initial.length - 1]!.id + 1 : 1
      }

      unlistenLine = await listen<string>('appendConsoleLine', (event: Event<string>) => {
        if (!isFrozen.value) {
          const parsed = parseLine(event.payload)
          logEntries.value.push(parsed)
          if (logEntries.value.length > maxBufferSize.value) {
            logEntries.value.shift()
          }
        }
      })
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Terminal Fault'), error.message || String(err), 'danger')
    }
  }

  async function uploadToMclogs(): Promise<void> {
    isUploading.value = true
    try {
      const res = await invoke<ConsoleExportResultDto>('upload_active_console_log')
      if (res.success && res.mclogsUrl) {
        await navigator.clipboard.writeText(res.mclogsUrl)
        showToast(t('Uploaded to mclo.gs'), 'URL copied to clipboard!', 'success')
      } else {
        showToast(t('Upload Failed'), res.message, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Fault'), error.message || String(err), 'danger')
    } finally {
      isUploading.value = false
    }
  }

  async function exportToFile(): Promise<void> {
    isExporting.value = true
    try {
      const res = await invoke<ConsoleExportResultDto>('export_console_log_to_file')
      if (res.success && res.filePath) {
        showToast(t('Exported'), `Log saved to ${res.filePath}`, 'success')
      } else {
        showToast(t('Export Failed'), res.message, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Fault'), error.message || String(err), 'danger')
    } finally {
      isExporting.value = false
    }
  }

  async function copyAllLogs(): Promise<void> {
    if (logEntries.value.length === 0) return
    const text = logEntries.value.map((e) => e.raw).join('\n')
    await navigator.clipboard.writeText(text)
    showToast(t('Copied'), 'Terminal lines copied to clipboard.', 'success')
  }

  function clearLogs(): void {
    logEntries.value = []
    showToast(t('Cleared'), 'Terminal buffer cleared.', 'info')
  }

  function cleanup(): void {
    if (unlistenLine) {
      unlistenLine()
      unlistenLine = null
    }
    invoke('toggle_console_streaming', { active: false }).catch(() => {})
  }

  return {
    logEntries,
    filteredEntries,
    stats,
    isAutoScroll,
    isFrozen,
    isUploading,
    isExporting,
    searchQuery,
    activeLevelFilter,
    maxBufferSize,
    initStreaming,
    uploadToMclogs,
    exportToFile,
    copyAllLogs,
    clearLogs,
    cleanup,
  }
}