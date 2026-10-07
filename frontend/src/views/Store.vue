<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import {
  Search,
  SearchX,
  Box,
  Flame,
  X,
  ChevronDown,
  User,
  Download,
  DownloadCloud,
  Check,
  CheckCircle2,
  Trash2,
  Loader,
  Cpu,
  Sparkles,
  Wrench,
  Globe2,
  Compass,
  Layers,
  Sword,
  Paintbrush,
  HardDrive,
} from 'lucide-vue-next'
import { t } from '@/store'
import type {
  StoreCategoryBadge,
  StoreFilterOption,
  StoreItemRecordDto,
  StoreDetailsResponseDto,
  StoreItemVersionDto,
} from '../types/store'
import { useStoreManager } from '../composables/useStoreManager'
import StoreDetailsModal from '../components/store/StoreDetailsModal.vue'

const {
  provider,
  projectType,
  loader,
  gameVersion,
  searchQuery,
  sortIndex,
  category,
  offset,
  isStoreLoading,
  isStoreLoadingMore,
  storeResults,
  searchCatalog,
  loadMore,
  fetchProjectDetails,
  installItem,
  unpinItem,
} = useStoreManager()

const activeDropdown = ref<'type' | 'version' | 'loader' | 'sort' | null>(null)
let debounceTimer: ReturnType<typeof setTimeout> | null = null

const isDetailsModalOpen = ref(false)
const selectedItem = ref<StoreItemRecordDto | null>(null)
const selectedDetails = ref<StoreDetailsResponseDto | null>(null)
const isDetailsLoading = ref(false)

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

const categoryBadges: StoreCategoryBadge[] = [
  { id: 'optimization', name: 'Optimization', icon: Cpu },
  { id: 'technology', name: 'Technology', icon: Wrench },
  { id: 'magic', name: 'Magic', icon: Sparkles },
  { id: 'utility', name: 'Utility', icon: Compass },
  { id: 'worldgen', name: 'World Gen', icon: Globe2 },
  { id: 'adventure', name: 'Adventure', icon: Sword },
  { id: 'decoration', name: 'Decoration', icon: Paintbrush },
  { id: 'storage', name: 'Storage', icon: HardDrive },
]

const typeOptions: StoreFilterOption[] = [
  { value: 'mod', label: 'Mods' },
  { value: 'modpack', label: 'Modpacks' },
  { value: 'resourcepack', label: 'Resourcepacks' },
  { value: 'shader', label: 'Shaders' },
]

const loaderOptions: StoreFilterOption[] = [
  { value: '', label: 'All Loaders' },
  { value: 'fabric', label: 'Fabric' },
  { value: 'neoforge', label: 'NeoForge' },
  { value: 'forge', label: 'Forge' },
  { value: 'quilt', label: 'Quilt' },
]

const sortOptions: StoreFilterOption[] = [
  { value: 'relevance', label: 'Relevance' },
  { value: 'downloads', label: 'Downloads' },
  { value: 'newest', label: 'Newest' },
  { value: 'updated', label: 'Updated' },
]

const versionCatalog = [
  '26.3',
  '26.2',
  '26.1',
  '1.21.4',
  '1.21.3',
  '1.21.2',
  '1.21.1',
  '1.21',
  '1.20.6',
  '1.20.4',
  '1.20.2',
  '1.20.1',
  '1.19.4',
  '1.19.2',
  '1.18.2',
  '1.16.5',
  '1.12.2',
  '1.7.10',
]

function formatNumber(num: number): string {
  if (!num) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

function onSearchInput(): void {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    searchCatalog(true)
  }, 350)
}

function handleScroll(e: Event): void {
  const el = e.target as HTMLElement
  if (el.scrollHeight - el.scrollTop <= el.clientHeight + 200) {
    loadMore()
  }
}

async function openDetails(item: StoreItemRecordDto): Promise<void> {
  selectedItem.value = item
  selectedDetails.value = null
  isDetailsLoading.value = true
  isDetailsModalOpen.value = true

  selectedDetails.value = await fetchProjectDetails(item.projectId)
  isDetailsLoading.value = false
}

function handleModalInstall(item: StoreItemRecordDto, ver?: StoreItemVersionDto): void {
  installItem(item, ver)
}

function handleModalUninstall(item: StoreItemRecordDto): void {
  unpinItem(item)
}

function closeDropdowns(): void {
  activeDropdown.value = null
}

