<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import {
  Mic,
  Activity,
  ServerCog,
  User,
  Zap,
} from 'lucide-vue-next'
import { t } from '@/store'
import type { NetworkSubTab } from '../types/network'
import { useNetworkManager } from '../composables/useNetworkManager'
import VoiceLobbyDeck from '../components/network/VoiceLobbyDeck.vue'
import ServerRadarDeck from '../components/network/ServerRadarDeck.vue'
import CloudDecks from '../components/network/CloudDecks.vue'
import SkinLabDeck from '../components/network/SkinLabDeck.vue'
import PingMasterDeck from '../components/network/PingMasterDeck.vue'

const activeSubTab = ref<NetworkSubTab | 'booster'>('booster')

const {
  netIp,
  isPinging,
  pingResult,
  recentPings,
  tunnelPort,
  activeTunnelEndpoint,
  pteroUrl,
  pteroKey,
  pteroServers,
  selectedPteroServerId,
  isPteroLoading,
  dockerCore,
  dockerVer,
  dockerPort,
  isDockerDeploying,
  pingTargetServer,
  startReverseTunnel,
  stopReverseTunnel,
  connectPterodactyl,
  sendPteroAction,
  deployDocker,
  setupListeners,
  cleanup,
} = useNetworkManager()

onMounted(async () => {
  await setupListeners()
})

onBeforeUnmount(() => {
  cleanup()
})
</script>

<template>
  <div class="h-full flex flex-col overflow-hidden select-none relative">
    <header class="flex justify-between items-center mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Network & Identity') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">High-Speed Relay, Server Radar & 3D Identity Lab</p>
      </div>

      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          @click="activeSubTab = 'booster'"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="activeSubTab === 'booster' ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/30 shadow-[0_0_20px_rgba(6,182,212,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <Zap class="w-3.5 h-3.5" /> Hardware Booster
        </button>
        <button
          @click="activeSubTab = 'voice'"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="activeSubTab === 'voice' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <Mic class="w-3.5 h-3.5" /> Lobby & Voice
        </button>
        <button
          @click="activeSubTab = 'radar'"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="activeSubTab === 'radar' ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-[0_0_20px_rgba(99,102,241,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <Activity class="w-3.5 h-3.5" /> Radar & Tunnel
        </button>
        <button
          @click="activeSubTab = 'remote'"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="activeSubTab === 'remote' ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30 shadow-[0_0_20px_rgba(245,158,11,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <ServerCog class="w-3.5 h-3.5" /> Cloud Decks
        </button>
        <button
          @click="activeSubTab = 'skin'"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2 cursor-pointer"
          :class="activeSubTab === 'skin' ? 'bg-purple-500/20 text-purple-400 border border-purple-500/30 shadow-[0_0_20px_rgba(168,85,247,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <User class="w-3.5 h-3.5" /> 3D Skin Lab
        </button>
      </div>
    </header>

    <main class="flex-1 overflow-y-auto custom-scroll pr-2 pb-6 min-h-0 z-10">
      <PingMasterDeck v-show="activeSubTab === 'booster'" />

      <VoiceLobbyDeck v-show="activeSubTab === 'voice'" />

      <ServerRadarDeck
        v-show="activeSubTab === 'radar'"
        :net-ip="netIp"
        :is-pinging="isPinging"
        :ping-result="pingResult"
        :recent-pings="recentPings"
        :tunnel-port="tunnelPort"
        :active-tunnel-endpoint="activeTunnelEndpoint"
        @update:net-ip="netIp = $event"
        @update:tunnel-port="tunnelPort = $event"
        @ping="pingTargetServer($event)"
        @start-tunnel="startReverseTunnel"
        @stop-tunnel="stopReverseTunnel"
      />

      <CloudDecks
        v-show="activeSubTab === 'remote'"
        :ptero-url="pteroUrl"
        :ptero-key="pteroKey"
        :ptero-servers="pteroServers"
        :selected-ptero-server-id="selectedPteroServerId"
        :is-ptero-loading="isPteroLoading"
        :docker-core="dockerCore"
        :docker-ver="dockerVer"
        :docker-port="dockerPort"
        :is-docker-deploying="isDockerDeploying"
        @update:ptero-url="pteroUrl = $event"
        @update:ptero-key="pteroKey = $event"
        @update:selected-ptero-server-id="selectedPteroServerId = $event"
        @update:docker-core="dockerCore = $event"
        @update:docker-ver="dockerVer = $event"
        @update:docker-port="dockerPort = $event"
        @connect-ptero="connectPterodactyl"
        @ptero-action="sendPteroAction"
        @deploy-docker="deployDocker"
      />

      <SkinLabDeck v-show="activeSubTab === 'skin'" />
    </main>
  </div>
</template>