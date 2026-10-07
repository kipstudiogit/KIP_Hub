<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import {
  Trash2,
  Copy,
  ArrowDownToLine,
  Search,
  Pause,
  Play,
  UploadCloud,
  FileDown,
  Terminal,
  Loader,
  AlertTriangle,
  AlertCircle,
  Info,
} from 'lucide-vue-next'
import { t } from '@/store'
import { useConsoleManager } from '../composables/useConsoleManager'
import type { LogLevelFilter } from '../types/console'

const consoleScrollRef = ref<HTMLElement | null>(null)

const {
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
} = useConsoleManager()

function scrollToBottom(): void {
  if (isAutoScroll.value && !isFrozen.value && consoleScrollRef.value) {
    nextTick(() => {
      if (consoleScrollRef.value) {
        consoleScrollRef.value.scrollTop = consoleScrollRef.value.scrollHeight
      }
    })
  }
}

watch(
  () => filteredEntries.value.length,
  () => {
    scrollToBottom()
  }
)

function getBadgeStyle(level: string): string {
  switch (level) {
    case 'ERROR':
      return 'bg-rose-500/20 text-rose-300 border-rose-500/40 shadow-[0_0_10px_rgba(244,63,94,0.3)]'
    case 'WARN':
      return 'bg-amber-500/20 text-amber-300 border-amber-500/40'
    case 'DEBUG':
      return 'bg-blue-500/20 text-blue-300 border-blue-500/40'
    default:
      return 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'
  }
}

function getLineTextStyle(level: string): string {
  switch (level) {
    case 'ERROR':
      return 'text-rose-400 font-bold'
    case 'WARN':
      return 'text-amber-300'
    case 'DEBUG':
      return 'text-blue-300'
    default:
      return 'text-white/80'
  }
}

onMounted(async () => {
  await initStreaming()
  scrollToBottom()
})

