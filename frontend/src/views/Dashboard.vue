<script setup lang="ts">
import { onMounted, onBeforeUnmount, computed } from 'vue'
import {
  Rocket,
  Zap,
  Cpu,
  HardDrive,
  Globe,
  Clock,
  Trash2,
  Play,
  Skull,
  RefreshCw,
  Sliders,
  ExternalLink,
  ShieldCheck,
  Maximize2,
  Terminal,
  Activity,
  Layers,
  Heart,
  HelpCircle,
} from 'lucide-vue-next'
import { state, t, sanitizeHTML, toggleMiniMode, showToast } from '@/store'
import { useDashboard } from '../composables/useDashboard'

const {
  overview,
  isRefreshing,
  isActionExecuting,
  cpuStatusColor,
  ramStatusColor,
  fetchOverview,
  executeQuickTool,
  instantIgnite,
  terminateProcess,
} = useDashboard()

let telemetryInterval: ReturnType<typeof setInterval> | null = null

const currentDate = computed<string>(() => {
  return new Date().toLocaleDateString(state.settings.lang || 'en', {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  })
})

function navigateTo(viewId: string): void {
  state.currentView = viewId
}

onMounted(async () => {
  await fetchOverview()
  telemetryInterval = setInterval(() => {
    fetchOverview()
  }, 4000)
})

onBeforeUnmount(() => {
  if (telemetryInterval) {
    clearInterval(telemetryInterval)
    telemetryInterval = null
  }
})
</script>

