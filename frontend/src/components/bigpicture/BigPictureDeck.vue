<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import {
  Gamepad2,
  X,
  Play,
  Skull,
  Wand2,
  Puzzle,
  ShoppingCart,
  Globe,
  Zap,
  Image as ImageIcon,
  LifeBuoy,
  Settings,
  BatteryCharging,
  Wifi,
  Clock,
  Sparkles,
  HardDrive,
  Cpu,
  Layers,
} from 'lucide-vue-next'
import { state, t } from '@/store'
import {
  activeGamepad,
  playNavSound,
  playConfirmSound,
  triggerHaptic,
  playLaunchSound,
} from '../../composables/useGamepad'
import { useLauncher } from '../../composables/useLauncher'
import { useDashboard } from '../../composables/useDashboard'

const emit = defineEmits<{
  (e: 'exit'): void
}>()

const currentTime = ref<string>('')
let timerInterval: ReturnType<typeof setInterval> | null = null

const { overview, fetchOverview, terminateProcess } = useDashboard()
const { ignite, isLaunching, currentProgress } = useLauncher()

const activeShelfIndex = ref<number>(0)

const consoleShelves = [
  { id: 'launcher', name: 'Ignition Pad', desc: 'Bootstrap instance & Java matrix', icon: Play, color: 'text-amber-400', border: 'hover:border-amber-500/50' },
  { id: 'builder', name: 'Auto-Builder', desc: 'AI modpack synthesizer', icon: Wand2, color: 'text-indigo-400', border: 'hover:border-indigo-500/50' },
  { id: 'mods', name: 'Content Suite', desc: 'Mods, shaders and packages', icon: Puzzle, color: 'text-cyan-400', border: 'hover:border-cyan-500/50' },
  { id: 'store', name: 'Mod Store', desc: 'Modrinth & CurseForge search', icon: ShoppingCart, color: 'text-emerald-400', border: 'hover:border-emerald-500/50' },
  { id: 'worlds', name: 'World Saves', desc: 'VCS snapshots & NBT restore', icon: Globe, color: 'text-purple-400', border: 'hover:border-purple-500/50' },
  { id: 'tools', name: 'Mod Doctor', desc: 'Bytecode & security audit', icon: Zap, color: 'text-rose-400', border: 'hover:border-rose-500/50' },
  { id: 'media', name: 'Captures', desc: '4K Screenshot gallery', icon: ImageIcon, color: 'text-blue-400', border: 'hover:border-blue-500/50' },
  { id: 'settings', name: 'System Deck', desc: 'HotSpot JVM & settings', icon: Settings, color: 'text-slate-400', border: 'hover:border-white/30' },
]