onMounted(() => {
  searchCatalog(true)
  window.addEventListener('click', closeDropdowns)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
  if (debounceTimer) clearTimeout(debounceTimer)
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <header class="flex justify-between items-start shrink-0 mb-6 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Store') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Automated Content Registry, Satellite Modpacks & Dependency Resolver</p>
      </div>

      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          @click="provider = 'modrinth'; searchCatalog(true)"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="provider === 'modrinth' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-white/40 hover:text-white'"
        >
          <Box class="w-3.5 h-3.5" /> Modrinth
        </button>
        <button
          @click="provider = 'curseforge'; searchCatalog(true)"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="provider === 'curseforge' ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30' : 'text-white/40 hover:text-white'"
        >
          <Flame class="w-3.5 h-3.5" /> CurseForge
        </button>
      </div>
    </header>

    <div class="flex gap-2 overflow-x-auto custom-scroll pb-2 mb-4 shrink-0 z-10">
      <button
        v-for="cat in categoryBadges"
        :key="cat.id"
        @click="category = category === cat.id ? '' : cat.id; searchCatalog(true)"
        class="px-4 py-1.5 rounded-full text-xs font-bold transition flex items-center gap-1.5 border cursor-pointer shrink-0"
        :class="category === cat.id ? 'bg-indigo-500 text-white border-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.5)]' : 'bg-white/5 border-white/5 text-white/60 hover:text-white'"
      >
        <component :is="cat.icon" class="w-3 h-3" />
        <span>{{ cat.name }}</span>
      </button>
      <button
        v-if="category"
        @click="category = ''; searchCatalog(true)"
        class="px-3 py-1.5 rounded-full text-xs font-bold bg-red-500/20 text-red-400 border border-red-500/30 flex items-center cursor-pointer"
      >
        <X class="w-3 h-3" />
      </button>
    </div>

    <div class="flex gap-3 mb-6 shrink-0 relative z-30">
      <div class="relative flex-1 max-w-[170px]" @click.stop>
        <div
          @click="activeDropdown = activeDropdown === 'type' ? null : 'type'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer transition hover:border-white/30"
          :class="activeDropdown === 'type' ? 'border-indigo-500' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ typeOptions.find(o => o.value === projectType)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2" :class="{'rotate-180': activeDropdown === 'type'}" />
        </div>
        <div v-if="activeDropdown === 'type'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
          <div
            v-for="opt in typeOptions"
            :key="opt.value"
            @click="projectType = opt.value as any; activeDropdown = null; searchCatalog(true)"
            class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase"
            :class="projectType === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
          >
            {{ opt.label }}
          </div>
        </div>
      </div>

      <div class="relative flex-1 max-w-[170px]" @click.stop>
        <div
          @click="activeDropdown = activeDropdown === 'version' ? null : 'version'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer hover:border-white/30"
          :class="activeDropdown === 'version' ? 'border-indigo-500' : 'border-white/10'"
        >
          <span class="font-mono text-xs font-bold text-white truncate">{{ gameVersion || t('All Versions') }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2" :class="{'rotate-180': activeDropdown === 'version'}" />
        </div>
        <div v-if="activeDropdown === 'version'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50 max-h-56 overflow-y-auto custom-scroll">
          <div
            @click="gameVersion = ''; activeDropdown = null; searchCatalog(true)"
            class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold border-b border-white/5"
            :class="!gameVersion ? 'text-indigo-400' : 'text-white/70'"
          >
            {{ t('All Versions') }}
          </div>
          <div
            v-for="v in versionCatalog"
            :key="v"
            @click="gameVersion = v; activeDropdown = null; searchCatalog(true)"
            class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold"
            :class="gameVersion === v ? 'text-indigo-400' : 'text-white/70'"
          >
            {{ v }}
          </div>
        </div>
      </div>

      <div v-if="projectType === 'mod' || projectType === 'modpack'" class="relative flex-1 max-w-[150px]" @click.stop>
        <div
          @click="activeDropdown = activeDropdown === 'loader' ? null : 'loader'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer hover:border-white/30"
          :class="activeDropdown === 'loader' ? 'border-indigo-500' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ loaderOptions.find(o => o.value === loader)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2" :class="{'rotate-180': activeDropdown === 'loader'}" />
        </div>
        <div v-if="activeDropdown === 'loader'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
          <div
            v-for="opt in loaderOptions"
            :key="opt.value"
            @click="loader = opt.value; activeDropdown = null; searchCatalog(true)"
            class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase"
            :class="loader === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
          >
            {{ opt.label }}
          </div>
        </div>
      </div>

      <div class="relative flex-1 max-w-[160px]" @click.stop>
        <div
          @click="activeDropdown = activeDropdown === 'sort' ? null : 'sort'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer hover:border-white/30"
          :class="activeDropdown === 'sort' ? 'border-indigo-500' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ sortOptions.find(o => o.value === sortIndex)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2" :class="{'rotate-180': activeDropdown === 'sort'}" />
        </div>
        <div v-if="activeDropdown === 'sort'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
          <div
            v-for="opt in sortOptions"
            :key="opt.value"
            @click="sortIndex = opt.value; activeDropdown = null; searchCatalog(true)"
            class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase"
            :class="sortIndex === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
          >
            {{ opt.label }}
          </div>
        </div>
      </div>

      <div class="relative flex-1">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
        <input
          v-model="searchQuery"
          @input="onSearchInput"
          type="text"
          :placeholder="t('Search modules, packages and libraries...')"
          class="kip-input pl-11 py-2 text-xs font-mono"
        >
      </div>
    </div>

    <main class="grid grid-cols-2 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10" @scroll="handleScroll">
      <div v-if="isStoreLoading && offset === 0" class="col-span-2 text-center py-24 flex flex-col items-center justify-center">
        <Loader class="w-10 h-10 animate-spin text-indigo-400 mb-3" />
        <span class="text-xs font-mono uppercase text-white/40">Querying registry index...</span>
      </div>

      <div v-else-if="storeResults.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
        <SearchX class="w-16 h-16 text-white/20" />
        <span class="text-white/40 font-mono text-xs">No matching packages discovered.</span>
      </div>

      <article
        v-for="item in storeResults"
        :key="item.projectId"
        @click="openDetails(item)"
        class="kip-card kip-card-hover p-5 flex flex-col justify-between cursor-pointer relative overflow-hidden group min-h-[220px] transition-all duration-300 border border-white/5 hover:border-indigo-500/30 hover:bg-black/50"
      >
        <div>
          <div class="flex gap-4 items-start mb-3 relative z-10">
            <div class="relative shrink-0">
              <img
                :src="item.iconUrl || fallbackModIcon"
                class="w-16 h-16 rounded-2xl object-cover bg-black/60 p-1 border border-white/10 shadow-xl group-hover:scale-105 transition-transform"
              >
              <span
                v-if="item.isInstalled"
                class="absolute -bottom-1 -right-1 p-1 bg-emerald-500 text-black rounded-lg shadow-md border border-emerald-300"
                :title="t('Installed in instance')"
              >
                <Check class="w-3 h-3 stroke-[3]" />
              </span>
            </div>

            <div class="flex-1 min-w-0">
              <div class="flex items-center justify-between gap-2 mb-1">
                <h3 class="font-black text-white text-base truncate leading-tight">{{ item.title }}</h3>
                <span
                  class="px-2 py-0.5 rounded text-[8px] font-mono font-bold uppercase tracking-wider border shrink-0"
                  :class="item.provider === 'curseforge' ? 'bg-orange-500/10 text-orange-400 border-orange-500/20' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'"
                >
                  {{ item.provider }}
                </span>
              </div>

              <div class="flex items-center gap-3 text-xs text-white/40 mb-2">
                <span class="flex items-center gap-1 truncate font-medium"><User class="w-3 h-3 text-white/30" /> {{ item.author }}</span>
                <span class="flex items-center gap-1 text-emerald-400 font-mono font-bold shrink-0"><Download class="w-3 h-3" /> {{ formatNumber(item.downloads) }}</span>
              </div>

              <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-[22px]">
                <span
                  v-for="cat in (item.categories || []).slice(0, 3)"
                  :key="cat"
                  class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-white/5 text-white/50 border border-white/10 uppercase"
                >
                  {{ cat }}
                </span>
              </div>
            </div>
          </div>

          <p class="text-xs text-white/60 line-clamp-2 leading-relaxed font-medium mb-4 relative z-10">{{ item.description }}</p>
        </div>

        <div class="relative z-10 flex items-center gap-2 pt-2">
          <template v-if="item.isInstalled">
            <button
              @click.stop="openDetails(item)"
              class="flex-1 py-2.5 rounded-xl font-bold text-xs uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex items-center justify-center gap-2 cursor-pointer"
            >
              <CheckCircle2 class="w-4 h-4" />
              <span>{{ t('Installed') }}</span>
            </button>
            <button
              @click.stop="unpinItem(item)"
              class="p-2.5 rounded-xl bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white border border-red-500/20 transition cursor-pointer"
              :title="t('Uninstall from instance')"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </template>

          <template v-else>
            <button
              @click.stop="installItem(item)"
              :disabled="item.downloading"
              class="w-full py-2.5 rounded-xl font-black text-xs uppercase tracking-widest transition-all duration-300 relative overflow-hidden flex items-center justify-center gap-2 border shadow-lg cursor-pointer"
              :class="item.downloading ? 'bg-indigo-950 text-indigo-300 border-indigo-500/30' : 'bg-indigo-500 hover:bg-indigo-400 text-white border-indigo-400 shadow-[0_0_20px_rgba(99,102,241,0.3)]'"
            >
              <div
                v-if="item.downloading"
                class="absolute top-0 left-0 h-full bg-indigo-600/40 transition-all duration-150 pointer-events-none"
                :style="{ width: (item.progress || 0) + '%' }"
              ></div>
              <Loader v-if="item.downloading" class="w-4 h-4 animate-spin relative z-10" />
              <DownloadCloud v-else class="w-4 h-4 relative z-10" />
              <span class="relative z-10">{{ item.downloading ? `${item.statusText || 'Installing'} (${Math.round(item.progress || 0)}%)` : t('Install 1-Click') }}</span>
            </button>
          </template>
        </div>
      </article>

      <div v-if="isStoreLoadingMore" class="col-span-2 flex justify-center py-6">
        <Loader class="w-8 h-8 animate-spin text-indigo-400" />
      </div>
    </main>

    <StoreDetailsModal
      v-model="isDetailsModalOpen"
      :item="selectedItem"
      :details="selectedDetails"
      :loading="isDetailsLoading"
      @install="handleModalInstall"
      @uninstall="handleModalUninstall"
    />
  </div>
</template>