<template>
  <div class="h-full flex flex-col custom-scroll overflow-y-auto pr-2 pb-10 select-none" :class="{'justify-center items-center text-center overflow-hidden pr-0 pb-0': state.isMiniMode}">
    <template v-if="!state.isMiniMode">
      <div class="kip-card p-10 mb-8 relative overflow-hidden group shrink-0 stagger-1">
        <div class="absolute top-0 right-0 w-[550px] h-[550px] bg-gradient-to-br from-indigo-500/15 via-purple-500/10 to-transparent blur-[120px] rounded-full group-hover:from-indigo-500/25 group-hover:via-purple-500/20 transition-all duration-700 pointer-events-none"></div>
        <div class="absolute -bottom-24 -left-24 w-72 h-72 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none transition-all duration-700"></div>

        <div class="relative z-10 flex justify-between items-end">
          <div class="flex flex-col gap-2 max-w-xl">
            <div class="flex items-center gap-3">
              <span class="text-indigo-400 font-mono font-bold tracking-[0.25em] text-xs uppercase">{{ currentDate }}</span>
              <span class="px-2.5 py-0.5 rounded-full text-[9px] font-black uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 flex items-center gap-1.5">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span> K.I.P. Engine v{{ state.version || '1.6.1' }}
              </span>
            </div>
            <h1 class="text-5xl font-black bg-clip-text text-transparent bg-gradient-to-r from-white via-white/90 to-white/60 tracking-tight flex items-center gap-3 mt-1">
              {{ t(state.greeting) }} <span class="animate-[pulse_3s_ease-in-out_infinite] inline-block">👋</span>
            </h1>
            <p class="text-white/50 text-base font-medium tracking-wide leading-relaxed mt-1">
              {{ t('Welcome to your ultimate manager.') }} Engine primed on <span class="text-white/80 font-mono">{{ formatCurrentDirectory(state.settings.mc_dir) }}</span>
            </p>
          </div>

          <div class="flex gap-3">
            <button @click="quickCleanLogs" :disabled="isCleaningLogs" class="kip-btn-ghost px-5 py-3 text-xs tracking-wider uppercase border-white/10 hover:border-white/25">
              <Loader v-if="isCleaningLogs" class="w-4 h-4 animate-spin text-indigo-400" />
              <Trash2 v-else class="w-4 h-4 text-indigo-400" />
              <span>Clean Caches</span>
            </button>
            <button @click="state.currentView = 'launcher'" class="kip-btn-primary px-7 py-3 text-xs tracking-widest uppercase font-black shadow-[0_0_25px_rgba(99,102,241,0.4)]">
              <Zap class="w-4 h-4 fill-current" />
              <span>Ignition Pad</span>
            </button>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-4 gap-5 mb-8 shrink-0 stagger-2">
        <div @click="state.currentView = 'mods'" class="kip-card kip-card-hover p-6 flex flex-col justify-between h-36 relative overflow-hidden group cursor-pointer border-white/5 hover:border-indigo-500/30">
          <div class="absolute -right-6 -top-6 w-20 h-20 bg-indigo-500/20 blur-2xl rounded-full group-hover:bg-indigo-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <div class="flex flex-col">
              <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('MC Size') }}</span>
              <span class="text-[9px] font-mono text-indigo-400 mt-0.5">{{ dynamicStats.mods_count || 0 }} mods installed</span>
            </div>
            <div class="p-2.5 bg-indigo-500/10 rounded-xl border border-indigo-500/20"><HardDrive class="text-indigo-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-white relative z-10 tracking-tight flex items-baseline justify-between">
            <span>{{ dynamicStats.size || state.stats.size }}</span>
            <ArrowRight class="w-4 h-4 text-indigo-400 opacity-0 -translate-x-3 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
          </div>
        </div>

        <div @click="state.currentView = 'worlds'" class="kip-card kip-card-hover p-6 flex flex-col justify-between h-36 relative overflow-hidden group cursor-pointer border-white/5 hover:border-emerald-500/30">
          <div class="absolute -right-6 -top-6 w-20 h-20 bg-emerald-500/20 blur-2xl rounded-full group-hover:bg-emerald-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <div class="flex flex-col">
              <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('Worlds') }}</span>
              <span class="text-[9px] font-mono text-emerald-400 mt-0.5 truncate max-w-[120px]">{{ dynamicStats.last_world_name || 'VCS Ready' }}</span>
            </div>
            <div class="p-2.5 bg-emerald-500/10 rounded-xl border border-emerald-500/20"><Globe class="text-emerald-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-white relative z-10 flex items-baseline justify-between tracking-tight">
            <span>{{ dynamicStats.saves || state.stats.saves }}</span>
            <ArrowRight class="w-4 h-4 text-emerald-400 opacity-0 -translate-x-3 group-hover:opacity-100 group-hover:translate-x-0 transition-all duration-300" />
          </div>
        </div>

        <div class="kip-card kip-card-hover p-6 flex flex-col justify-between h-36 relative overflow-hidden group border-white/5 hover:border-purple-500/30">
          <div class="absolute -right-6 -top-6 w-20 h-20 bg-purple-500/20 blur-2xl rounded-full group-hover:bg-purple-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <div class="flex flex-col">
              <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">{{ t('Playtime') }}</span>
              <span class="text-[9px] font-mono text-purple-400 mt-0.5">Global Metric</span>
            </div>
            <div class="p-2.5 bg-purple-500/10 rounded-xl border border-purple-500/20"><Clock class="text-purple-400 w-4 h-4" /></div>
          </div>
          <div class="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-indigo-400 relative z-10 truncate tracking-tight">
            {{ dynamicStats.playtime || state.stats.playtime }}
          </div>
        </div>

        <div class="kip-card kip-card-hover p-6 flex flex-col justify-between h-36 relative overflow-hidden group border-white/5 hover:border-amber-500/30">
          <div class="absolute -right-6 -top-6 w-20 h-20 bg-amber-500/20 blur-2xl rounded-full group-hover:bg-amber-500/40 transition-all duration-500 pointer-events-none"></div>
          <div class="flex items-start justify-between relative z-10">
            <div class="flex flex-col">
              <span class="text-white/50 font-bold text-[10px] uppercase tracking-widest">System Load</span>
              <span class="text-[9px] font-mono text-amber-400 mt-0.5">RAM: {{ dynamicStats.ram_usage_percent || 0 }}% in use</span>
            </div>
            <div class="p-2.5 bg-amber-500/10 rounded-xl border border-amber-500/20"><Cpu class="text-amber-400 w-4 h-4" /></div>
          </div>
          <div class="text-2xl font-black text-white relative z-10 truncate tracking-tight flex items-baseline justify-between">
            <span class="truncate">{{ dynamicStats.java || state.stats.java }}</span>
            <span class="text-xs font-mono font-bold text-white/40">Java</span>
          </div>
        </div>
      </div>

      <div class="grid grid-cols-3 gap-5 shrink-0 stagger-3">
        <div class="col-span-2 kip-card p-8 border border-blue-500/20 relative overflow-hidden group transition-all duration-500 flex flex-col justify-between">
          <div class="absolute top-0 left-0 w-full h-1 bg-gradient-to-r from-blue-500 to-cyan-500 opacity-50 group-hover:opacity-100 transition-opacity duration-500"></div>
          <div class="absolute -bottom-24 -right-24 w-72 h-72 bg-blue-500/10 blur-[90px] rounded-full pointer-events-none group-hover:bg-blue-500/20 transition-all duration-500"></div>

          <div>
            <div class="flex items-center justify-between mb-6 relative z-10">
              <div class="flex items-center gap-3">
                <div class="p-2.5 bg-blue-500/10 rounded-xl border border-blue-500/20"><Newspaper class="w-5 h-5 text-blue-400" /></div>
                <h3 class="text-xl font-bold text-white">{{ t('Project News') }}</h3>
                <span class="px-2.5 py-1 bg-blue-500/20 text-blue-400 text-[10px] font-bold uppercase tracking-widest rounded-lg border border-blue-500/30">Release v{{ state.version || '1.6.1' }}</span>
              </div>
              <button @click="refreshDashboard" class="text-white/40 hover:text-white transition p-2" :title="t('Refresh')">
                <RefreshCw class="w-4 h-4" :class="isRefreshing ? 'animate-spin text-blue-400' : ''" />
              </button>
            </div>

            <div v-if="state.newsText" class="text-sm text-white/70 leading-relaxed relative z-10 markdown-body max-w-none" v-html="sanitizeHTML(state.newsText)"></div>
            <div v-else class="text-sm text-white/30 relative z-10 flex items-center gap-3 font-medium py-6">
              <Loader class="w-4 h-4 animate-spin text-blue-500" /> Fetching engine broadcasts...
            </div>
          </div>

          <div class="pt-6 mt-6 border-t border-white/5 flex items-center justify-between relative z-10">
            <div class="flex items-center gap-2 text-xs font-mono text-white/40">
              <ShieldCheck class="w-4 h-4 text-emerald-400" />
              <span>GPL-3.0 Pure Open-Source Suite</span>
            </div>
            <div class="flex gap-3">
              <button @click="state.currentView = 'media'" class="text-xs font-bold text-white/60 hover:text-white transition">Gallery</button>
              <span class="text-white/20">•</span>
              <button @click="state.currentView = 'tools'" class="text-xs font-bold text-white/60 hover:text-white transition">Diagnostics</button>
              <span class="text-white/20">•</span>
              <button @click="state.currentView = 'support'" class="text-xs font-bold text-white/60 hover:text-white transition">AI Analysis</button>
            </div>
          </div>
        </div>

        <div class="col-span-1 kip-card p-8 flex flex-col justify-between items-center text-center relative overflow-hidden transition-all duration-500 border" :class="state.isMcRunning ? 'border-emerald-500/40 bg-emerald-900/10 shadow-[0_0_35px_rgba(16,185,129,0.15)]' : 'border-white/10'">
          <div class="absolute inset-0 flex items-center justify-center pointer-events-none z-0">
            <div v-if="state.isMcRunning" class="w-48 h-48 border border-emerald-500/20 rounded-full animate-[ping_3s_cubic-bezier(0,0,0.2,1)_infinite]"></div>
            <div v-if="state.isMcRunning" class="w-32 h-32 border border-emerald-500/30 rounded-full animate-[ping_2s_cubic-bezier(0,0,0.2,1)_infinite]"></div>
          </div>

          <div class="relative z-10 flex flex-col items-center w-full">
            <div class="w-20 h-20 rounded-3xl flex items-center justify-center mb-5 transition-all duration-500 shadow-2xl relative" :class="state.isMcRunning ? 'bg-emerald-500/20 border border-emerald-500/50 shadow-[0_0_25px_rgba(16,185,129,0.5)]' : 'bg-black/50 border border-white/10'">
              <div v-if="state.isMcRunning" class="absolute inset-0 bg-emerald-500/20 blur-xl rounded-3xl animate-pulse"></div>
              <Rocket class="w-10 h-10 relative z-10" :class="state.isMcRunning ? 'text-emerald-400 animate-bounce' : 'text-white/30'" />
            </div>

            <h3 class="text-xl font-extrabold mb-1.5">{{ t('System Status') }}</h3>
            <p class="text-[10px] font-bold uppercase tracking-widest px-4 py-1.5 rounded-full border transition-all duration-500" :class="state.isMcRunning ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30 shadow-[0_0_12px_rgba(52,211,153,0.3)]' : 'bg-white/5 text-white/50 border-white/10'">
              {{ t(state.mcStatusText) }}
            </p>
          </div>

          <div class="w-full relative z-10 mt-6 flex flex-col gap-2.5">
            <button v-if="!state.isMcRunning" @click="quickStartVanilla" class="w-full py-3.5 bg-emerald-500 hover:bg-emerald-400 text-black font-black text-xs uppercase tracking-widest rounded-xl transition shadow-[0_0_20px_rgba(16,185,129,0.3)] flex items-center justify-center gap-2 cursor-pointer">
              <Play class="w-4 h-4 fill-current" /> Instant Ignite
            </button>
            <button v-else @click="quickKillGame" class="w-full py-3.5 bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white border border-red-500/30 font-black text-xs uppercase tracking-widest rounded-xl transition flex items-center justify-center gap-2 cursor-pointer">
              <Skull class="w-4 h-4" /> Terminate Instance
            </button>
            <button @click="state.currentView = 'launcher'" class="w-full kip-btn-ghost py-3 text-xs tracking-wider uppercase font-bold border-white/10">
              Configure Profiles
            </button>
          </div>
        </div>
      </div>
    </template>

    <template v-else>
      <div class="kip-card p-8 w-full max-w-sm flex flex-col justify-center items-center text-center relative overflow-hidden transition-all duration-500" :class="state.isMcRunning ? 'border-emerald-500/40 bg-emerald-900/10 shadow-[0_0_40px_rgba(16,185,129,0.15)]' : 'border-indigo-500/30 shadow-[0_0_40px_rgba(99,102,241,0.1)]'">
        <div class="absolute -top-20 -left-20 w-48 h-48 blur-[80px] rounded-full pointer-events-none transition-colors duration-500" :class="state.isMcRunning ? 'bg-emerald-500/20' : 'bg-indigo-500/20'"></div>

        <div class="relative w-24 h-24 mb-6 flex items-center justify-center">
          <div class="absolute inset-0 border-2 rounded-full border-dashed animate-[spin_10s_linear_infinite]" :class="state.isMcRunning ? 'border-emerald-500/30' : 'border-indigo-500/30'"></div>
          <div class="absolute inset-2 border-2 rounded-full animate-[spin_5s_linear_infinite_reverse]" :class="state.isMcRunning ? 'border-emerald-500/20' : 'border-indigo-500/20'"></div>
          <Hexagon class="w-10 h-10 transition-colors duration-500" :class="state.isMcRunning ? 'text-emerald-400 animate-pulse' : 'text-indigo-400'" />
        </div>

        <h3 class="font-extrabold text-2xl tracking-widest mb-3 uppercase">K.I.P. Engine</h3>

        <div class="flex items-center gap-2 px-4 py-2 rounded-full border bg-black/40 mb-8 transition-colors duration-500" :class="state.isMcRunning ? 'border-emerald-500/30' : 'border-white/10'">
          <span class="w-2 h-2 rounded-full" :class="state.isMcRunning ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-white/30'"></span>
          <span class="text-[10px] font-bold uppercase tracking-widest" :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/50'">{{ t(state.mcStatusText) }}</span>
        </div>

        <button @click="toggleMiniMode" class="kip-btn-ghost w-full py-3.5 text-xs tracking-wider">
          <Maximize2 class="w-4 h-4" /> {{ t('Expand Dashboard') }}
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  HardDrive,
  Globe,
  Clock,
  Hexagon,
  ArrowRight,
  Cpu,
  Newspaper,
  Rocket,
  Play,
  Maximize2,
  Loader,
  RefreshCw,
  Zap,
  Trash2,
  ShieldCheck,
  Skull,
} from 'lucide-vue-next'
import { state, t, sanitizeHTML, toggleMiniMode, showToast, loadDashboardStats } from '@/store'
import { bridge, invokeSafe, type DashboardStats, type ToolExecutionResult } from '@/bridge'

