<script setup lang="ts">
import {
  Activity,
  Radio,
  Play,
  Square,
  Loader,
  Copy,
  Flame,
  Globe2,
  Share2,
} from 'lucide-vue-next'
import { t, showToast, sanitizeHTML, voiceState, broadcastLobbyEndpoint } from '@/store'
import type { ServerPingResultDto, DiscoveredLanServerDto } from '../../types/network'

defineProps<{
  netIp: string
  isPinging: boolean
  pingResult: ServerPingResultDto | null
  recentPings: string[]
  tunnelPort: string
  activeTunnelEndpoint: string
  discoveredLanServers: DiscoveredLanServerDto[]
}>()

const emit = defineEmits<{
  (e: 'update:netIp', value: string): void
  (e: 'update:tunnelPort', value: string): void
  (e: 'ping', target?: string): void
  (e: 'start-tunnel', port?: string): void
  (e: 'stop-tunnel'): void
}>()

const fallbackServerIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"/><rect x="2" y="14" width="20" height="8" rx="2" ry="2"/><line x1="6" y1="6" x2="6.01" y2="6"/><line x1="6" y1="18" x2="6.01" y2="18"/></svg>'

function getPingColorClass(ping: number): string {
  if (ping < 45) return 'border-emerald-500/40 text-emerald-300 bg-emerald-500/15'
  if (ping < 110) return 'border-cyan-500/40 text-cyan-300 bg-cyan-500/15'
  if (ping < 180) return 'border-amber-500/40 text-amber-300 bg-amber-500/15'
  return 'border-rose-500/40 text-rose-300 bg-rose-500/15'
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
  showToast(t('Copied'), 'Address copied to clipboard!', 'success')
}

function broadcastTunnelToLobby(endpoint: string): void {
  broadcastLobbyEndpoint(endpoint)
  showToast(t('Broadcasted'), 'Public address transmitted to active voice lobby.', 'success')
}
</script>

