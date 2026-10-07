<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from 'vue'
import {
  Hexagon,
  X,
  Pin,
  Minimize2,
  Activity,
  BrainCircuit,
  Send,
  Loader,
  Mic,
  Clock,
  Zap,
  StickyNote,
  Compass,
  Copy,
  MemoryStick,
  MapPin,
  Wrench,
  DoorOpen,
} from 'lucide-vue-next'
import {
  voiceState,
  toggleMute,
  toggleDeafen,
  joinVoiceChannel,
  showToast,
} from '@/store'
import { useOverlay } from '../composables/useOverlay'
import { useBooster } from '../composables/useBooster'
import { invokeSafe, bridge } from '@/bridge'
import type { OverlayWidgetKey } from '../types/overlay'

const {
  widgets,
  waypoints,
  telemetryHistory,
  isInteractive,
  hasPinnedWidgets,
  bringToFront,
  toggleWidget,
  togglePinWidget,
  toggleCollapseWidget,
  updateWindowMode,
  exitOverlayMode,
  loadLayout,
  saveLayout,
  loadWaypoints,
  addWaypoint,
  copyWaypointTp,
  pushTelemetryPoint,
} = useOverlay()

const {
  chunkAcceleratorStatus,
  isPrebakingChunks,
  chunkPrebakeProgress,
  memoryMatrixStatus,
  isPingMasterRunning,
  currentMetric,
  triggerChunkPrebake,
  prefaultMemoryMatrix,
} = useBooster()

const currentTime = ref<string>('')
const memoryUsage = ref<number>(0)
const realPlaytime = ref<string>('0h 0m')
const aiPrompt = ref<string>('')
const isAiThinking = ref<boolean>(false)
const chatScroll = ref<HTMLElement | null>(null)
const connectChannel = ref<string>('')
const notesText = ref<string>('')
let noteSaveTimeout: ReturnType<typeof setTimeout> | null = null

const chatHistory = ref<{ role: 'user' | 'ai'; text: string }[]>([
  { role: 'ai', text: 'Neural Oracle HUD online. Ask crafting recipes, game coordinates or crash diagnostics.' },
])

const newWaypoint = ref({
  name: '',
  dimension: 'overworld',
  x: 0,
  y: 64,
  z: 0,
})

let activeDrag: OverlayWidgetKey | null = null
let startX = 0
let startY = 0

function onDrag(e: MouseEvent): void {
  if (!activeDrag) return
  const nextX = Math.max(10, Math.min(window.innerWidth - 300, e.clientX - startX))
  const nextY = Math.max(70, Math.min(window.innerHeight - 150, e.clientY - startY))
  widgets[activeDrag].x = nextX
  widgets[activeDrag].y = nextY
}

