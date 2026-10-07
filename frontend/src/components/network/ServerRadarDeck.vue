<script setup lang="ts">
import {
  Activity,
  Radio,
  Play,
  Square,
  Loader,
  Copy,
} from 'lucide-vue-next'
import { t, showToast, sanitizeHTML, voiceState, broadcastLobbyEndpoint } from '@/store'
import type { ServerPingResultDto } from '../../types/network'

defineProps<{
  netIp: string
  isPinging: boolean
  pingResult: ServerPingResultDto | null
  recentPings: string[]
  tunnelPort: string
  activeTunnelEndpoint: string
}>()

const emit = defineEmits<{
  (e: 'update:netIp', value: string): void
  (e: 'update:tunnelPort', value: string): void
  (e: 'ping', target?: string): void
  (e: 'start-tunnel'): void
  (e: 'stop-tunnel'): void
}>()

const fallbackServerIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>'

function getPingColorClass(ping: number): string {
  if (ping < 50) return 'border-emerald-500/30 text-emerald-400 bg-emerald-500/10'
  if (ping < 150) return 'border-amber-500/30 text-amber-400 bg-amber-500/10'
  return 'border-red-500/30 text-red-400 bg-red-500/10'
}

function calculatePlayerPercentage(playersStr: string): number {
  try {
    const [online, max] = playersStr.split('/').map(Number)
    if (!max || max === 0 || Number.isNaN(online) || Number.isNaN(max)) return 0
    return Math.min(Math.round((online / max) * 100), 100)
  } catch {
    return 0
  }
}

async function copyAddress(addr: string): Promise<void> {
  await navigator.clipboard.writeText(addr)
  showToast(t('Copied'), 'Server address copied to clipboard!', 'success')
}

function broadcastTunnelToLobby(endpoint: string): void {
  broadcastLobbyEndpoint(endpoint)
  showToast(t('Broadcasted'), 'Server address transmitted to active lobby.', 'success')
}
</script>

<template>
  <div class="grid grid-cols-2 gap-6">
    <!-- Server Radar Card -->
    <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-indigo-500/30 transition bg-black/40">
      <div class="flex justify-between items-start mb-6">
        <div>
          <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
            <Activity class="w-5 h-5 text-indigo-400" /> Server Radar
          </h3>
          <p class="text-xs text-white/40 mt-1">Real-time latency check and MOTD diagnostic</p>
        </div>
      </div>

      <div class="flex gap-3 mb-4">
        <input
          :value="netIp"
          @input="emit('update:netIp', ($event.target as HTMLInputElement).value)"
          @keyup.enter="emit('ping')"
          type="text"
          placeholder="mc.hypixel.net"
          class="kip-input font-mono"
        >
        <button @click="emit('ping')" :disabled="isPinging" class="kip-btn-primary px-6 min-w-[110px] text-xs font-black uppercase">
          <Loader v-if="isPinging" class="w-4 h-4 animate-spin" />
          <span v-else>Inspect</span>
        </button>
      </div>

      <!-- Quick Server Recents -->
      <div v-if="recentPings.length > 0" class="flex gap-1.5 mb-6 flex-wrap">
        <button
          v-for="s in recentPings"
          :key="s"
          @click="emit('ping', s)"
          class="px-2.5 py-0.5 rounded text-[10px] font-mono bg-white/5 hover:bg-white/10 text-white/60 hover:text-white border border-white/5"
        >
          {{ s }}
        </button>
      </div>

      <div v-if="pingResult" class="bg-black/50 p-5 rounded-2xl border border-white/5 flex flex-col gap-4 mt-auto">
        <div class="flex items-center gap-4">
          <img :src="pingResult.icon || fallbackServerIcon" class="w-14 h-14 rounded-xl bg-black/60 p-1 object-contain border border-white/10">
          <div class="flex-1 min-w-0">
            <div class="flex justify-between items-center mb-1">
              <span class="text-xs font-black uppercase tracking-wider text-emerald-400 flex items-center gap-1.5">
                <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span> Online
              </span>
              <span class="text-xs font-mono font-bold px-2 py-0.5 rounded border" :class="getPingColorClass(pingResult.ping || 0)">
                {{ pingResult.ping ?? 0 }} ms
              </span>
            </div>
            <p class="text-xs text-white/60 truncate font-mono" v-html="sanitizeHTML(pingResult.motd || '')"></p>
          </div>
        </div>

        <div>
          <div class="flex justify-between text-[10px] font-mono uppercase text-white/40 mb-1.5 font-bold">
            <span>Players Online</span>
            <span>{{ pingResult.players || '0/0' }}</span>
          </div>
          <div class="w-full h-1.5 bg-black/60 rounded-full overflow-hidden border border-white/5">
            <div class="h-full bg-indigo-500 rounded-full shadow-[0_0_10px_rgba(99,102,241,0.8)]" :style="{ width: calculatePlayerPercentage(pingResult.players || '0/0') + '%' }"></div>
          </div>
        </div>
      </div>
    </div>

    <!-- LAN Reverse Tunnel Card -->
    <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-amber-500/30 transition bg-black/40">
      <div class="flex justify-between items-start mb-6">
        <div>
          <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
            <Radio class="w-5 h-5 text-amber-400" /> LAN Reverse Tunnel
          </h3>
          <p class="text-xs text-white/40 mt-1">Expose singleplayer LAN game to friends via encrypted TCP bridge</p>
        </div>
      </div>

      <div class="flex flex-col gap-4">
        <div class="flex gap-3">
          <input
            :value="tunnelPort"
            @input="emit('update:tunnelPort', ($event.target as HTMLInputElement).value)"
            type="text"
            placeholder="25565"
            class="kip-input font-mono w-1/3 text-center"
          >
          <button @click="emit('start-tunnel')" class="kip-btn-primary flex-1 bg-amber-500 hover:bg-amber-400 text-black font-black uppercase text-xs shadow-[0_0_20px_rgba(245,158,11,0.3)] flex items-center justify-center gap-2">
            <Play class="w-4 h-4 fill-current" /> Open Tunnel
          </button>
          <button @click="emit('stop-tunnel')" class="kip-btn-danger px-5 bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white border border-red-500/30">
            <Square class="w-4 h-4 fill-current" />
          </button>
        </div>

        <div v-if="activeTunnelEndpoint" class="bg-black/60 p-5 rounded-2xl border border-amber-500/30 flex flex-col gap-3 shadow-inner">
          <span class="text-[9px] font-mono uppercase text-amber-400 font-bold tracking-widest">Live Tunnel Public Address:</span>
          <div class="flex items-center justify-between bg-black/80 px-4 py-3 rounded-xl border border-white/10">
            <span class="font-mono text-sm text-white font-bold select-all">{{ activeTunnelEndpoint }}</span>
            <button @click="copyAddress(activeTunnelEndpoint)" class="kip-btn-ghost px-3 py-1.5 text-xs text-amber-400 border-amber-500/30 hover:bg-amber-500/10">
              <Copy class="w-3.5 h-3.5" /> Copy IP
            </button>
          </div>
          <button
            v-if="voiceState.isConnected"
            @click="broadcastTunnelToLobby(activeTunnelEndpoint)"
            class="w-full py-2 bg-indigo-500/20 hover:bg-indigo-500/30 border border-indigo-500/40 text-indigo-300 font-mono text-xs rounded-xl font-bold uppercase transition"
          >
            Broadcast IP to Active Lobby
          </button>
        </div>
      </div>
    </div>
  </div>
</template>