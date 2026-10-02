<template>
  <div class="fixed inset-0 z-[9999] bg-black/40 backdrop-blur-sm flex flex-col pointer-events-auto font-sans text-white transition-all duration-500 overflow-hidden">
    <div class="absolute top-6 left-1/2 -translate-x-1/2 z-[10000] bg-black/60 backdrop-blur-2xl border border-white/10 p-2 rounded-full flex items-center gap-2 shadow-[0_0_30px_rgba(0,0,0,0.5)]">
      <div class="flex items-center gap-3 px-4 mr-2">
        <Hexagon class="w-5 h-5 text-indigo-400 animate-pulse" />
        <span class="font-extrabold tracking-widest text-sm">K.I.P. <span class="text-indigo-400">OS</span></span>
      </div>
      <div class="w-px h-6 bg-white/10 mx-1"></div>

      <button @click="toggleWidget('voice')" class="p-3 rounded-full transition-all duration-300 relative group" :class="widgets.voice.show ? 'bg-emerald-500/20 text-emerald-400' : 'hover:bg-white/10 text-white/50 hover:text-white'">
        <Mic class="w-5 h-5" />
        <span v-if="voiceState.isConnected" class="absolute top-0 right-0 w-2.5 h-2.5 bg-emerald-400 rounded-full animate-pulse"></span>
      </button>

      <button @click="toggleWidget('ai')" class="p-3 rounded-full transition-all duration-300 relative group" :class="widgets.ai.show ? 'bg-purple-500/20 text-purple-400' : 'hover:bg-white/10 text-white/50 hover:text-white'">
        <BrainCircuit class="w-5 h-5" />
      </button>

      <button @click="toggleWidget('telemetry')" class="p-3 rounded-full transition-all duration-300 relative group" :class="widgets.telemetry.show ? 'bg-blue-500/20 text-blue-400' : 'hover:bg-white/10 text-white/50 hover:text-white'">
        <Activity class="w-5 h-5" />
      </button>

      <button @click="toggleWidget('actions')" class="p-3 rounded-full transition-all duration-300 relative group" :class="widgets.actions.show ? 'bg-amber-500/20 text-amber-400' : 'hover:bg-white/10 text-white/50 hover:text-white'">
        <Zap class="w-5 h-5" />
      </button>

      <button @click="toggleWidget('notes')" class="p-3 rounded-full transition-all duration-300 relative group" :class="widgets.notes.show ? 'bg-pink-500/20 text-pink-400' : 'hover:bg-white/10 text-white/50 hover:text-white'">
        <StickyNote class="w-5 h-5" />
      </button>

      <div class="w-px h-6 bg-white/10 mx-1"></div>

      <div class="px-4 font-mono text-sm font-bold text-white/80 tracking-widest">
        {{ currentTime }}
      </div>

      <div class="w-px h-6 bg-white/10 mx-1"></div>

      <button @click="closeOverlay" class="p-3 rounded-full hover:bg-red-500 hover:text-white text-red-400 transition-all duration-300 bg-red-500/10">
        <X class="w-5 h-5" />
      </button>
    </div>

    <transition name="fade">
      <div v-if="widgets.voice.show" @mousedown="bringToFront('voice')" class="absolute w-80 flex flex-col bg-black/70 backdrop-blur-xl border border-emerald-500/30 rounded-2xl shadow-2xl overflow-hidden" :style="{ left: widgets.voice.x + 'px', top: widgets.voice.y + 'px', zIndex: widgets.voice.z }">
        <div @mousedown="startDrag($event, 'voice')" class="p-3 bg-gradient-to-r from-emerald-500/20 to-transparent border-b border-emerald-500/30 flex justify-between items-center cursor-move">
          <span class="text-xs font-bold text-emerald-400 uppercase tracking-widest flex items-center gap-2">
            <Mic class="w-3 h-3" /> K.I.P. Connect
          </span>
          <button @click.stop="toggleWidget('voice')" class="text-white/50 hover:text-red-400 transition"><X class="w-4 h-4" /></button>
        </div>

        <div class="p-4 flex flex-col gap-4">
          <div v-if="!voiceState.isConnected" class="flex flex-col gap-3">
            <div class="relative">
              <Hash class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
              <input v-model="connectChannel" @keyup.enter="handleJoinVoice" type="text" :placeholder="t('Room ID...')" class="w-full bg-black/40 border border-white/10 rounded-xl pl-9 pr-10 py-2.5 text-sm text-white focus:outline-none focus:border-emerald-500 transition font-mono">
              <button @click="connectChannel = generateRandomRoom()" class="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-white/40 hover:text-emerald-400 transition rounded-lg hover:bg-white/5">
                <Dices class="w-4 h-4" />
              </button>
            </div>
            <button @click="handleJoinVoice" class="w-full py-2.5 bg-emerald-500 hover:bg-emerald-400 text-black font-extrabold rounded-xl transition flex justify-center items-center gap-2 uppercase text-xs">
              <PhoneCall class="w-4 h-4" /> {{ connectChannel ? t('Join Room') : t('Quick Start') }}
            </button>
          </div>

          <div v-else class="flex flex-col gap-3">
            <div class="flex justify-between items-center bg-emerald-500/10 border border-emerald-500/30 p-2.5 rounded-xl">
              <div class="flex items-center gap-2">
                <span class="w-2 h-2 bg-emerald-400 rounded-full animate-pulse"></span>
                <span class="text-xs font-mono font-bold text-white truncate max-w-[120px]">{{ voiceState.channelId }}</span>
              </div>
              <button @click="toggleVoiceSettings" class="text-emerald-400/70 hover:text-emerald-400 p-1.5 rounded-md hover:bg-emerald-500/20 transition">
                <Settings class="w-4 h-4" />
              </button>
            </div>

            <div v-if="voiceState.showSettings" class="bg-black/40 border border-white/10 rounded-xl p-3 flex flex-col gap-3">
              <select v-model="voiceState.selectedInputId" @change="setAudioInput(voiceState.selectedInputId)" class="w-full bg-black/40 border border-white/10 rounded-lg px-2 py-1.5 text-xs text-white">
                <option value="default">Default Mic</option>
                <option v-for="d in voiceState.inputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Mic ' + d.deviceId.substring(0,4) }}</option>
              </select>
              <select v-model="voiceState.selectedOutputId" @change="setAudioOutput(voiceState.selectedOutputId)" class="w-full bg-black/40 border border-white/10 rounded-lg px-2 py-1.5 text-xs text-white">
                <option value="default">Default Speaker</option>
                <option v-for="d in voiceState.outputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Speaker ' + d.deviceId.substring(0,4) }}</option>
              </select>
              <div class="flex items-center gap-2">
                <button @click="toggleMicTest" class="px-3 py-1.5 rounded-md text-[9px] font-bold uppercase border flex-shrink-0" :class="voiceState.isTestingMic ? 'bg-red-500/10 text-red-400 border-red-500/30' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'">
                  {{ voiceState.isTestingMic ? 'Stop' : 'Test' }}
                </button>
                <div class="flex-1 h-1.5 bg-black/60 rounded-full overflow-hidden border border-white/5 relative">
                  <div class="absolute top-0 left-0 h-full bg-emerald-400 transition-all duration-75" :style="{ width: voiceState.testMicVolume + '%' }"></div>
                </div>
              </div>
            </div>

            <div class="flex flex-col gap-2 max-h-48 overflow-y-auto custom-scroll pr-1">
              <div class="flex items-center gap-2 bg-black/40 border border-white/5 p-2 rounded-xl" :class="voiceState.localSpeaking ? 'border-emerald-500/50 bg-emerald-500/10' : ''">
                <img :src="getAvatarUrl(state.settings.ms_name || state.settings.offline_username || 'Guest')" class="w-8 h-8 rounded-md object-cover" :class="voiceState.localSpeaking ? 'border border-emerald-400' : 'opacity-70'">
                <div class="flex-1 min-w-0">
                  <span class="text-xs font-bold truncate block" :class="voiceState.localSpeaking ? 'text-emerald-400' : 'text-white'">{{ state.settings.ms_name || state.settings.offline_username || 'Guest' }}</span>
                </div>
                <div class="flex gap-1 shrink-0">
                  <MicOff v-if="voiceState.isMuted" class="w-3.5 h-3.5 text-red-400" />
                  <Headphones v-if="voiceState.isDeafened" class="w-3.5 h-3.5 text-amber-400" />
                </div>
              </div>
              <div v-for="p in voiceState.participants" :key="p.id" class="flex items-center gap-2 bg-black/40 border border-white/5 p-2 rounded-xl" :class="p.speaking ? 'border-emerald-500/50 bg-emerald-500/10' : ''">
                <img :src="getAvatarUrl(p.name)" class="w-8 h-8 rounded-md object-cover" :class="p.speaking ? 'border border-emerald-400' : 'opacity-70'">
                <span class="text-xs font-bold truncate flex-1" :class="p.speaking ? 'text-emerald-400' : 'text-white/80'">{{ p.name }}</span>
                <div class="flex gap-1 shrink-0">
                  <MicOff v-if="p.muted" class="w-3.5 h-3.5 text-red-400" />
                  <Headphones v-if="p.deafened" class="w-3.5 h-3.5 text-amber-400" />
                </div>
              </div>
            </div>

            <div class="flex gap-2 pt-2 border-t border-white/10">
              <button @click="toggleMute" class="flex-1 py-2 rounded-xl flex justify-center items-center transition border" :class="voiceState.isMuted ? 'bg-red-500/10 text-red-400 border-red-500/30' : 'bg-white/5 border-white/10 hover:bg-white/10 text-white'">
                <MicOff v-if="voiceState.isMuted" class="w-4 h-4" />
                <Mic v-else class="w-4 h-4" />
              </button>
              <button @click="toggleDeafen" class="flex-1 py-2 rounded-xl flex justify-center items-center transition border" :class="voiceState.isDeafened ? 'bg-amber-500/10 text-amber-400 border-amber-500/30' : 'bg-white/5 border-white/10 hover:bg-white/10 text-white'">
                <Headphones class="w-4 h-4" />
              </button>
              <button @click="leaveVoiceChannel" class="flex-1 py-2 bg-red-500 hover:bg-red-400 text-white rounded-xl flex justify-center items-center transition border border-red-400">
                <PhoneOff class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="widgets.ai.show" @mousedown="bringToFront('ai')" class="absolute w-96 flex flex-col bg-black/70 backdrop-blur-xl border border-purple-500/30 rounded-2xl shadow-2xl overflow-hidden" :style="{ left: widgets.ai.x + 'px', top: widgets.ai.y + 'px', zIndex: widgets.ai.z }">
        <div @mousedown="startDrag($event, 'ai')" class="p-3 bg-gradient-to-r from-purple-500/20 to-transparent border-b border-purple-500/30 flex justify-between items-center cursor-move">
          <span class="text-xs font-bold text-purple-400 uppercase tracking-widest flex items-center gap-2">
            <BrainCircuit class="w-3 h-3" /> Neural Assistant
          </span>
          <button @click.stop="toggleWidget('ai')" class="text-white/50 hover:text-red-400 transition"><X class="w-4 h-4" /></button>
        </div>

        <div class="flex flex-col h-96">
          <div ref="chatScroll" class="flex-1 overflow-y-auto custom-scroll p-4 flex flex-col gap-4">
            <div v-for="(msg, idx) in chatHistory" :key="idx" class="flex w-full" :class="msg.role === 'user' ? 'justify-end' : 'justify-start'">
              <div class="max-w-[90%] p-3 rounded-2xl text-xs leading-relaxed border shadow-xl" :class="msg.role === 'user' ? 'bg-purple-600/90 border-purple-400/50 text-white rounded-br-sm' : 'bg-black/60 border-white/10 text-white/90 rounded-bl-sm'">
                <div class="flex items-center gap-1.5 mb-1" :class="msg.role === 'user' ? 'text-purple-200' : 'text-purple-400'">
                  <User v-if="msg.role === 'user'" class="w-3 h-3" />
                  <BrainCircuit v-else class="w-3 h-3" />
                  <span class="text-[9px] font-bold uppercase tracking-wider">{{ msg.role === 'user' ? 'You' : 'K.I.P.' }}</span>
                </div>
                <div class="markdown-body text-xs break-words" v-html="msg.content"></div>
              </div>
            </div>
          </div>

          <div class="p-3 border-t border-white/10 bg-black/40">
            <div class="flex gap-2 bg-black/60 p-1.5 rounded-xl border border-white/10">
              <input v-model="aiPrompt" @keyup.enter="sendToAi" type="text" placeholder="Ask anything..." class="flex-1 bg-transparent px-3 py-1.5 focus:outline-none font-medium text-xs text-white placeholder-white/30">
              <button @click="sendToAi" :disabled="isAiThinking" class="px-4 py-2 bg-purple-500 hover:bg-purple-400 text-white rounded-lg transition disabled:opacity-50 flex items-center justify-center">
                <Loader v-if="isAiThinking" class="w-4 h-4 animate-spin" />
                <Send v-else class="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="widgets.telemetry.show" @mousedown="bringToFront('telemetry')" class="absolute w-64 flex flex-col bg-black/70 backdrop-blur-xl border border-blue-500/30 rounded-2xl shadow-2xl overflow-hidden" :style="{ left: widgets.telemetry.x + 'px', top: widgets.telemetry.y + 'px', zIndex: widgets.telemetry.z }">
        <div @mousedown="startDrag($event, 'telemetry')" class="p-3 bg-gradient-to-r from-blue-500/20 to-transparent border-b border-blue-500/30 flex justify-between items-center cursor-move">
          <span class="text-xs font-bold text-blue-400 uppercase tracking-widest flex items-center gap-2">
            <Activity class="w-3 h-3" /> Telemetry
          </span>
          <button @click.stop="toggleWidget('telemetry')" class="text-white/50 hover:text-red-400 transition"><X class="w-4 h-4" /></button>
        </div>

        <div class="p-4 flex flex-col gap-5">
          <div>
            <div class="flex justify-between items-end mb-2">
              <span class="text-[9px] font-bold text-white/50 uppercase tracking-widest flex items-center gap-1.5"><Cpu class="w-3 h-3" /> RAM</span>
              <span class="text-xs font-mono font-bold text-white">{{ memoryUsage }}%</span>
            </div>
            <div class="w-full h-1.5 bg-black/60 border border-white/5 rounded-full overflow-hidden">
              <div class="h-full bg-blue-500 shadow-[0_0_10px_rgba(59,130,246,0.8)] transition-all duration-1000 ease-out" :style="{ width: memoryUsage + '%' }"></div>
            </div>
          </div>

          <div class="flex justify-between items-center">
            <span class="text-[9px] font-bold text-white/50 uppercase tracking-widest flex items-center gap-1.5"><Clock class="w-3 h-3" /> Session</span>
            <span class="text-xs font-mono font-bold text-blue-400 bg-blue-500/10 px-2.5 py-1 rounded-lg border border-blue-500/20">{{ realPlaytime }}</span>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="widgets.actions.show" @mousedown="bringToFront('actions')" class="absolute w-56 flex flex-col bg-black/70 backdrop-blur-xl border border-amber-500/30 rounded-2xl shadow-2xl overflow-hidden" :style="{ left: widgets.actions.x + 'px', top: widgets.actions.y + 'px', zIndex: widgets.actions.z }">
        <div @mousedown="startDrag($event, 'actions')" class="p-3 bg-gradient-to-r from-amber-500/20 to-transparent border-b border-amber-500/30 flex justify-between items-center cursor-move">
          <span class="text-xs font-bold text-amber-400 uppercase tracking-widest flex items-center gap-2">
            <Zap class="w-3 h-3" /> Actions
          </span>
          <button @click.stop="toggleWidget('actions')" class="text-white/50 hover:text-red-400 transition"><X class="w-4 h-4" /></button>
        </div>

        <div class="p-4 flex flex-col gap-2">
          <button @click="runQuickTool('ai_fps')" class="w-full py-2.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-xs font-bold transition flex items-center justify-between px-3 group">
            <span class="text-white/80 group-hover:text-white">Optimize FPS</span>
            <Cpu class="w-3 h-3 text-amber-400" />
          </button>
          <button @click="runQuickTool('clean_logs')" class="w-full py-2.5 bg-white/5 hover:bg-white/10 border border-white/10 rounded-xl text-xs font-bold transition flex items-center justify-between px-3 group">
            <span class="text-white/80 group-hover:text-white">Clear Logs</span>
            <Trash2 class="w-3 h-3 text-emerald-400" />
          </button>
          <button @click="runQuickTool('kill_java')" class="w-full py-2.5 bg-red-500/10 hover:bg-red-500/20 border border-red-500/30 rounded-xl text-xs font-bold transition flex items-center justify-between px-3 text-red-400">
            <span>Force Stop</span>
            <Skull class="w-3 h-3" />
          </button>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="widgets.notes.show" @mousedown="bringToFront('notes')" class="absolute w-72 flex flex-col bg-black/70 backdrop-blur-xl border border-pink-500/30 rounded-2xl shadow-2xl overflow-hidden" :style="{ left: widgets.notes.x + 'px', top: widgets.notes.y + 'px', zIndex: widgets.notes.z }">
        <div @mousedown="startDrag($event, 'notes')" class="p-3 bg-gradient-to-r from-pink-500/20 to-transparent border-b border-pink-500/30 flex justify-between items-center cursor-move">
          <span class="text-xs font-bold text-pink-400 uppercase tracking-widest flex items-center gap-2">
            <StickyNote class="w-3 h-3" /> Scratchpad
          </span>
          <button @click.stop="toggleWidget('notes')" class="text-white/50 hover:text-red-400 transition"><X class="w-4 h-4" /></button>
        </div>

        <div class="p-2 h-48 flex flex-col">
          <textarea v-model="notesText" @input="debouncedSaveNote" class="flex-1 bg-transparent border-none resize-none focus:outline-none text-sm text-white/80 p-2 custom-scroll" placeholder="Write coordinates or to-do..."></textarea>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import {
  Hexagon,
  X,
  Activity,
  BrainCircuit,
  Send,
  Loader,
  Mic,
  MicOff,
  Headphones,
  PhoneOff,
  Cpu,
  Clock,
  Trash2,
  Skull,
  Zap,
  User,
  Hash,
  Dices,
  PhoneCall,
  Settings,
  StickyNote,
} from 'lucide-vue-next'
import {
  state,
  getAvatarUrl,
  sanitizeHTML,
  voiceState,
  toggleMute,
  toggleDeafen,
  leaveVoiceChannel,
  joinVoiceChannel,
  showToast,
  t,
  toggleVoiceSettings,
  setAudioInput,
  setAudioOutput,
  startMicTest,
  stopMicTest,
} from '@/store'
import { invokeSafe, bridge, type DashboardStats, type ToolExecutionResult } from '@/bridge'
import { marked } from 'marked'

