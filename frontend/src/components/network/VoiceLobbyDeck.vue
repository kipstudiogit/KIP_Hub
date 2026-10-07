<script setup lang="ts">
import { ref } from 'vue'
import {
  Users,
  Copy,
  Settings,
  Mic,
  MicOff,
  Headphones,
  PhoneOff,
  Globe,
  Radio,
  Hash,
  Dices,
  PhoneCall,
  Volume2,
} from 'lucide-vue-next'
import {
  state,
  t,
  showToast,
  getAvatarUrl,
  voiceState,
  toggleMute,
  toggleDeafen,
  leaveVoiceChannel,
  joinVoiceChannel,
  toggleVoiceSettings,
  setAudioInput,
  setAudioOutput,
  startMicTest,
  stopMicTest,
} from '@/store'

const networkMode = ref<'global' | 'lan'>('global')
const connectChannel = ref('')

function generateRandomRoom(): string {
  return 'kip-' + Math.random().toString(36).substring(2, 6)
}

function handleJoinVoice(): void {
  if (!connectChannel.value) {
    connectChannel.value = generateRandomRoom()
  }
  const cleanInput = connectChannel.value.trim().toLowerCase()
  if (networkMode.value === 'lan') {
    if (cleanInput.includes(':')) {
      const parts = cleanInput.split('/')
      const hostPart = parts[0] || '127.0.0.1:8765'
      const roomPart = parts[1] || 'lan-room'
      joinVoiceChannel(roomPart, hostPart)
    } else {
      joinVoiceChannel(cleanInput, '127.0.0.1:8765')
    }
  } else {
    joinVoiceChannel(cleanInput, 'wss://kip-backend.noisyfutlor98.workers.dev/ws')
  }
}

async function copyLobbyCode(): Promise<void> {
  if (!voiceState.channelId) return
  await navigator.clipboard.writeText(voiceState.channelId.toUpperCase())
  showToast(t('Copied'), 'Lobby Code copied to clipboard!', 'success')
}

async function copyHostAddress(): Promise<void> {
  if (!voiceState.activeHostAddress) return
  await navigator.clipboard.writeText(voiceState.activeHostAddress)
  showToast(t('Copied'), 'Minecraft Host Address copied!', 'success')
}

function toggleMicTestHandler(): void {
  if (voiceState.isTestingMic) {
    stopMicTest()
  } else {
    startMicTest()
  }
}
</script>

