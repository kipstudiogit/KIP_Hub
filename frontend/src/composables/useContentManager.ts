import { ref, computed } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import { bridge, type LocalModRecord } from '../bridge'
import type {
  ContentTabType,
  ModpackRecordDto,
  ModUpdateItemDto,
  ModUpdatesCheckDto,
  ContentUpdateProgressDto,
  ContentActionResultDto,
  HubPreset,
  SwarmStatusDto,
} from '../types/content'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useContentManager() {
  const activeContentTab = ref<ContentTabType>('mods')
  const contentList = ref<LocalModRecord[]>([])
  const modpacksList = ref<ModpackRecordDto[]>([])
  const isContentLoading = ref(true)
  const isCheckingUpdates = ref(false)
  const isExporting = ref(false)
  const isImporting = ref(false)
  const isDraggingOver = ref(false)
  const localSearchQuery = ref('')
  const statusFilter = ref<'all' | 'active' | 'disabled'>('all')
  const sortBy = ref<'name' | 'size' | 'date'>('name')
  const selectedFilenames = ref<Set<string>>(new Set())

  // Updates state
  const availableUpdatesList = ref<ModUpdateItemDto[]>([])
  const isUpdatesModalOpen = ref(false)
  const isApplyingUpdates = ref(false)
  const updateProgressPercent = ref(0)
  const updateCurrentFileName = ref('')

  // Hub modal state
  const isHubModalOpen = ref(false)
  const hubPresets = ref<HubPreset[]>([])
  const isHubLoading = ref(false)

  // Swarm modal state
  const isSwarmModalOpen = ref(false)
  const swarmSeeds = ref<SwarmStatusDto[]>([])
  const isSwarmLoading = ref(false)

  // Details modal state
  const isDetailsModalOpen = ref(false)
  const selectedInspectItem = ref<LocalModRecord | null>(null)

  const filteredContent = computed<LocalModRecord[]>(() => {
    let list = [...contentList.value]
    const query = localSearchQuery.value.trim().toLowerCase()

    if (query) {
      list = list.filter(
        (m) =>
          m.name.toLowerCase().includes(query) ||
          m.filename.toLowerCase().includes(query) ||
          m.author.toLowerCase().includes(query) ||
          m.id.toLowerCase().includes(query)
      )
    }

    if (statusFilter.value === 'active') {
      list = list.filter((m) => !m.disabled)
    } else if (statusFilter.value === 'disabled') {
      list = list.filter((m) => m.disabled)
    }

    if (sortBy.value === 'size') {
      list.sort((a, b) => b.size_bytes - a.size_bytes)
    } else if (sortBy.value === 'date') {
      list.sort((a, b) => b.date_modified.localeCompare(a.date_modified))
    } else {
      list.sort((a, b) => a.name.localeCompare(b.name))
    }

    return list
  })

  async function loadContent(): Promise<void> {
    isContentLoading.value = true
    try {
      if (activeContentTab.value === 'modpacks') {
        modpacksList.value = await invoke<ModpackRecordDto[]>('get_installed_modpacks')
      } else {
        contentList.value = await bridge.getLocalMods(activeContentTab.value)
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    } finally {
      isContentLoading.value = false
    }
  }

  async function toggleContentState(item: LocalModRecord): Promise<void> {
    try {
      const success = await bridge.toggleMod(item.filename, activeContentTab.value)
      if (success) {
        item.disabled = !item.disabled
        item.filename = item.disabled ? `${item.filename}.disabled` : item.filename.replace('.disabled', '')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function deleteContentItem(item: LocalModRecord): Promise<void> {
    try {
      const res = await bridge.deleteMod(item.filename, activeContentTab.value)
      if (res.success) {
        showToast(t('Deleted'), res.msg, 'success')
        selectedFilenames.value.delete(item.filename)
        await loadContent()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function batchToggle(enable: boolean): Promise<void> {
    if (selectedFilenames.value.size === 0) return
    try {
      const targets = Array.from(selectedFilenames.value)
      const count = await bridge.batchToggleMods(targets, enable, activeContentTab.value)
      showToast(t('Batch Operation'), `${enable ? 'Activated' : 'Suspended'} ${count} packages.`, 'success')
      selectedFilenames.value.clear()
      await loadContent()
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function batchDelete(): Promise<void> {
    if (selectedFilenames.value.size === 0) return
    try {
      const targets = Array.from(selectedFilenames.value)
      const count = await bridge.batchDeleteMods(targets, activeContentTab.value)
      showToast(t('Batch Operation'), `Purged ${count} packages from disk.`, 'success')
      selectedFilenames.value.clear()
      await loadContent()
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function checkModUpdates(): Promise<void> {
    if (isCheckingUpdates.value) return
    isCheckingUpdates.value = true
    showToast(t('Scanning'), 'Auditing JAR hashes via Modrinth API...', 'info')

    try {
      const res = await invoke<ModUpdatesCheckDto>('check_content_updates', {
        mcVersion: '26.3',
        loader: 'fabric',
      })

      if (res.success && res.updates && res.updates.length > 0) {
        availableUpdatesList.value = res.updates
        isUpdatesModalOpen.value = true
        showToast(
          t('Updates Available'),
          `Discovered ${res.updates.length} verified upgrades across ${res.checkedCount} JARs.`,
          'success'
        )
      } else if (res.success) {
        availableUpdatesList.value = []
        showToast(t('Up to date'), `All ${res.checkedCount} installed JARs are on the latest versions.`, 'success')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    } finally {
      isCheckingUpdates.value = false
    }
  }

  async function applyUpdatesBatch(): Promise<void> {
    if (availableUpdatesList.value.length === 0 || isApplyingUpdates.value) return
    isApplyingUpdates.value = true
    updateProgressPercent.value = 0

    const channel = new Channel<ContentUpdateProgressDto>()
    channel.onmessage = (progress) => {
      updateProgressPercent.value = progress.percent
      updateCurrentFileName.value = progress.currentFile
    }

    try {
      const res = await invoke<ContentActionResultDto>('apply_content_updates_stream', {
        updates: availableUpdatesList.value,
        progressChannel: channel,
      })

      if (res.success) {
        showToast(t('Success'), res.msg, 'success')
        isUpdatesModalOpen.value = false
        availableUpdatesList.value = []
        await loadContent()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    } finally {
      isApplyingUpdates.value = false
    }
  }

  async function mountModpack(filename: string): Promise<void> {
    try {
      showToast(t('Mounting Modpack'), 'Extracting modpack overrides into instance...', 'info')
      const res = await invoke<ContentActionResultDto>('mount_modpack_archive', { filename })
      if (res.success) {
        showToast(t('Modpack Mounted'), res.msg, 'success')
        await loadContent()
      } else {
        showToast(t('Mount Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function exportModpack(): Promise<void> {
    if (isExporting.value) return
    isExporting.value = true
    showToast(t('Exporting'), t('Generating modpack archive...'), 'info')

    try {
      const res = await invoke<ContentActionResultDto>('export_active_modpack')
      if (res.success) {
        showToast(t('Exported'), res.msg, 'success')
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    } finally {
      isExporting.value = false
    }
  }

  async function importDroppedFiles(paths: string[]): Promise<void> {
    try {
      const res = await invoke<ContentActionResultDto>('import_dropped_content', { files: paths })
      if (res.success) {
        showToast(t('Imported'), res.msg, 'success')
        await loadContent()
      } else {
        showToast(t('Import Error'), res.msg, 'danger')
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg, 'danger')
    }
  }

  async function openHub(): Promise<void> {
    isHubModalOpen.value = true
    isHubLoading.value = true
    try {
      hubPresets.value = await bridge.fetchHub()
    } catch {
      hubPresets.value = []
    } finally {
      isHubLoading.value = false
    }
  }

  async function openSwarm(): Promise<void> {
    isSwarmModalOpen.value = true
    try {
      swarmSeeds.value = await bridge.swarmSeedStatus()
    } catch {
      swarmSeeds.value = []
    }
  }

  function inspectItem(item: LocalModRecord): void {
    selectedInspectItem.value = item
    isDetailsModalOpen.value = true
  }

  return {
    activeContentTab,
    contentList,
    modpacksList,
    isContentLoading,
    isCheckingUpdates,
    isExporting,
    isImporting,
    isDraggingOver,
    localSearchQuery,
    statusFilter,
    sortBy,
    selectedFilenames,
    availableUpdatesList,
    isUpdatesModalOpen,
    isApplyingUpdates,
    updateProgressPercent,
    updateCurrentFileName,
    isHubModalOpen,
    hubPresets,
    isHubLoading,
    isSwarmModalOpen,
    swarmSeeds,
    isSwarmLoading,
    isDetailsModalOpen,
    selectedInspectItem,
    filteredContent,
    loadContent,
    toggleContentState,
    deleteContentItem,
    batchToggle,
    batchDelete,
    checkModUpdates,
    applyUpdatesBatch,
    mountModpack,
    exportModpack,
    importDroppedFiles,
    openHub,
    openSwarm,
    inspectItem,
  }
}