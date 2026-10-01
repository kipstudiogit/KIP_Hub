<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="flex justify-between items-end mb-8 shrink-0 stagger-1">
      <div>
        <h2 class="text-3xl font-extrabold mb-1">{{ t('Worlds') }}</h2>
        <p class="text-white/50 text-sm">{{ t('Manage saves, view maps, and travel through time.') }}</p>
      </div>
      <button @click="loadWorlds" class="kip-btn-ghost p-2.5">
        <RefreshCw class="w-5 h-5 text-white/70" />
      </button>
    </div>

    <div class="grid grid-cols-2 gap-6 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 stagger-2 min-h-0">
      <div v-if="isWorldsLoading" class="col-span-2 text-center py-20 flex flex-col items-center justify-center">
        <Loader class="w-10 h-10 animate-spin text-emerald-500 mb-4" />
        <span class="text-white/50 font-bold tracking-widest uppercase">{{ t('Mounting Universes...') }}</span>
      </div>
      <div v-else-if="worldsList.length === 0" class="col-span-2 text-center py-20 flex flex-col items-center gap-4">
        <Earth class="w-16 h-16 text-white/20" />
        <span class="text-white/40 font-medium">{{ t('No worlds found.') }}</span>
      </div>

      <div v-for="w in worldsList" :key="w.name" class="kip-card kip-card-hover p-6 flex flex-col group relative overflow-hidden transition-all duration-500 border-white/5 hover:border-emerald-500/30">
        <div class="absolute -right-10 -top-10 w-32 h-32 bg-emerald-500/10 blur-3xl rounded-full pointer-events-none group-hover:bg-emerald-500/20 transition-all duration-500"></div>

        <div class="flex items-start gap-5 mb-6 relative z-10">
          <img :src="w.icon || fallbackModIcon" class="w-20 h-20 rounded-2xl object-cover bg-black/40 p-1 flex-shrink-0 border border-white/10 shadow-lg group-hover:scale-105 transition-transform duration-500">
          <div class="flex-1 min-w-0 pt-1">
            <h3 class="font-extrabold text-white truncate text-xl mb-2">{{ w.name }}</h3>

            <div class="flex flex-wrap gap-2 mb-2">
              <span class="px-2.5 py-0.5 bg-white/10 rounded-md text-[10px] font-bold uppercase tracking-wider text-white/80 border border-white/5">
                {{ w.mode }}
              </span>
              <span v-if="w.datapacks > 0" class="px-2.5 py-0.5 bg-indigo-500/20 rounded-md text-[10px] font-bold uppercase tracking-wider text-indigo-400 border border-indigo-500/30">
                {{ w.datapacks }} {{ t('Datapacks') }}
              </span>
            </div>

            <p class="text-xs text-white/40 flex items-center gap-2 mt-2">
              <span class="font-bold uppercase tracking-wider">{{ t('Seed:') }}</span>
              <span class="font-mono bg-black/40 px-2 py-0.5 rounded border border-white/5 select-all text-white/70">{{ w.seed }}</span>
            </p>
          </div>
        </div>

        <div class="mt-auto pt-4 border-t border-white/5 flex justify-between gap-2 relative z-10">
          <div class="flex gap-2">
            <button @click="healPlayer(w)" :disabled="w.healing" class="kip-btn-ghost px-4 py-2 text-xs hover:text-emerald-400 hover:border-emerald-500/40 hover:bg-emerald-500/10">
              <Heart class="w-4 h-4" :class="w.healing ? 'animate-ping text-emerald-400' : ''" /> <span v-if="!w.healing">{{ t('Heal') }}</span>
            </button>
            <button @click="syncCloudWorld(w)" :disabled="w.syncing" class="kip-btn-ghost px-4 py-2 text-xs hover:text-blue-400 hover:border-blue-500/40 hover:bg-blue-500/10">
              <UploadCloud class="w-4 h-4" :class="w.syncing ? 'animate-bounce text-blue-400' : ''" /> <span v-if="!w.syncing">{{ t('Sync') }}</span>
            </button>
            <button @click="openWorldMap(w.name)" class="kip-btn-ghost px-4 py-2 text-xs hover:text-cyan-400 hover:border-cyan-500/40 hover:bg-cyan-500/10">
              <Map class="w-4 h-4" /> {{ t('Map') }}
            </button>
            <button @click="openWorldHistory(w.name)" class="kip-btn-ghost px-4 py-2 text-xs hover:text-purple-400 hover:border-purple-500/40 hover:bg-purple-500/10">
              <History class="w-4 h-4" /> {{ t('Timeline') }}
            </button>
          </div>

          <button @click="deleteWorld(w)" class="kip-btn-danger p-2.5 bg-red-500/10 hover:bg-red-500 hover:text-white text-red-400 border-red-500/20 shadow-none border">
            <Trash2 class="w-4 h-4" />
          </button>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="mapModal.isOpen" class="fixed inset-0 bg-black/90 backdrop-blur-xl z-[200] flex items-center justify-center p-6 cursor-default" @click.self="mapModal.isOpen = false">
        <div class="relative w-full h-full kip-card p-0 border-cyan-500/30 flex flex-col max-w-6xl shadow-[0_0_50px_rgba(6,182,212,0.2)] overflow-hidden">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
            <div class="absolute top-0 right-0 w-64 h-64 bg-cyan-500/10 blur-[100px] rounded-full pointer-events-none"></div>
            <h3 class="font-extrabold text-2xl flex items-center gap-3 text-cyan-400 relative z-10">
              <Map class="w-7 h-7" />
              {{ t('World Map:') }} <span class="text-white">{{ mapModal.worldName }}</span>
            </h3>
            <button @click="mapModal.isOpen = false" class="kip-btn-ghost p-2 relative z-10 border-transparent hover:bg-white/10 hover:text-red-400"><X class="w-6 h-6" /></button>
          </div>

          <div class="flex-1 overflow-auto flex items-center justify-center bg-[#020202] p-6 relative">
            <div v-if="mapModal.loading" class="flex flex-col items-center text-cyan-500 relative z-10">
              <div class="w-24 h-24 border-4 border-cyan-500/30 border-t-cyan-400 rounded-full animate-spin mb-6"></div>
              <span class="font-bold tracking-widest uppercase animate-pulse">{{ t('Generating chunks...') }}</span>
            </div>

            <div v-else-if="mapModal.image" class="relative group max-w-full max-h-full flex items-center justify-center">
              <img :src="mapModal.image" class="max-w-full max-h-full object-contain shadow-2xl rounded-xl border border-white/10" style="image-rendering: pixelated;">
            </div>

            <div v-else class="flex flex-col items-center text-white/30 gap-4">
              <Globe class="w-16 h-16 opacity-50" />
              <p class="font-medium">{{ t('Map generation failed or region is empty.') }}</p>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="historyModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="historyModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[85vh] shadow-2xl relative overflow-hidden border-purple-500/30">
          <div class="absolute -bottom-32 -left-32 w-96 h-96 bg-purple-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="p-8 border-b border-white/5 flex justify-between items-center bg-black/40 shrink-0 relative z-10">
            <div>
              <h3 class="text-2xl font-extrabold flex items-center gap-3 text-purple-400">
                <History class="w-6 h-6" /> {{ t('Timeline') }}
              </h3>
              <p class="text-white/50 text-sm mt-1.5 font-medium">{{ t('World:') }} <span class="text-white">{{ historyModal.worldName }}</span></p>
            </div>
            <button @click="historyModal.isOpen = false" class="kip-btn-ghost p-2 border-transparent hover:bg-white/10 hover:text-red-400"><X class="w-6 h-6" /></button>
          </div>

          <div class="p-6 border-b border-white/5 bg-black/20 shrink-0 relative z-10">
            <button @click="commitWorld" :disabled="historyModal.loading" class="kip-btn-primary w-full py-4 bg-purple-500 hover:bg-purple-400 shadow-[0_0_15px_rgba(168,85,247,0.3)] border border-purple-400">
              <Loader v-if="historyModal.loading" class="w-5 h-5 animate-spin" />
              <Save v-else class="w-5 h-5" />
              {{ historyModal.loading ? t('Creating Snapshot...') : t('Create Snapshot') }}
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#050505]/80 relative z-10 min-h-0">
            <div v-if="historyModal.commits.length === 0" class="py-10 text-center text-white/30 font-medium">
              {{ t('No snapshots available.') }}
            </div>

            <div v-else class="relative border-l-2 border-purple-500/20 ml-4 space-y-8 pb-4">
              <div v-for="(commit, index) in historyModal.commits" :key="commit.id" class="relative pl-8 group">
                <div class="absolute -left-[9px] top-1.5 w-4 h-4 rounded-full border-4 border-[#050505] transition-colors" :class="index === 0 ? 'bg-purple-400 shadow-[0_0_10px_rgba(168,85,247,0.8)]' : 'bg-purple-500/40 group-hover:bg-purple-400'"></div>

                <div class="bg-black/40 border border-white/5 group-hover:border-purple-500/30 rounded-2xl p-5 transition-all duration-300 flex justify-between items-center shadow-inner group-hover:bg-black/60">
                  <div>
                    <div class="flex items-center gap-3 mb-1">
                      <span class="text-sm font-bold text-white flex items-center gap-1.5">
                        <GitCommit class="w-4 h-4 text-purple-400" />
                        <span class="font-mono uppercase">{{ commit.id }}</span>
                      </span>
                      <span v-if="index === 0" class="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-[9px] font-bold uppercase tracking-wider rounded border border-emerald-500/30">{{ t('Latest') }}</span>
                    </div>
                    <div class="text-xs text-white/50 flex items-center gap-1.5 mt-2">
                      <Clock class="w-3.5 h-3.5" />
                      {{ new Date(commit.timestamp).toLocaleString() }}
                    </div>
                  </div>

                  <button @click="restoreCommit(commit.id)" class="kip-btn-ghost px-5 py-2.5 text-sm text-purple-400 hover:text-white border-purple-500/20 hover:border-purple-500/50 hover:bg-purple-500/10 opacity-0 group-hover:opacity-100">
                    <History class="w-4 h-4" /> {{ t('Restore') }}
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