onBeforeUnmount(() => {
  cleanup()
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <!-- Top Terminal Control Deck -->
    <header class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          <Terminal class="w-8 h-8 text-indigo-400" />
          <span>{{ t('Live Terminal') }}</span>
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Real-time JVM Event Stream, Structured Log Filter & Forensic Cloud Dump</p>
      </div>

      <div class="flex items-center gap-2.5">
        <button
          @click="uploadToMclogs"
          :disabled="isUploading"
          class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase tracking-wider text-blue-400 border-blue-500/20 hover:bg-blue-500/10 flex items-center gap-2"
          title="Export sanitized log to mclo.gs"
        >
          <Loader v-if="isUploading" class="w-3.5 h-3.5 animate-spin" />
          <UploadCloud v-else class="w-3.5 h-3.5" />
          <span>Upload mclo.gs</span>
        </button>

        <button
          @click="exportToFile"
          :disabled="isExporting"
          class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase tracking-wider text-purple-400 border-purple-500/20 hover:bg-purple-500/10 flex items-center gap-2"
          title="Save dump to .log file"
        >
          <Loader v-if="isExporting" class="w-3.5 h-3.5 animate-spin" />
          <FileDown v-else class="w-3.5 h-3.5" />
          <span>Dump .log</span>
        </button>

        <button
          @click="copyAllLogs"
          class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase tracking-wider text-white/70 hover:text-white border-white/10 flex items-center gap-2"
        >
          <Copy class="w-3.5 h-3.5" />
          <span>Copy</span>
        </button>

        <button
          @click="clearLogs"
          class="kip-btn-danger px-4 py-2 text-xs font-mono font-bold uppercase tracking-wider flex items-center gap-2"
        >
          <Trash2 class="w-3.5 h-3.5" />
          <span>Clear</span>
        </button>
      </div>
    </header>

    <!-- Toolbar Strip (Filters, Search & Stream Toggles) -->
    <div class="flex justify-between items-center gap-4 mb-4 shrink-0 z-10">
      <!-- Filter Level Pills -->
      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          @click="activeLevelFilter = 'ALL'"
          class="px-3.5 py-1.5 rounded-xl text-xs font-mono font-bold uppercase tracking-wider transition"
          :class="activeLevelFilter === 'ALL' ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/30' : 'text-white/40 hover:text-white'"
        >
          All ({{ stats.total }})
        </button>

        <button
          @click="activeLevelFilter = 'ERROR'"
          class="px-3.5 py-1.5 rounded-xl text-xs font-mono font-bold uppercase tracking-wider transition flex items-center gap-1.5"
          :class="activeLevelFilter === 'ERROR' ? 'bg-rose-500/20 text-rose-300 border border-rose-500/40' : 'text-rose-400/60 hover:text-rose-400'"
        >
          <AlertCircle class="w-3 h-3" /> Errors ({{ stats.errors }})
        </button>

        <button
          @click="activeLevelFilter = 'WARN'"
          class="px-3.5 py-1.5 rounded-xl text-xs font-mono font-bold uppercase tracking-wider transition flex items-center gap-1.5"
          :class="activeLevelFilter === 'WARN' ? 'bg-amber-500/20 text-amber-300 border border-amber-500/40' : 'text-amber-400/60 hover:text-amber-400'"
        >
          <AlertTriangle class="w-3 h-3" /> Warn ({{ stats.warns }})
        </button>

        <button
          @click="activeLevelFilter = 'INFO'"
          class="px-3.5 py-1.5 rounded-xl text-xs font-mono font-bold uppercase tracking-wider transition flex items-center gap-1.5"
          :class="activeLevelFilter === 'INFO' ? 'bg-emerald-500/20 text-emerald-300 border border-emerald-500/40' : 'text-emerald-400/60 hover:text-emerald-400'"
        >
          <Info class="w-3 h-3" /> Info ({{ stats.infos }})
        </button>
      </div>

      <!-- Controls Right -->
      <div class="flex items-center gap-3">
        <div class="relative w-64">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30 w-3.5 h-3.5" />
          <input
            v-model="searchQuery"
            type="text"
            placeholder="Search terminal logs..."
            class="kip-input py-1.5 pl-8 text-xs font-mono"
          >
        </div>

        <button
          @click="isFrozen = !isFrozen"
          class="kip-btn-ghost px-3.5 py-1.5 text-xs font-mono font-bold uppercase tracking-wider border flex items-center gap-1.5"
          :class="isFrozen ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 shadow' : 'text-white/50 hover:text-white border-white/10'"
          title="Pause stream (Freeze screen without dropping incoming lines)"
        >
          <Play v-if="isFrozen" class="w-3.5 h-3.5 fill-current" />
          <Pause v-else class="w-3.5 h-3.5" />
          <span>{{ isFrozen ? 'Resume' : 'Freeze' }}</span>
        </button>

        <button
          @click="isAutoScroll = !isAutoScroll"
          class="kip-btn-ghost px-3.5 py-1.5 text-xs font-mono font-bold uppercase tracking-wider border flex items-center gap-1.5"
          :class="isAutoScroll ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' : 'text-white/40 hover:text-white border-white/10'"
          title="Lock view to latest incoming log line"
        >
          <ArrowDownToLine class="w-3.5 h-3.5" />
          <span>Auto-Scroll</span>
        </button>

        <select
          v-model.number="maxBufferSize"
          class="kip-input py-1 px-2 text-xs font-mono w-28 bg-black/50 border border-white/10"
        >
          <option :value="500">500 lines</option>
          <option :value="1000">1000 lines</option>
          <option :value="2500">2500 lines</option>
        </select>
      </div>
    </div>

    <!-- Main Terminal Console Display -->
    <main class="kip-card flex-1 overflow-hidden flex flex-col bg-black/80 border border-white/10 relative shadow-2xl min-h-0">
      <!-- Title bar header inside terminal -->
      <div class="bg-black/90 px-5 py-2.5 flex justify-between items-center border-b border-white/5 shrink-0">
        <div class="flex gap-2 items-center">
          <div class="w-3 h-3 rounded-full bg-red-500/80 border border-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]"></div>
          <div class="w-3 h-3 rounded-full bg-amber-500/80 border border-amber-500 shadow-[0_0_10px_rgba(245,158,11,0.5)]"></div>
          <div class="w-3 h-3 rounded-full bg-emerald-500/80 border border-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]"></div>
          <span class="text-[11px] font-mono text-white/40 ml-2">latest.log • live stream socket</span>
        </div>

        <div class="flex items-center gap-2">
          <span class="relative flex h-2 w-2">
            <span v-if="!isFrozen" class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2 w-2" :class="isFrozen ? 'bg-amber-400' : 'bg-emerald-500'"></span>
          </span>
          <span class="text-[10px] font-mono font-bold uppercase tracking-wider" :class="isFrozen ? 'text-amber-400' : 'text-emerald-400'">
            {{ isFrozen ? 'STREAM FROZEN' : 'LIVE STREAM' }}
          </span>
        </div>
      </div>

      <!-- Scrollable Log Lines Matrix -->
      <div
        ref="consoleScrollRef"
        class="flex-1 p-5 font-mono text-[12px] overflow-y-auto custom-scroll leading-relaxed select-text space-y-1.5 shadow-inner"
      >
        <div v-if="filteredEntries.length === 0" class="py-24 text-center text-white/30 font-mono text-xs">
          Awaiting log entries from Minecraft process...
        </div>

        <div
          v-for="entry in filteredEntries"
          :key="entry.id"
          class="flex items-start gap-3 hover:bg-white/[0.03] px-2 py-0.5 rounded transition-colors group"
        >
          <!-- Line Index -->
          <span class="text-white/20 select-none text-[10px] w-10 text-right shrink-0 pt-0.5 font-bold">
            {{ entry.id }}
          </span>

          <!-- Timestamp -->
          <span v-if="entry.timestamp" class="text-indigo-400/70 select-none shrink-0 text-[11px] pt-0.5">
            [{{ entry.timestamp }}]
          </span>

          <!-- Level Badge -->
          <span
            class="px-1.5 py-0.2 rounded text-[9px] font-black uppercase tracking-wider border select-none shrink-0 leading-none self-center"
            :class="getBadgeStyle(entry.level)"
          >
            {{ entry.level }}
          </span>

          <!-- Thread -->
          <span v-if="entry.thread" class="text-white/30 select-none text-[11px] shrink-0 pt-0.5">
            [{{ entry.thread }}]:
          </span>

          <!-- Message Body -->
          <span class="break-all flex-1" :class="getLineTextStyle(entry.level)">
            {{ entry.message }}
          </span>
        </div>
      </div>
    </main>
  </div>
</template>

<style scoped>
.kip-card {
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7);
}
</style>