function stopDrag(): void {
  if (activeDrag) {
    saveLayout()
  }
  activeDrag = null
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

function startDrag(e: MouseEvent, id: OverlayWidgetKey): void {
  if (!isInteractive.value) return
  activeDrag = id
  bringToFront(id)
  startX = e.clientX - widgets[id].x
  startY = e.clientY - widgets[id].y
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

const telemetrySparklinePoints = computed(() => {
  const data = telemetryHistory.value
  const step = 200 / Math.max(data.length - 1, 1)
  return data
    .map((val, idx) => {
      const x = idx * step
      const y = 40 - (val / 100) * 35
      return `${x},${y}`
    })
    .join(' ')
})

function updateTime(): void {
  currentTime.value = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

async function updateStats(): Promise<void> {
  try {
    const info = await invokeSafe<string>('get_sys_info')
    if (info) {
      const ramLine = info.split('\n').find((l) => l.startsWith('RAM:'))
      if (ramLine) {
        const totalGb = parseFloat(ramLine.replace('RAM:', '').replace('GB', '').trim())
        if (!isNaN(totalGb) && totalGb > 0) {
          const calculated = Math.min(Math.max(Math.round((4.0 / totalGb) * 100), 20), 95)
          memoryUsage.value = calculated
          pushTelemetryPoint(calculated)
        }
      }
    }

    const s = await invokeSafe<{ playtime?: string }>('get_dashboard_stats')
    if (s && s.playtime) {
      realPlaytime.value = s.playtime
    }
  } catch {}
}

async function dismissOverlay(): Promise<void> {
  if (hasPinnedWidgets.value) {
    await updateWindowMode(false)
  } else {
    await exitOverlayMode()
  }
}

async function runQuickTool(toolId: string): Promise<void> {
  try {
    const res = await bridge.executeSystemTool(toolId)
    if (res && res.success) {
      showToast('Action Executed', res.msg, 'success')
    }
  } catch (err: unknown) {
    showToast('Action Failed', String(err), 'danger')
  }
}

async function sendToAi(): Promise<void> {
  const userText = aiPrompt.value.trim()
  if (!userText || isAiThinking.value) return

  chatHistory.value.push({ role: 'user', text: userText })
  aiPrompt.value = ''
  isAiThinking.value = true

  nextTick(() => {
    if (chatScroll.value) {
      chatScroll.value.scrollTop = chatScroll.value.scrollHeight
    }
  })

  try {
    const prompt = `You are an in-game Minecraft HUD expert. Brief, technical reply: ${userText}`
    const res = await invokeSafe<{ success: boolean; answer: string }>('analyze_crash_ai', {
      logSnippet: prompt,
    })
    if (res && res.success) {
      chatHistory.value.push({ role: 'ai', text: res.answer })
    } else {
      chatHistory.value.push({ role: 'ai', text: res?.answer || 'Oracle service unreachable.' })
    }
  } catch (err: unknown) {
    chatHistory.value.push({ role: 'ai', text: `IPC Failure: ${String(err)}` })
  } finally {
    isAiThinking.value = false
    nextTick(() => {
      if (chatScroll.value) {
        chatScroll.value.scrollTop = chatScroll.value.scrollHeight
      }
    })
  }
}

function handleAddWaypoint(): void {
  addWaypoint(
    newWaypoint.value.name,
    newWaypoint.value.dimension,
    newWaypoint.value.x,
    newWaypoint.value.y,
    newWaypoint.value.z
  )
  newWaypoint.value.name = ''
}

function handleJoinVoice(): void {
  if (!connectChannel.value.trim()) return
  joinVoiceChannel(connectChannel.value.trim().toLowerCase())
}

async function loadNote(): Promise<void> {
  try {
    const text = await invokeSafe<string>('get_note')
    notesText.value = text || ''
  } catch {
    notesText.value = ''
  }
}

function debouncedSaveNote(): void {
  if (noteSaveTimeout) clearTimeout(noteSaveTimeout)
  noteSaveTimeout = setTimeout(async () => {
    try {
      await invokeSafe<boolean>('save_note', { text: notesText.value })
    } catch {}
  }, 800)
}

function shouldRenderWidget(key: OverlayWidgetKey): boolean {
  if (isInteractive.value) {
    return widgets[key].show
  }
  return widgets[key].show && widgets[key].pinned
}

function handleOverlayKeyDown(e: KeyboardEvent): void {
  if (e.key === 'Escape' && isInteractive.value) {
    e.preventDefault()
    e.stopPropagation()
    dismissOverlay()
  }
}

let timeInterval: ReturnType<typeof setInterval> | null = null
let statsInterval: ReturnType<typeof setInterval> | null = null

onMounted(async () => {
  updateTime()
  await updateStats()
  await loadNote()
  await loadLayout()
  await loadWaypoints()

  window.addEventListener('keydown', handleOverlayKeyDown)
  timeInterval = setInterval(updateTime, 1000)
  statsInterval = setInterval(updateStats, 4000)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleOverlayKeyDown)
  if (timeInterval) clearInterval(timeInterval)
  if (statsInterval) clearInterval(statsInterval)
  if (noteSaveTimeout) clearTimeout(noteSaveTimeout)
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
})
</script>

<template>
  <div
    class="fixed inset-0 select-none overflow-hidden font-sans text-white transition-colors duration-300"
    :class="isInteractive ? 'bg-black/60 backdrop-blur-md pointer-events-auto z-[9999]' : 'bg-transparent pointer-events-none z-[8000]'"
  >
    <header
      v-if="isInteractive"
      class="absolute top-5 left-1/2 -translate-x-1/2 z-[10000] pointer-events-auto bg-[#0a0a0f]/90 backdrop-blur-2xl border border-indigo-500/40 p-2 rounded-2xl flex items-center gap-2 shadow-[0_0_50px_rgba(0,0,0,0.9)]"
    >
      <div class="flex items-center gap-2.5 px-3">
        <Hexagon class="w-5 h-5 text-indigo-400 animate-pulse stroke-[2.2]" />
        <span class="font-black tracking-[0.18em] text-xs uppercase leading-none">
          K.I.P. <span class="text-indigo-400">Bar</span>
        </span>
      </div>

      <div class="w-px h-5 bg-white/10 mx-0.5"></div>

      <div class="flex items-center gap-1">
        <button
          @click="toggleWidget('chunks')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.chunks.show ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Quantum Chunk Matrix"
        >
          <Compass class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('pvp')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.pvp.show ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Ping-Master PvP Accelerator"
        >
          <Zap class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('matrix')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.matrix.show ? 'bg-indigo-500/20 text-indigo-300 border-indigo-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Kernel Memory Matrix"
        >
          <MemoryStick class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('telemetry')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.telemetry.show ? 'bg-blue-500/20 text-blue-300 border-blue-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Hardware Telemetry"
        >
          <Activity class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('voice')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.voice.show ? 'bg-purple-500/20 text-purple-300 border-purple-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Multiplayer Voice Matrix"
        >
          <Mic class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('ai')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.ai.show ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Neural Oracle Assistant"
        >
          <BrainCircuit class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('waypoints')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.waypoints.show ? 'bg-teal-500/20 text-teal-300 border-teal-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Voxel Coordinates & POIs"
        >
          <MapPin class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('actions')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.actions.show ? 'bg-rose-500/20 text-rose-300 border-rose-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Quick Direct Tools"
        >
          <Wrench class="w-4 h-4" />
        </button>

        <button
          @click="toggleWidget('notes')"
          class="p-2 rounded-xl transition border cursor-pointer"
          :class="widgets.notes.show ? 'bg-yellow-500/20 text-yellow-300 border-yellow-500/40 shadow' : 'bg-white/5 border-transparent text-white/50 hover:text-white'"
          title="Persistent Scratchpad"
        >
          <StickyNote class="w-4 h-4" />
        </button>
      </div>

      <div class="w-px h-5 bg-white/10 mx-0.5"></div>

      <div class="px-3 font-mono text-xs font-bold text-white/80 tracking-widest flex items-center gap-3">
        <span class="text-emerald-400 flex items-center gap-1">
          <Zap class="w-3 h-3 fill-current" /> {{ isPingMasterRunning ? `${currentMetric.currentPingMs}ms` : 'Offline' }}
        </span>
        <span class="flex items-center gap-1">
          <Clock class="w-3.5 h-3.5 text-indigo-400" /> {{ currentTime }}
        </span>
      </div>

      <div class="w-px h-5 bg-white/10 mx-0.5"></div>

      <button
        @click="dismissOverlay"
        class="p-2 rounded-xl bg-amber-500/20 hover:bg-amber-500 text-amber-300 hover:text-black transition flex items-center gap-1.5 text-xs font-mono font-bold cursor-pointer"
        title="Return to Game [ESC]"
      >
        <X class="w-4 h-4" />
        <span>{{ hasPinnedWidgets ? 'HUD Mode [ESC]' : 'Close [ESC]' }}</span>
      </button>

      <button
        @click="exitOverlayMode"
        class="p-2 rounded-xl bg-rose-500/20 hover:bg-rose-500 text-rose-300 hover:text-white transition flex items-center gap-1.5 text-xs font-mono font-bold cursor-pointer"
        title="Exit to Desktop Launcher"
      >
        <DoorOpen class="w-4 h-4" />
        <span>Launcher</span>
      </button>
    </header>

    <div class="relative w-full h-full pointer-events-none">
      <transition name="fade">
        <div
          v-if="shouldRenderWidget('chunks')"
          @mousedown="bringToFront('chunks')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-emerald-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.chunks.x + 'px', top: widgets.chunks.y + 'px', width: widgets.chunks.width + 'px', zIndex: widgets.chunks.z }"
        >
          <div @mousedown="startDrag($event, 'chunks')" class="p-3 bg-emerald-500/10 border-b border-emerald-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-emerald-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <Compass class="w-3.5 h-3.5" /> Quantum Chunks
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('chunks')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.chunks.pinned ? 'text-emerald-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.chunks.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleCollapseWidget('chunks')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer">
                <Minimize2 class="w-3 h-3" />
              </button>
              <button @click.stop="toggleWidget('chunks')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div v-show="!widgets.chunks.collapsed" class="p-3.5 flex flex-col gap-2.5 text-xs font-mono">
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">SIMD Vector Math:</span>
              <span class="text-emerald-400 font-bold text-[10px]">{{ chunkAcceleratorStatus.simdCapabilities.vectorWidthBits }}-bit AVX</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">MCA Ring Buffered:</span>
              <span class="text-cyan-300 font-bold text-[10px]">{{ chunkAcceleratorStatus.totalChunksBuffered }} chunks</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Zero-GC Off-Heap:</span>
              <span class="text-purple-300 font-bold text-[10px]">{{ chunkAcceleratorStatus.arenaMetrics.zeroGcEfficiencyPercent }}% bypass</span>
            </div>

            <div v-if="isInteractive" class="pt-2 border-t border-white/5">
              <button
                @click="triggerChunkPrebake(0, 0, 16)"
                :disabled="isPrebakingChunks"
                class="kip-btn-primary w-full py-2 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-[10px] tracking-wider cursor-pointer"
              >
                <Loader v-if="isPrebakingChunks" class="w-3 h-3 animate-spin" />
                <Zap v-else class="w-3 h-3 fill-current" />
                <span>{{ isPrebakingChunks ? `${Math.round(chunkPrebakeProgress.percent)}%` : '1-Click Pre-Bake Cone' }}</span>
              </button>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('pvp')"
          @mousedown="bringToFront('pvp')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-cyan-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.pvp.x + 'px', top: widgets.pvp.y + 'px', width: widgets.pvp.width + 'px', zIndex: widgets.pvp.z }"
        >
          <div @mousedown="startDrag($event, 'pvp')" class="p-3 bg-cyan-500/10 border-b border-cyan-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-cyan-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <Zap class="w-3.5 h-3.5" /> Ping-Master PvP
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('pvp')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.pvp.pinned ? 'text-cyan-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.pvp.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleCollapseWidget('pvp')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer">
                <Minimize2 class="w-3 h-3" />
              </button>
              <button @click.stop="toggleWidget('pvp')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div v-show="!widgets.pvp.collapsed" class="p-3.5 flex flex-col gap-2.5 text-xs font-mono">
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Real-time Latency:</span>
              <span class="text-cyan-400 font-bold">{{ currentMetric.currentPingMs }} ms</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Network Jitter:</span>
              <span class="text-amber-300 font-bold">{{ currentMetric.jitterMs }} ms</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Hit-Reg Quality:</span>
              <span class="text-emerald-400 font-bold text-[10px] truncate max-w-[170px]">{{ currentMetric.hitRegQuality }}</span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Active Route:</span>
              <span class="text-white/80 font-bold text-[9px] truncate max-w-[170px]">{{ currentMetric.activeRoute }}</span>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('matrix')"
          @mousedown="bringToFront('matrix')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-indigo-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.matrix.x + 'px', top: widgets.matrix.y + 'px', width: widgets.matrix.width + 'px', zIndex: widgets.matrix.z }"
        >
          <div @mousedown="startDrag($event, 'matrix')" class="p-3 bg-indigo-500/10 border-b border-indigo-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-indigo-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <MemoryStick class="w-3.5 h-3.5" /> Memory Matrix
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('matrix')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.matrix.pinned ? 'text-indigo-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.matrix.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('matrix')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3.5 flex flex-col gap-2.5 text-xs font-mono">
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">2MB HugeTLB Pages:</span>
              <span :class="memoryMatrixStatus.largePagesActive ? 'text-emerald-400 font-bold' : 'text-white/40'">
                {{ memoryMatrixStatus.largePagesActive ? 'Active' : 'Disabled' }}
              </span>
            </div>
            <div class="flex justify-between items-center">
              <span class="text-white/50 text-[10px]">Compact Heap Savings:</span>
              <span class="text-indigo-300 font-bold">~{{ memoryMatrixStatus.savedMemoryMb }} MB</span>
            </div>
            <button
              v-if="isInteractive"
              @click="prefaultMemoryMatrix"
              class="kip-btn-ghost w-full py-1.5 text-[10px] font-bold uppercase text-indigo-300 border-indigo-500/30 hover:bg-indigo-500/10 mt-1 cursor-pointer"
            >
              Prefault Standby Cache
            </button>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('telemetry')"
          @mousedown="bringToFront('telemetry')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-blue-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.telemetry.x + 'px', top: widgets.telemetry.y + 'px', width: widgets.telemetry.width + 'px', zIndex: widgets.telemetry.z }"
        >
          <div @mousedown="startDrag($event, 'telemetry')" class="p-3 bg-blue-500/10 border-b border-blue-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-blue-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <Activity class="w-3.5 h-3.5" /> Host Sensors
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('telemetry')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.telemetry.pinned ? 'text-blue-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.telemetry.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('telemetry')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3.5 flex flex-col gap-3">
            <div>
              <div class="flex justify-between items-center text-[10px] font-mono text-white/50 uppercase mb-1">
                <span>RAM Load Sensor</span>
                <span class="text-blue-400 font-bold">{{ memoryUsage }}%</span>
              </div>
              <svg class="w-full h-10 bg-black/60 rounded-xl p-1 border border-white/5" viewBox="0 0 200 40">
                <polyline fill="none" stroke="#60a5fa" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" :points="telemetrySparklinePoints" />
              </svg>
            </div>
            <div class="flex justify-between items-center text-xs font-mono pt-1 border-t border-white/5">
              <span class="text-white/50 text-[10px]">Session Time:</span>
              <span class="text-white font-bold">{{ realPlaytime }}</span>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('voice')"
          @mousedown="bringToFront('voice')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-purple-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.voice.x + 'px', top: widgets.voice.y + 'px', width: widgets.voice.width + 'px', zIndex: widgets.voice.z }"
        >
          <div @mousedown="startDrag($event, 'voice')" class="p-3 bg-purple-500/10 border-b border-purple-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-purple-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <Mic class="w-3.5 h-3.5" /> Voice Matrix
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('voice')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.voice.pinned ? 'text-purple-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.voice.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('voice')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3.5 flex flex-col gap-3 font-mono text-xs">
            <div v-if="!voiceState.isConnected" class="flex flex-col gap-2">
              <input v-model="connectChannel" @keyup.enter="handleJoinVoice" type="text" placeholder="Lobby code..." class="kip-input py-1 text-xs">
              <button @click="handleJoinVoice" class="kip-btn-primary py-1.5 text-xs font-bold uppercase bg-purple-500 hover:bg-purple-400 cursor-pointer">Join</button>
            </div>
            <div v-else class="flex flex-col gap-2">
              <div class="flex justify-between items-center text-[10px]">
                <span class="text-white/60">Lobby: {{ voiceState.channelId }}</span>
                <span class="text-purple-300 font-bold">{{ voiceState.participants.length + 1 }} peers</span>
              </div>
              <div class="flex gap-1.5 pt-1 border-t border-white/5">
                <button @click="toggleMute" class="flex-1 py-1 rounded bg-white/5 border border-white/10 text-white text-[10px] font-bold cursor-pointer">
                  {{ voiceState.isMuted ? 'Unmute' : 'Mute' }}
                </button>
                <button @click="toggleDeafen" class="flex-1 py-1 rounded bg-white/5 border border-white/10 text-white text-[10px] font-bold cursor-pointer">
                  {{ voiceState.isDeafened ? 'Undeaf' : 'Deaf' }}
                </button>
              </div>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('ai')"
          @mousedown="bringToFront('ai')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-amber-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.ai.x + 'px', top: widgets.ai.y + 'px', width: widgets.ai.width + 'px', zIndex: widgets.ai.z }"
        >
          <div @mousedown="startDrag($event, 'ai')" class="p-3 bg-amber-500/10 border-b border-amber-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-amber-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <BrainCircuit class="w-3.5 h-3.5" /> Neural Oracle
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('ai')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.ai.pinned ? 'text-amber-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.ai.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('ai')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="flex flex-col h-72">
            <div ref="chatScroll" class="flex-1 overflow-y-auto custom-scroll p-3 flex flex-col gap-2 min-h-0 select-text text-xs leading-relaxed">
              <div v-for="(msg, idx) in chatHistory" :key="idx" class="flex flex-col" :class="msg.role === 'user' ? 'items-end' : 'items-start'">
                <div class="p-2.5 rounded-xl text-xs max-w-[90%]" :class="msg.role === 'user' ? 'bg-amber-500/20 text-amber-200 border border-amber-500/30' : 'bg-black/60 border border-white/10 text-white/90'">
                  <p class="whitespace-pre-wrap">{{ msg.text }}</p>
                </div>
              </div>
            </div>
            <div v-if="isInteractive" class="p-2.5 border-t border-white/5 bg-black/40 flex gap-1.5">
              <input v-model="aiPrompt" @keyup.enter="sendToAi" type="text" placeholder="Ask Oracle..." class="kip-input py-1 text-xs flex-1">
              <button @click="sendToAi" :disabled="isAiThinking" class="kip-btn-primary px-3 bg-amber-500 hover:bg-amber-400 text-black cursor-pointer">
                <Send class="w-3 h-3" />
              </button>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('waypoints')"
          @mousedown="bringToFront('waypoints')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-teal-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.waypoints.x + 'px', top: widgets.waypoints.y + 'px', width: widgets.waypoints.width + 'px', zIndex: widgets.waypoints.z }"
        >
          <div @mousedown="startDrag($event, 'waypoints')" class="p-3 bg-teal-500/10 border-b border-teal-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-teal-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <MapPin class="w-3.5 h-3.5" /> Waypoints
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('waypoints')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.waypoints.pinned ? 'text-teal-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.waypoints.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('waypoints')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3.5 flex flex-col gap-2 font-mono text-xs">
            <div class="flex flex-col gap-1.5 max-h-40 overflow-y-auto custom-scroll pr-1">
              <div v-for="wp in waypoints" :key="wp.id" class="p-2 bg-black/60 rounded-xl border border-white/5 flex justify-between items-center">
                <div>
                  <span class="text-white font-bold block">{{ wp.name }}</span>
                  <span class="text-[9px] text-teal-300">[{{ wp.x }}, {{ wp.y }}, {{ wp.z }}]</span>
                </div>
                <button @click="copyWaypointTp(wp)" class="p-1 text-teal-300 hover:bg-teal-500/20 rounded cursor-pointer">
                  <Copy class="w-3 h-3" />
                </button>
              </div>
            </div>
            <div v-if="isInteractive" class="flex gap-1 pt-2 border-t border-white/5">
              <input v-model="newWaypoint.name" placeholder="POI Name" class="kip-input py-1 text-[10px] flex-1">
              <input v-model.number="newWaypoint.x" placeholder="X" class="kip-input py-1 text-[10px] w-12 text-center">
              <input v-model.number="newWaypoint.y" placeholder="Y" class="kip-input py-1 text-[10px] w-12 text-center">
              <input v-model.number="newWaypoint.z" placeholder="Z" class="kip-input py-1 text-[10px] w-12 text-center">
              <button @click="handleAddWaypoint" class="kip-btn-primary px-2.5 py-1 text-[10px] bg-teal-500 hover:bg-teal-400 text-black font-black uppercase cursor-pointer">Add</button>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('actions')"
          @mousedown="bringToFront('actions')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-rose-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.actions.x + 'px', top: widgets.actions.y + 'px', width: widgets.actions.width + 'px', zIndex: widgets.actions.z }"
        >
          <div @mousedown="startDrag($event, 'actions')" class="p-3 bg-rose-500/10 border-b border-rose-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-rose-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <Wrench class="w-3.5 h-3.5" /> Quick Actions
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="toggleWidget('actions')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3 flex flex-col gap-1.5 font-mono text-xs">
            <button @click="runQuickTool('ai_fps')" class="p-2 bg-white/5 hover:bg-white/10 rounded-lg text-left text-white text-[10px] font-bold cursor-pointer">Auto-Tune FPS</button>
            <button @click="runQuickTool('clean_logs')" class="p-2 bg-white/5 hover:bg-white/10 rounded-lg text-left text-white text-[10px] font-bold cursor-pointer">Clear Logs</button>
            <button @click="runQuickTool('flush_dns')" class="p-2 bg-white/5 hover:bg-white/10 rounded-lg text-left text-white text-[10px] font-bold cursor-pointer">Flush DNS</button>
            <button @click="runQuickTool('kill_java')" class="p-2 bg-rose-500/10 hover:bg-rose-500/20 text-rose-300 rounded-lg text-left text-[10px] font-bold cursor-pointer">Kill Zombie JVMs</button>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div
          v-if="shouldRenderWidget('notes')"
          @mousedown="bringToFront('notes')"
          class="absolute flex flex-col bg-[#08080c]/90 backdrop-blur-2xl border border-yellow-500/40 rounded-2xl shadow-2xl overflow-hidden select-none"
          :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none opacity-90'"
          :style="{ left: widgets.notes.x + 'px', top: widgets.notes.y + 'px', width: widgets.notes.width + 'px', zIndex: widgets.notes.z }"
        >
          <div @mousedown="startDrag($event, 'notes')" class="p-3 bg-yellow-500/10 border-b border-yellow-500/30 flex justify-between items-center cursor-move">
            <span class="text-xs font-black text-yellow-400 uppercase tracking-widest flex items-center gap-2 font-mono">
              <StickyNote class="w-3.5 h-3.5" /> Scratchpad
            </span>
            <div class="flex items-center gap-1" v-if="isInteractive">
              <button @click.stop="togglePinWidget('notes')" class="p-1 rounded hover:bg-white/10 cursor-pointer" :class="widgets.notes.pinned ? 'text-yellow-400' : 'text-white/40'">
                <Pin class="w-3 h-3" :class="widgets.notes.pinned ? 'fill-current' : ''" />
              </button>
              <button @click.stop="toggleWidget('notes')" class="p-1 rounded hover:bg-white/10 text-white/40 cursor-pointer"><X class="w-3 h-3" /></button>
            </div>
          </div>

          <div class="p-3 h-36 flex flex-col">
            <textarea
              v-model="notesText"
              @input="debouncedSaveNote"
              class="flex-1 bg-transparent border-none resize-none focus:outline-none text-xs text-white/80 font-mono custom-scroll leading-relaxed"
              :class="isInteractive ? 'pointer-events-auto' : 'pointer-events-none'"
              placeholder="Jot down notes, trades or seeds..."
            ></textarea>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>