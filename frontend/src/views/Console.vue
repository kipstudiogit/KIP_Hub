<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="flex justify-between items-end mb-6 shrink-0 stagger-1">
      <div>
        <h2 class="text-3xl font-extrabold mb-1">{{ t('Live Terminal') }}</h2>
        <p class="text-white/50 text-sm">{{ t('Real-time engine and game event logs.') }}</p>
      </div>
      <div class="flex items-center gap-3">
        <div class="relative w-64">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-white/40 w-4 h-4" />
          <input v-model="searchQuery" type="text" :placeholder="t('Filter logs...')" class="kip-input py-2 pl-9 text-xs">
        </div>

        <div class="w-px h-6 bg-white/10 mx-1"></div>

        <button @click="isAutoScroll = !isAutoScroll" class="kip-btn-ghost px-4 py-2 text-xs transition-colors" :class="isAutoScroll ? 'text-emerald-400 border-emerald-500/30 bg-emerald-500/10' : 'text-white/50 hover:text-white'">
          <ArrowDownToLine class="w-4 h-4" /> Auto-Scroll
        </button>
        <button @click="copyLogs" class="kip-btn-ghost px-4 py-2 text-xs text-white/50 hover:text-white">
          <Copy class="w-4 h-4" /> Copy
        </button>
        <button @click="clearLogs" class="kip-btn-danger px-4 py-2 text-xs bg-red-500/10 hover:bg-red-500 hover:text-white text-red-400 border border-red-500/20 shadow-none">
          <Trash2 class="w-4 h-4" /> Clear
        </button>
      </div>
    </div>

    <div class="kip-card flex-1 overflow-hidden flex flex-col bg-black/60 relative stagger-2 border-white/5 shadow-2xl">
      <div class="bg-black/40 px-5 py-3 flex justify-between items-center border-b border-white/5 shrink-0">
        <div class="flex gap-2">
          <div class="w-3 h-3 rounded-full bg-red-500/80 border border-red-500 shadow-[0_0_10px_rgba(239,68,68,0.5)]"></div>
          <div class="w-3 h-3 rounded-full bg-amber-500/80 border border-amber-500 shadow-[0_0_10px_rgba(245,158,11,0.5)]"></div>
          <div class="w-3 h-3 rounded-full bg-emerald-500/80 border border-emerald-500 shadow-[0_0_10px_rgba(16,185,129,0.5)]"></div>
        </div>
        <div class="flex items-center gap-2">
          <span class="relative flex h-2 w-2">
            <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
            <span class="relative inline-flex rounded-full h-2 w-2 bg-emerald-500"></span>
          </span>
          <span class="text-[10px] font-bold text-white/50 uppercase tracking-widest">latest.log</span>
        </div>
      </div>

      <div ref="consoleContainer" class="flex-1 p-6 font-mono text-[13px] overflow-y-auto custom-scroll text-white/70 whitespace-pre-wrap leading-relaxed shadow-inner" v-html="filteredConsoleHtml"></div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import { Trash2, Copy, ArrowDownToLine, Search } from 'lucide-vue-next'
import { state, api, t, showToast } from '@/store.js'

const consoleContainer = ref(null)
const isAutoScroll = ref(true)
const searchQuery = ref('')
const debouncedQuery = ref('')
let searchTimeout = null

watch(searchQuery, (newVal) => {
  clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    debouncedQuery.value = newVal.toLowerCase()
  }, 300)
})

const filteredConsoleHtml = computed(() => {
  if (!debouncedQuery.value) return state.consoleHtml

  const query = debouncedQuery.value
  const lines = state.consoleHtml.split('<br>')

  return lines.filter(line => {
    const textContent = line.replace(/<[^>]*>?/gm, '').toLowerCase()
    return textContent.includes(query)
  }).join('<br>')
})

const scrollToBottom = () => {
  if (isAutoScroll.value && consoleContainer.value) {
    nextTick(() => {
      consoleContainer.value.scrollTop = consoleContainer.value.scrollHeight
    })
  }
}

watch(() => state.consoleHtml, () => {
  scrollToBottom()
})

const clearLogs = () => {
  state._consoleBuffer = []
  state.consoleHtml = ''
  showToast(t("Cleared"), t("Console output cleared."), "success")
}

const copyLogs = () => {
  const plainText = state.consoleHtml.replace(/<br>/g, '\n').replace(/<[^>]*>?/gm, '')
  navigator.clipboard.writeText(plainText).then(() => {
    showToast(t("Copied"), t("Logs copied to clipboard."), "success")
  }).catch(() => {
    showToast(t("Error"), t("Failed to copy logs."), "danger")
  })
}

onMounted(() => {
  if (api.value) {
    try {
      api.value.toggle_console_stream(true)
    } catch (e) {}
  }
  scrollToBottom()
})

onBeforeUnmount(() => {
  clearTimeout(searchTimeout)
  if (api.value) {
    try {
      api.value.toggle_console_stream(false)
    } catch (e) {}
  }
})
</script>