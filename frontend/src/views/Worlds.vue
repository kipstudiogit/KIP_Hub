<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <div class="flex justify-between items-end mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Worlds') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">NBT Universe Inspector, Emergency Patient Recovery & Time Machine</p>
      </div>

      <div class="flex items-center gap-3">
        <div class="kip-card px-4 py-2 flex items-center gap-3 bg-black/40 border border-white/10">
          <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Total Worlds Disk Space:</span>
          <span class="text-xs font-mono font-black text-indigo-400">{{ totalDiskUsage }} MB</span>
        </div>

        <button @click="loadWorlds" :disabled="isWorldsLoading" class="kip-btn-ghost p-2.5 border-white/10 text-white/70 hover:text-white" :title="t('Refresh universes')">
          <RefreshCw class="w-4 h-4" :class="isWorldsLoading ? 'animate-spin text-indigo-400' : ''" />
        </button>
      </div>
    </div>

    <div class="flex justify-between items-center gap-4 mb-6 shrink-0 z-10">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
        <input v-model="searchQuery" type="text" :placeholder="t('Filter worlds by name, seed or version...')" class="kip-input pl-11 py-2 text-xs font-mono">
      </div>

      <div class="flex items-center gap-2">
        <div class="flex p-1 bg-black/40 rounded-xl border border-white/10">
          <button v-for="f in (['all', 'survival', 'hardcore', 'creative'] as const)" :key="f" @click="activeFilter = f" class="px-3.5 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition" :class="activeFilter === f ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'">
            {{ f }}
          </button>
        </div>

        <select v-model="sortBy" class="kip-input py-2 text-xs w-44 font-mono">
          <option value="recent">Sort: Recently Played</option>
          <option value="size">Sort: Largest Disk Size</option>
          <option value="name">Sort: Alphabetical</option>
        </select>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10">
      <div v-if="isWorldsLoading && worldsList.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center justify-center">
        <Loader class="w-10 h-10 animate-spin text-indigo-400 mb-4" />
        <span class="text-white/40 font-mono text-xs uppercase tracking-widest">{{ t('Mounting universes & parsing NBT data...') }}</span>
      </div>

      <div v-else-if="filteredWorlds.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
        <Earth class="w-16 h-16 text-white/10" />
        <span class="text-white/40 font-mono text-xs">{{ searchQuery ? 'No worlds matching your filter query.' : t('No worlds found in current instance.') }}</span>
      </div>

      <div v-for="w in filteredWorlds" :key="w.name" class="kip-card p-6 flex flex-col justify-between group relative overflow-hidden transition-all duration-300 border border-white/5 hover:border-indigo-500/30 hover:bg-black/50">
        <div class="absolute -right-12 -top-12 w-36 h-36 rounded-full blur-3xl pointer-events-none transition-all duration-500" :class="w.hardcore ? 'bg-red-500/10 group-hover:bg-red-500/20' : 'bg-indigo-500/10 group-hover:bg-indigo-500/20'"></div>

        <div class="flex items-start gap-5 mb-5 relative z-10">
          <div class="relative shrink-0">
            <img :src="w.icon || fallbackWorldIcon" class="w-20 h-20 rounded-2xl object-cover bg-black/60 p-1 border border-white/10 shadow-xl group-hover:scale-105 transition-transform duration-300">
            <span v-if="w.hardcore" class="absolute -bottom-2 -right-2 p-1.5 bg-red-600 text-white rounded-lg shadow-lg border border-red-400" :title="t('Hardcore Mode Active')">
              <Skull class="w-3.5 h-3.5" />
            </span>
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2 mb-1.5">
              <h3 class="font-black text-white text-xl truncate leading-tight">{{ w.name }}</h3>
              <span class="text-[10px] font-mono text-white/30 shrink-0">{{ w.last_played }}</span>
            </div>

            <div class="flex flex-wrap gap-1.5 mb-2.5">
              <span class="px-2 py-0.5 rounded text-[9px] font-black uppercase tracking-wider border" :class="w.hardcore ? 'bg-red-500/10 text-red-400 border-red-500/20' : 'bg-white/5 text-white/70 border-white/10'">
                {{ w.hardcore ? 'Hardcore' : w.mode }}
              </span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                MC {{ w.mc_version }}
              </span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center gap-1">
                <Sun class="w-2.5 h-2.5" /> Day {{ w.day_count }}
              </span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-white/5 text-white/50 border border-white/10">
                {{ w.size_mb }} MB
              </span>
            </div>

            <div class="flex items-center justify-between text-xs text-white/40 font-mono">
              <span class="truncate">Spawn: {{ w.spawn_x }}, {{ w.spawn_y }}, {{ w.spawn_z }}</span>
              <button @click="copySeed(w.seed)" class="hover:text-indigo-400 transition flex items-center gap-1 text-[10px]">
                <span>Seed: {{ w.seed.slice(0, 10) }}...</span>
                <Copy class="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>

        <div class="pt-4 border-t border-white/5 flex items-center justify-between gap-2 relative z-10">
          <div class="flex gap-2">
            <button @click="healPlayer(w)" :disabled="w.healing" class="kip-btn-ghost px-3.5 py-2 text-xs hover:text-emerald-400 hover:border-emerald-500/40 hover:bg-emerald-500/10" :title="t('Recover player from void fall, clear negative effects and reset health to 20')">
              <Heart class="w-4 h-4 text-emerald-400" :class="w.healing ? 'animate-ping' : ''" />
              <span>{{ w.healing ? t('Rescuing...') : t('Heal & Fix') }}</span>
            </button>
            <button @click="syncCloudWorld(w)" :disabled="w.syncing" class="kip-btn-ghost px-3.5 py-2 text-xs hover:text-blue-400 hover:border-blue-500/40 hover:bg-blue-500/10">
              <UploadCloud class="w-4 h-4 text-blue-400" :class="w.syncing ? 'animate-bounce' : ''" />
              <span>{{ t('Snapshot') }}</span>
            </button>
            <button @click="openWorldMap(w.name)" class="kip-btn-ghost px-3.5 py-2 text-xs hover:text-cyan-400 hover:border-cyan-500/40 hover:bg-cyan-500/10">
              <Map class="w-4 h-4 text-cyan-400" />
              <span>{{ t('Map') }}</span>
            </button>
            <button @click="openWorldHistory(w.name)" class="kip-btn-ghost px-3.5 py-2 text-xs hover:text-purple-400 hover:border-purple-500/40 hover:bg-purple-500/10">
              <History class="w-4 h-4 text-purple-400" />
              <span>{{ t('Timeline') }}</span>
            </button>
          </div>

          <button @click="confirmDeleteWorld(w)" class="p-2.5 rounded-xl text-red-400/60 hover:text-red-400 hover:bg-red-500/10 transition" :title="t('Delete universe')">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="mapModal.isOpen" class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default" @click.self="mapModal.isOpen = false">
        <div class="relative w-full h-full kip-card p-0 border-cyan-500/30 flex flex-col max-w-6xl shadow-[0_0_60px_rgba(6,182,212,0.25)] overflow-hidden">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
            <div class="flex items-center gap-3">
              <div class="p-2.5 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
                <Map class="w-6 h-6" />
              </div>
              <div>
                <h3 class="font-black text-2xl text-white">Cartographer Satellite Radar</h3>
                <span class="text-xs font-mono text-cyan-400">Universe: {{ mapModal.worldName }}</span>
              </div>
            </div>

            <div class="flex items-center gap-3">
              <button v-if="mapModal.image" @click="downloadMapImage" class="kip-btn-ghost px-4 py-2 text-xs border-cyan-500/30 text-cyan-400 hover:bg-cyan-500/10 flex items-center gap-2">
                <Download class="w-4 h-4" /> Export Satellite PNG
              </button>
              <button @click="mapModal.isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition">
                <X class="w-6 h-6" />
              </button>
            </div>
          </div>

          <div class="flex-1 overflow-auto flex items-center justify-center bg-[#020202] p-6 relative">
            <div v-if="mapModal.loading" class="flex flex-col items-center text-cyan-400 relative z-10">
              <div class="w-20 h-20 border-4 border-cyan-500/20 border-t-cyan-400 rounded-full animate-spin mb-5"></div>
              <span class="font-mono text-xs uppercase font-bold tracking-widest animate-pulse">{{ t('Decompressing MCA region chunks and raycasting surface voxels...') }}</span>
            </div>

            <div v-else-if="mapModal.image" class="relative max-w-full max-h-full flex items-center justify-center">
              <img :src="mapModal.image" class="max-w-full max-h-full object-contain shadow-2xl rounded-2xl border border-white/10" style="image-rendering: pixelated;">
            </div>

            <div v-else class="flex flex-col items-center text-white/30 gap-4">
              <Earth class="w-16 h-16 opacity-30" />
              <p class="font-mono text-xs">{{ t('Region files are empty or ungenerated.') }}</p>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="historyModal.isOpen" class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default" @click.self="historyModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[85vh] shadow-[0_0_60px_rgba(168,85,247,0.25)] relative overflow-hidden border-purple-500/30">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
            <div class="flex items-center gap-3">
              <div class="p-2.5 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400">
                <History class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-2xl font-black text-white">VCS Time Machine</h3>
                <span class="text-xs font-mono text-purple-400">Snapshot Registry: {{ historyModal.worldName }}</span>
              </div>
            </div>
            <button @click="historyModal.isOpen = false" class="p-2 text-white/40 hover:text-white hover:bg-white/5 rounded-xl transition">
              <X class="w-6 h-6" />
            </button>
          </div>

          <div class="p-5 border-b border-white/5 bg-black/40 shrink-0">
            <button @click="commitWorld" :disabled="historyModal.loading" class="kip-btn-primary w-full py-3.5 bg-purple-500 hover:bg-purple-400 text-white font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(168,85,247,0.4)]">
              <Loader v-if="historyModal.loading" class="w-4 h-4 animate-spin" />
              <Save v-else class="w-4 h-4" />
              <span>Capture Incremental Snapshot</span>
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#050505]/90 relative z-10 min-h-0">
            <div v-if="historyModal.commits.length === 0" class="py-12 text-center text-white/30 font-mono text-xs">
              {{ t('No snapshot commits available for this universe.') }}
            </div>

            <div v-else class="relative border-l-2 border-purple-500/30 ml-4 space-y-6 pb-2">
              <div v-for="(commit, index) in historyModal.commits" :key="commit.id" class="relative pl-6 group">
                <div class="absolute -left-[9px] top-1.5 w-4 h-4 rounded-full border-4 border-[#050505] transition-colors" :class="index === 0 ? 'bg-purple-400 shadow-[0_0_12px_rgba(168,85,247,0.9)]' : 'bg-purple-500/40 group-hover:bg-purple-400'"></div>

                <div class="bg-black/40 border border-white/5 group-hover:border-purple-500/40 rounded-2xl p-4 transition-all duration-300 flex justify-between items-center shadow-inner">
                  <div>
                    <div class="flex items-center gap-2 mb-1">
                      <GitCommit class="w-4 h-4 text-purple-400" />
                      <span class="font-mono text-xs font-bold text-white uppercase">{{ commit.id }}</span>
                      <span v-if="index === 0" class="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-[8px] font-black uppercase tracking-wider rounded border border-emerald-500/30">Head</span>
                    </div>
                    <div class="text-[10px] text-white/40 flex items-center gap-1.5 font-mono">
                      <Clock class="w-3 h-3" />
                      {{ new Date(commit.timestamp).toLocaleString() }}
                    </div>
                  </div>

                  <button @click="restoreCommit(commit.id)" class="kip-btn-ghost px-4 py-2 text-xs text-purple-400 hover:text-white border-purple-500/20 hover:border-purple-500/50 hover:bg-purple-500/20">
                    <History class="w-3.5 h-3.5" /> Revert
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  Loader,
  Heart,
  UploadCloud,
  Map,
  History,
  Trash2,
  X,
  Save,
  RefreshCw,
  Earth,
  GitCommit,
  Clock,
  Search,
  Copy,
  Download,
  Skull,
  Sun,
} from 'lucide-vue-next'
import { t, showToast } from '@/store'
import { invokeSafe, type GenericActionResult } from '@/bridge'

