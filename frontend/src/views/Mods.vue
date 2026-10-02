<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="flex justify-between items-end mb-6 shrink-0 stagger-1">
      <div>
        <h2 class="text-3xl font-extrabold mb-1">{{ t('Content Manager') }}</h2>
        <p class="text-white/50 text-sm">{{ t('Enable, disable, or import modifications.') }}</p>
      </div>
      <div class="flex gap-2 items-center">
        <button @click="openHub" class="kip-btn-ghost px-4 py-2 text-sm text-blue-400 hover:text-white border-blue-500/20 hover:border-blue-500/50 hover:bg-blue-500/20">
          <Globe class="w-4 h-4" /> {{ t('Hub') }}
        </button>
        <button @click="openSwarm" class="kip-btn-ghost px-4 py-2 text-sm text-amber-400 hover:text-white border-amber-500/20 hover:border-amber-500/50 hover:bg-amber-500/20">
          <Magnet class="w-4 h-4" /> {{ t('Swarm') }}
        </button>

        <div class="w-px h-6 bg-white/10 mx-1"></div>

        <button @click="checkModUpdates" :disabled="isCheckingUpdates" class="kip-btn-ghost px-4 py-2 text-sm text-emerald-400 hover:text-white border-emerald-500/20 hover:border-emerald-500/50 hover:bg-emerald-500/20">
          <Loader v-if="isCheckingUpdates" class="w-4 h-4 animate-spin" />
          <ArrowUpCircle v-else class="w-4 h-4" />
          {{ t('Update') }}
        </button>
        <button @click="exportModpack" :disabled="isExporting" class="kip-btn-ghost px-4 py-2 text-sm text-purple-400 hover:text-white border-purple-500/20 hover:border-purple-500/50 hover:bg-purple-500/20">
          <Loader v-if="isExporting" class="w-4 h-4 animate-spin" />
          <Package v-else class="w-4 h-4" />
          {{ t('Export') }}
        </button>

        <div class="w-px h-6 bg-white/10 mx-1"></div>

        <button @click="isGraphView = !isGraphView" class="kip-btn-ghost px-4 py-2 text-sm" :class="isGraphView ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30' : ''">
          <LayoutGrid v-if="isGraphView" class="w-4 h-4" />
          <GitMerge v-else class="w-4 h-4" />
          {{ isGraphView ? t('Grid View') : t('Node Graph') }}
        </button>
        <button @click="importMods" :disabled="isImporting" class="kip-btn-ghost px-4 py-2 text-sm">
          <Loader v-if="isImporting" class="w-4 h-4 animate-spin" />
          <Download v-else class="w-4 h-4" />
          {{ t('Import') }}
        </button>
        <button @click="loadMods" class="kip-btn-ghost p-2.5">
          <RefreshCw class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div v-show="!isGraphView" class="mb-6 shrink-0 stagger-2">
      <div class="relative max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/40 w-5 h-5" />
        <input v-model="localSearchQuery" type="text" :placeholder="t('Search installed mods...')" class="kip-input pl-12">
      </div>
    </div>

    <div v-show="!isGraphView" class="grid grid-cols-2 gap-4 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 stagger-3 min-h-0">
      <template v-if="isModsLoading">
        <div v-for="i in 6" :key="i" class="kip-card p-5 flex items-center gap-4">
          <div class="w-16 h-16 rounded-2xl skeleton-box flex-shrink-0"></div>
          <div class="flex-1 space-y-3">
            <div class="h-4 w-3/4 rounded skeleton-box"></div>
            <div class="h-3 w-1/2 rounded skeleton-box"></div>
          </div>
        </div>
      </template>
      <template v-else>
        <div v-if="filteredMods.length === 0" class="col-span-2 text-center py-20 text-white/30 flex flex-col items-center gap-4">
          <SearchX class="w-16 h-16 opacity-50" />
          <span>{{ t('No mods found.') }}</span>
        </div>

        <div v-for="mod in filteredMods" :key="mod.filename" class="kip-card p-4 flex items-center gap-5 cursor-pointer relative overflow-hidden group transition-all duration-500" :class="mod.disabled ? 'opacity-50 grayscale hover:grayscale-0 hover:opacity-100' : 'kip-card-hover'" @click="openModDetails(mod)">
          <div v-if="!mod.disabled" class="absolute -right-10 -top-10 w-24 h-24 bg-indigo-500/10 blur-2xl rounded-full pointer-events-none group-hover:bg-indigo-500/20 transition-all duration-500"></div>

          <img :src="mod.icon || fallbackModIcon" class="w-14 h-14 rounded-2xl object-contain bg-black/40 p-1 border border-white/10 flex-shrink-0 shadow-lg relative z-10">

          <div class="flex-1 min-w-0 relative z-10">
            <h3 class="font-bold text-white truncate text-base leading-tight mb-1" :class="mod.disabled ? 'text-white/70' : 'text-white'">{{ mod.name }}</h3>
            <p class="text-xs text-white/50 truncate mb-2 font-mono">{{ mod.version }} • {{ mod.author }}</p>
            <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-[22px]">
              <span v-for="loader in mod.loaders" :key="loader" class="px-2 py-0.5 rounded text-[9px] font-bold uppercase tracking-wider border" :class="getLoaderColor(loader)">
                {{ loader }}
              </span>
            </div>
          </div>

          <div class="flex flex-col items-end gap-4 relative z-10 shrink-0 pr-2">
            <div class="relative inline-block w-10 align-middle select-none cursor-pointer" @click.stop="toggleMod(mod)">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none transition-transform" :checked="!mod.disabled" style="pointer-events: none;">
              <label class="toggle-label block overflow-hidden h-5 rounded-full transition-colors border border-white/10" style="pointer-events: none;"></label>
            </div>
            <button @click.stop="deleteMod(mod)" class="text-white/30 hover:text-red-400 transition" :title="t('Delete Mod')">
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>
      </template>
    </div>

    <div v-show="isGraphView" class="w-full h-full kip-card mt-2 relative overflow-hidden shrink-0 min-h-[500px] stagger-3">
      <div ref="graphContainer" class="w-full h-full"></div>
      <div class="absolute top-4 left-4 kip-card p-4 text-xs text-white/70 z-10 flex flex-col gap-3 pointer-events-none">
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]"></span> <span class="font-bold">{{ t('Requires') }}</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]"></span> <span class="font-bold">{{ t('Incompatible') }}</span></div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="modDetailsModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="modDetailsModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl flex flex-col max-h-[80vh] overflow-hidden relative">
          <div class="flex justify-between items-start p-8 border-b border-white/5 bg-black/40 shrink-0 relative overflow-hidden">
            <div class="absolute -top-32 -right-32 w-96 h-96 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none"></div>
            <div class="flex items-center gap-5 relative z-10">
              <img :src="modDetailsModal.mod?.icon || fallbackModIcon" class="w-20 h-20 rounded-2xl bg-black/40 p-1 object-contain border border-white/10 shadow-lg">
              <div>
                <h3 class="text-3xl font-extrabold text-white mb-2">{{ modDetailsModal.mod?.name }}</h3>
                <div class="flex items-center gap-3">
                  <span class="text-sm font-mono text-indigo-400 bg-indigo-500/10 px-2 py-0.5 rounded border border-indigo-500/20">{{ modDetailsModal.mod?.version }}</span>
                  <span class="text-sm text-white/50">{{ t('by') }} <span class="text-white/80 font-medium">{{ modDetailsModal.mod?.author }}</span></span>
                </div>
              </div>
            </div>
            <button @click="modDetailsModal.isOpen = false" class="kip-btn-ghost p-2 relative z-10"><X class="w-5 h-5" /></button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#050505]/80 flex flex-col gap-6 min-h-0">
            <div class="grid grid-cols-2 gap-6">
              <div>
                <h4 class="text-xs font-bold text-white/40 uppercase tracking-wider mb-2 flex items-center gap-2"><Info class="w-4 h-4" /> Mod ID</h4>
                <p class="font-mono text-sm text-white/80 bg-white/5 p-2 rounded-lg border border-white/5 select-all">{{ modDetailsModal.meta.id || 'Unknown' }}</p>
              </div>
              <div>
                <h4 class="text-xs font-bold text-white/40 uppercase tracking-wider mb-2 flex items-center gap-2"><Layers class="w-4 h-4" /> Loaders</h4>
                <div class="flex gap-2 flex-wrap">
                  <span v-for="loader in modDetailsModal.mod?.loaders || []" :key="loader" class="px-2 py-1 rounded text-xs font-bold uppercase tracking-wider border" :class="getLoaderColor(loader)">
                    {{ loader }}
                  </span>
                </div>
              </div>
            </div>

            <div v-if="modDetailsModal.meta.depends.length > 0">
              <h4 class="text-xs font-bold text-emerald-400 uppercase tracking-wider mb-3 flex items-center gap-2"><ArrowUpCircle class="w-4 h-4" /> {{ t('Requires') }}</h4>
              <div class="flex gap-2 flex-wrap">
                <span v-for="dep in modDetailsModal.meta.depends" :key="dep" class="bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-3 py-1.5 rounded-lg text-xs font-mono font-medium">
                  {{ dep }}
                </span>
              </div>
            </div>

            <div v-if="modDetailsModal.meta.breaks.length > 0">
              <h4 class="text-xs font-bold text-red-400 uppercase tracking-wider mb-3 flex items-center gap-2"><AlertTriangle class="w-4 h-4" /> {{ t('Incompatible') }}</h4>
              <div class="flex gap-2 flex-wrap">
                <span v-for="br in modDetailsModal.meta.breaks" :key="br" class="bg-red-500/10 text-red-400 border border-red-500/20 px-3 py-1.5 rounded-lg text-xs font-mono font-medium">
                  {{ br }}
                </span>
              </div>
            </div>

            <div v-if="modDetailsModal.meta.jij.length > 0">
              <h4 class="text-xs font-bold text-indigo-400 uppercase tracking-wider mb-3 flex items-center gap-2"><Package class="w-4 h-4" /> Included Libraries (JiJ)</h4>
              <div class="flex flex-col gap-2">
                <span v-for="jij in modDetailsModal.meta.jij" :key="jij" class="text-xs font-mono text-white/60 bg-white/5 px-3 py-2 rounded-lg border border-white/5 truncate">
                  {{ jij }}
                </span>
              </div>
            </div>

            <div>
              <h4 class="text-xs font-bold text-white/40 uppercase tracking-wider mb-2 flex items-center gap-2"><FileText class="w-4 h-4" /> Filename</h4>
              <p class="font-mono text-xs text-white/50">{{ modDetailsModal.mod?.filename }}</p>
            </div>
          </div>

          <div class="p-6 border-t border-white/5 bg-black/40 shrink-0 flex justify-between items-center relative z-20">
            <button v-if="modDetailsModal.mod" @click="toggleMod(modDetailsModal.mod)" class="px-6 py-2.5 rounded-xl font-bold transition flex items-center gap-2" :class="modDetailsModal.mod.disabled ? 'bg-emerald-500/10 text-emerald-400 hover:bg-emerald-500/20 border border-emerald-500/20' : 'bg-amber-500/10 text-amber-400 hover:bg-amber-500/20 border border-amber-500/20'">
              {{ modDetailsModal.mod.disabled ? t('ENABLE') : t('Disable') }}
            </button>
            <div class="flex gap-3">
              <button @click="modDetailsModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5">{{ t('Close') }}</button>
              <button v-if="modDetailsModal.mod" @click="deleteMod(modDetailsModal.mod); modDetailsModal.isOpen = false" class="kip-btn-danger px-6 py-2.5">
                <Trash2 class="w-4 h-4" /> {{ t('Delete') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="hubModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="hubModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-4xl border-blue-500/30 flex flex-col h-[85vh] relative overflow-hidden">
          <div class="absolute -top-32 -left-32 w-96 h-96 bg-blue-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="p-8 border-b border-white/5 flex justify-between items-center bg-black/40 shrink-0 relative z-10">
            <h3 class="text-2xl font-extrabold flex items-center gap-3 text-white">
              <Globe class="w-7 h-7 text-blue-400" /> Community Hub
            </h3>
            <button @click="hubModal.isOpen = false" class="kip-btn-ghost p-2"><X class="w-5 h-5" /></button>
          </div>

          <div class="flex border-b border-white/5 bg-black/20 shrink-0 relative z-10">
            <button @click="hubModal.tab = 'browse'; loadHubPresets()" :class="hubModal.tab === 'browse' ? 'text-blue-400 border-b-2 border-blue-400 bg-white/5' : 'text-white/50 hover:text-white'" class="flex-1 py-4 font-bold transition flex items-center justify-center gap-2">
              <Search class="w-4 h-4" /> Browse Profiles
            </button>
            <button @click="hubModal.tab = 'publish'" :class="hubModal.tab === 'publish' ? 'text-blue-400 border-b-2 border-blue-400 bg-white/5' : 'text-white/50 hover:text-white'" class="flex-1 py-4 font-bold transition flex items-center justify-center gap-2">
              <UploadCloud class="w-4 h-4" /> Publish Active Profile
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#050505]/80 relative z-10 min-h-0">
            <div v-if="hubModal.tab === 'browse'" class="h-full">
              <div v-if="hubModal.loading" class="flex justify-center items-center h-full">
                <Loader class="w-8 h-8 animate-spin text-blue-500" />
              </div>
              <div v-else-if="hubModal.presets.length === 0" class="flex flex-col justify-center items-center h-full text-white/40">
                <CloudOff class="w-12 h-12 mb-4 opacity-50" />
                <p>{{ t('Hub is empty! Be the first to publish.') }}</p>
              </div>
              <div v-else class="grid grid-cols-2 gap-5">
                <div v-for="preset in hubModal.presets" :key="preset.id" class="kip-card p-6 kip-card-hover group border-white/5">
                  <h4 class="font-bold text-xl text-white mb-1">{{ preset.title }}</h4>
                  <p class="text-xs text-white/50 mb-3">{{ t('by') }} <span class="font-medium text-white/80">{{ preset.author }}</span></p>
                  <p class="text-sm text-white/70 line-clamp-2 mb-5 leading-relaxed">{{ preset.description }}</p>
                  <button @click="applyHubPreset(preset)" class="w-full py-2.5 bg-blue-500/10 hover:bg-blue-500 hover:text-white text-blue-400 font-bold rounded-xl transition flex items-center justify-center gap-2 border border-blue-500/20 shadow-lg">
                    <DownloadCloud class="w-4 h-4" /> Apply Preset
                  </button>
                </div>
              </div>
            </div>

            <div v-if="hubModal.tab === 'publish'" class="max-w-xl mx-auto h-full flex flex-col justify-center">
              <div class="kip-card p-8 flex flex-col gap-5 border-white/5">
                <div>
                  <label class="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1.5 block">Title</label>
                  <input v-model="hubModal.form.title" type="text" :placeholder="t('Modpack Title')" class="kip-input">
                </div>
                <div>
                  <label class="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1.5 block">Author</label>
                  <input v-model="hubModal.form.author" type="text" :placeholder="t('Author Name')" class="kip-input">
                </div>
                <div>
                  <label class="text-[10px] font-bold text-white/40 uppercase tracking-wider mb-1.5 block">Description</label>
                  <textarea v-model="hubModal.form.desc" :placeholder="t('Description...')" class="kip-input h-32 resize-none custom-scroll leading-relaxed"></textarea>
                </div>
                <button @click="publishToHub" :disabled="hubModal.loading || !hubModal.form.title.trim()" class="kip-btn-primary py-3.5 mt-2 bg-blue-500 hover:bg-blue-400 shadow-[0_0_15px_rgba(59,130,246,0.3)]">
                  <Loader v-if="hubModal.loading" class="w-5 h-5 animate-spin" />
                  <UploadCloud v-else class="w-5 h-5" />
                  {{ t('Publish Profile') }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="p2pModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="p2pModal.isOpen = false">
        <div class="kip-card p-8 w-full max-w-xl border-amber-500/30 flex flex-col relative overflow-hidden">
          <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-amber-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <h3 class="text-2xl font-extrabold flex items-center gap-3 mb-2 relative z-10 text-amber-400">
            <Magnet class="w-7 h-7" /> Swarm Download
          </h3>
          <p class="text-white/50 text-sm mb-6 relative z-10">{{ t('Enter a magnet link to download a modpack via P2P.') }}</p>

          <input v-model="p2pModal.magnet" type="text" placeholder="magnet:?xt=urn:btih:..." class="kip-input mb-8 font-mono text-amber-400 relative z-10 focus:border-amber-500">

          <div class="flex justify-end gap-3 relative z-10">
            <button @click="p2pModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5">{{ t('Cancel') }}</button>
            <button @click="startSwarm" :disabled="!p2pModal.magnet.trim()" class="kip-btn-primary px-8 py-2.5 bg-amber-500 hover:bg-amber-400 text-black border-amber-400 shadow-[0_0_15px_rgba(245,158,11,0.3)]">
              <Download class="w-4 h-4 fill-current" /> Download
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { Network } from 'vis-network'
import {
  ArrowUpCircle,
  Package,
  Globe,
  Magnet,
  LayoutGrid,
  GitMerge,
  Download,
  RefreshCw,
  Trash2,
  Loader,
  Search,
  SearchX,
  X,
  Info,
  Layers,
  AlertTriangle,
  FileText,
  UploadCloud,
  CloudOff,
  DownloadCloud,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type LocalModRecord,
  type ModGraphDataDto,
  type GraphEdge,
  type GenericActionResult,
} from '@/bridge'

export interface ModMetaInfo {
  id: string
  depends: string[]
  breaks: string[]
  jij: string[]
}

export interface HubPreset {
  id: string
  title: string
  author: string
  description: string
  preset: string[]
}

interface ModDetailsModalState {
  isOpen: boolean
  mod: LocalModRecord | null
  meta: ModMetaInfo
}

interface HubModalState {
  isOpen: boolean
  tab: 'browse' | 'publish'
  presets: HubPreset[]
  loading: boolean
  form: {
    title: string
    author: string
    desc: string
  }
}

interface P2PModalState {
  isOpen: boolean
  magnet: string
}

const modsList = ref<LocalModRecord[]>([])
const isModsLoading = ref<boolean>(true)
const isGraphView = ref<boolean>(false)
const isImporting = ref<boolean>(false)
const isCheckingUpdates = ref<boolean>(false)
const isExporting = ref<boolean>(false)
const graphContainer = ref<HTMLElement | null>(null)
const localSearchQuery = ref<string>('')
let networkInstance: Network | null = null

const modDetailsModal = ref<ModDetailsModalState>({
  isOpen: false,
  mod: null,
  meta: { id: '', depends: [], breaks: [], jij: [] },
})

const p2pModal = ref<P2PModalState>({
  isOpen: false,
  magnet: '',
})

const hubModal = ref<HubModalState>({
  isOpen: false,
  tab: 'browse',
  presets: [],
  loading: false,
  form: {
    title: '',
    author: state.settings.ms_name || 'Guest',
    desc: '',
  },
})

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

const filteredMods = computed<LocalModRecord[]>(() => {
  const q = localSearchQuery.value.trim().toLowerCase()
  if (!q) return modsList.value
  return modsList.value.filter(
    (m) => m.name.toLowerCase().includes(q) || m.filename.toLowerCase().includes(q)
  )
})

const getLoaderColor = (loader: string): string => {
  const l = loader.toLowerCase()
  if (l === 'fabric') return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
  if (l === 'forge') return 'bg-rose-500/10 text-rose-400 border-rose-500/20'
  if (l === 'neoforge') return 'bg-orange-500/10 text-orange-400 border-orange-500/20'
  if (l === 'quilt') return 'bg-purple-500/10 text-purple-400 border-purple-500/20'
  return 'bg-white/10 text-white/70 border-white/20'
}

const loadMods = async (): Promise<void> => {
  isModsLoading.value = true
  try {
    modsList.value = await bridge.getLocalMods()
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to load mods.'), 'danger')
  } finally {
    isModsLoading.value = false
  }
}

const toggleMod = async (mod: LocalModRecord): Promise<void> => {
  try {
    const success = await bridge.toggleMod(mod.filename)
    if (success) {
      mod.disabled = !mod.disabled
      mod.filename = mod.disabled
        ? mod.filename + '.disabled'
        : mod.filename.replace('.disabled', '')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to toggle mod state.'), 'danger')
  }
}

const deleteMod = async (mod: LocalModRecord): Promise<void> => {
  try {
    const res: GenericActionResult = await bridge.deleteMod(mod.filename)
    if (res.success) {
      showToast(t('Deleted'), res.msg, 'success')
      await loadMods()
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to delete mod.'), 'danger')
  }
}

const importMods = async (): Promise<void> => {
  if (isImporting.value) return
  isImporting.value = true
  try {
    const res: GenericActionResult = await bridge.importModsDialog()
    if (res.success) {
      showToast(t('Import Complete'), res.msg, 'success')
      await loadMods()
    } else if (res.msg !== 'No files selected.') {
      showToast(t('Import Failed'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to import mods.'), 'danger')
  } finally {
    isImporting.value = false
  }
}

const checkModUpdates = async (): Promise<void> => {
  if (isCheckingUpdates.value) return
  isCheckingUpdates.value = true
  showToast(t('Scanning'), t('Checking for mod updates...'), 'info')

  try {
    const res = await bridge.checkModUpdates()
    if (res.success && res.updates && res.updates.length > 0) {
      showToast(
        t('Updating'),
        `${t('Found')} ${res.updates.length} ${t('updates. Downloading...')}`,
        'info'
      )
      const applyRes = await bridge.applyModUpdates(res.updates)
      if (applyRes.success) {
        showToast(t('Success'), applyRes.msg, 'success')
        await loadMods()
      } else {
        showToast(t('Error'), applyRes.msg, 'danger')
      }
    } else if (res.success) {
      showToast(t('Up to date'), t('All mods are on the latest version.'), 'success')
    } else {
      showToast(t('Error'), t('Failed to verify mod updates.'), 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to check updates.'), 'danger')
  } finally {
    isCheckingUpdates.value = false
  }
}

const exportModpack = async (): Promise<void> => {
  if (isExporting.value) return
  isExporting.value = true
  showToast(t('Exporting'), t('Generating modpack archive...'), 'info')

  try {
    const res: GenericActionResult = await bridge.exportModpack()
    if (res.success) {
      showToast(t('Exported'), res.msg, 'success')
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to export modpack.'), 'danger')
  } finally {
    isExporting.value = false
  }
}

const openModDetails = async (mod: LocalModRecord): Promise<void> => {
  modDetailsModal.value.mod = mod
  modDetailsModal.value.meta = { id: '', depends: [], breaks: [], jij: [] }
  modDetailsModal.value.isOpen = true

  try {
    const graphData = await invokeSafe<ModGraphDataDto>('get_mod_graph_data')
    if (graphData && graphData.edges) {
      const modId = mod.name.toLowerCase().replace(/ /g, '-')
      const deps = graphData.edges
        .filter((e: GraphEdge) => e.from === modId && e.color?.color === '#10B981')
        .map((e: GraphEdge) => e.to)
      const breaks = graphData.edges
        .filter((e: GraphEdge) => e.from === modId && e.color?.color === '#EF4444')
        .map((e: GraphEdge) => e.to)

      modDetailsModal.value.meta = {
        id: modId,
        depends: deps,
        breaks: breaks,
        jij: [],
      }
    }
  } catch {
    modDetailsModal.value.meta = {
      id: mod.name.toLowerCase().replace(/ /g, '-'),
      depends: [],
      breaks: [],
      jij: [],
    }
  }
}

const renderGraph = async (): Promise<void> => {
  if (!graphContainer.value) return
  try {
    const data = await invokeSafe<ModGraphDataDto>('get_mod_graph_data')
    if (networkInstance) {
      networkInstance.destroy()
      networkInstance = null
    }

    networkInstance = new Network(
      graphContainer.value,
      {
        nodes: data.nodes,
        edges: data.edges,
      },
      {
        nodes: { font: { color: '#ffffff' }, borderWidth: 2 },
        edges: {
          smooth: {
            enabled: true,
            type: 'continuous',
            roundness: 0.5,
          },
        },
        physics: {
          barnesHut: { gravitationalConstant: -2000, centralGravity: 0.3, springLength: 95 },
        },
      }
    )
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to build mod graph.'), 'danger')
  }
}

const openHub = (): void => {
  hubModal.value.isOpen = true
  hubModal.value.tab = 'browse'
  loadHubPresets()
}

const loadHubPresets = async (): Promise<void> => {
  hubModal.value.loading = true
  try {
    const presets = await invokeSafe<HubPreset[]>('fetch_hub')
    hubModal.value.presets = presets || []
  } catch {
    hubModal.value.presets = []
  } finally {
    hubModal.value.loading = false
  }
}

const publishToHub = async (): Promise<void> => {
  if (!hubModal.value.form.title.trim()) return
  hubModal.value.loading = true

  try {
    const activeMods = modsList.value
      .filter((m) => !m.disabled)
      .map((m) => m.name || m.filename)

    if (activeMods.length === 0) {
      showToast(t('Error'), t('No active mods found to share.'), 'danger')
      hubModal.value.loading = false
      return
    }

    const success = await invokeSafe<boolean>('publish_hub', {
      title: hubModal.value.form.title,
      author: hubModal.value.form.author,
      desc: hubModal.value.form.desc,
      mods: activeMods,
    })

    if (success) {
      showToast(t('Success'), t('Profile published to Hub!'), 'success')
      hubModal.value.form = {
        title: '',
        author: state.settings.ms_name || 'Guest',
        desc: '',
      }
      hubModal.value.tab = 'browse'
      await loadHubPresets()
    } else {
      showToast(t('Error'), t('Failed to publish profile.'), 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to publish profile.'), 'danger')
  } finally {
    hubModal.value.loading = false
  }
}

const applyHubPreset = async (preset: HubPreset): Promise<void> => {
  hubModal.value.isOpen = false
  showToast(t('Applying'), `Downloading preset: ${preset.title}`, 'info')

  try {
    const issues = preset.preset.map((modName) => ({
      action: 'DOWNLOAD',
      target: modName,
      selected: true,
    }))

    const res = await invokeSafe<{ success: boolean; downloaded: number }>('apply_doctor_fixes', {
      issues,
    })

    if (res && (res.success || res.downloaded > 0)) {
      showToast(t('Success'), `Downloaded ${res.downloaded || 0} mods.`, 'success')
      await loadMods()
    } else {
      showToast(t('Error'), t('Failed to download preset mods.'), 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Failed to download preset mods.'), 'danger')
  }
}

const openSwarm = (): void => {
  p2pModal.value.isOpen = true
  p2pModal.value.magnet = ''
}

const startSwarm = async (): Promise<void> => {
  if (!p2pModal.value.magnet.trim()) return

  try {
    const res = await invokeSafe<{ success: boolean; msg?: string }>('swarm_download', {
      magnet: p2pModal.value.magnet.trim(),
      targetDir: 'MODS_DIR',
    })

    if (res && res.success) {
      showToast(t('Swarm'), t('P2P download started...'), 'info')
      p2pModal.value.isOpen = false
    } else {
      showToast(t('Error'), res?.msg || t('Swarm download failed.'), 'danger')
    }
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), message || t('Swarm download failed.'), 'danger')
  }
}

watch(isGraphView, (val) => {
  if (val) {
    nextTick(() => {
      renderGraph()
    })
  } else if (networkInstance) {
    networkInstance.destroy()
    networkInstance = null
  }
})

onMounted(() => {
  loadMods()
})

onBeforeUnmount(() => {
  if (networkInstance) {
    networkInstance.destroy()
    networkInstance = null
  }
})
</script>