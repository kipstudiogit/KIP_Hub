<script setup lang="ts">
import { onMounted } from 'vue'
import {
  Loader,
  Heart,
  UploadCloud,
  Map,
  History,
  Trash2,
  RefreshCw,
  Earth,
  Search,
  Copy,
  Skull,
  Sun,
  CopyPlus,
  Compass,
  Lock,
} from 'lucide-vue-next'
import { t, showToast } from '@/store'
import { useWorldsManager } from '../composables/useWorldsManager'
import WorldMapModal from '../components/worlds/WorldMapModal.vue'
import WorldHistoryModal from '../components/worlds/WorldHistoryModal.vue'

const {
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
  loadWorlds,
  healPlayer,
  changeGamemode,
  cloneWorld,
  deleteWorld,
  openMapModal,
  openHistoryModal,
  commitActiveWorld,
  revertCommit,
} = useWorldsManager()

const fallbackWorldIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>'

async function copySeed(seed: string): Promise<void> {
  await navigator.clipboard.writeText(seed)
  showToast(t('Copied'), `Seed: ${seed}`, 'success')
}

onMounted(() => {
  loadWorlds()
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <!-- Header -->
    <header class="flex justify-between items-end mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Worlds') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">NBT Universe Inspector, Emergency Patient Recovery & Time Machine</p>
      </div>

      <div class="flex items-center gap-3">
        <div class="kip-card px-4 py-2 flex items-center gap-3 bg-black/40 border border-white/10 shadow-lg">
          <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Total Worlds Storage:</span>
          <span class="text-xs font-mono font-black text-indigo-400">{{ totalDiskUsage }} MB</span>
        </div>

        <button
          @click="loadWorlds"
          :disabled="isWorldsLoading"
          class="kip-btn-ghost p-2.5 border-white/10 text-white/70 hover:text-white"
          :title="t('Refresh universes')"
        >
          <RefreshCw class="w-4 h-4" :class="isWorldsLoading ? 'animate-spin text-indigo-400' : ''" />
        </button>
      </div>
    </header>

    <!-- Filters Strip -->
    <div class="flex justify-between items-center gap-4 mb-6 shrink-0 z-10">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
        <input
          v-model="searchQuery"
          type="text"
          :placeholder="t('Filter worlds by name, seed or version...')"
          class="kip-input pl-11 py-2 text-xs font-mono"
        >
      </div>

      <div class="flex items-center gap-2">
        <div class="flex p-1 bg-black/40 rounded-xl border border-white/10">
          <button
            v-for="f in (['all', 'survival', 'hardcore', 'creative', 'spectator'] as const)"
            :key="f"
            @click="activeFilter = f"
            class="px-3.5 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider transition"
            :class="activeFilter === f ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'"
          >
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

    <!-- Worlds Universe Grid -->
    <main class="grid grid-cols-2 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10">
      <div v-if="isWorldsLoading && worldsList.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center justify-center">
        <Loader class="w-10 h-10 animate-spin text-indigo-400 mb-4" />
        <span class="text-white/40 font-mono text-xs uppercase tracking-widest">{{ t('Mounting universes & parsing NBT data...') }}</span>
      </div>

      <div v-else-if="filteredWorlds.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
        <Earth class="w-16 h-16 text-white/10" />
        <span class="text-white/40 font-mono text-xs">{{ searchQuery ? 'No worlds matching your query.' : t('No worlds found.') }}</span>
      </div>

      <article
        v-for="w in filteredWorlds"
        :key="w.name"
        class="kip-card p-6 flex flex-col justify-between group relative overflow-hidden transition-all duration-300 border border-white/5 hover:border-indigo-500/30 hover:bg-black/50"
      >
        <div class="flex items-start gap-5 mb-5 relative z-10">
          <div class="relative shrink-0">
            <img :src="w.icon || fallbackWorldIcon" class="w-20 h-20 rounded-2xl object-cover bg-black/60 p-1 border border-white/10 shadow-xl group-hover:scale-105 transition-transform duration-300">
            <span v-if="w.hardcore" class="absolute -bottom-2 -right-2 p-1.5 bg-red-600 text-white rounded-lg shadow-lg border border-red-400" title="Hardcore World">
              <Skull class="w-3.5 h-3.5" />
            </span>
            <span v-else-if="w.isLocked" class="absolute -bottom-2 -right-2 p-1.5 bg-amber-600 text-white rounded-lg shadow-lg border border-amber-400" title="Locked by session.lock">
              <Lock class="w-3.5 h-3.5" />
            </span>
          </div>

          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2 mb-1.5">
              <h3 class="font-black text-white text-xl truncate leading-tight">{{ w.name }}</h3>
              <span class="text-[10px] font-mono text-white/30 shrink-0">{{ w.lastPlayed }}</span>
            </div>

            <!-- Gamemode & Specs Tags -->
            <div class="flex flex-wrap gap-1.5 mb-2.5">
              <!-- Inline Gamemode Selector -->
              <select
                :value="w.mode.toLowerCase() === 'survival' ? 0 : w.mode.toLowerCase() === 'creative' ? 1 : w.mode.toLowerCase() === 'adventure' ? 2 : 3"
                @change="changeGamemode(w, Number(($event.target as HTMLSelectElement).value))"
                class="px-2 py-0.5 rounded text-[9px] font-black uppercase tracking-wider border bg-black/60 cursor-pointer focus:outline-none"
                :class="w.hardcore ? 'bg-red-500/10 text-red-400 border-red-500/20' : 'text-white/80 border-white/15'"
              >
                <option value="0">Survival</option>
                <option value="1">Creative</option>
                <option value="2">Adventure</option>
                <option value="3">Spectator</option>
              </select>

              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
                MC {{ w.mcVersion }}
              </span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-amber-500/10 text-amber-400 border border-amber-500/20 flex items-center gap-1">
                <Sun class="w-2.5 h-2.5" /> Day {{ w.dayCount }}
              </span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-white/5 text-white/50 border border-white/10">
                {{ w.sizeMb }} MB
              </span>
            </div>

            <div class="flex items-center justify-between text-xs text-white/40 font-mono">
              <span class="truncate flex items-center gap-1"><Compass class="w-3 h-3 text-cyan-400" /> Spawn: {{ w.spawnX }}, {{ w.spawnY }}, {{ w.spawnZ }}</span>
              <button @click="copySeed(w.seed)" class="hover:text-indigo-400 transition flex items-center gap-1 text-[10px]">
                <span>Seed: {{ w.seed.slice(0, 10) }}...</span>
                <Copy class="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>

        <!-- Action Commands Deck -->
        <footer class="pt-4 border-t border-white/5 flex items-center justify-between gap-2 relative z-10">
          <div class="flex gap-2 flex-wrap">
            <button
              @click="healPlayer(w)"
              :disabled="w.healing"
              class="kip-btn-ghost px-3 py-2 text-xs hover:text-emerald-400 hover:border-emerald-500/40 hover:bg-emerald-500/10"
              :title="t('Recover player from void fall and clear negative effects')"
            >
              <Heart class="w-3.5 h-3.5 text-emerald-400" :class="w.healing ? 'animate-ping' : ''" />
              <span>{{ w.healing ? 'Healing...' : 'Heal & Fix' }}</span>
            </button>
            <button
              @click="openHistoryModal(w.name)"
              class="kip-btn-ghost px-3 py-2 text-xs hover:text-purple-400 hover:border-purple-500/40 hover:bg-purple-500/10"
              title="VCS Time Machine Snapshots"
            >
              <History class="w-3.5 h-3.5 text-purple-400" />
              <span>Timeline</span>
            </button>
            <button
              @click="openMapModal(w.name)"
              class="kip-btn-ghost px-3 py-2 text-xs hover:text-cyan-400 hover:border-cyan-500/40 hover:bg-cyan-500/10"
              title="Cartographer Satellite Radar"
            >
              <Map class="w-3.5 h-3.5 text-cyan-400" />
              <span>Map</span>
            </button>
            <button
              @click="cloneWorld(w)"
              :disabled="w.cloning"
              class="kip-btn-ghost px-3 py-2 text-xs hover:text-amber-400 hover:border-amber-500/40 hover:bg-amber-500/10"
              title="Duplicate Universe for Testing"
            >
              <CopyPlus class="w-3.5 h-3.5 text-amber-400" :class="w.cloning ? 'animate-spin' : ''" />
              <span>Clone</span>
            </button>
          </div>

          <button
            @click="deleteWorld(w)"
            class="p-2 rounded-xl text-red-400/60 hover:text-red-400 hover:bg-red-500/10 transition"
            :title="t('Delete universe')"
          >
            <Trash2 class="w-4 h-4" />
          </button>
        </footer>
      </article>
    </main>

    <!-- Map Satellite Modal Component -->
    <WorldMapModal
      v-model="isMapModalOpen"
      :world-name="mapWorldName"
      :image="mapImageBase64"
      :spawn-x="mapSpawnX"
      :spawn-z="mapSpawnZ"
      :loading="isMapLoading"
      :radius="mapRadius"
      @change-radius="openMapModal(mapWorldName, $event)"
    />

    <!-- VCS History Modal Component -->
    <WorldHistoryModal
      v-model="isHistoryModalOpen"
      :world-name="historyWorldName"
      :commits="worldCommits"
      :loading="isHistoryLoading"
      @commit="commitActiveWorld"
      @revert="revertCommit"
    />
  </div>
</template>