<script setup>
import { ref, onMounted } from 'vue'
import {
  Loader, Heart, UploadCloud, Map, History, Trash2,
  X, Save, RefreshCw, Earth, GitCommit, Clock
} from 'lucide-vue-next'
import { api, t, showToast } from '@/store.js'

const worldsList = ref([])
const isWorldsLoading = ref(false)

const mapModal = ref({ isOpen: false, worldName: '', loading: false, image: '' })
const historyModal = ref({ isOpen: false, worldName: '', loading: false, commits: [] })

const fallbackModIcon = 'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>'

const loadWorlds = async () => {
  if (!api.value) return
  isWorldsLoading.value = true
  try {
    const res = await api.value.get_worlds()
    worldsList.value = res.map(w => ({ ...w, healing: false, syncing: false }))
  } catch (e) {}
  isWorldsLoading.value = false
}

const healPlayer = async (world) => {
  if (!api.value || world.healing) return
  world.healing = true
  try {
    const res = await api.value.heal_world_player(world.name)
    if (res && res.success) {
      showToast(t("Player Healed"), `${t('Restored in')} ${world.name}.`, "success")
    } else {
      showToast(t("Error"), t("Failed to heal player."), "danger")
    }
  } catch (e) {}
  setTimeout(() => { world.healing = false }, 2000)
}