<template>
  <div class="flex flex-col gap-6">
    <!-- Auto-Discovered Local LAN Beacon Strip -->
    <div
      v-if="discoveredLanServers.length > 0"
      class="kip-card p-5 border border-emerald-500/40 bg-gradient-to-r from-emerald-950/30 to-black/60 shadow-xl flex items-center justify-between"
    >
      <div class="flex items-center gap-3.5">
        <div class="p-3 bg-emerald-500/20 text-emerald-400 rounded-2xl border border-emerald-500/40 animate-pulse">
          <Globe2 class="w-6 h-6" />
        </div>
        <div>
          <span class="text-[9px] font-mono uppercase font-black tracking-widest text-emerald-400 block">
            Local Multicast Beacon (224.0.2.60:4445)
          </span>
          <h4 class="text-base font-black text-white">
            Detected Minecraft World: {{ discoveredLanServers[0].motd }}
          </h4>
          <p class="text-xs text-white/50 font-mono mt-0.5">
            Host: {{ discoveredLanServers[0].address }}:{{ discoveredLanServers[0].port }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2.5">
        <button
          @click="copyAddress(`${discoveredLanServers[0].address}:${discoveredLanServers[0].port}`)"
          class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase text-white/80 border-white/10 flex items-center gap-1.5 cursor-pointer"
        >
          <Copy class="w-3.5 h-3.5" />
          <span>Copy LAN IP</span>
        </button>

        <button
          @click="emit('start-tunnel', String(discoveredLanServers[0].port))"
          class="kip-btn-primary px-5 py-2 text-xs font-black uppercase tracking-wider bg-emerald-500 hover:bg-emerald-400 text-black flex items-center gap-1.5 cursor-pointer shadow-[0_0_15px_rgba(16,185,129,0.3)]"
        >
          <Share2 class="w-3.5 h-3.5" />
          <span>1-Click Public Tunnel</span>
        </button>
      </div>
    </div>

    <div class="grid grid-cols-2 gap-6">
      <!-- Enhanced Server Radar Card -->
      <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-indigo-500/30 transition bg-black/40">
        <div class="flex justify-between items-start mb-6">
          <div>
            <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
              <Activity class="w-5 h-5 text-indigo-400" /> Server Radar Pro
            </h3>
            <p class="text-xs text-white/40 mt-1">Full SLP Stream Frame Ingestion & DNS SRV Resolution</p>
          </div>
        </div>

        <div class="flex gap-3 mb-3">
          <input
            :value="netIp"
            @input="emit('update:netIp', ($event.target as HTMLInputElement).value)"
            @keyup.enter="emit('ping')"
            type="text"
            placeholder="mc.hypixel.net, 2b2t.org..."
            class="kip-input font-mono text-xs"
          >
          <button
            @click="emit('ping')"
            :disabled="isPinging"
            class="kip-btn-primary px-6 min-w-[120px] text-xs font-black uppercase cursor-pointer"
          >
            <Loader v-if="isPinging" class="w-4 h-4 animate-spin" />
            <span v-else>Inspect</span>
          </button>
        </div>

        <div v-if="recentPings.length > 0" class="flex gap-1.5 mb-6 flex-wrap">
          <button
            v-for="s in recentPings"
            :key="s"
            @click="emit('ping', s)"
            class="px-2.5 py-0.5 rounded text-[10px] font-mono bg-white/5 hover:bg-white/10 text-white/60 hover:text-white border border-white/5 cursor-pointer transition"
          >
            {{ s }}
          </button>
        </div>

        <div v-if="pingResult" class="bg-black/60 p-5 rounded-2xl border border-white/10 flex flex-col gap-4 mt-auto shadow-inner">
          <div class="flex items-center gap-4">
            <img
              :src="pingResult.icon || fallbackServerIcon"
              class="w-14 h-14 rounded-xl bg-black/60 p-1 object-contain border border-white/10 shadow-lg"
            >
            <div class="flex-1 min-w-0">
              <div class="flex justify-between items-center mb-1">
                <span class="text-xs font-black uppercase tracking-wider text-emerald-400 flex items-center gap-1.5">
                  <span class="w-2 h-2 rounded-full bg-emerald-400 animate-pulse"></span> Online
                </span>
                <span class="text-xs font-mono font-bold px-2 py-0.5 rounded border" :class="getPingColorClass(pingResult.ping || 0)">
                  {{ pingResult.ping ?? 0 }} ms
                </span>
              </div>
              <p class="text-xs text-white/70 font-mono truncate" v-html="sanitizeHTML(pingResult.motd || '')"></p>
            </div>
          </div>

          <div>
            <div class="flex justify-between text-[10px] font-mono uppercase text-white/40 mb-1.5 font-bold">
              <span>Players Online</span>
              <span class="text-white font-bold">{{ pingResult.players || '0/0' }}</span>
            </div>
            <div class="w-full h-1.5 bg-black/60 rounded-full overflow-hidden border border-white/5">
              <div
                class="h-full bg-indigo-500 rounded-full shadow-[0_0_10px_rgba(99,102,241,0.8)]"
                :style="{ width: calculatePlayerPercentage(pingResult.players || '0/0') + '%' }"
              ></div>
            </div>
          </div>
        </div>
      </div>

      <!-- LAN Reverse Tunnel Card -->
      <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-amber-500/30 transition bg-black/40 justify-between">
        <div>
          <div class="flex justify-between items-start mb-6">
            <div>
              <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
                <Radio class="w-5 h-5 text-amber-400" /> LAN Reverse Tunnel Bridge
              </h3>
              <p class="text-xs text-white/40 mt-1">Expose local worlds to friends worldwide with zero port-forwarding</p>
            </div>
          </div>

          <div class="flex flex-col gap-4">
            <div class="flex gap-3">
              <input
                :value="tunnelPort"
                @input="emit('update:tunnelPort', ($event.target as HTMLInputElement).value)"
                type="text"
                placeholder="25565"
                class="kip-input font-mono w-1/3 text-center text-xs"
              >
              <button
                @click="emit('start-tunnel')"
                class="kip-btn-primary flex-1 bg-amber-500 hover:bg-amber-400 text-black font-black uppercase text-xs shadow-[0_0_20px_rgba(245,158,11,0.3)] flex items-center justify-center gap-2 cursor-pointer"
              >
                <Play class="w-4 h-4 fill-current" /> Open Bridge
              </button>
              <button
                @click="emit('stop-tunnel')"
                class="kip-btn-danger px-5 bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white border border-red-500/30 cursor-pointer"
              >
                <Square class="w-4 h-4 fill-current" />
              </button>
            </div>

            <div v-if="activeTunnelEndpoint" class="bg-black/60 p-5 rounded-2xl border border-amber-500/30 flex flex-col gap-3 shadow-inner">
              <span class="text-[9px] font-mono uppercase text-amber-400 font-bold tracking-widest">
                Public Gateway Address for Friends:
              </span>
              <div class="flex items-center justify-between bg-black/80 px-4 py-3 rounded-xl border border-white/10">
                <span class="font-mono text-sm text-white font-bold select-all">{{ activeTunnelEndpoint }}</span>
                <button @click="copyAddress(activeTunnelEndpoint)" class="kip-btn-ghost px-3 py-1.5 text-xs text-amber-400 border-amber-500/30 hover:bg-amber-500/10 cursor-pointer">
                  <Copy class="w-3.5 h-3.5" /> Copy IP
                </button>
              </div>
              <button
                v-if="voiceState.isConnected"
                @click="broadcastTunnelToLobby(activeTunnelEndpoint)"
                class="w-full py-2 bg-indigo-500/20 hover:bg-indigo-500/30 border border-indigo-500/40 text-indigo-300 font-mono text-xs rounded-xl font-bold uppercase transition cursor-pointer"
              >
                Broadcast IP to Active Voice Lobby
              </button>
            </div>
          </div>
        </div>

        <div class="text-[10px] font-mono text-white/40 pt-4 border-t border-white/5">
          Encrypted TLS reverse proxy stream • Powered by Pinggy / TCP FastPath
        </div>
      </div>
    </div>
  </div>
</template>