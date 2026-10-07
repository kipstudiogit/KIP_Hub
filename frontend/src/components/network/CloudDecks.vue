<script setup lang="ts">
import {
  ServerCog,
  Container,
  Loader,
} from 'lucide-vue-next'
import type { PteroServerDto, DockerCoreType } from '../../types/network'

defineProps<{
  pteroUrl: string
  pteroKey: string
  pteroServers: PteroServerDto[]
  selectedPteroServerId: string
  isPteroLoading: boolean
  dockerCore: DockerCoreType
  dockerVer: string
  dockerPort: string
  isDockerDeploying: boolean
}>()

const emit = defineEmits<{
  (e: 'update:pteroUrl', value: string): void
  (e: 'update:pteroKey', value: string): void
  (e: 'update:selectedPteroServerId', value: string): void
  (e: 'update:dockerCore', value: DockerCoreType): void
  (e: 'update:dockerVer', value: string): void
  (e: 'update:dockerPort', value: string): void
  (e: 'connect-ptero'): void
  (e: 'ptero-action', action: 'start' | 'restart' | 'kill'): void
  (e: 'deploy-docker'): void
}>()
</script>

<template>
  <div class="grid grid-cols-2 gap-6">
    <!-- Pterodactyl Panel Link -->
    <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-blue-500/30 transition bg-black/40">
      <h3 class="text-xl font-black uppercase text-white flex items-center gap-2 mb-2">
        <ServerCog class="w-5 h-5 text-blue-400" /> Pterodactyl Panel Link
      </h3>
      <p class="text-xs text-white/40 mb-6">Manage remote dedicated gaming servers via official client API</p>

      <div class="flex flex-col gap-4">
        <input
          :value="pteroUrl"
          @input="emit('update:pteroUrl', ($event.target as HTMLInputElement).value)"
          type="text"
          placeholder="https://panel.example.com"
          class="kip-input"
        >
        <div class="flex gap-3">
          <input
            :value="pteroKey"
            @input="emit('update:pteroKey', ($event.target as HTMLInputElement).value)"
            type="password"
            placeholder="Client API Key"
            class="kip-input font-mono flex-1"
          >
          <button @click="emit('connect-ptero')" :disabled="isPteroLoading" class="kip-btn-primary px-6 bg-blue-500 hover:bg-blue-400 text-white font-black uppercase text-xs shadow-[0_0_20px_rgba(59,130,246,0.3)]">
            <Loader v-if="isPteroLoading" class="w-4 h-4 animate-spin" />
            <span v-else>Link</span>
          </button>
        </div>

        <div v-if="pteroServers.length > 0" class="flex flex-col gap-3 mt-4 pt-4 border-t border-white/5">
          <select
            :value="selectedPteroServerId"
            @change="emit('update:selectedPteroServerId', ($event.target as HTMLSelectElement).value)"
            class="kip-input text-xs"
          >
            <option v-for="s in pteroServers" :key="s.id" :value="s.id">{{ s.name }} ({{ s.state }})</option>
          </select>

          <div class="flex items-center justify-between bg-black/60 p-4 rounded-xl border border-white/5">
            <span class="text-xs font-mono font-bold uppercase text-white/80">Active Control Node</span>
            <div class="flex gap-2">
              <button @click="emit('ptero-action', 'start')" class="kip-btn-ghost px-4 py-1.5 text-xs text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10">Start</button>
              <button @click="emit('ptero-action', 'restart')" class="kip-btn-ghost px-4 py-1.5 text-xs text-amber-400 border-amber-500/20 hover:bg-amber-500/10">Restart</button>
              <button @click="emit('ptero-action', 'kill')" class="kip-btn-ghost px-4 py-1.5 text-xs text-red-400 border-red-500/20 hover:bg-red-500/10">Kill</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 1-Click Docker Container Instance -->
    <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-cyan-500/30 transition bg-black/40">
      <h3 class="text-xl font-black uppercase text-white flex items-center gap-2 mb-2">
        <Container class="w-5 h-5 text-cyan-400" /> 1-Click Docker Instance
      </h3>
      <p class="text-xs text-white/40 mb-6">Deploy isolated sandboxed containerized server on your local machine</p>

      <div class="flex flex-col gap-4">
        <div class="grid grid-cols-3 gap-3">
          <select
            :value="dockerCore"
            @change="emit('update:dockerCore', ($event.target as HTMLSelectElement).value as DockerCoreType)"
            class="kip-input text-xs font-bold uppercase"
          >
            <option value="paper">PaperMC</option>
            <option value="fabric">Fabric</option>
            <option value="forge">Forge</option>
            <option value="vanilla">Vanilla</option>
          </select>
          <input
            :value="dockerVer"
            @input="emit('update:dockerVer', ($event.target as HTMLInputElement).value)"
            type="text"
            placeholder="1.21.1"
            class="kip-input font-mono text-center text-xs"
          >
          <input
            :value="dockerPort"
            @input="emit('update:dockerPort', ($event.target as HTMLInputElement).value)"
            type="text"
            placeholder="25565"
            class="kip-input font-mono text-center text-xs"
          >
        </div>

        <button @click="emit('deploy-docker')" :disabled="isDockerDeploying" class="kip-btn-primary py-3.5 bg-cyan-500 hover:bg-cyan-400 text-black font-black uppercase text-xs shadow-[0_0_20px_rgba(6,182,212,0.3)]">
          <Loader v-if="isDockerDeploying" class="w-4 h-4 animate-spin" />
          <span v-else>Deploy Docker Stack</span>
        </button>
      </div>
    </div>
  </div>
</template>