<template>
  <div class="h-full flex flex-col custom-scroll overflow-y-auto pr-2 pb-12 select-none relative">
    <!-- Mission Control Master Banner -->
    <section class="kip-card p-8 mb-8 relative overflow-hidden shrink-0 border border-white/10 group bg-gradient-to-r from-black/80 via-black/50 to-indigo-950/20 shadow-2xl">
      <div class="absolute -top-32 -right-32 w-[500px] h-[500px] bg-indigo-600/15 blur-[120px] rounded-full pointer-events-none transition-all duration-700 group-hover:bg-indigo-600/25"></div>
      <div class="absolute -bottom-24 -left-24 w-72 h-72 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none"></div>

      <div class="relative z-10 flex justify-between items-center">
        <div class="flex flex-col gap-2 max-w-2xl">
          <div class="flex items-center gap-3">
            <span class="text-indigo-400 font-mono font-bold tracking-[0.25em] text-xs uppercase">{{ currentDate }}</span>
            <span class="px-2.5 py-0.5 rounded-full text-[9px] font-black uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex items-center gap-1.5">
              <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-ping"></span>
              <span>Active Profile: {{ overview.currentInstanceName }}</span>
            </span>
          </div>

          <h1 class="text-4xl font-black text-white tracking-tight flex items-center gap-3">
            {{ t(state.greeting) }} <span class="text-indigo-400">• Command Deck</span>
          </h1>

          <p class="text-white/50 text-xs font-medium tracking-wide leading-relaxed font-mono">
            HotSpot {{ overview.java }} • Memory Bandwidth: {{ overview.usedRamGb }} GB / {{ overview.totalRamGb }} GB Allocatable
          </p>
        </div>

        <!-- Quick Command Triggers -->
        <div class="flex items-center gap-3">
          <button
            @click="fetchOverview"
            :disabled="isRefreshing"
            class="kip-btn-ghost p-3 rounded-2xl border-white/10 hover:border-white/20 text-white/60 hover:text-white"
            title="Refresh Host Telemetry"
          >
            <RefreshCw class="w-4 h-4" :class="isRefreshing ? 'animate-spin text-indigo-400' : ''" />
          </button>

          <button
            @click="executeQuickTool('clean_logs')"
            :disabled="isActionExecuting"
            class="kip-btn-ghost px-5 py-3 rounded-2xl text-xs font-mono font-bold uppercase tracking-wider border-white/10 hover:border-white/25 flex items-center gap-2"
          >
            <Trash2 class="w-4 h-4 text-amber-400" />
            <span>Clean Caches</span>
          </button>

          <button
            v-if="!overview.isGameRunning"
            @click="instantIgnite"
            class="kip-btn-primary px-8 py-3.5 rounded-2xl text-xs tracking-widest uppercase font-black shadow-[0_0_30px_rgba(99,102,241,0.5)] flex items-center gap-2"
          >
            <Zap class="w-4 h-4 fill-current text-white" />
            <span>Ignition Pad</span>
          </button>

          <button
            v-else
            @click="terminateProcess"
            class="kip-btn-danger px-8 py-3.5 rounded-2xl text-xs tracking-widest uppercase font-black shadow-[0_0_30px_rgba(239,68,68,0.5)] flex items-center gap-2"
          >
            <Skull class="w-4 h-4 fill-current text-white" />
            <span>Terminate (PID: {{ overview.runningGamePid }})</span>
          </button>
        </div>
      </div>
    </section>

    <!-- Dynamic Quad Telemetry Matrix -->
    <section class="grid grid-cols-4 gap-5 mb-8 shrink-0">
      <!-- Storage Matrix Card -->
      <article
        @click="navigateTo('mods')"
        class="kip-card p-6 flex flex-col justify-between h-40 cursor-pointer border border-white/5 hover:border-indigo-500/30 transition-all duration-300 group bg-black/40 hover:bg-black/60 relative overflow-hidden"
      >
        <div class="flex items-start justify-between relative z-10">
          <div>
            <span class="text-white/40 font-mono text-[9px] font-bold uppercase tracking-widest block">Instance Footprint</span>
            <span class="text-3xl font-black text-white mt-1 block tracking-tight">{{ overview.size }}</span>
          </div>
          <div class="p-3 bg-indigo-500/10 rounded-2xl border border-indigo-500/20 text-indigo-400 group-hover:scale-110 transition-transform">
            <HardDrive class="w-5 h-5" />
          </div>
        </div>

        <div class="space-y-1 relative z-10">
          <div class="flex justify-between text-[9px] font-mono text-white/50">
            <span>Mods: {{ overview.storage.modsSizeMb }}MB</span>
            <span>Saves: {{ overview.storage.savesSizeMb }}MB</span>
          </div>
          <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden flex border border-white/5">
            <div class="bg-indigo-500 h-full" :style="{ width: `${(overview.storage.modsSizeMb / (overview.storage.totalSizeMb || 1)) * 100}%` }"></div>
            <div class="bg-emerald-500 h-full" :style="{ width: `${(overview.storage.savesSizeMb / (overview.storage.totalSizeMb || 1)) * 100}%` }"></div>
            <div class="bg-purple-500 h-full" :style="{ width: `${(overview.storage.screenshotsSizeMb / (overview.storage.totalSizeMb || 1)) * 100}%` }"></div>
          </div>
        </div>
      </article>

      <!-- CPU Load Sensor -->
      <article class="kip-card p-6 flex flex-col justify-between h-40 border border-white/5 bg-black/40 relative overflow-hidden">
        <div class="flex items-start justify-between relative z-10">
          <div class="min-w-0 pr-2">
            <span class="text-white/40 font-mono text-[9px] font-bold uppercase tracking-widest block">Host Processor</span>
            <span class="text-3xl font-black text-white mt-1 block tracking-tight">{{ overview.cpuUsagePercent }}%</span>
            <span class="text-[9px] font-mono text-white/40 truncate block mt-0.5">{{ overview.cpuBrand }}</span>
          </div>
          <div class="p-3 rounded-2xl border transition-colors shrink-0" :class="cpuStatusColor">
            <Cpu class="w-5 h-5" />
          </div>
        </div>

        <div class="relative z-10">
          <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden border border-white/5">
            <div class="h-full bg-gradient-to-r from-emerald-500 via-amber-500 to-rose-500 transition-all duration-500" :style="{ width: `${overview.cpuUsagePercent}%` }"></div>
          </div>
        </div>
      </article>

      <!-- RAM Sensor -->
      <article class="kip-card p-6 flex flex-col justify-between h-40 border border-white/5 bg-black/40 relative overflow-hidden">
        <div class="flex items-start justify-between relative z-10">
          <div>
            <span class="text-white/40 font-mono text-[9px] font-bold uppercase tracking-widest block">System Memory</span>
            <span class="text-3xl font-black text-white mt-1 block tracking-tight">{{ overview.ramUsagePercent }}%</span>
            <span class="text-[9px] font-mono text-white/40 mt-0.5 block">{{ overview.usedRamGb }} GB of {{ overview.totalRamGb }} GB in use</span>
          </div>
          <div class="p-3 rounded-2xl border transition-colors shrink-0" :class="ramStatusColor">
            <Activity class="w-5 h-5" />
          </div>
        </div>

        <div class="relative z-10">
          <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden border border-white/5">
            <div class="h-full bg-cyan-400 shadow-[0_0_10px_rgba(6,182,212,0.6)] transition-all duration-500" :style="{ width: `${overview.ramUsagePercent}%` }"></div>
          </div>
        </div>
      </article>

      <!-- Cumulative Playtime -->
      <article
        @click="navigateTo('worlds')"
        class="kip-card p-6 flex flex-col justify-between h-40 cursor-pointer border border-white/5 hover:border-purple-500/30 transition-all duration-300 group bg-black/40 hover:bg-black/60 relative overflow-hidden"
      >
        <div class="flex items-start justify-between relative z-10">
          <div>
            <span class="text-white/40 font-mono text-[9px] font-bold uppercase tracking-widest block">Logged Immersion</span>
            <span class="text-3xl font-black text-transparent bg-clip-text bg-gradient-to-r from-purple-400 to-indigo-400 mt-1 block tracking-tight">
              {{ overview.playtime }}
            </span>
            <span class="text-[9px] font-mono text-purple-400 block mt-0.5">{{ overview.saves }} registered universes</span>
          </div>
          <div class="p-3 bg-purple-500/10 rounded-2xl border border-purple-500/20 text-purple-400 group-hover:scale-110 transition-transform">
            <Clock class="w-5 h-5" />
          </div>
        </div>

        <div class="flex items-center justify-between text-[10px] font-mono text-white/40 relative z-10 pt-2 border-t border-white/5">
          <span>VCS Snapshots Primed</span>
          <span class="text-purple-400 font-bold group-hover:translate-x-1 transition-transform">&rarr; Inspect</span>
        </div>
      </article>
    </section>

    <!-- Central Grid: Quick Resume Universe & Live Broadcaster -->
    <section class="grid grid-cols-3 gap-6 shrink-0 mb-8">
      <!-- Quick Resume Universe Card -->
      <article class="col-span-1 kip-card p-6 border border-white/5 flex flex-col justify-between bg-black/40 relative overflow-hidden">
        <div class="flex items-center justify-between mb-4">
          <span class="text-[10px] font-mono font-bold uppercase tracking-wider text-white/40 flex items-center gap-2">
            <Globe class="w-3.5 h-3.5 text-emerald-400" /> Recent Universe
          </span>
          <span v-if="overview.lastWorld?.hardcore" class="px-2 py-0.5 rounded bg-rose-500/20 text-rose-300 text-[8px] font-black uppercase tracking-wider border border-rose-500/30">
            Hardcore
          </span>
        </div>

        <div v-if="overview.lastWorld" class="flex items-center gap-4 my-2">
          <img
            :src="overview.lastWorld.icon || 'data:image/svg+xml;utf8,<svg xmlns=\'http://www.w3.org/2000/svg\' width=\'64\' height=\'64\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'%2310b981\' stroke-width=\'2\'><circle cx=\'12\' cy=\'12\' r=\'10\'/></svg>'"
            class="w-16 h-16 rounded-2xl object-cover border border-white/10 bg-black/60 p-1 shadow-lg shrink-0"
          >
          <div class="min-w-0 flex-1">
            <h4 class="font-black text-lg text-white truncate">{{ overview.lastWorld.name }}</h4>
            <p class="text-[10px] font-mono text-white/50 truncate">Last played: {{ overview.lastWorld.lastPlayed }}</p>
            <span class="text-[9px] font-mono text-emerald-400 font-bold uppercase">{{ overview.lastWorld.mode }} Mode</span>
          </div>
        </div>

        <div v-else class="py-8 text-center text-white/30 font-mono text-xs">
          No save games detected in active instance.
        </div>

        <div class="pt-4 border-t border-white/5 flex gap-2">
          <button
            @click="navigateTo('worlds')"
            class="kip-btn-ghost flex-1 py-2.5 text-xs font-mono font-bold uppercase tracking-wider text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10"
          >
            Open Universe Deck
          </button>
        </div>
      </article>

      <!-- Project News & Broadcaster Deck -->
      <article class="col-span-2 kip-card p-6 border border-white/5 flex flex-col justify-between bg-black/40 relative overflow-hidden">
        <div class="flex items-center justify-between mb-4">
          <div class="flex items-center gap-3">
            <div class="p-2.5 bg-blue-500/10 rounded-xl border border-blue-500/20 text-blue-400">
              <Terminal class="w-4 h-4" />
            </div>
            <div>
              <h3 class="font-black text-sm uppercase text-white tracking-wider">K.I.P. Dispatch & Telemetry</h3>
              <span class="text-[9px] font-mono text-white/40">Engine Release v{{ state.version }} • Changelog Feed</span>
            </div>
          </div>

          <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-bold uppercase tracking-widest bg-blue-500/10 text-blue-400 border border-blue-500/20">
            Live Feed
          </span>
        </div>

        <div
          v-if="state.newsText"
          class="p-4 bg-black/60 border border-white/5 rounded-2xl text-xs text-white/70 font-mono leading-relaxed max-h-36 overflow-y-auto custom-scroll markdown-body"
          v-html="sanitizeHTML(state.newsText)"
        ></div>

        <div v-else class="py-8 text-center text-white/30 font-mono text-xs">
          Awaiting cloud telemetry broadcast...
        </div>

        <div class="pt-4 border-t border-white/5 flex items-center justify-between text-xs font-mono text-white/40">
          <div class="flex items-center gap-2">
            <ShieldCheck class="w-4 h-4 text-emerald-400" />
            <span>GPL-3.0 Pure Open-Source Architecture</span>
          </div>
          <div class="flex gap-3">
            <button @click="navigateTo('tools')" class="hover:text-white transition">Doctor</button>
            <span>•</span>
            <button @click="navigateTo('settings')" class="hover:text-white transition">Config</button>
            <span>•</span>
            <button @click="navigateTo('support')" class="hover:text-white transition">Support</button>
          </div>
        </div>
      </article>
    </section>

    <!-- Fast Access Diagnostic Pad -->
    <section class="grid grid-cols-4 gap-4 shrink-0">
      <button
        @click="executeQuickTool('ai_fps')"
        :disabled="isActionExecuting"
        class="kip-card p-4 flex items-center gap-3 border border-white/5 hover:border-emerald-500/30 transition-all text-left bg-black/40 hover:bg-black/60 group"
      >
        <div class="p-3 bg-emerald-500/10 rounded-xl text-emerald-400 group-hover:scale-110 transition-transform">
          <Zap class="w-4 h-4" />
        </div>
        <div>
          <span class="font-black text-xs text-white block">AI FPS Optimizer</span>
          <span class="text-[9px] font-mono text-white/40 block">Auto-tune options.txt</span>
        </div>
      </button>

      <button
        @click="executeQuickTool('backup')"
        :disabled="isActionExecuting"
        class="kip-card p-4 flex items-center gap-3 border border-white/5 hover:border-blue-500/30 transition-all text-left bg-black/40 hover:bg-black/60 group"
      >
        <div class="p-3 bg-blue-500/10 rounded-xl text-blue-400 group-hover:scale-110 transition-transform">
          <Layers class="w-4 h-4" />
        </div>
        <div>
          <span class="font-black text-xs text-white block">Instant Snapshot</span>
          <span class="text-[9px] font-mono text-white/40 block">Backup all world saves</span>
        </div>
      </button>

      <button
        @click="executeQuickTool('flush_dns')"
        :disabled="isActionExecuting"
        class="kip-card p-4 flex items-center gap-3 border border-white/5 hover:border-cyan-500/30 transition-all text-left bg-black/40 hover:bg-black/60 group"
      >
        <div class="p-3 bg-cyan-500/10 rounded-xl text-cyan-400 group-hover:scale-110 transition-transform">
          <Activity class="w-4 h-4" />
        </div>
        <div>
          <span class="font-black text-xs text-white block">Flush DNS Cache</span>
          <span class="text-[9px] font-mono text-white/40 block">Purge network resolver</span>
        </div>
      </button>

      <button
        @click="executeQuickTool('kill_java')"
        :disabled="isActionExecuting"
        class="kip-card p-4 flex items-center gap-3 border border-white/5 hover:border-rose-500/30 transition-all text-left bg-black/40 hover:bg-black/60 group"
      >
        <div class="p-3 bg-rose-500/10 rounded-xl text-rose-400 group-hover:scale-110 transition-transform">
          <Skull class="w-4 h-4" />
        </div>
        <div>
          <span class="font-black text-xs text-white block">Zombie Killer</span>
          <span class="text-[9px] font-mono text-white/40 block">Halt stuck JVM threads</span>
        </div>
      </button>
    </section>
  </div>
</template>