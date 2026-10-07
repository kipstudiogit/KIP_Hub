<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import {
  Hexagon,
  Pin,
  Maximize2,
  Minus,
  X,
  Zap,
  Play,
  Skull,
  Activity,
  Cpu,
  HardDrive,
  Mic,
  MicOff,
  Headphones,
  FolderOpen,
  FolderSync,
} from 'lucide-vue-next'
import { state, t } from '@/store'
import {
  isPinned,
  minimizeWindow,
  closeWindow,
  togglePinWindow,
  toggleMiniMode,
} from '../../composables/useWindow'
import { useDashboard } from '../../composables/useDashboard'
import { useLauncher } from '../../composables/useLauncher'
import { voiceState, toggleMute, toggleDeafen, joinVoiceChannel } from '../../composables/useVoice'
import { bridge, invokeSafe } from '@/bridge'

const emit = defineEmits<{
  (e: 'expand'): void
}>()

const {
  overview,
  cpuStatusColor,
  ramStatusColor,
  fetchOverview,
  terminateProcess,
} = useDashboard()

const { ignite, isLaunching, currentProgress } = useLauncher()

let pollingInterval: ReturnType<typeof setInterval> | null = null

function formatInstanceName(path: string): string {
  if (!path) return 'Default (.minecraft)'
  const clean = path.replace(/[\\/]+$/, '')
  const name = clean.split(/[\\/]/).pop()
  return name || 'Default (.minecraft)'
}

async function handleIgnition(): Promise<void> {
  if (overview.value.isGameRunning) {
    await terminateProcess()
  } else {
    await ignite('26.3', 'fabric', null, 0, '', '1920x1080', true, false)
  }
}

async function cycleNextInstance(): Promise<void> {
  const instances = state.settings.instances || []
  if (instances.length <= 1) return
  const currentIdx = instances.indexOf(state.settings.mc_dir)
  const nextIdx = (currentIdx + 1) % instances.length
  const nextDir = instances[nextIdx]
  if (nextDir) {
    state.settings.mc_dir = nextDir
    await invokeSafe<boolean>('change_instance', { newDir: nextDir })
    await fetchOverview()
  }
}

async function openGameDir(): Promise<void> {
  try {
    await invokeSafe('open_instance_directory')
  } catch {}
}

onMounted(() => {
  fetchOverview()
  pollingInterval = setInterval(fetchOverview, 3000)
})

onBeforeUnmount(() => {
  if (pollingInterval) clearInterval(pollingInterval)
})
</script>

