<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { marked } from 'marked'
import {
  X,
  Clock,
  Layers,
  FileText,
  DownloadCloud,
  Trash2,
  Loader,
  ExternalLink,
  ChevronRight,
  ShieldCheck,
  Search,
  Tag,
  Check,
} from 'lucide-vue-next'
import type {
  StoreItemRecordDto,
  StoreDetailsResponseDto,
  StoreItemVersionDto,
} from '../../types/store'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

const props = defineProps<{
  item: StoreItemRecordDto | null
  details: StoreDetailsResponseDto | null
  loading: boolean
}>()

const emit = defineEmits<{
  (e: 'install', item: StoreItemRecordDto, version?: StoreItemVersionDto): void
  (e: 'uninstall', item: StoreItemRecordDto): void
}>()

const activeTab = ref<'description' | 'versions'>('description')
const markdownHtml = ref<string>('')
const versionFilterQuery = ref('')
const selectedModalGameVersion = ref('')

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

function formatNumber(num: number): string {
  if (!num) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

function openExternalUrl(url: string): void {
  window.open(url, '_blank')
}

const availableGameVersionsInModal = computed(() => {
  if (!props.details?.versions) return []
  const set = new Set<string>()
  for (const v of props.details.versions) {
    if (v.gameVersions) {
      for (const gv of v.gameVersions) {
        set.add(gv)
      }
    }
  }
  return Array.from(set).slice(0, 16)
})

const filteredProjectVersions = computed(() => {
  if (!props.details?.versions) return []
  const query = versionFilterQuery.value.trim().toLowerCase()
  const gvFilter = selectedModalGameVersion.value

  return props.details.versions.filter((ver) => {
    if (gvFilter && !ver.gameVersions?.includes(gvFilter)) {
      return false
    }

    if (query) {
      const matchName = (ver.name || '').toLowerCase().includes(query)
      const matchNum = (ver.versionNumber || '').toLowerCase().includes(query)
      const matchGv = (ver.gameVersions || []).some((g) => g.toLowerCase().includes(query))
      const matchLdr = (ver.loaders || []).some((l) => l.toLowerCase().includes(query))
      return matchName || matchNum || matchGv || matchLdr
    }

    return true
  })
})

watch(
  () => props.details,
  async (newVal) => {
    selectedModalGameVersion.value = ''
    versionFilterQuery.value = ''
    if (newVal && newVal.details && newVal.details.body) {
      markdownHtml.value = (await marked.parse(newVal.details.body)) as string
    } else {
      markdownHtml.value =
        '<div class="text-white/40 font-mono text-xs">No descriptive manifest payload provided.</div>'
    }
  }
)
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && item"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-5xl shadow-[0_0_60px_rgba(0,0,0,0.8)] overflow-hidden border border-white/10">
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
          <div
            class="absolute -top-32 -right-32 w-80 h-80 blur-[100px] rounded-full pointer-events-none opacity-20"
            :class="item.provider === 'curseforge' ? 'bg-orange-500' : 'bg-emerald-500'"
          ></div>

          <div class="flex items-center gap-5 relative z-10">
            <img
              :src="item.iconUrl || fallbackModIcon"
              class="w-16 h-16 rounded-2xl bg-black/60 p-1 object-cover border border-white/10 shadow-2xl"
            >
            <div>
              <h3 class="text-2xl font-black text-white leading-tight mb-1">{{ item.title }}</h3>
              <p class="text-xs text-white/50 flex items-center gap-3 font-mono">
                <span>By <span class="text-white font-bold">{{ item.author }}</span></span>
                <span class="text-white/20">•</span>
                <span class="text-emerald-400 font-bold">{{ formatNumber(item.downloads) }} downloads</span>
                <span class="text-white/20">•</span>
                <span
                  class="uppercase text-[9px] px-2 py-0.5 rounded border"
                  :class="item.provider === 'curseforge' ? 'bg-orange-500/10 text-orange-400 border-orange-500/30' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'"
                >
                  {{ item.provider }}
                </span>
              </p>
            </div>
          </div>

          <div class="flex items-center gap-3 relative z-10">
            <button
              v-if="item.isInstalled"
              @click="emit('uninstall', item)"
              class="kip-btn-danger px-4 py-2 text-xs uppercase tracking-wider font-bold cursor-pointer"
            >
              <Trash2 class="w-3.5 h-3.5" />
              <span>{{ t('Uninstall') }}</span>
            </button>
            <button
              v-else
              @click="emit('install', item)"
              :disabled="item.downloading"
              class="kip-btn-primary px-6 py-2.5 text-xs uppercase tracking-wider font-black shadow-[0_0_20px_rgba(99,102,241,0.4)] cursor-pointer"
            >
              <Loader v-if="item.downloading" class="w-4 h-4 animate-spin" />
              <DownloadCloud v-else class="w-4 h-4" />
              <span>{{ t('Install') }}</span>
            </button>

            <button
              @click="isOpen = false"
              class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition cursor-pointer"
            >
              <X class="w-5 h-5" />
            </button>
          </div>
        </header>

        <div class="flex border-b border-white/5 bg-black/30 shrink-0">
          <button
            @click="activeTab = 'description'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2 cursor-pointer"
            :class="activeTab === 'description' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'"
          >
            <FileText class="w-3.5 h-3.5" /> Overview & Gallery
          </button>
          <button
            @click="activeTab = 'versions'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2 cursor-pointer"
            :class="activeTab === 'versions' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'"
          >
            <Layers class="w-3.5 h-3.5" /> Release Catalog ({{ details?.versions?.length || 0 }})
          </button>
        </div>

        <main class="flex-1 overflow-y-auto custom-scroll bg-[#030305]/95 p-8 relative min-h-0">
          <div v-if="loading" class="py-24 text-center flex flex-col items-center justify-center">
            <Loader class="w-8 h-8 animate-spin text-indigo-400 mb-3" />
            <span class="text-xs font-mono uppercase text-white/40">Loading remote manifest & telemetry...</span>
          </div>

          <div v-else-if="activeTab === 'description'" class="flex flex-col gap-6">
            <div
              v-if="details?.details?.gallery && details.details.gallery.length > 0"
              class="flex gap-4 overflow-x-auto custom-scroll pb-4 snap-x shrink-0"
            >
              <img
                v-for="img in details.details.gallery"
                :key="img.url"
                :src="img.url"
                class="h-56 rounded-2xl object-cover border border-white/10 shadow-2xl cursor-pointer hover:border-indigo-500 transition-all duration-300 snap-center"
                @click="openExternalUrl(img.url)"
              >
            </div>

            <div
              class="p-6 bg-black/40 border border-white/5 rounded-3xl markdown-body text-white/80 leading-relaxed font-sans select-text"
              v-html="markdownHtml"
            ></div>
          </div>

          <div v-else-if="activeTab === 'versions'" class="flex flex-col gap-4">
            <div class="flex flex-col gap-3 pb-3 border-b border-white/5 bg-black/30 p-4 rounded-2xl">
              <div class="relative">
                <Search class="w-3.5 h-3.5 text-white/40 absolute left-3 top-1/2 -translate-y-1/2" />
                <input
                  v-model="versionFilterQuery"
                  type="text"
                  placeholder="Filter versions by title, Minecraft version (e.g. 1.20.1) or loader..."
                  class="kip-input py-1.5 pl-9 text-xs font-mono w-full"
                >
              </div>

              <div v-if="availableGameVersionsInModal.length > 0" class="flex items-center gap-1.5 flex-wrap">
                <span class="text-[9px] font-mono uppercase text-white/40 font-bold mr-1">Game Version:</span>
                <button
                  @click="selectedModalGameVersion = ''"
                  class="px-2 py-0.5 rounded text-[9px] font-mono border transition cursor-pointer"
                  :class="!selectedModalGameVersion ? 'bg-indigo-500/20 text-indigo-300 border-indigo-500/40' : 'bg-black/40 text-white/40 border-white/5 hover:border-white/20'"
                >
                  All
                </button>
                <button
                  v-for="gv in availableGameVersionsInModal"
                  :key="gv"
                  @click="selectedModalGameVersion = selectedModalGameVersion === gv ? '' : gv"
                  class="px-2 py-0.5 rounded text-[9px] font-mono border transition cursor-pointer"
                  :class="selectedModalGameVersion === gv ? 'bg-indigo-500/20 text-indigo-300 border-indigo-500/40' : 'bg-black/40 text-white/50 border-white/5 hover:border-white/20'"
                >
                  {{ gv }}
                </button>
              </div>
            </div>

            <div v-if="filteredProjectVersions.length === 0" class="py-16 text-center text-white/30 font-mono text-xs">
              No releases match criteria.
            </div>

            <div
              v-for="ver in filteredProjectVersions"
              :key="ver.id"
              class="p-5 bg-black/40 border border-white/5 rounded-2xl flex flex-col gap-3 hover:border-white/15 transition-colors"
            >
              <div class="flex justify-between items-start">
                <div class="min-w-0 pr-4">
                  <div class="flex items-center gap-3 mb-1.5 flex-wrap">
                    <h4 class="font-black text-white text-base leading-tight truncate">
                      {{ ver.name || ver.versionNumber }}
                    </h4>
                    <span class="px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-mono text-[9px] font-bold">
                      v{{ ver.versionNumber }}
                    </span>
                    <span v-if="ver.files[0]?.size" class="text-[9px] font-mono text-white/40">
                      {{ formatBytes(ver.files[0].size) }}
                    </span>
                  </div>

                  <span class="text-[10px] font-mono text-white/40 flex items-center gap-1 mb-2">
                    <Clock class="w-3 h-3" /> Published {{ new Date(ver.date).toLocaleDateString() }}
                  </span>

                  <div class="flex flex-wrap gap-1.5 items-center">
                    <span
                      v-for="gv in (ver.gameVersions || []).slice(0, 8)"
                      :key="gv"
                      class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-300 border border-emerald-500/20 text-[9px] font-mono font-bold"
                    >
                      MC {{ gv }}
                    </span>
                    <span
                      v-if="(ver.gameVersions || []).length > 8"
                      class="px-1.5 py-0.5 rounded bg-white/5 text-white/40 text-[8px] font-mono"
                    >
                      +{{ ver.gameVersions.length - 8 }} more
                    </span>

                    <span
                      v-for="ldr in (ver.loaders || [])"
                      :key="ldr"
                      class="px-2 py-0.5 rounded bg-purple-500/10 text-purple-300 border border-purple-500/20 text-[9px] font-mono uppercase font-black"
                    >
                      {{ ldr }}
                    </span>
                  </div>
                </div>

                <button
                  @click="emit('install', item, ver)"
                  class="kip-btn-primary px-5 py-2.5 text-xs font-black uppercase tracking-wider shrink-0 cursor-pointer"
                >
                  <DownloadCloud class="w-3.5 h-3.5" />
                  <span>Download</span>
                </button>
              </div>

              <div
                v-if="ver.dependencies && ver.dependencies.length > 0"
                class="pt-3 border-t border-white/5 flex flex-wrap gap-2 items-center"
              >
                <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Required Dependencies:</span>
                <span
                  v-for="dep in ver.dependencies"
                  :key="dep.projectId"
                  class="px-2 py-0.5 rounded bg-amber-500/10 text-amber-300 border border-amber-500/20 text-[9px] font-mono font-bold"
                >
                  {{ dep.projectId }}
                </span>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>