interface WidgetPosition {
  show: boolean
  x: number
  y: number
  z: number
}

type WidgetKey = 'voice' | 'ai' | 'telemetry' | 'actions' | 'notes'

interface ChatMessage {
  role: 'user' | 'ai'
  content: string
}

const currentTime = ref<string>('')
const memoryUsage = ref<number>(0)
const aiPrompt = ref<string>('')
const isAiThinking = ref<boolean>(false)
const chatScroll = ref<HTMLElement | null>(null)
const realPlaytime = ref<string>('0h 0m')
const connectChannel = ref<string>('')
const notesText = ref<string>('')
let noteSaveTimeout: ReturnType<typeof setTimeout> | null = null

const chatHistory = ref<ChatMessage[]>([
  { role: 'ai', content: 'Neural link established. Awaiting input for game analysis or assistance.' },
])

const zIndexCounter = ref<number>(10000)

const widgets = ref<Record<WidgetKey, WidgetPosition>>({
  voice: { show: true, x: 40, y: 100, z: 10001 },
  ai: { show: false, x: 380, y: 100, z: 10002 },
  telemetry: { show: true, x: 40, y: 450, z: 10003 },
  actions: { show: false, x: 380, y: 450, z: 10004 },
  notes: { show: false, x: 700, y: 100, z: 10005 },
})