<template>
  <div class="kip-card p-8 relative overflow-hidden flex flex-col border border-white/5 bg-black/40">
    <div class="absolute -right-20 -top-20 w-80 h-80 bg-emerald-500/10 blur-[110px] rounded-full pointer-events-none"></div>

    <!-- Header bar -->
    <div class="flex justify-between items-center mb-6 relative z-10">
      <div class="flex items-center gap-3">
        <div class="p-3 rounded-2xl border" :class="voiceState.isConnected ? 'bg-emerald-500/20 border-emerald-500/40 text-emerald-400' : 'bg-white/5 border-white/10 text-white/40'">
          <Users class="w-6 h-6" :class="voiceState.isConnected ? 'animate-pulse' : ''" />
        </div>
        <div>
          <h3 class="text-xl font-black text-white uppercase tracking-wider">K.I.P. Connect • Multiplayer Hub</h3>
          <span class="text-[9px] font-mono text-white/40 uppercase tracking-widest">
            P2P Mesh Topology • WebRTC Web Audio DSP Matrix • Adaptive Acoustic Gate
          </span>
        </div>
      </div>

      <div v-if="voiceState.isConnected" class="flex items-center gap-3">
        <button @click="copyLobbyCode" class="flex items-center gap-2 bg-emerald-500/20 hover:bg-emerald-500/30 border border-emerald-500/40 px-4 py-2 rounded-xl font-mono text-xs text-emerald-300 font-bold transition">
          <Copy class="w-3.5 h-3.5" />
          <span>CODE: {{ voiceState.channelId.toUpperCase() }}</span>
        </button>
        <button @click="toggleVoiceSettings" class="kip-btn-ghost p-2.5 border-emerald-500/20 text-emerald-400 hover:bg-emerald-500/10">
          <Settings class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Host sync bar -->
    <div v-if="voiceState.activeHostAddress && voiceState.isConnected" class="mb-6 bg-indigo-500/10 border border-indigo-500/30 p-4 rounded-xl flex items-center justify-between relative z-10 shadow-inner">
      <div class="flex items-center gap-3">
        <Radio class="w-5 h-5 text-indigo-400 animate-pulse" />
        <div>
          <span class="text-[9px] font-mono uppercase text-indigo-300 font-bold block">Synchronized Host Address:</span>
          <span class="text-sm font-mono font-black text-white select-all">{{ voiceState.activeHostAddress }}</span>
        </div>
      </div>
      <button @click="copyHostAddress" class="kip-btn-primary px-4 py-2 text-xs font-bold uppercase tracking-wider bg-indigo-500 hover:bg-indigo-400">
        <Copy class="w-3.5 h-3.5" /> Copy Server IP
      </button>
    </div>

    <!-- Connect view (Disconnected) -->
    <div v-if="!voiceState.isConnected" class="flex flex-col gap-5 max-w-2xl relative z-10">
      <p class="text-xs text-white/60 leading-relaxed font-medium">
        Join your team's multiplayer lobby using their Code, or initialize a room. Peers synchronize voice channels and Minecraft server IPs automatically.
      </p>

      <div class="flex gap-2 p-1 bg-black/40 rounded-xl border border-white/10 w-fit">
        <button @click="networkMode = 'global'" class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2" :class="networkMode === 'global' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'text-white/40 hover:text-white'">
          <Globe class="w-3.5 h-3.5" /> Global Cloud (Edge)
        </button>
        <button @click="networkMode = 'lan'" class="px-4 py-2 rounded-lg text-xs font-bold transition flex items-center gap-2" :class="networkMode === 'lan' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'">
          <Radio class="w-3.5 h-3.5" /> Local Network (LAN)
        </button>
      </div>

      <div class="flex gap-3">
        <div class="relative flex-1">
          <Hash class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
          <input v-model="connectChannel" @keyup.enter="handleJoinVoice" type="text" :placeholder="networkMode === 'global' ? t('Enter Lobby Code (e.g. survival or KIP-99)') : t('Local Host or IP:Port (e.g. 192.168.1.5:8765)')" class="kip-input pl-11 pr-12 font-mono uppercase">
          <button @click="connectChannel = generateRandomRoom()" class="absolute right-2 top-1/2 -translate-y-1/2 p-2 text-white/40 hover:text-emerald-400 transition" :title="t('Generate key')">
            <Dices class="w-4 h-4" />
          </button>
        </div>
        <button @click="handleJoinVoice" class="kip-btn-primary px-8 py-3 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(16,185,129,0.3)]">
          <PhoneCall class="w-4 h-4 fill-current" /> Connect Lobby
        </button>
      </div>
    </div>

    <!-- Active Connected Roster -->
    <div v-else class="flex flex-col gap-6 relative z-10">
      <!-- Expanded Audio Device Settings -->
      <transition name="fade">
        <div v-if="voiceState.showSettings" class="bg-black/60 border border-white/10 rounded-2xl p-6 flex flex-col gap-5 shadow-2xl">
          <div class="grid grid-cols-2 gap-6">
            <div>
              <label class="text-[10px] font-black text-white/40 uppercase tracking-widest mb-2 block">Audio Input Device</label>
              <select v-model="voiceState.selectedInputId" @change="setAudioInput(voiceState.selectedInputId)" class="kip-input py-2.5 text-xs">
                <option value="default">Default Recording Device</option>
                <option v-for="d in voiceState.inputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Microphone ' + d.deviceId.substring(0,4) }}</option>
              </select>
            </div>
            <div>
              <label class="text-[10px] font-black text-white/40 uppercase tracking-widest mb-2 block">Audio Output Device</label>
              <select v-model="voiceState.selectedOutputId" @change="setAudioOutput(voiceState.selectedOutputId)" class="kip-input py-2.5 text-xs">
                <option value="default">Default Playback Device</option>
                <option v-for="d in voiceState.outputDevices" :key="d.deviceId" :value="d.deviceId">{{ d.label || 'Speaker ' + d.deviceId.substring(0,4) }}</option>
              </select>
            </div>
          </div>

          <div class="pt-4 border-t border-white/5 flex items-center justify-between gap-5">
            <button @click="toggleMicTestHandler" :disabled="voiceState.isListenOnly" class="px-5 py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition border disabled:opacity-30" :class="voiceState.isTestingMic ? 'bg-red-500/10 text-red-400 border-red-500/30' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30'">
              {{ voiceState.isTestingMic ? 'Stop Test' : 'Test Loopback' }}
            </button>
            <div class="flex-1 h-2 bg-black/60 rounded-full overflow-hidden border border-white/5 relative">
              <div class="absolute top-0 left-0 h-full bg-emerald-400 transition-all duration-75 shadow-[0_0_10px_rgba(52,211,153,0.8)]" :style="{ width: voiceState.testMicVolume + '%' }"></div>
            </div>
          </div>
        </div>
      </transition>

      <!-- Participants Grid -->
      <div class="grid grid-cols-4 gap-4">
        <div class="bg-black/50 border rounded-2xl p-4 flex flex-col justify-between transition-all duration-300" :class="voiceState.localSpeaking ? 'border-emerald-500/60 bg-emerald-500/10 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'border-white/5'">
          <div class="flex items-center gap-3">
            <img :src="getAvatarUrl(state.settings.ms_name || state.settings.offline_username || 'Guest')" class="w-10 h-10 rounded-xl object-cover border border-white/10">
            <div class="flex-1 min-w-0">
              <h4 class="font-black text-sm text-white truncate">{{ state.settings.ms_name || state.settings.offline_username || 'Guest' }} (You)</h4>
              <span class="text-[9px] font-mono text-emerald-400 font-bold uppercase">
                {{ voiceState.isListenOnly ? 'Spectator' : voiceState.localSpeaking ? 'Transmitting' : 'Idle' }}
              </span>
            </div>
          </div>

          <div class="flex items-center justify-between mt-4 pt-3 border-t border-white/5">
            <div class="flex gap-1.5">
              <button @click="toggleMute" :disabled="voiceState.isListenOnly" class="p-2 rounded-lg transition disabled:opacity-40" :class="voiceState.isMuted ? 'bg-red-500/20 text-red-400' : 'bg-white/5 text-white/60 hover:text-white'">
                <MicOff v-if="voiceState.isMuted" class="w-3.5 h-3.5" />
                <Mic v-else class="w-3.5 h-3.5" />
              </button>
              <button @click="toggleDeafen" class="p-2 rounded-lg transition" :class="voiceState.isDeafened ? 'bg-red-500/20 text-red-400' : 'bg-white/5 text-white/60 hover:text-white'">
                <Headphones class="w-3.5 h-3.5" />
              </button>
            </div>
            <span class="text-[9px] font-mono text-white/30 uppercase">Local Operator</span>
          </div>
        </div>

        <div v-for="p in voiceState.participants" :key="p.id" class="bg-black/50 border rounded-2xl p-4 flex flex-col justify-between transition-all duration-300" :class="p.speaking ? 'border-emerald-500/60 bg-emerald-500/10 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'border-white/5'">
          <div class="flex items-center gap-3">
            <img :src="getAvatarUrl(p.name)" class="w-10 h-10 rounded-xl object-cover border border-white/10">
            <div class="flex-1 min-w-0">
              <h4 class="font-black text-sm text-white truncate">{{ p.name }}</h4>
              <span class="text-[9px] font-mono font-bold uppercase" :class="p.speaking ? 'text-emerald-400' : 'text-white/40'">{{ p.speaking ? 'Speaking' : 'Listening' }}</span>
            </div>
          </div>

          <div class="flex items-center justify-between mt-4 pt-3 border-t border-white/5">
            <div class="flex gap-1.5">
              <MicOff v-if="p.muted" class="w-3.5 h-3.5 text-red-400" />
              <Headphones v-if="p.deafened" class="w-3.5 h-3.5 text-amber-400" />
              <span v-if="!p.muted && !p.deafened" class="w-2 h-2 rounded-full bg-emerald-400"></span>
            </div>
            <span class="text-[9px] font-mono text-white/30 uppercase">Remote Peer</span>
          </div>
        </div>
      </div>

      <div class="flex justify-end pt-2">
        <button @click="leaveVoiceChannel" class="kip-btn-danger px-6 py-2.5 text-xs uppercase font-black tracking-wider">
          <PhoneOff class="w-4 h-4" /> Disconnect Lobby
        </button>
      </div>
    </div>
  </div>
</template>