function updateClock(): void {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

function handleShelfSelect(viewId: string): void {
  playConfirmSound()
  triggerHaptic(70, 0.4, 0.2)
  state.currentView = viewId
  emit('exit')
}

async function handleInstantIgnite(): Promise<void> {
  if (overview.value.isGameRunning) {
    playConfirmSound()
    triggerHaptic(100, 0.6, 0.4)
    await terminateProcess()
  } else {
    playLaunchSound()
    triggerHaptic(200, 0.8, 1.0)
    await ignite('26.3', 'fabric', null, 0, '', '1920x1080', true, false)
  }
}

onMounted(() => {
  updateClock()
  timerInterval = setInterval(updateClock, 1000)
  fetchOverview()
})

onBeforeUnmount(() => {
  if (timerInterval) clearInterval(timerInterval)
})
</script>

<template>
  <div class="fixed inset-0 z-[7500] bg-[#020204]/95 backdrop-blur-3xl flex flex-col justify-between p-12 text-white select-none overflow-hidden font-sans">
    <!-- Top HUD Console Bar -->
    <header class="flex justify-between items-center shrink-0 z-20 pb-6 border-b border-white/10">
      <!-- Player & Matrix Identity -->
      <div class="flex items-center gap-4">
        <div class="p-3.5 rounded-2xl bg-indigo-500/20 border border-indigo-500/40 text-indigo-400 shadow-[0_0_25px_rgba(99,102,241,0.5)]">
          <Gamepad2 class="w-8 h-8 stroke-[2.2] animate-pulse" />
        </div>
        <div>
          <h2 class="text-2xl font-black uppercase tracking-wider text-white flex items-center gap-2">
            <span>K.I.P. Hub</span>
            <span class="text-xs px-2.5 py-0.5 rounded-lg bg-indigo-500/20 text-indigo-300 font-mono font-bold border border-indigo-500/40">10-Foot Deck</span>
          </h2>
          <p class="text-xs font-mono text-white/40 tracking-widest uppercase mt-0.5">
            Operator: <strong class="text-emerald-400">{{ state.settings.ms_name || state.settings.offline_username || 'Guest' }}</strong>
          </p>
        </div>
      </div>

      <!-- Telemetry Center Pill -->
      <div class="flex items-center gap-6 px-6 py-2.5 rounded-2xl bg-black/60 border border-white/10 font-mono text-xs shadow-inner">
        <div class="flex items-center gap-2 text-indigo-300">
          <Gamepad2 class="w-4 h-4 text-emerald-400" />
          <span class="font-bold">{{ activeGamepad?.modelName || 'Keyboard Emulation Matrix' }}</span>
        </div>

        <div class="w-px h-4 bg-white/10"></div>

        <div class="flex items-center gap-2 text-emerald-400 font-bold">
          <Wifi class="w-4 h-4" />
          <span>LAN Relay: 0 ms</span>
        </div>

        <div class="w-px h-4 bg-white/10"></div>

        <div class="flex items-center gap-2 text-white/80 font-bold tracking-widest">
          <Clock class="w-4 h-4 text-purple-400" />
          <span>{{ currentTime }}</span>
        </div>
      </div>

      <!-- Exit Button -->
      <button
        @click="emit('exit')"
        class="kip-btn-ghost px-5 py-3 rounded-2xl border-white/10 hover:border-rose-500/40 hover:bg-rose-500/10 text-white/70 hover:text-white flex items-center gap-2.5 text-xs font-mono uppercase font-bold transition shadow-lg"
      >
        <X class="w-4 h-4 text-rose-400" />
        <span>Return to Desktop [B]</span>
      </button>
    </header>

    <!-- Main Stage: Hero Launch Center Card -->
    <main class="flex-1 flex flex-col justify-center my-6 z-10 max-w-6xl mx-auto w-full min-h-0">
      <div class="grid grid-cols-12 gap-8 items-center">
        <!-- Hero Primary Ignition Deck -->
        <section class="col-span-7 kip-card p-10 border border-indigo-500/40 bg-gradient-to-br from-indigo-950/40 via-black/80 to-purple-950/20 shadow-[0_0_80px_rgba(99,102,241,0.25)] flex flex-col justify-between h-[380px] relative overflow-hidden group">
          <div class="absolute -top-32 -right-32 w-80 h-80 bg-indigo-500/20 blur-[100px] rounded-full pointer-events-none group-hover:bg-indigo-500/35 transition-colors"></div>

          <div>
            <div class="flex justify-between items-start mb-4">
              <span class="text-[10px] font-mono uppercase tracking-[0.25em] text-indigo-400 font-black flex items-center gap-2">
                <Sparkles class="w-3.5 h-3.5" /> Core Instance Ready
              </span>
              <span class="px-3 py-1 rounded-xl text-xs font-mono uppercase font-bold" :class="overview.isGameRunning ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/40 animate-pulse' : 'bg-white/10 text-white/60 border border-white/10'">
                {{ overview.isGameRunning ? 'PROCESS ACTIVE' : 'STANDBY' }}
              </span>
            </div>

            <h1 class="text-4xl font-black uppercase text-white tracking-tight leading-none mb-2">
              Minecraft 26.3
            </h1>
            <p class="text-sm text-white/60 font-mono">Fabric Turbo • OpenJDK 25 HotSpot JVM • Pure Direct Launch</p>
          </div>

          <!-- Mid Metrics Grid -->
          <div class="grid grid-cols-3 gap-3 my-4">
            <div class="bg-black/60 p-3.5 rounded-2xl border border-white/5 flex flex-col">
              <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Total Playtime</span>
              <span class="text-lg font-black text-indigo-300 mt-0.5">{{ overview.playtime }}</span>
            </div>
            <div class="bg-black/60 p-3.5 rounded-2xl border border-white/5 flex flex-col">
              <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Instance Storage</span>
              <span class="text-lg font-black text-white mt-0.5">{{ overview.size }}</span>
            </div>
            <div class="bg-black/60 p-3.5 rounded-2xl border border-white/5 flex flex-col">
              <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Saved Worlds</span>
              <span class="text-lg font-black text-emerald-400 mt-0.5">{{ overview.saves }} Universes</span>
            </div>
          </div>

          <!-- Hero Action Trigger -->
          <button
            data-gamepad-action="ignite"
            @click="handleInstantIgnite"
            :disabled="isLaunching"
            class="w-full py-5 rounded-2xl font-black text-lg tracking-[0.2em] uppercase transition-all duration-300 flex items-center justify-center gap-3 shadow-2xl cursor-pointer"
            :class="overview.isGameRunning ? 'bg-rose-500 hover:bg-rose-400 text-white border border-rose-400 shadow-[0_0_40px_rgba(244,63,94,0.6)]' : 'bg-gradient-to-r from-amber-400 via-emerald-400 to-indigo-500 hover:brightness-110 text-black border border-white/30 shadow-[0_0_50px_rgba(245,158,11,0.5)]'"
          >
            <template v-if="overview.isGameRunning">
              <Skull class="w-6 h-6 fill-current animate-pulse" />
              <span>TERMINATE GAME (PID: {{ overview.runningGamePid }}) [PRESS Y]</span>
            </template>
            <template v-else-if="isLaunching">
              <span class="w-6 h-6 border-3 border-black border-t-transparent rounded-full animate-spin"></span>
              <span>{{ currentProgress.message }}</span>
            </template>
            <template v-else>
              <Play class="w-6 h-6 fill-current" />
              <span>IGNITE INSTANCE [PRESS Y OR A]</span>
            </template>
          </button>
        </section>

        <!-- Live Voxel Region / World Snapshot Poster -->
        <section class="col-span-5 kip-card p-8 border border-white/10 bg-black/60 shadow-2xl flex flex-col justify-between h-[380px]">
          <div>
            <span class="text-[9px] font-mono uppercase tracking-widest text-emerald-400 font-bold block mb-2">Most Recent Exploration</span>
            <div v-if="overview.lastWorld" class="flex items-center gap-4 mb-4">
              <img
                :src="overview.lastWorld.icon || 'data:image/svg+xml;utf8,<svg xmlns=\'http://www.w3.org/2000/svg\' width=\'64\' height=\'64\' viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'%2310b981\' stroke-width=\'2\'><circle cx=\'12\' cy=\'12\' r=\'10\'/></svg>'"
                class="w-20 h-20 rounded-2xl object-cover border border-white/10 bg-black/60 p-1 shadow-2xl"
              >
              <div class="min-w-0">
                <h3 class="text-xl font-black text-white truncate">{{ overview.lastWorld.name }}</h3>
                <span class="text-xs font-mono text-emerald-400 uppercase font-bold">{{ overview.lastWorld.mode }} Mode</span>
                <p class="text-[10px] font-mono text-white/40 mt-1">Last Played: {{ overview.lastWorld.lastPlayed }}</p>
              </div>
            </div>
            <div v-else class="py-12 text-center text-white/30 font-mono text-xs">
              No recent saves discovered in profile.
            </div>
          </div>

          <div class="p-4 bg-black/80 rounded-2xl border border-white/5 font-mono text-xs space-y-1.5">
            <div class="flex justify-between text-white/60">
              <span>RAM Allocation:</span>
              <span class="text-white font-bold">{{ overview.usedRamGb }}G / {{ overview.totalRamGb }}G</span>
            </div>
            <div class="flex justify-between text-white/60">
              <span>Host CPU Load:</span>
              <span class="text-emerald-400 font-bold">{{ overview.cpuUsagePercent }}% ({{ overview.cpuBrand }})</span>
            </div>
          </div>
        </section>
      </div>

      <!-- Horizontal Console Shelf Carousel -->
      <section class="mt-8">
        <h3 class="text-xs font-mono font-bold uppercase tracking-widest text-white/40 mb-3 flex items-center gap-2">
          <Layers class="w-4 h-4 text-indigo-400" /> Navigation Shelves [LB / RB]
        </h3>

        <div class="grid grid-cols-4 gap-4">
          <button
            v-for="(shelf, idx) in consoleShelves.slice(0, 4)"
            :key="shelf.id"
            @click="handleShelfSelect(shelf.id)"
            class="kip-card p-5 text-left border border-white/10 bg-black/50 hover:bg-white/10 transition-all duration-200 group flex items-center gap-4 cursor-pointer"
            :class="shelf.border"
          >
            <div class="p-3.5 rounded-2xl bg-black/60 border border-white/10 group-hover:scale-110 transition-transform shrink-0" :class="shelf.color">
              <component :is="shelf.icon" class="w-6 h-6 stroke-[2]" />
            </div>
            <div class="min-w-0">
              <h4 class="font-black text-sm text-white uppercase tracking-wider group-hover:text-indigo-300 transition-colors">{{ shelf.name }}</h4>
              <p class="text-[10px] text-white/40 font-mono truncate mt-0.5">{{ shelf.desc }}</p>
            </div>
          </button>
        </div>
      </section>
    </main>

    <!-- Bottom Gamepad Legend Bar -->
    <footer class="flex justify-between items-center shrink-0 z-20 pt-6 border-t border-white/10 font-mono text-xs">
      <div class="flex items-center gap-8">
        <div class="flex items-center gap-2.5">
          <span class="w-6 h-6 rounded-full bg-emerald-500 text-black font-black text-xs flex items-center justify-center shadow-[0_0_12px_rgba(16,185,129,0.7)]">A</span>
          <span class="text-white/80 font-bold uppercase">Select</span>
        </div>

        <div class="flex items-center gap-2.5">
          <span class="w-6 h-6 rounded-full bg-rose-500 text-white font-black text-xs flex items-center justify-center shadow-[0_0_12px_rgba(244,63,94,0.7)]">B</span>
          <span class="text-white/80 font-bold uppercase">Back / Exit</span>
        </div>

        <div class="flex items-center gap-2.5">
          <span class="w-6 h-6 rounded-full bg-blue-500 text-white font-black text-xs flex items-center justify-center shadow-[0_0_12px_rgba(59,130,246,0.7)]">X</span>
          <span class="text-white/80 font-bold uppercase">Mod Doctor</span>
        </div>

        <div class="flex items-center gap-2.5">
          <span class="w-6 h-6 rounded-full bg-amber-400 text-black font-black text-xs flex items-center justify-center shadow-[0_0_12px_rgba(245,158,11,0.7)]">Y</span>
          <span class="text-white/80 font-bold uppercase">Quick Ignite</span>
        </div>

        <div class="flex items-center gap-2.5">
          <span class="px-2 py-0.5 rounded bg-white/15 text-white font-mono font-bold border border-white/20">LB / RB</span>
          <span class="text-white/80 font-bold uppercase">Switch Deck</span>
        </div>
      </div>

      <div class="text-[10px] text-white/40 uppercase tracking-widest font-black">
        K.I.P. Studio • Console Experience Engine
      </div>
    </footer>
  </div>
</template>