<template>
  <div class="h-full w-full bg-[#030305]/95 backdrop-blur-2xl flex flex-col justify-between p-4 text-white select-none relative font-sans border border-indigo-500/30 rounded-2xl overflow-hidden shadow-[0_0_50px_rgba(0,0,0,0.9)]">
    <!-- Ambient Mini Backlight -->
    <div
      class="absolute -top-24 -right-24 w-60 h-60 blur-[90px] rounded-full pointer-events-none transition-colors duration-700"
      :class="state.isMcRunning ? 'bg-emerald-500/20' : 'bg-indigo-500/20'"
    ></div>

    <!-- Integrated Mini Drag Titlebar -->
    <header
      class="flex justify-between items-center pb-3 border-b border-white/10 shrink-0 z-20 cursor-move"
      data-tauri-drag-region
    >
      <div class="flex items-center gap-2 pointer-events-none">
        <div class="p-1.5 rounded-lg bg-indigo-500/20 border border-indigo-500/40 text-indigo-400">
          <Hexagon class="w-3.5 h-3.5 animate-pulse stroke-[2.2]" />
        </div>
        <div class="flex flex-col">
          <span class="font-black text-[11px] uppercase tracking-wider leading-none text-white">K.I.P. Widget</span>
          <span class="text-[7px] font-mono text-white/40 uppercase tracking-widest mt-0.5">Desktop Matrix</span>
        </div>
      </div>

      <div class="flex items-center gap-1 titlebar-btn pointer-events-auto">
        <button
          @click="togglePinWindow"
          class="p-1.5 rounded-lg transition"
          :class="isPinned ? 'bg-indigo-500/20 text-indigo-300 border border-indigo-500/40 shadow-[0_0_10px_rgba(99,102,241,0.5)]' : 'text-white/40 hover:text-white hover:bg-white/5'"
          :title="isPinned ? 'Unpin' : 'Pin Always-on-Top'"
        >
          <Pin class="w-3 h-3" :class="isPinned ? 'fill-current' : ''" />
        </button>

        <button
          @click="toggleMiniMode"
          class="p-1.5 text-white/40 hover:text-white hover:bg-white/5 rounded-lg transition"
          title="Expand View"
        >
          <Maximize2 class="w-3 h-3" />
        </button>

        <button
          @click="minimizeWindow"
          class="p-1.5 text-white/40 hover:text-white hover:bg-white/5 rounded-lg transition"
          title="Minimize"
        >
          <Minus class="w-3 h-3" />
        </button>

        <button
          @click="closeWindow"
          class="p-1.5 text-white/40 hover:text-white hover:bg-rose-600 rounded-lg transition"
          title="Close"
        >
          <X class="w-3 h-3" />
        </button>
      </div>
    </header>

    <!-- Mini Main Stage -->
    <main class="flex-1 flex flex-col justify-between py-3 gap-3 z-10 min-h-0">
      <!-- Active Instance Card -->
      <section class="bg-black/60 p-3 rounded-xl border border-white/5 flex items-center justify-between shadow-inner">
        <div class="min-w-0 pr-2">
          <span class="text-[8px] font-mono text-white/40 uppercase tracking-widest font-bold block">Active Universe</span>
          <span class="text-xs font-black text-white truncate block mt-0.5 font-mono">{{ formatInstanceName(state.settings.mc_dir) }}</span>
        </div>

        <div class="flex items-center gap-1 shrink-0">
          <button
            @click="cycleNextInstance"
            class="p-1.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg text-white/60 hover:text-white transition"
            title="Cycle Next Instance"
          >
            <FolderSync class="w-3.5 h-3.5 text-indigo-400" />
          </button>
          <button
            @click="openGameDir"
            class="p-1.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-lg text-white/60 hover:text-white transition"
            title="Open Directory"
          >
            <FolderOpen class="w-3.5 h-3.5 text-emerald-400" />
          </button>
        </div>
      </section>

      <!-- Central Hero Ignition Dial -->
      <section class="kip-card p-4 border border-white/10 flex flex-col items-center text-center justify-between relative overflow-hidden bg-black/60 shadow-xl">
        <div class="flex items-center justify-between w-full mb-2 text-[9px] font-mono font-bold uppercase">
          <span class="flex items-center gap-1.5">
            <span
              class="w-2 h-2 rounded-full"
              :class="overview.isGameRunning ? 'bg-emerald-400 animate-ping' : 'bg-white/30'"
            ></span>
            <span :class="overview.isGameRunning ? 'text-emerald-400' : 'text-white/50'">
              {{ overview.isGameRunning ? 'GAME ACTIVE' : 'ENGINE READY' }}
            </span>
          </span>
          <span class="text-white/40">MC 26.3 • Fabric</span>
        </div>

        <!-- Master Circular Ignition Trigger -->
        <button
          @click="handleIgnition"
          :disabled="isLaunching"
          class="w-full py-4 rounded-xl font-black text-xs uppercase tracking-widest transition-all duration-300 flex items-center justify-center gap-2.5 shadow-2xl cursor-pointer my-1 border"
          :class="overview.isGameRunning ? 'bg-rose-500 hover:bg-rose-400 text-white border-rose-400 shadow-[0_0_20px_rgba(244,63,94,0.5)]' : 'bg-gradient-to-r from-emerald-400 to-indigo-500 hover:brightness-110 text-black border-emerald-300 shadow-[0_0_25px_rgba(16,185,129,0.4)]'"
        >
          <template v-if="overview.isGameRunning">
            <Skull class="w-4 h-4 fill-current animate-pulse" />
            <span>TERMINATE (PID: {{ overview.runningGamePid }})</span>
          </template>
          <template v-else-if="isLaunching">
            <span class="w-4 h-4 border-2 border-black border-t-transparent rounded-full animate-spin"></span>
            <span>{{ currentProgress.message }}</span>
          </template>
          <template v-else>
            <Zap class="w-4 h-4 fill-current" />
            <span>IGNITE INSTANCE</span>
          </template>
        </button>

        <div v-if="isLaunching" class="w-full mt-2">
          <div class="w-full h-1 bg-black/80 rounded-full overflow-hidden">
            <div class="h-full bg-emerald-400 transition-all duration-150" :style="{ width: `${currentProgress.percent}%` }"></div>
          </div>
        </div>
      </section>

      <!-- Compact Dual Resource Sensors -->
      <section class="grid grid-cols-2 gap-2">
        <div class="bg-black/60 p-2.5 rounded-xl border border-white/5 flex flex-col justify-between">
          <div class="flex justify-between items-center text-[8px] font-mono font-bold uppercase text-white/40 mb-1">
            <span class="flex items-center gap-1"><Cpu class="w-2.5 h-2.5" /> CPU</span>
            <span :class="cpuStatusColor">{{ overview.cpuUsagePercent }}%</span>
          </div>
          <div class="w-full h-1 bg-black/80 rounded-full overflow-hidden">
            <div class="h-full bg-gradient-to-r from-emerald-500 to-rose-500 transition-all duration-300" :style="{ width: `${overview.cpuUsagePercent}%` }"></div>
          </div>
        </div>

        <div class="bg-black/60 p-2.5 rounded-xl border border-white/5 flex flex-col justify-between">
          <div class="flex justify-between items-center text-[8px] font-mono font-bold uppercase text-white/40 mb-1">
            <span class="flex items-center gap-1"><Activity class="w-2.5 h-2.5" /> RAM</span>
            <span :class="ramStatusColor">{{ overview.ramUsagePercent }}%</span>
          </div>
          <div class="w-full h-1 bg-black/80 rounded-full overflow-hidden">
            <div class="h-full bg-cyan-400 transition-all duration-300" :style="{ width: `${overview.ramUsagePercent}%` }"></div>
          </div>
        </div>
      </section>

      <!-- Mini Voice Matrix Pill Bar -->
      <section class="bg-black/60 p-2.5 rounded-xl border border-white/5 flex items-center justify-between text-xs font-mono">
        <div class="flex items-center gap-2 min-w-0 pr-1">
          <span
            class="w-2 h-2 rounded-full"
            :class="voiceState.isConnected ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.9)] animate-pulse' : 'bg-white/20'"
          ></span>
          <span class="text-[10px] font-bold truncate text-white/80">
            {{ voiceState.isConnected ? `Lobby: ${voiceState.channelId}` : 'Voice Offline' }}
          </span>
        </div>

        <div v-if="voiceState.isConnected" class="flex items-center gap-1 shrink-0">
          <button
            @click="toggleMute"
            class="p-1 rounded-lg transition"
            :class="voiceState.isMuted ? 'bg-rose-500/20 text-rose-300' : 'bg-white/5 text-white/60 hover:text-white'"
          >
            <MicOff v-if="voiceState.isMuted" class="w-3 h-3" />
            <Mic v-else class="w-3 h-3" />
          </button>
          <button
            @click="toggleDeafen"
            class="p-1 rounded-lg transition"
            :class="voiceState.isDeafened ? 'bg-amber-500/20 text-amber-300' : 'bg-white/5 text-white/60 hover:text-white'"
          >
            <Headphones class="w-3 h-3" />
          </button>
        </div>

        <button
          v-else
          @click="joinVoiceChannel('kip-global')"
          class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 text-[9px] font-bold uppercase hover:bg-emerald-500/20 transition"
        >
          Join
        </button>
      </section>
    </main>

    <!-- Bottom Restore Dock Trigger -->
    <footer class="pt-2 border-t border-white/10 shrink-0 z-20 flex justify-between items-center text-[9px] font-mono text-white/40">
      <span>Playtime: {{ overview.playtime }}</span>
      <button
        @click="emit('expand')"
        class="text-indigo-400 hover:text-indigo-300 font-bold uppercase tracking-wider transition flex items-center gap-1"
      >
        <Maximize2 class="w-3 h-3" />
        <span>Expand Deck</span>
      </button>
    </footer>
  </div>
</template>