let activeDrag: WidgetKey | null = null
let startX = 0
let startY = 0

const toggleWidget = (id: WidgetKey): void => {
  widgets.value[id].show = !widgets.value[id].show
  if (widgets.value[id].show) bringToFront(id)
}

const bringToFront = (id: WidgetKey): void => {
  zIndexCounter.value++
  widgets.value[id].z = zIndexCounter.value
}

const onDrag = (e: MouseEvent): void => {
  if (!activeDrag) return
  widgets.value[activeDrag].x = e.clientX - startX
  widgets.value[activeDrag].y = e.clientY - startY
}

const stopDrag = (): void => {
  activeDrag = null
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
}

const startDrag = (e: MouseEvent, id: WidgetKey): void => {
  activeDrag = id
  bringToFront(id)
  startX = e.clientX - widgets.value[id].x
  startY = e.clientY - widgets.value[id].y
  document.addEventListener('mousemove', onDrag)
  document.addEventListener('mouseup', stopDrag)
}

let timeInterval: ReturnType<typeof setInterval> | null = null
let statsInterval: ReturnType<typeof setInterval> | null = null

const generateRandomRoom = (): string => {
  return 'kip-' + Math.random().toString(36).substring(2, 6)
}

const handleJoinVoice = (): void => {
  if (!connectChannel.value) {
    connectChannel.value = generateRandomRoom()
  }
  joinVoiceChannel(connectChannel.value)
}