export interface WorldCardItem {
  name: string
  seed: string
  mode: string
  hardcore: boolean
  difficulty: string
  day_count: number
  mc_version: string
  spawn_x: number
  spawn_y: number
  spawn_z: number
  size_mb: number
  last_played: string
  datapacks: number
  icon: string
  healing?: boolean
  syncing?: boolean
}

export interface VcsCommitItem {
  id: string
  timestamp: string
  tree?: Record<string, string>
}

interface MapModalState {
  isOpen: boolean
  worldName: string
  loading: boolean
  image: string
}

interface HistoryModalState {
  isOpen: boolean
  worldName: string
  loading: boolean
  commits: VcsCommitItem[]
}

const worldsList = ref<WorldCardItem[]>([])
const isWorldsLoading = ref<boolean>(false)
const searchQuery = ref<string>('')
const activeFilter = ref<'all' | 'survival' | 'hardcore' | 'creative'>('all')
const sortBy = ref<'recent' | 'size' | 'name'>('recent')

const mapModal = ref<MapModalState>({
  isOpen: false,
  worldName: '',
  loading: false,
  image: '',
})

const historyModal = ref<HistoryModalState>({
  isOpen: false,
  worldName: '',
  loading: false,
  commits: [],
})

const fallbackWorldIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>'

