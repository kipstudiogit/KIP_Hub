<script setup lang="ts">
import { ref } from 'vue'
import {
  Magnet,
  X,
  Download,
  Loader,
} from 'lucide-vue-next'
import type { SwarmStatusDto } from '../../types/content'
import { bridge } from '@/bridge'
import { showToast } from '../../composables/useToasts'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

defineProps<{
  activeSeeds: SwarmStatusDto[]
}>()

const magnetUri = ref('')
const isLoading = ref(false)

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

async function startDownload(): Promise<void> {
  const clean = magnetUri.value.trim()
  if (!clean) return
  isLoading.value = true

  try {
    const res = await bridge.swarmDownload(clean, 'MODS_DIR')
    if (res && res.success) {
      showToast(t('Swarm'), 'P2P background download started...', 'success')
      isOpen.value = false
      magnetUri.value = ''
    } else {
      showToast(t('Error'), res?.msg || 'P2P download initialization failed.', 'danger')
    }
  } catch (err: unknown) {
    showToast(t('Error'), String(err), 'danger')
  } finally {
    isLoading.value = false
  }
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="kip-card p-8 w-full max-w-xl border-amber-500/30 flex flex-col relative overflow-hidden shadow-[0_0_60px_rgba(245,158,11,0.25)]">
        <header class="flex items-center gap-4 mb-4 relative z-10">
          <div class="p-3 bg-amber-500/20 border border-amber-500/40 rounded-2xl text-amber-400">
            <Magnet class="w-6 h-6" />
          </div>
          <div>
            <h3 class="text-2xl font-black uppercase text-white tracking-wider">Swarm P2P Acceleration</h3>
            <p class="text-xs text-white/40 font-mono">BitTorrent Mesh Ingestion & Peer-to-Peer Synchronization</p>
          </div>
        </header>

        <div class="flex flex-col gap-4 relative z-10 my-4">
          <label class="text-[10px] font-mono uppercase text-white/40 font-bold block">BitTorrent Magnet URI</label>
          <input
            v-model="magnetUri"
            type="text"
            placeholder="magnet:?xt=urn:btih:..."
            class="kip-input font-mono text-xs text-amber-400 focus:border-amber-500"
          >

          <div v-if="activeSeeds.length > 0" class="flex flex-col gap-2 p-3 bg-black/40 rounded-xl border border-white/5">
            <span class="text-[9px] font-mono uppercase text-amber-400 font-bold">Active P2P Seeding Torrents:</span>
            <div v-for="seed in activeSeeds" :key="seed.name" class="flex justify-between items-center text-xs font-mono">
              <span class="text-white/80 truncate max-w-xs">{{ seed.name }}</span>
              <span class="text-amber-400">{{ formatBytes(seed.total_upload) }} uploaded</span>
            </div>
          </div>
        </div>

        <footer class="flex justify-end gap-3 relative z-10 pt-2 border-t border-white/5">
          <button @click="isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">
            {{ t('Cancel') }}
          </button>
          <button
            @click="startDownload"
            :disabled="!magnetUri.trim() || isLoading"
            class="kip-btn-primary px-8 py-2.5 bg-amber-500 hover:bg-amber-400 text-black border-amber-400 font-black text-xs uppercase tracking-wider"
          >
            <Loader v-if="isLoading" class="w-4 h-4 animate-spin" />
            <Download v-else class="w-4 h-4 fill-current" />
            <span>Initiate P2P Ingestion</span>
          </button>
        </footer>
      </div>
    </div>
  </transition>
</template>