const toggleMicTest = (): void => {
  if (voiceState.isTestingMic) {
    stopMicTest()
  } else {
    startMicTest()
  }
}

const scrollToBottom = (): void => {
  nextTick(() => {
    if (chatScroll.value) {
      chatScroll.value.scrollTop = chatScroll.value.scrollHeight
    }
  })
}

const updateTime = (): void => {
  currentTime.value = new Date().toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
}

const updateStats = async (): Promise<void> => {
  try {
    const info = await invokeSafe<string>('get_sys_info')
    const ramLine = info.split('\n').find((l) => l.startsWith('RAM:'))
    if (ramLine) {
      const totalGb = parseFloat(ramLine.replace('RAM:', '').replace('GB', '').trim())
      if (!isNaN(totalGb) && totalGb > 0) {
        const simulatedLoad = Math.min(Math.max(Math.round((4.0 / totalGb) * 100), 20), 95)
        memoryUsage.value = simulatedLoad
      }
    }

    const s = await invokeSafe<DashboardStats>('get_dashboard_stats')
    if (s && s.playtime) realPlaytime.value = s.playtime
  } catch {
    // Retains previous telemetry
  }
}

const closeOverlay = (): void => {
  try {
    bridge.toggleOverlay().catch(() => {})
  } catch {
    // Ignored
  }
}

