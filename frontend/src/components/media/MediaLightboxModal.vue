<script setup lang="ts">
import { onMounted, onBeforeUnmount } from 'vue'
import {
  X,
  Trash2,
  Copy,
  ChevronLeft,
  ChevronRight,
  Loader,
  Maximize2,
  Calendar,
  Layers,
} from 'lucide-vue-next'
import type { MediaItemDto } from '../../types/media'

const isOpen = defineModel<boolean>({ required: true })

const props = defineProps<{
  item: MediaItemDto | null
  fullImage: string
  loading: boolean
}>()

const emit = defineEmits<{
  (e: 'next'): void
  (e: 'prev'): void
  (e: 'delete', filename: string): void
  (e: 'copy'): void
}>()

function handleKeyDown(e: KeyboardEvent): void {
  if (!isOpen.value) return
  if (e.key === 'ArrowRight') emit('next')
  if (e.key === 'ArrowLeft') emit('prev')
  if (e.key === 'Escape') isOpen.value = false
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleKeyDown)
})
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && item"
      class="fixed inset-0 bg-black/95 backdrop-blur-2xl z-[300] flex flex-col items-center justify-center p-6 cursor-default select-none"
      @click.self="isOpen = false"
    >
      <!-- Floating Top HUD Header -->
      <header class="w-full flex justify-between items-start shrink-0 mb-4 px-6 pointer-events-none absolute top-6 z-50">
        <div class="pointer-events-auto kip-card px-6 py-3.5 flex items-center gap-4 bg-black/70 border-white/10 shadow-2xl">
          <div>
            <h3 class="font-bold text-sm text-white truncate max-w-sm">{{ item.filename }}</h3>
            <p class="text-[10px] font-mono text-white/50 flex items-center gap-2 mt-0.5">
              <span>{{ item.sizeMb }} MB</span>
              <span>•</span>
              <span class="text-indigo-400 font-bold uppercase">{{ item.isPng ? 'PNG' : 'JPG' }}</span>
              <span>•</span>
              <span class="flex items-center gap-1"><Maximize2 class="w-3 h-3" /> {{ item.width }}x{{ item.height }}</span>
              <span>•</span>
              <span class="flex items-center gap-1"><Calendar class="w-3 h-3" /> {{ item.dateModified }}</span>
            </p>
          </div>
        </div>

        <!-- Quick Action Tools -->
        <div class="flex gap-2.5 pointer-events-auto">
          <button
            @click="emit('copy')"
            class="kip-btn-ghost px-4 py-3 bg-black/60 border-white/10 text-white/80 hover:text-white"
            title="Copy Image to Clipboard"
          >
            <Copy class="w-4 h-4" />
          </button>
          <button
            @click="emit('delete', item.filename)"
            class="kip-btn-danger px-4 py-3 shadow-[0_0_15px_rgba(239,68,68,0.3)]"
            title="Delete Capture"
          >
            <Trash2 class="w-4 h-4" />
          </button>
          <button
            @click="isOpen = false"
            class="kip-btn-ghost px-4 py-3 bg-black/60 border-white/10 hover:bg-white/10"
            title="Close [ESC]"
          >
            <X class="w-4 h-4" />
          </button>
        </div>
      </header>

      <!-- Carousel Steppers -->
      <button
        @click="emit('prev')"
        class="absolute left-6 top-1/2 -translate-y-1/2 p-4 rounded-full bg-black/60 hover:bg-white/10 text-white/60 hover:text-white border border-white/10 z-50 transition"
      >
        <ChevronLeft class="w-6 h-6" />
      </button>

      <button
        @click="emit('next')"
        class="absolute right-6 top-1/2 -translate-y-1/2 p-4 rounded-full bg-black/60 hover:bg-white/10 text-white/60 hover:text-white border border-white/10 z-50 transition"
      >
        <ChevronRight class="w-6 h-6" />
      </button>

      <!-- Main Stage Canvas -->
      <main class="flex-1 w-full flex items-center justify-center relative overflow-hidden" @click.self="isOpen = false">
        <Loader v-if="loading" class="w-12 h-12 animate-spin text-indigo-400 absolute" />
        <img
          v-if="fullImage"
          :src="fullImage"
          class="max-w-[92vw] max-h-[84vh] object-contain shadow-[0_0_60px_rgba(0,0,0,0.9)] rounded-2xl border border-white/10 transition-opacity duration-300"
          :class="loading ? 'opacity-40' : 'opacity-100'"
        >
      </main>
    </div>
  </transition>
</template>