const deleteWorld = async (world) => {
  if (!api.value) return
  try {
    const res = await api.value.delete_world(world.name)
    if (res && res.success) {
      showToast(t("Deleted"), res.msg, "success")
      loadWorlds()
    } else {
      showToast(t("Error"), res.msg || t("Failed to delete world directory."), "danger")
    }
  } catch (e) {}
}

const syncCloudWorld = async (world) => {
  if (!api.value || world.syncing) return
  world.syncing = true
  try {
    const res = await api.value.sync_cloud_world(world.name)
    if (res && res.success) {
      showToast(t("Cloud Sync"), res.msg, "success")
    } else {
      showToast(t("Sync Error"), res.msg || t("Failed to sync world."), "danger")
    }
  } catch (e) {}
  setTimeout(() => { world.syncing = false }, 2000)
}

const openWorldMap = async (worldName) => {
  if (!api.value) return
  mapModal.value.worldName = worldName
  mapModal.value.image = ''
  mapModal.value.loading = true
  mapModal.value.isOpen = true
  try {
    const res = await api.value.get_world_map(worldName)
    if (res && res.success) {
      mapModal.value.image = res.image
    }
  } catch (e) {}
  mapModal.value.loading = false
}

const openWorldHistory = async (worldName) => {
  if (!api.value) return
  historyModal.value.worldName = worldName
  historyModal.value.commits = []
  historyModal.value.loading = true
  historyModal.value.isOpen = true
  try {
    historyModal.value.commits = await api.value.vcs_get_history(worldName)
  } catch (e) {}
  historyModal.value.loading = false
}

const commitWorld = async () => {
  if (!api.value || historyModal.value.loading) return
  historyModal.value.loading = true
  try {
    const res = await api.value.vcs_commit(historyModal.value.worldName)
    if (res && res.success) {
      showToast(t("Snapshot Created"), t("World state saved."), "success")
      historyModal.value.commits = await api.value.vcs_get_history(historyModal.value.worldName)
    } else {
      showToast(t("Error"), res.msg || t("Failed to create snapshot."), "danger")
    }
  } catch (e) {}
  historyModal.value.loading = false
}

const restoreCommit = async (commitId) => {
  if (!api.value) return
  try {
    const res = await api.value.vcs_restore(historyModal.value.worldName, commitId)
    if (res && res.success) {
      showToast(t("Restored"), t("World reverted successfully."), "success")
      historyModal.value.isOpen = false
    } else {
      showToast(t("Error"), res.msg || t("Failed to restore world state."), "danger")
    }
  } catch (e) {}
}

onMounted(() => {
  loadWorlds()
})
</script>