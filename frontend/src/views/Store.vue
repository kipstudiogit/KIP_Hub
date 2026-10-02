<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="flex justify-between items-start shrink-0 mb-6 stagger-1">
      <div>
        <h2 class="text-3xl font-extrabold mb-1">{{ t('Store') }}</h2>
        <p class="text-white/50 text-sm">{{ t('Discover millions of creations.') }}</p>
      </div>
      <div class="flex gap-1 bg-black/40 border border-white/10 p-1 rounded-xl">
        <button @click="setProvider('modrinth')" :class="storeData.provider === 'modrinth' ? 'bg-emerald-500 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)]' : 'text-white/60 hover:text-white'" class="px-5 py-2 rounded-lg text-sm font-bold transition">Modrinth</button>
        <button @click="setProvider('curseforge')" :class="storeData.provider === 'curseforge' ? 'bg-orange-500 text-black shadow-[0_0_15px_rgba(249,115,22,0.3)]' : 'text-white/60 hover:text-white'" class="px-5 py-2 rounded-lg text-sm font-bold transition">CurseForge</button>
      </div>
    </div>

    <div class="flex gap-2 overflow-x-auto custom-scroll pb-2 mb-4 shrink-0 stagger-1">
      <button @click="setStoreCategory('optimization')" :class="storeData.category === 'optimization' ? 'bg-indigo-500 text-white shadow-[0_0_15px_rgba(99,102,241,0.4)]' : 'bg-white/5 text-white/70 hover:bg-white/10'" class="px-5 py-2 rounded-full text-xs font-bold transition whitespace-nowrap">{{ t('Optimization') }}</button>
      <button @click="setStoreCategory('technology')" :class="storeData.category === 'technology' ? 'bg-indigo-500 text-white shadow-[0_0_15px_rgba(99,102,241,0.4)]' : 'bg-white/5 text-white/70 hover:bg-white/10'" class="px-5 py-2 rounded-full text-xs font-bold transition whitespace-nowrap">{{ t('Tech') }}</button>
      <button @click="setStoreCategory('magic')" :class="storeData.category === 'magic' ? 'bg-indigo-500 text-white shadow-[0_0_15px_rgba(99,102,241,0.4)]' : 'bg-white/5 text-white/70 hover:bg-white/10'" class="px-5 py-2 rounded-full text-xs font-bold transition whitespace-nowrap">{{ t('Magic') }}</button>
      <button @click="setStoreCategory('utility')" :class="storeData.category === 'utility' ? 'bg-indigo-500 text-white shadow-[0_0_15px_rgba(99,102,241,0.4)]' : 'bg-white/5 text-white/70 hover:bg-white/10'" class="px-5 py-2 rounded-full text-xs font-bold transition whitespace-nowrap">{{ t('Utility') }}</button>
      <button @click="setStoreCategory('worldgen')" :class="storeData.category === 'worldgen' ? 'bg-indigo-500 text-white shadow-[0_0_15px_rgba(99,102,241,0.4)]' : 'bg-white/5 text-white/70 hover:bg-white/10'" class="px-5 py-2 rounded-full text-xs font-bold transition whitespace-nowrap">{{ t('World Gen') }}</button>
      <button @click="setStoreCategory('')" v-if="storeData.category" class="px-4 py-2 rounded-full text-xs font-bold transition whitespace-nowrap bg-red-500/20 text-red-400 hover:bg-red-500/30 flex items-center"><X class="w-3 h-3" /></button>
    </div>

    <div class="flex gap-3 mb-8 shrink-0 relative z-30 stagger-2">
      <div class="relative flex-1 max-w-[150px] custom-dropdown">
        <div @click="activeDropdown = activeDropdown === 'type' ? null : 'type'"
             class="bg-black/40 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition h-full"
             :class="activeDropdown === 'type' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10 hover:border-white/20'">
          <span class="font-medium text-sm text-white/90 truncate">{{ typeOptions.find(o => o.value === storeData.type)?.label }}</span>
          <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0 ml-2" :class="{'rotate-180': activeDropdown === 'type'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'type'" class="absolute top-full left-0 w-full mt-2 bg-[#121214] border border-white/10 rounded-xl shadow-2xl overflow-hidden py-1 z-50">
            <div v-for="opt in typeOptions" :key="opt.value" @click="storeData.type = opt.value; activeDropdown = null; performStoreSearch()" class="px-4 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-medium" :class="storeData.type === opt.value ? 'text-indigo-400' : 'text-white/70'">
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1 max-w-[150px] custom-dropdown">
        <div @click="activeDropdown = activeDropdown === 'version' ? null : 'version'"
             class="bg-black/40 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition h-full"
             :class="activeDropdown === 'version' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10 hover:border-white/20'">
          <span class="font-medium text-sm text-white/90 truncate">{{ storeData.game_version || t('All Versions') }}</span>
          <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0 ml-2" :class="{'rotate-180': activeDropdown === 'version'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'version'" class="absolute top-full left-0 w-full mt-2 bg-[#121214] border border-white/10 rounded-xl shadow-2xl overflow-hidden py-1 z-50 max-h-60 overflow-y-auto custom-scroll">
            <div @click="storeData.game_version = ''; activeDropdown = null; performStoreSearch()" class="px-4 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-medium" :class="!storeData.game_version ? 'text-indigo-400' : 'text-white/70'">
              {{ t('All Versions') }}
            </div>
            <div v-for="v in state.mcVersions" :key="v" @click="storeData.game_version = v; activeDropdown = null; performStoreSearch()" class="px-4 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-medium" :class="storeData.game_version === v ? 'text-indigo-400' : 'text-white/70'">
              {{ v }}
            </div>
          </div>
        </transition>
      </div>

      <div v-if="storeData.type === 'mod'" class="relative flex-1 max-w-[150px] custom-dropdown">
        <div @click="activeDropdown = activeDropdown === 'loader' ? null : 'loader'"
             class="bg-black/40 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition h-full"
             :class="activeDropdown === 'loader' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10 hover:border-white/20'">
          <span class="font-medium text-sm text-white/90 truncate">{{ loaderOptions.find(o => o.value === storeData.loader)?.label }}</span>
          <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0 ml-2" :class="{'rotate-180': activeDropdown === 'loader'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'loader'" class="absolute top-full left-0 w-full mt-2 bg-[#121214] border border-white/10 rounded-xl shadow-2xl overflow-hidden py-1 z-50">
            <div v-for="opt in loaderOptions" :key="opt.value" @click="storeData.loader = opt.value; activeDropdown = null; performStoreSearch()" class="px-4 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-medium" :class="storeData.loader === opt.value ? 'text-indigo-400' : 'text-white/70'">
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1 max-w-[150px] custom-dropdown">
        <div @click="activeDropdown = activeDropdown === 'sort' ? null : 'sort'"
             class="bg-black/40 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition h-full"
             :class="activeDropdown === 'sort' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10 hover:border-white/20'">
          <span class="font-medium text-sm text-white/90 truncate">{{ sortOptions.find(o => o.value === storeData.sort_index)?.label }}</span>
          <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0 ml-2" :class="{'rotate-180': activeDropdown === 'sort'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'sort'" class="absolute top-full left-0 w-full mt-2 bg-[#121214] border border-white/10 rounded-xl shadow-2xl overflow-hidden py-1 z-50">
            <div v-for="opt in sortOptions" :key="opt.value" @click="storeData.sort_index = opt.value; activeDropdown = null; performStoreSearch()" class="px-4 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-medium" :class="storeData.sort_index === opt.value ? 'text-indigo-400' : 'text-white/70'">
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1 h-full">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/40 w-5 h-5" />
        <input v-model="storeData.query" @input="debounceStoreSearch" type="text" :placeholder="t('Search for content...')" class="kip-input h-full pl-12">
      </div>
    </div>

    <div class="grid grid-cols-2 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 stagger-3 min-h-0" @scroll="handleScroll">
      <template v-if="isStoreLoading && storeData.offset === 0">
        <div v-for="i in 6" :key="i" class="kip-card p-5 flex flex-col justify-between min-h-[220px]">
          <div class="flex items-start gap-4">
            <div class="w-14 h-14 rounded-xl skeleton-box flex-shrink-0"></div>
            <div class="flex-1 space-y-3 mt-1">
              <div class="h-4 w-3/4 rounded skeleton-box"></div>
              <div class="h-3 w-1/2 rounded skeleton-box"></div>
            </div>
          </div>
          <div class="h-10 w-full rounded-xl skeleton-box mt-auto"></div>
        </div>
      </template>
      <template v-else>
        <div v-if="storeResults.length === 0" class="col-span-2 text-center py-20 text-white/30 flex flex-col items-center gap-4">
          <SearchX class="w-16 h-16 opacity-50" />
          <span>{{ t('No results found.') }}</span>
        </div>

        <div v-for="item in storeResults" :key="item.project_id" class="kip-card kip-card-hover p-5 flex flex-col cursor-pointer relative overflow-hidden group min-h-[220px]" @click="openStoreItemDetails(item)">
          <div class="absolute -right-10 -top-10 w-32 h-32 blur-3xl rounded-full pointer-events-none transition-all duration-500" :class="item.provider === 'curseforge' ? 'bg-orange-500/10 group-hover:bg-orange-500/20' : 'bg-emerald-500/10 group-hover:bg-emerald-500/20'"></div>

          <div class="flex gap-4 items-start mb-3 relative z-10">
            <img :src="item.icon_url || fallbackModIcon" class="w-14 h-14 rounded-xl object-contain bg-black/40 p-1 border border-white/10 flex-shrink-0 shadow-lg group-hover:scale-105 transition-transform duration-500">
            <div class="flex-1 min-w-0 pt-0.5">
              <h3 class="font-bold text-white truncate text-base leading-tight mb-1">{{ item.title }}</h3>
              <p class="text-xs text-white/50 mb-2 flex items-center gap-2">
                <span class="flex items-center gap-1 truncate"><User class="w-3 h-3" /> {{ item.author }}</span>
                <span class="flex items-center gap-1 text-emerald-400/80 shrink-0"><Download class="w-3 h-3" /> {{ formatNumber(item.downloads) }}</span>
              </p>
              <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-[20px]">
                <span v-for="cat in (item.categories || []).slice(0, 3)" :key="cat" class="px-2 py-0.5 bg-white/10 rounded text-[9px] font-bold text-white/70 uppercase tracking-wider">{{ cat }}</span>
              </div>
            </div>
          </div>

          <p class="text-sm text-white/60 line-clamp-2 mb-4 relative z-10 leading-relaxed">{{ item.description }}</p>

          <div class="mt-auto relative z-10">
            <button @click.stop="downloadStoreItem(item)" :disabled="item.downloading" class="w-full py-2.5 rounded-xl transition-all text-sm font-bold relative overflow-hidden flex items-center justify-center gap-2 border shadow-lg" :class="item.downloaded ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' : 'bg-indigo-500 hover:bg-indigo-400 text-white border-indigo-500/50 hover:border-indigo-400'">
              <div class="absolute top-0 left-0 h-full bg-white/20 transition-all duration-300 pointer-events-none" :style="{ width: item.progress + '%' }"></div>
              <Loader v-if="item.downloading" class="w-4 h-4 animate-spin relative z-10" />
              <DownloadCloud v-else-if="!item.downloaded" class="w-4 h-4 relative z-10" />
              <CheckCircle v-else class="w-4 h-4 relative z-10" />
              <span class="relative z-10">{{ item.downloaded ? t('Installed') : item.downloading ? t('Downloading...') : t('Quick Download') }}</span>
            </button>
          </div>
        </div>

        <div v-if="isStoreLoadingMore" class="col-span-2 flex justify-center py-6">
          <Loader class="w-8 h-8 animate-spin text-indigo-500" />
        </div>
      </template>
    </div>

    <transition name="fade">
      <div v-if="storeDetailsModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="storeDetailsModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-4xl flex flex-col h-[85vh] overflow-hidden relative">
          <div class="flex justify-between items-start p-8 border-b border-white/5 bg-black/40 shrink-0 relative overflow-hidden">
            <div class="absolute -top-32 -right-32 w-96 h-96 blur-[100px] rounded-full pointer-events-none transition-all duration-1000" :class="storeDetailsModal.item?.provider === 'curseforge' ? 'bg-orange-500/20' : 'bg-emerald-500/20'"></div>

            <div class="flex items-center gap-5 relative z-10">
              <img :src="storeDetailsModal.item?.icon_url || fallbackModIcon" class="w-16 h-16 rounded-2xl bg-black/40 p-0.5 object-contain border border-white/10 shadow-lg">
              <div>
                <h3 class="text-3xl font-extrabold text-white mb-1">{{ storeDetailsModal.item?.title }}</h3>
                <p class="text-sm text-white/50 mb-3">{{ t('by') }} <span class="text-white/80 font-medium">{{ storeDetailsModal.item?.author }}</span></p>
                <div class="flex gap-4">
                  <span class="flex items-center gap-1.5 text-xs font-bold text-emerald-400 bg-emerald-500/10 px-3 py-1 rounded-lg border border-emerald-500/20"><Download class="w-3 h-3" /> {{ formatNumber(storeDetailsModal.item?.downloads || 0) }}</span>
                  <span class="flex items-center gap-1.5 text-xs font-bold text-amber-400 bg-amber-500/10 px-3 py-1 rounded-lg border border-amber-500/20"><Star class="w-3 h-3" /> {{ formatNumber(storeDetailsModal.item?.follows || 0) }}</span>
                </div>
              </div>
            </div>
            <button @click="storeDetailsModal.isOpen = false" class="kip-btn-ghost p-2 relative z-10"><X class="w-5 h-5" /></button>
          </div>

          <div class="flex border-b border-white/5 bg-black/20 shrink-0 relative z-10">
            <button @click="storeDetailsModal.tab = 'description'" :class="storeDetailsModal.tab === 'description' ? 'text-indigo-400 border-b-2 border-indigo-400 bg-white/5' : 'text-white/50 hover:text-white'" class="flex-1 py-4 font-bold transition flex items-center justify-center gap-2">
              <FileText class="w-4 h-4" /> {{ t('Description') }}
            </button>
            <button @click="storeDetailsModal.tab = 'versions'" :class="storeDetailsModal.tab === 'versions' ? 'text-indigo-400 border-b-2 border-indigo-400 bg-white/5' : 'text-white/50 hover:text-white'" class="flex-1 py-4 font-bold transition flex items-center justify-center gap-2">
              <Layers class="w-4 h-4" /> {{ t('Versions') }}
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll bg-[#050505]/80 flex flex-col min-h-0 relative z-10">
            <div v-show="storeDetailsModal.tab === 'description'">
              <div v-if="storeDetailsModal.itemData?.gallery && storeDetailsModal.itemData.gallery.length > 0" class="p-8 pb-0 flex gap-4 overflow-x-auto custom-scroll snap-x shrink-0">
                <img v-for="img in storeDetailsModal.itemData.gallery" :key="img.url" :src="img.url" class="h-64 object-cover rounded-xl border border-white/10 snap-center shrink-0 shadow-lg cursor-pointer hover:opacity-80 transition" @click="openUrl(img.url)" :title="img.title || ''">
              </div>
              <div class="p-8 markdown-body text-white/80" v-html="storeDetailsModal.html"></div>
            </div>

            <div v-show="storeDetailsModal.tab === 'versions'" class="p-6 flex flex-col gap-3">
              <div v-if="!storeDetailsModal.versions || storeDetailsModal.versions.length === 0" class="text-center py-20 text-white/50">{{ t('No matching versions found.') }}</div>
              <div v-else v-for="ver in storeDetailsModal.versions" :key="ver.id" class="bg-black/40 border border-white/5 rounded-xl overflow-hidden flex flex-col group transition" :class="ver.expanded ? 'border-indigo-500/30 shadow-lg' : 'hover:border-indigo-500/20'">
                <div class="p-5 flex items-center justify-between cursor-pointer" @click="ver.expanded = !ver.expanded">
                  <div>
                    <h4 class="font-bold text-white text-base">{{ ver.name }}</h4>
                    <p class="text-xs text-white/50 mt-1.5 font-mono">{{ ver.version_number }} • {{ new Date(ver.date).toLocaleDateString() }}</p>
                  </div>
                  <div class="flex items-center gap-3">
                    <button v-if="ver.changelog" class="text-[11px] font-bold text-indigo-400 hover:text-indigo-300 transition uppercase tracking-wider flex items-center gap-1.5 bg-indigo-500/10 px-3 py-2 rounded-lg" @click.stop="ver.expanded = !ver.expanded">
                      <FileCode class="w-3.5 h-3.5" /> {{ t('Changelog') }}
                    </button>
                    <button @click.stop="downloadSpecificVersion(ver)" :disabled="ver.downloading" class="kip-btn-primary px-5 py-2 text-sm">
                      <Loader v-if="ver.downloading" class="w-4 h-4 animate-spin" />
                      <Download v-else class="w-4 h-4" />
                      {{ ver.downloaded ? t('Installed') : ver.downloading ? t('Downloading...') : t('Download') }}
                    </button>
                  </div>
                </div>
                <div v-if="ver.expanded && ver.changelog" class="p-5 border-t border-white/5 bg-black/60 text-sm text-white/70 markdown-body" v-html="renderMarkdown(ver.changelog)"></div>
              </div>
            </div>
          </div>

          <div v-show="storeDetailsModal.tab === 'description'" class="p-6 border-t border-white/5 bg-black/40 shrink-0 flex justify-end gap-4 relative z-20">
            <button @click="storeDetailsModal.isOpen = false" class="kip-btn-ghost px-6 py-3 text-white/70 hover:text-white">{{ t('Close') }}</button>
            <button v-if="storeDetailsModal.item" @click="downloadStoreItem(storeDetailsModal.item)" :disabled="storeDetailsModal.item.downloading" class="kip-btn-primary px-8 py-3">
              <Loader v-if="storeDetailsModal.item.downloading" class="w-5 h-5 animate-spin" />
              <DownloadCloud v-else-if="!storeDetailsModal.item.downloaded" class="w-5 h-5" />
              <CheckCircle v-else class="w-5 h-5" />
              {{ storeDetailsModal.item.downloaded ? t('Installed') : storeDetailsModal.item.downloading ? t('Downloading...') : t('Quick Download') }}
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { marked } from 'marked'
import {
  X,
  Search,
  SearchX,
  Download,
  Loader,
  FileText,
  Layers,
  FileCode,
  ChevronDown,
  User,
  DownloadCloud,
  CheckCircle,
  Star,
} from 'lucide-vue-next'
import { state, t, showToast, sanitizeHTML } from '@/store'
import { invokeSafe, bridge } from '@/bridge'

export interface StoreItemFile {
  filename: string
  url: string
  primary?: boolean
}

export interface StoreItemVersion {
  id: string
  version_number: string
  name: string
  date: string
  changelog?: string
  files: StoreItemFile[]
  expanded?: boolean
  downloading?: boolean
  downloaded?: boolean
  progress?: number
  downloadTarget?: string
}

export interface StoreItemDetailsData {
  body?: string
  gallery?: Array<{ url: string; title?: string }>
}

export interface StoreItemRecord {
  project_id: string
  title: string
  author: string
  description: string
  icon_url: string
  downloads: number
  follows: number
  categories: string[]
  provider: 'modrinth' | 'curseforge'
  downloading?: boolean
  downloaded?: boolean
  progress?: number
  downloadTarget?: string
}

export interface StoreSearchResult {
  success: boolean
  hits?: StoreItemRecord[]
  msg?: string
}

export interface StoreDetailsResponse {
  success: boolean
  details: StoreItemDetailsData
  versions: StoreItemVersion[]
  msg?: string
}

interface SelectOption<T = string> {
  value: T
  label: string
}

type DropdownName = 'type' | 'version' | 'loader' | 'sort' | null

const storeData = ref({
  provider: 'modrinth' as 'modrinth' | 'curseforge',
  type: 'mod',
  loader: '',
  game_version: '',
  query: '',
  offset: 0,
  sort_index: 'relevance',
  category: '',
})

const isStoreLoading = ref<boolean>(false)
const isStoreLoadingMore = ref<boolean>(false)
const hasMoreStoreItems = ref<boolean>(true)
const storeResults = ref<StoreItemRecord[]>([])
let storeSearchTimeout: ReturnType<typeof setTimeout> | null = null

const storeDetailsModal = ref<{
  isOpen: boolean
  item: StoreItemRecord | null
  itemData: StoreItemDetailsData
  html: string
  tab: 'description' | 'versions'
  versions: StoreItemVersion[]
}>({
  isOpen: false,
  item: null,
  itemData: {},
  html: '',
  tab: 'description',
  versions: [],
})

const activeDropdown = ref<DropdownName>(null)

const typeOptions: SelectOption[] = [
  { value: 'mod', label: t('Mods') },
  { value: 'resourcepack', label: t('Resourcepacks') },
  { value: 'shader', label: t('Shaders') },
]

const loaderOptions: SelectOption[] = [
  { value: '', label: t('All Loaders') },
  { value: 'fabric', label: t('Fabric') },
  { value: 'forge', label: t('Forge') },
  { value: 'quilt', label: t('Quilt') },
  { value: 'neoforge', label: t('NeoForge') },
]

const sortOptions: SelectOption[] = [
  { value: 'relevance', label: t('Relevance') },
  { value: 'downloads', label: t('Downloads') },
  { value: 'newest', label: t('Newest') },
  { value: 'updated', label: t('Updated') },
]

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    activeDropdown.value = null
  }
}

const formatNumber = (num: number): string => {
  if (!num) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

const openUrl = (url?: string): void => {
  if (url) window.open(url, '_blank')
}

const renderMarkdown = (text?: string): string => {
  if (!text) return ''
  try {
    return marked.parse(text) as string
  } catch {
    return sanitizeHTML(text)
  }
}

const setProvider = (providerName: 'modrinth' | 'curseforge'): void => {
  storeData.value.provider = providerName
  performStoreSearch()
}

const setStoreCategory = (cat: string): void => {
  if (storeData.value.category === cat) {
    storeData.value.category = ''
  } else {
    storeData.value.category = cat
  }
  performStoreSearch()
}

const debounceStoreSearch = (): void => {
  if (storeSearchTimeout) clearTimeout(storeSearchTimeout)
  storeSearchTimeout = setTimeout(() => {
    performStoreSearch()
  }, 400)
}

const fetchStoreData = async (): Promise<StoreSearchResult> => {
  try {
    const res = await invokeSafe<StoreSearchResult>('search_store', {
      provider: storeData.value.provider,
      query: storeData.value.query.trim() || null,
      projectType: storeData.value.type || null,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
      category: storeData.value.category || null,
      sortIndex: storeData.value.sort_index || null,
      offset: storeData.value.offset,
    })
    return res || { success: true, hits: [] }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    return { success: false, hits: [], msg: message }
  }
}

const performStoreSearch = async (): Promise<void> => {
  isStoreLoading.value = true
  storeData.value.offset = 0
  hasMoreStoreItems.value = true

  try {
    const data = await fetchStoreData()
    if (!data || !data.success) {
      if (data && data.msg) {
        showToast(t('Store Notice'), data.msg, 'danger')
      }
      storeResults.value = []
      hasMoreStoreItems.value = false
    } else {
      if (!data.hits || data.hits.length === 0) {
        hasMoreStoreItems.value = false
      }
      storeResults.value = (data.hits || []).map((item) => ({
        ...item,
        downloading: false,
        downloaded: false,
        progress: 0,
        downloadTarget: '',
      }))
    }
  } catch (err: unknown) {
    hasMoreStoreItems.value = false
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isStoreLoading.value = false
  }
}

const loadMoreStoreItems = async (): Promise<void> => {
  if (isStoreLoadingMore.value || isStoreLoading.value || !hasMoreStoreItems.value) return
  isStoreLoadingMore.value = true
  storeData.value.offset += 12

  try {
    const data = await fetchStoreData()
    if (data && data.success && data.hits && data.hits.length > 0) {
      const newItems: StoreItemRecord[] = data.hits.map((item) => ({
        ...item,
        downloading: false,
        downloaded: false,
        progress: 0,
        downloadTarget: '',
      }))
      storeResults.value = [...storeResults.value, ...newItems]
    } else {
      hasMoreStoreItems.value = false
    }
  } catch {
    hasMoreStoreItems.value = false
  } finally {
    isStoreLoadingMore.value = false
  }
}

const handleScroll = (e: Event): void => {
  const el = e.target as HTMLElement
  if (el.scrollHeight - el.scrollTop <= el.clientHeight + 150) {
    loadMoreStoreItems()
  }
}

const openStoreItemDetails = async (item: StoreItemRecord): Promise<void> => {
  storeDetailsModal.value.item = item
  storeDetailsModal.value.itemData = {}
  storeDetailsModal.value.versions = []
  storeDetailsModal.value.tab = 'description'
  storeDetailsModal.value.html = `<div class="flex justify-center py-20"><svg class="w-10 h-10 animate-spin text-indigo-500" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 12a9 9 0 1 1-6.219-8.56"/></svg></div>`
  storeDetailsModal.value.isOpen = true

  try {
    const res = await invokeSafe<StoreDetailsResponse>('get_store_full_details', {
      provider: item.provider,
      projectId: item.project_id,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (res && res.success) {
      storeDetailsModal.value.itemData = res.details
      storeDetailsModal.value.versions = (res.versions || []).map((v) => ({
        ...v,
        expanded: false,
        downloading: false,
        downloaded: false,
        progress: 0,
        downloadTarget: '',
      }))

      if (res.details.body) {
        storeDetailsModal.value.html = marked.parse(res.details.body) as string
      } else {
        storeDetailsModal.value.html = `<div class="text-center text-white/50 py-20">${t('No description provided.')}</div>`
      }
    } else {
      storeDetailsModal.value.html = `<div class="text-center text-red-400 py-20">${res?.msg || t('Failed to load description.')}</div>`
    }
  } catch {
    storeDetailsModal.value.html = `<div class="text-center text-red-400 py-20">${t('Failed to load description.')}</div>`
  }
}

const downloadStoreItem = async (item: StoreItemRecord): Promise<void> => {
  if (item.downloading) return
  item.downloading = true

  try {
    const res = await invokeSafe<StoreDetailsResponse>('get_store_full_details', {
      provider: item.provider,
      projectId: item.project_id,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (!res || !res.success || !res.versions || res.versions.length === 0) {
      showToast(t('Error'), t('No versions found.'), 'danger')
      item.downloading = false
      return
    }

    const firstVer = res.versions[0]
    if (!firstVer || !firstVer.files || firstVer.files.length === 0) {
      showToast(t('Error'), t('No files in this version.'), 'danger')
      item.downloading = false
      return
    }

    const targetFile = firstVer.files.find((f) => f.primary) || firstVer.files[0]
    if (!targetFile) {
      showToast(t('Error'), t('Target file not found.'), 'danger')
      item.downloading = false
      return
    }

    item.downloadTarget = targetFile.filename
    const success = await invokeSafe<boolean>('download_store_item', {
      url: targetFile.url,
      filename: targetFile.filename,
      projectType: storeData.value.type,
    })

    if (success) {
      item.downloaded = true
      item.progress = 100
      showToast(t('Success'), `${targetFile.filename} downloaded.`, 'success')
    } else {
      showToast(t('Error'), t('Download failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Network error.'), 'danger')
  } finally {
    item.downloading = false
  }
}

const downloadSpecificVersion = async (ver: StoreItemVersion): Promise<void> => {
  if (ver.downloading) return
  ver.downloading = true

  try {
    if (!ver.files || ver.files.length === 0) {
      showToast(t('Error'), t('No files in this version.'), 'danger')
      ver.downloading = false
      return
    }

    const targetFile = ver.files.find((f) => f.primary) || ver.files[0]
    if (!targetFile) {
      showToast(t('Error'), t('File could not be resolved.'), 'danger')
      ver.downloading = false
      return
    }

    ver.downloadTarget = targetFile.filename
    const success = await invokeSafe<boolean>('download_specific_file', {
      url: targetFile.url,
      filename: targetFile.filename,
      projectType: storeData.value.type,
    })

    if (success) {
      ver.downloaded = true
      showToast(t('Installed'), `${targetFile.filename} downloaded.`, 'success')
    } else {
      showToast(t('Error'), t('Download failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Network error.'), 'danger')
  } finally {
    ver.downloading = false
  }
}

declare global {
  interface Window {
    updateDownloadProgress?: ((filename: string, p: number) => void) | null
  }
}

onMounted(async () => {
  window.addEventListener('click', closeDropdowns)

  if (state.mcVersions.length === 0) {
    try {
      state.mcVersions = await bridge.getMcVersions()
    } catch {
      // Ignored
    }
  }

  await performStoreSearch()

  window.updateDownloadProgress = (filename: string, p: number): void => {
    const item = storeResults.value.find((i) => i.downloadTarget === filename)
    if (item) item.progress = p
    if (storeDetailsModal.value.item && storeDetailsModal.value.item.downloadTarget === filename) {
      storeDetailsModal.value.item.progress = p
    }
    if (storeDetailsModal.value.versions) {
      const ver = storeDetailsModal.value.versions.find((v) => v.downloadTarget === filename)
      if (ver) ver.progress = p
    }
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
  window.updateDownloadProgress = null
  if (storeSearchTimeout) clearTimeout(storeSearchTimeout)
})
</script>