const runQuickTool = async (toolId: string): Promise<void> => {
  try {
    const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId })
    if (res && res.success) {
      showToast('Action Executed', res.msg, 'success')
    } else {
      showToast('Action Failed', res?.msg || 'Execution failed', 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast('Action Failed', msg, 'danger')
  }
}

const sendToAi = async (): Promise<void> => {
  const userText = aiPrompt.value.trim()
  if (!userText || isAiThinking.value) return

  chatHistory.value.push({ role: 'user', content: sanitizeHTML(userText) })
  aiPrompt.value = ''
  isAiThinking.value = true
  scrollToBottom()

  try {
    const prompt = `You are an in-game HUD assistant for a Minecraft player. Keep your answer brief, highly technical but helpful, and use markdown. Player asks: ${userText}`
    const res = await invokeSafe<{ success: boolean; answer: string }>('analyze_crash_ai', {
      logSnippet: prompt,
    })

    if (res && res.success) {
      chatHistory.value.push({ role: 'ai', content: marked.parse(res.answer) as string })
    } else {
      chatHistory.value.push({
        role: 'ai',
        content: `<span class="text-red-400">Error connecting to Neural Network: ${sanitizeHTML(res?.answer || 'Service unavailable')}</span>`,
      })
    }
  } catch {
    chatHistory.value.push({
      role: 'ai',
      content: `<span class="text-red-400">Critical Backend Failure.</span>`,
    })
  } finally {
    isAiThinking.value = false
    scrollToBottom()
  }
}