const totalDiskUsage = computed<string>(() => {
  const total = worldsList.value.reduce((acc, curr) => acc + (curr.size_mb || 0), 0)
  return total.toFixed(1)
})

const filteredWorlds = computed<WorldCardItem[]>(() => {
  let list = [...worldsList.value]

  if (searchQuery.value.trim()) {
    const q = searchQuery.value.trim().toLowerCase()
    list = list.filter((w) =>
      w.name.toLowerCase().includes(q) ||
      w.seed.toLowerCase().includes(q) ||
      w.mc_version.toLowerCase().includes(q)
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
    list.sort((a, b) => b.size_mb - a.size_mb)
  } else if (sortBy.value === 'name') {
    list.sort((a, b) => a.name.localeCompare(b.name))
  } else {
    list.sort((a, b) => b.last_played.localeCompare(a.last_played))
  }

  return list
})

const loadWorlds = async (): Promise<void> => {
  isWorldsLoading.value = true
  try {
    const res = await invokeSafe<WorldCardItem[]>('get_worlds')
    worldsList.value = (res || []).map((w) => ({
      ...w,
      healing: false,
      syncing: false,
    }))
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Failed to load universes.'), 'danger')
  } finally {
    isWorldsLoading.value = false
  }
}

const copySeed = async (seed: string): Promise<void> => {
  try {
    await navigator.clipboard.writeText(seed)
    showToast(t('Copied'), `Seed ${seed} copied to clipboard!`, 'success')
  } catch {
  }
}

const healPlayer = async (world: WorldCardItem): Promise<void> => {
  if (world.healing) return
  world.healing = true

  try {
    const res = await invokeSafe<GenericActionResult>('heal_world_player', {
      worldName: world.name,
    })
    if (res && res.success) {
      showToast(t('Emergency Rescue Successful'), res.msg, 'success')
    } else {
      showToast(t('Recovery Failed'), res?.msg || t('Failed to heal player.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Recovery Failed'), msg, 'danger')
  } finally {
    setTimeout(() => {
      world.healing = false
    }, 1500)
  }
}

const confirmDeleteWorld = async (world: WorldCardItem): Promise<void> => {
  try {
    const res = await invokeSafe<GenericActionResult>('delete_world', {
      worldName: world.name,
    })
    if (res && res.success) {
      showToast(t('Purged'), res.msg, 'success')
      await loadWorlds()
    } else {
      showToast(t('Error'), res?.msg || t('Failed to delete world directory.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const syncCloudWorld = async (world: WorldCardItem): Promise<void> => {
  if (world.syncing) return
  world.syncing = true

  try {
    const res = await invokeSafe<GenericActionResult>('sync_cloud_world', {
      worldName: world.name,
    })
    if (res && res.success) {
      showToast(t('Snapshot Captured'), res.msg, 'success')
    } else {
      showToast(t('Snapshot Error'), res?.msg || t('Failed to capture snapshot.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Snapshot Error'), msg, 'danger')
  } finally {
    setTimeout(() => {
      world.syncing = false
    }, 1500)
  }
}

const openWorldMap = async (worldName: string): Promise<void> => {
  mapModal.value.worldName = worldName
  mapModal.value.image = ''
  mapModal.value.loading = true
  mapModal.value.isOpen = true

  try {
    const res = await invokeSafe<{ success: boolean; image?: string; msg?: string }>('get_world_map', {
      worldName,
    })
    if (res && res.success && res.image) {
      mapModal.value.image = res.image
    }
  } catch {
    mapModal.value.image = ''
  } finally {
    mapModal.value.loading = false
  }
}

const downloadMapImage = (): void => {
  if (!mapModal.value.image) return
  const link = document.createElement('a')
  link.download = `${mapModal.value.worldName}_satellite_map.png`
  link.href = mapModal.value.image
  link.click()
}

const openWorldHistory = async (worldName: string): Promise<void> => {
  historyModal.value.worldName = worldName
  historyModal.value.commits = []
  historyModal.value.loading = true
  historyModal.value.isOpen = true

  try {
    const commits = await invokeSafe<VcsCommitItem[]>('vcs_get_history', {
      worldName,
    })
    historyModal.value.commits = commits || []
  } catch {
    historyModal.value.commits = []
  } finally {
    historyModal.value.loading = false
  }
}

const commitWorld = async (): Promise<void> => {
  if (historyModal.value.loading) return
  historyModal.value.loading = true

  try {
    const res = await invokeSafe<{ success: boolean; commit?: VcsCommitItem; msg?: string }>('vcs_commit', {
      worldName: historyModal.value.worldName,
    })
    if (res && res.success) {
      showToast(t('Snapshot Created'), t('Differential state recorded.'), 'success')
      const commits = await invokeSafe<VcsCommitItem[]>('vcs_get_history', {
        worldName: historyModal.value.worldName,
      })
      historyModal.value.commits = commits || []
    } else {
      showToast(t('Error'), res?.msg || t('Failed to create snapshot.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    historyModal.value.loading = false
  }
}

const restoreCommit = async (commitId: string): Promise<void> => {
  try {
    const res = await invokeSafe<GenericActionResult>('vcs_restore', {
      worldName: historyModal.value.worldName,
      commitId,
    })
    if (res && res.success) {
      showToast(t('State Restored'), t('Universe restored to target snapshot commit.'), 'success')
      historyModal.value.isOpen = false
      await loadWorlds()
    } else {
      showToast(t('Error'), res?.msg || t('Failed to revert universe state.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

onMounted(() => {
  loadWorlds()
})
</script>