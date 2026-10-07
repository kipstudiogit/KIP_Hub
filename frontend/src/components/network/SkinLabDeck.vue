<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { User, Box, Loader, Download } from 'lucide-vue-next'
import { SkinViewer, WalkingAnimation, RunningAnimation, IdleAnimation } from 'skinview3d'
import { showToast } from '@/store'
import type { SkinAnimationType } from '../../types/network'

const netNick = ref('')
const isSkinLoading = ref(false)
const hasSkinLoaded = ref(false)
const skinContainer = ref<HTMLElement | null>(null)
const skinCanvas = ref<HTMLCanvasElement | null>(null)
const skinAnim = ref<SkinAnimationType>('idle')

let skinViewerInstance: SkinViewer | null = null
let resizeObserver: ResizeObserver | null = null

async function loadSkin3D(): Promise<void> {
  const nickname = netNick.value.trim()
  if (!nickname || !skinContainer.value || !skinCanvas.value) return
  isSkinLoading.value = true

  const width = skinContainer.value.clientWidth || 350
  const height = skinContainer.value.clientHeight || 400
  const url = `https://mc-heads.net/skin/${encodeURIComponent(nickname)}`

  try {
    if (!skinViewerInstance) {
      skinViewerInstance = new SkinViewer({
        canvas: skinCanvas.value,
        width,
        height,
        skin: url,
      })
      skinViewerInstance.fov = 70
      skinViewerInstance.zoom = 0.9
      skinViewerInstance.autoRotate = true
      skinViewerInstance.autoRotateSpeed = 0.5

      if (skinViewerInstance.controls) {
        skinViewerInstance.controls.enableRotate = true
        skinViewerInstance.controls.enableZoom = true
        skinViewerInstance.controls.enablePan = false
      }
      skinViewerInstance.animation = new IdleAnimation()
      skinAnim.value = 'idle'
    } else {
      await skinViewerInstance.loadSkin(url)
    }
    hasSkinLoaded.value = true
  } catch {
    showToast('Skin Error', 'Failed to load skin for this player handle.', 'danger')
  } finally {
    isSkinLoading.value = false
  }
}

function setAnimation(type: SkinAnimationType): void {
  if (!skinViewerInstance) return
  skinAnim.value = type
  if (type === 'walk') {
    skinViewerInstance.animation = new WalkingAnimation()
  } else if (type === 'run') {
    skinViewerInstance.animation = new RunningAnimation()
  } else {
    skinViewerInstance.animation = new IdleAnimation()
  }
}

function downloadSkinTexture(): void {
  if (!netNick.value.trim()) return
  const link = document.createElement('a')
  link.download = `${netNick.value.trim()}_skin.png`
  link.href = `https://mc-heads.net/skin/${encodeURIComponent(netNick.value.trim())}`
  link.click()
}

onMounted(() => {
  resizeObserver = new ResizeObserver(() => {
    if (skinViewerInstance && skinContainer.value) {
      const w = skinContainer.value.clientWidth || 350
      const h = skinContainer.value.clientHeight || 400
      skinViewerInstance.setSize(w, h)
    }
  })

  if (skinContainer.value) {
    resizeObserver.observe(skinContainer.value)
  }
})

onBeforeUnmount(() => {
  if (resizeObserver) {
    resizeObserver.disconnect()
    resizeObserver = null
  }
  if (skinViewerInstance) {
    skinViewerInstance.dispose()
    skinViewerInstance = null
  }
})
</script>

<template>
  <div class="kip-card p-8 flex flex-col border border-white/5 hover:border-purple-500/30 transition relative overflow-hidden min-h-[500px] bg-black/40">
    <div class="absolute -top-32 -right-32 w-96 h-96 bg-purple-500/10 blur-[120px] rounded-full pointer-events-none"></div>

    <div class="flex justify-between items-center mb-6 relative z-20">
      <div>
        <h3 class="text-xl font-black uppercase text-white flex items-center gap-2">
          <User class="w-5 h-5 text-purple-400" /> Holographic 3D Skin Lab
        </h3>
        <p class="text-xs text-white/40 mt-1">Real-time WebGL Minecraft player model renderer</p>
      </div>

      <div class="flex gap-3">
        <input v-model="netNick" @keyup.enter="loadSkin3D" type="text" placeholder="Minecraft Nickname" class="kip-input py-2 text-xs font-mono w-48">
        <button @click="loadSkin3D" :disabled="isSkinLoading" class="kip-btn-primary px-6 bg-purple-500 hover:bg-purple-400 text-white font-black text-xs uppercase shadow-[0_0_20px_rgba(168,85,247,0.3)]">
          <Loader v-if="isSkinLoading" class="w-4 h-4 animate-spin" />
          <span v-else>Project</span>
        </button>
        <button v-if="hasSkinLoaded" @click="downloadSkinTexture" class="kip-btn-ghost px-4 py-2 text-xs text-purple-400 border-purple-500/30 hover:bg-purple-500/10" title="Export Skin PNG">
          <Download class="w-4 h-4" />
        </button>
      </div>
    </div>

    <div ref="skinContainer" class="flex-1 w-full flex justify-center items-center relative z-10 min-h-[350px]">
      <div v-show="!hasSkinLoaded" class="text-white/30 flex flex-col items-center gap-3">
        <Box class="w-12 h-12 opacity-30" />
        <span class="text-xs font-mono">Enter Minecraft nickname above to generate hologram</span>
      </div>
      <canvas ref="skinCanvas" class="drop-shadow-[0_20px_40px_rgba(0,0,0,0.9)] transition-opacity duration-500" :class="hasSkinLoaded ? 'opacity-100' : 'opacity-0'"></canvas>
    </div>

    <div v-if="hasSkinLoaded" class="flex justify-center gap-2 mt-4 relative z-20">
      <button @click="setAnimation('idle')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'idle' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Idle</button>
      <button @click="setAnimation('walk')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'walk' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Walk</button>
      <button @click="setAnimation('run')" class="px-5 py-2 rounded-xl text-xs font-bold transition" :class="skinAnim === 'run' ? 'bg-purple-500 text-white shadow-[0_0_15px_rgba(168,85,247,0.4)]' : 'text-white/40 hover:text-white bg-white/5'">Run</button>
    </div>
  </div>
</template>