const isRefreshing = ref<boolean>(false)
const isCleaningLogs = ref<boolean>(false)

const dynamicStats = ref<DashboardStats & { mods_count?: number; ram_usage_percent?: number; last_world_name?: string | null }>({
  size: '...',
  saves: '...',
  playtime: '...',
  java: '...',
  mods_count: 0,
  ram_usage_percent: 0,
  last_world_name: null,
})

const currentDate = computed<string>(() => {
  try {
    const lang = state.settings.lang || 'en'
    const localeMap: Record<string, string> = {
      zh: 'zh-CN',
      pt: 'pt-BR',
      ja: 'ja-JP',
      ko: 'ko-KR',
      ru: 'ru-RU',
      es: 'es-ES',
      de: 'de-DE',
      fr: 'fr-FR',
      it: 'it-IT',
      pl: 'pl-PL',
      tr: 'tr-TR',
    }
    const resolvedLocale = localeMap[lang] || lang
    return new Date().toLocaleDateString(resolvedLocale, {
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    })
  } catch {
    return new Date().toLocaleDateString('en-US', {
      month: 'long',
      day: 'numeric',
      year: 'numeric',
    })
  }
})

const formatCurrentDirectory = (path?: string): string => {
  if (!path) return 'Default (.minecraft)'
  const normalized = path.replace(/\\/g, '/')
  const segments = normalized.split('/').filter(Boolean)
  return segments.pop() || path
}

