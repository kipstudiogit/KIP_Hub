<script setup lang="ts">
import { ref } from 'vue'
import {
  Download,
  X,
  Earth,
  Map,
  ZoomIn,
  ZoomOut,
  RotateCcw,
  Crosshair,
  Loader,
} from 'lucide-vue-next'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

const props = defineProps<{
  worldName: string
  image: string
  spawnX: number
  spawnZ: number
  loading: boolean
  radius: number
}>()

const emit = defineEmits<{
  (e: 'change-radius', radius: number): void
}>()

const zoomLevel = ref(1.0)
const panX = ref(0)
const panY = ref(0)
const isDragging = ref(false)
const startX = ref(0)
const startY = ref(0)

function handleWheel(e: WheelEvent): void {
  e.preventDefault()
  if (e.deltaY < 0) {
    zoomLevel.value = Math.min(zoomLevel.value + 0.25, 4.0)
  } else {
    zoomLevel.value = Math.max(zoomLevel.value - 0.25, 0.5)
  }
}

function startPan(e: MouseEvent): void {
  isDragging.value = true
  startX.value = e.clientX - panX.value
  startY.value = e.clientY - panY.value
}

function onPan(e: MouseEvent): void {
  if (!isDragging.value) return
  panX.value = e.clientX - startX.value
  panY.value = e.clientY - startY.value
}

function stopPan(): void {
  isDragging.value = false
}

function resetTransform(): void {
  zoomLevel.value = 1.0
  panX.value = 0
  panY.value = 0
}

function downloadMap(): void {
  if (!props.image) return
  const link = document.createElement('a')
  link.download = `${props.worldName}_satellite_r${props.radius}_[${props.spawnX}_${props.spawnZ}].png`
  link.href = props.image
  link.click()
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="relative w-full h-full kip-card p-0 border-cyan-500/30 flex flex-col max-w-6xl shadow-[0_0_60px_rgba(6,182,212,0.25)] overflow-hidden">
        <!-- Top Toolbar -->
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
          <div class="flex items-center gap-3">
            <div class="p-2.5 rounded-xl bg-cyan-500/10 border border-cyan-500/20 text-cyan-400">
              <Map class="w-6 h-6" />
            </div>
            <div>
              <h3 class="font-black text-2xl text-white">Cartographer Satellite Radar</h3>
              <p class="text-xs font-mono text-cyan-400 flex items-center gap-2">
                <span>Universe: {{ worldName }}</span>
                <span>•</span>
                <span class="flex items-center gap-1"><Crosshair class="w-3 h-3" /> Spawn: [X: {{ spawnX }}, Z: {{ spawnZ }}]</span>
              </p>
            </div>
          </div>

          <!-- Controls & Radius Selector -->
          <div class="flex items-center gap-2">
            <div class="flex items-center gap-1 bg-black/50 p-1 rounded-xl border border-white/10 font-mono text-xs">
              <button
                @click="emit('change-radius', 1)"
                class="px-2.5 py-1 rounded-lg uppercase font-bold transition"
                :class="radius === 1 ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40' : 'text-white/40 hover:text-white'"
              >
                512m
              </button>
              <button
                @click="emit('change-radius', 2)"
                class="px-2.5 py-1 rounded-lg uppercase font-bold transition"
                :class="radius === 2 ? 'bg-cyan-500/20 text-cyan-300 border border-cyan-500/40' : 'text-white/40 hover:text-white'"
              >
                1024m
              </button>
            </div>

            <div class="flex items-center gap-1 bg-black/50 p-1 rounded-xl border border-white/10">
              <button @click="zoomLevel = Math.min(zoomLevel + 0.25, 4.0)" class="p-1.5 text-white/50 hover:text-white" title="Zoom In">
                <ZoomIn class="w-4 h-4" />
              </button>
              <button @click="zoomLevel = Math.max(zoomLevel - 0.25, 0.5)" class="p-1.5 text-white/50 hover:text-white" title="Zoom Out">
                <ZoomOut class="w-4 h-4" />
              </button>
              <button @click="resetTransform" class="p-1.5 text-white/50 hover:text-white" title="Reset View">
                <RotateCcw class="w-4 h-4" />
              </button>
            </div>

            <button
              v-if="image"
              @click="downloadMap"
              class="kip-btn-ghost px-4 py-2 text-xs border-cyan-500/30 text-cyan-400 hover:bg-cyan-500/10 flex items-center gap-2 font-mono uppercase font-bold"
            >
              <Download class="w-4 h-4" /> Export PNG
            </button>

            <button @click="isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white transition">
              <X class="w-6 h-6" />
            </button>
          </div>
        </header>

        <!-- Interactive Map Canvas Area -->
        <main
          class="flex-1 overflow-hidden flex items-center justify-center bg-[#020204] relative select-none cursor-grab active:cursor-grabbing"
          @mousedown="startPan"
          @mousemove="onPan"
          @mouseup="stopPan"
          @mouseleave="stopPan"
          @wheel="handleWheel"
        >
          <div v-if="loading" class="flex flex-col items-center text-cyan-400 relative z-10 pointer-events-none">
            <Loader class="w-16 h-16 animate-spin mb-4" />
            <span class="font-mono text-xs uppercase font-bold tracking-widest animate-pulse">
              Reconstructing MCA voxel regions and heightmap buffer...
            </span>
          </div>

          <div
            v-else-if="image"
            class="relative transition-transform duration-75 origin-center pointer-events-none"
            :style="{
              transform: `translate(${panX}px, ${panY}px) scale(${zoomLevel})`
            }"
          >
            <img
              :src="image"
              class="max-w-none shadow-2xl rounded-2xl border border-white/10"
              style="image-rendering: pixelated; width: 850px; height: 850px;"
            >
            <!-- Center Crosshair Marker -->
            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
              <div class="relative flex items-center justify-center">
                <Crosshair class="w-8 h-8 text-rose-500 animate-pulse drop-shadow-[0_0_8px_rgba(244,63,94,0.9)]" />
                <span class="absolute -bottom-5 px-2 py-0.5 rounded bg-black/80 text-[9px] font-mono font-bold text-rose-400 border border-rose-500/40 whitespace-nowrap">
                  SPAWN
                </span>
              </div>
            </div>
          </div>

          <div v-else class="flex flex-col items-center text-white/30 gap-4 pointer-events-none">
            <Earth class="w-16 h-16 opacity-30" />
            <p class="font-mono text-xs">{{ t('Region files are empty or ungenerated.') }}</p>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>