import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type {
  WorldCardDto,
  VcsCommitDto,
  WorldActionResultDto,
  WorldMapResultDto,
} from '../types/worlds'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useWorldsManager() {
  const worldsList = ref<WorldCardDto[]>([])
  const isWorldsLoading = ref(false)
  const searchQuery = ref('')
  const activeFilter = ref<'all' | 'survival' | 'hardcore' | 'creative' | 'spectator'>('all')
  const sortBy = ref<'recent' | 'size' | 'name'>('recent')

  const isMapModalOpen = ref(false)
  const mapWorldName = ref('')
  const isMapLoading = ref(false)
  const mapImageBase64 = ref('')
  const mapSpawnX = ref(0)
  const mapSpawnZ = ref(0)
  const mapRadius = ref(1)

  const isHistoryModalOpen = ref(false)
  const historyWorldName = ref('')
  const isHistoryLoading = ref(false)
  const worldCommits = ref<VcsCommitDto[]>([])
  const commitMessageInput = ref('')

  const totalDiskUsage = computed(() => {
    const total = worldsList.value.reduce((acc, curr) => acc + (curr.sizeMb || 0), 0)
    return total.toFixed(1)
  })

  const filteredWorlds = computed(() => {
    let list = [...worldsList.value]

    if (searchQuery.value.trim()) {
      const q = searchQuery.value.trim().toLowerCase()
      list = list.filter(
        (w) =>
          w.name.toLowerCase().includes(q) ||
          w.seed.toLowerCase().includes(q) ||
          w.mcVersion.toLowerCase().includes(q)
      )
    }

    if (activeFilter.value !== 'all') {
      if (activeFilter.value === 'hardcore') {
        list = list.filter((w) => w.hardcore)
      } else {
        list = list.filter((w) => w.mode.toLowerCase() === activeFilter.value)
      }
    }

    if (sortBy.value === 'size') {
      list.sort((a, b) => b.sizeMb - a.sizeMb)
    } else if (sortBy.value === 'name') {
      list.sort((a, b) => a.name.localeCompare(b.name))
    } else {
      list.sort((a, b) => b.lastPlayed.localeCompare(a.lastPlayed))
    }

    return list
  })

  async function loadWorlds(): Promise<void> {
    isWorldsLoading.value = true
    try {
      const res = await invoke<WorldCardDto[]>('get_worlds_catalog')
      worldsList.value = res.map((w) => ({
        ...w,
        healing: false,
        syncing: false,
        cloning: false,
      }))
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isWorldsLoading.value = false
    }
  }

  async function healPlayer(world: WorldCardDto): Promise<void> {
    world.healing = true
    try {
      const res = await invoke<WorldActionResultDto>('heal_world_entity_player', {
        worldName: world.name,
      })
      if (res.success) {
        showToast(t('Emergency Recovery'), res.msg, 'success')
        await loadWorlds()
      } else {
        showToast(t('Recovery Failed'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      world.healing = false
    }
  }

  async function changeGamemode(world: WorldCardDto, modeIndex: number): Promise<void> {
    try {
      const res = await invoke<WorldActionResultDto>('toggle_world_gamemode', {
        worldName: world.name,
        targetMode: modeIndex,
      })
      if (res.success) {
        showToast(t('Gamemode Adjusted'), res.msg, 'success')
        await loadWorlds()
      } else {
        showToast(t('Adjustment Error'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Gamemode Fault'), error.message || String(err), 'danger')
    }
  }

  async function cloneWorld(world: WorldCardDto): Promise<void> {
    world.cloning = true
    try {
      const res = await invoke<WorldActionResultDto>('clone_saved_world', {
        worldName: world.name,
      })
      if (res.success) {
        showToast(t('Universe Cloned'), res.msg, 'success')
        await loadWorlds()
      } else {
        showToast(t('Clone Error'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Clone Fault'), error.message || String(err), 'danger')
    } finally {
      world.cloning = false
    }
  }

  async function deleteWorld(world: WorldCardDto): Promise<void> {
    try {
      const res = await invoke<WorldActionResultDto>('delete_saved_world', {
        worldName: world.name,
      })
      if (res.success) {
        showToast(t('Purged'), res.msg, 'success')
        await loadWorlds()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  async function openMapModal(worldName: string, radius = 1): Promise<void> {
    mapWorldName.value = worldName
    mapImageBase64.value = ''
    mapRadius.value = radius
    isMapLoading.value = true
    isMapModalOpen.value = true

    try {
      const res = await invoke<WorldMapResultDto>('generate_world_satellite_map', {
        worldName,
        radius,
      })
      if (res.success && res.image) {
        mapImageBase64.value = res.image
        mapSpawnX.value = res.spawnX
        mapSpawnZ.value = res.spawnZ
      } else {
        showToast(t('Notice'), res.msg || 'Region unpopulated.', 'info')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Cartographer Error'), error.message || String(err), 'danger')
    } finally {
      isMapLoading.value = false
    }
  }

  async function openHistoryModal(worldName: string): Promise<void> {
    historyWorldName.value = worldName
    worldCommits.value = []
    commitMessageInput.value = ''
    isHistoryLoading.value = true
    isHistoryModalOpen.value = true

    try {
      worldCommits.value = await invoke<VcsCommitDto[]>('get_world_vcs_history', {
        worldName,
      })
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('VCS Error'), error.message || String(err), 'danger')
    } finally {
      isHistoryLoading.value = false
    }
  }

  async function commitActiveWorld(customMessage?: string): Promise<void> {
    if (!historyWorldName.value) return
    isHistoryLoading.value = true

    try {
      const commit = await invoke<VcsCommitDto>('capture_world_vcs_commit', {
        worldName: historyWorldName.value,
        message: customMessage || commitMessageInput.value.trim() || null,
      })
      showToast(t('Snapshot Created'), `Committed snapshot ID: ${commit.id}`, 'success')
      commitMessageInput.value = ''
      worldCommits.value = await invoke<VcsCommitDto[]>('get_world_vcs_history', {
        worldName: historyWorldName.value,
      })
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Snapshot Fault'), error.message || String(err), 'danger')
    } finally {
      isHistoryLoading.value = false
    }
  }

  async function revertCommit(commitId: string): Promise<void> {
    try {
      const res = await invoke<WorldActionResultDto>('restore_world_vcs_commit', {
        worldName: historyWorldName.value,
        commitId,
      })
      if (res.success) {
        showToast(t('State Reverted'), res.msg, 'success')
        isHistoryModalOpen.value = false
        await loadWorlds()
      } else {
        showToast(t('Error'), res.msg, 'danger')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Revert Fault'), error.message || String(err), 'danger')
    }
  }

  return {
    worldsList,
    isWorldsLoading,
    searchQuery,
    activeFilter,
    sortBy,
    totalDiskUsage,
    filteredWorlds,
    isMapModalOpen,
    mapWorldName,
    isMapLoading,
    mapImageBase64,
    mapSpawnX,
    mapSpawnZ,
    mapRadius,
    isHistoryModalOpen,
    historyWorldName,
    isHistoryLoading,
    worldCommits,
    commitMessageInput,
    loadWorlds,
    healPlayer,
    changeGamemode,
    cloneWorld,
    deleteWorld,
    openMapModal,
    openHistoryModal,
    commitActiveWorld,
    revertCommit,
  }
}