const loadNote = async (): Promise<void> => {
  try {
    const text = await invokeSafe<string>('get_note')
    notesText.value = text || ''
  } catch {
    notesText.value = ''
  }
}

const debouncedSaveNote = (): void => {
  if (noteSaveTimeout) clearTimeout(noteSaveTimeout)
  noteSaveTimeout = setTimeout(async () => {
    try {
      await invokeSafe<boolean>('save_note', { text: notesText.value })
    } catch {
      // Ignored
    }
  }, 1000)
}

onMounted(() => {
  updateTime()
  updateStats()
  loadNote()
  timeInterval = setInterval(updateTime, 1000)
  statsInterval = setInterval(updateStats, 5000)
  scrollToBottom()
})

onBeforeUnmount(() => {
  if (timeInterval) clearInterval(timeInterval)
  if (statsInterval) clearInterval(statsInterval)
  if (noteSaveTimeout) clearTimeout(noteSaveTimeout)
  document.removeEventListener('mousemove', onDrag)
  document.removeEventListener('mouseup', stopDrag)
})
</script>

<style scoped>
.markdown-body :deep(p) { margin-bottom: 0.5em; }
.markdown-body :deep(p:last-child) { margin-bottom: 0; }
.markdown-body :deep(code) { background: rgba(255,255,255,0.1); padding: 2px 4px; border-radius: 4px; font-family: monospace; font-size: 0.9em; }
.markdown-body :deep(pre) { background: rgba(0,0,0,0.5); padding: 10px; border-radius: 8px; margin: 10px 0; border: 1px solid rgba(255,255,255,0.1); overflow-x: auto; }
.markdown-body :deep(ul) { list-style-type: disc; padding-left: 20px; margin-bottom: 0.5em; }
.markdown-body :deep(ol) { list-style-type: decimal; padding-left: 20px; margin-bottom: 0.5em; }
</style>