const refreshDashboard = async (): Promise<void> => {
  if (isRefreshing.value) return
  isRefreshing.value = true
  try {
    const res = await invokeSafe<DashboardStats & { mods_count?: number; ram_usage_percent?: number; last_world_name?: string | null }>('get_dashboard_stats')
    if (res) {
      dynamicStats.value = res
      state.stats.size = res.size
      state.stats.saves = res.saves
      state.stats.playtime = res.playtime
      state.stats.java = res.java
    }
    await loadDashboardStats()
    showToast(t('Telemetry Synchronized'), 'Dashboard metrics updated.', 'success')
  } catch {
    showToast(t('Error'), 'Failed to refresh telemetry.', 'danger')
  } finally {
    isRefreshing.value = false
  }
}

const quickCleanLogs = async (): Promise<void> => {
  if (isCleaningLogs.value) return
  isCleaningLogs.value = true
  try {
    const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: 'clean_logs' })
    if (res && res.success) {
      showToast(t('Cleaned'), res.msg, 'success')
      await refreshDashboard()
    } else {
      showToast(t('Error'), res?.msg || 'Failed to clean logs.', 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || 'Cleanup error.', 'danger')
  } finally {
    isCleaningLogs.value = false
  }
}

const quickStartVanilla = async (): Promise<void> => {
  state.currentView = 'launcher'
}

const quickKillGame = async (): Promise<void> => {
  try {
    const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: 'kill_java' })
    if (res && res.success) {
      showToast(t('Terminated'), res.msg, 'success')
      state.isMcRunning = false
      state.mcStatusText = 'Minecraft stopped'
    }
  } catch {
    showToast(t('Error'), 'Could not terminate processes.', 'danger')
  }
}

onMounted(() => {
  refreshDashboard()
})
</script>

<style scoped>
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  color: #fff;
  font-weight: 800;
  margin-top: 0.5rem;
  margin-bottom: 0.25rem;
}
.markdown-body :deep(ul) {
  list-style-type: disc;
  padding-left: 1.25rem;
}
.markdown-body :deep(p) {
  margin-bottom: 0.5rem;
}
</style>