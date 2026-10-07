<script setup lang="ts">
import { onMounted } from 'vue'
import {
  FolderOpen,
  Minimize,
  ImageOff,
  Loader,
  Maximize,
  Trash2,
  Search,
  Check,
  CheckSquare,
  Sparkles,
  Camera,
} from 'lucide-vue-next'
import { t } from '@/store'
import type { MediaFormatFilter } from '../types/media'
import { useMediaManager } from '../composables/useMediaManager'
import MediaLightboxModal from '../components/media/MediaLightboxModal.vue'

const {
  filteredMedia,
  isLoading,
  isLoadingMore,
  hasMore,
  activeFormatFilter,
  searchQuery,
  selectedFilenames,
  isCompressing,
  compressionProgress,
  isLightboxOpen,
  activeLightboxItem,
  fullImageBase64,
  isFullImageLoading,
  uncompressedPngSavingsMb,
  loadMedia,
  openLightbox,
  nextImage,
  prevImage,
  copyActiveImageToClipboard,
  deleteSingle,
  batchDeleteSelected,
  runLosslessCompression,
  openNativeFolder,
} = useMediaManager()

function setFilter(filter: MediaFormatFilter): void {
  activeFormatFilter.value = filter
  selectedFilenames.value.clear()
  loadMedia(true)
}

function toggleSelect(filename: string): void {
  if (selectedFilenames.value.has(filename)) {
    selectedFilenames.value.delete(filename)
  } else {
    selectedFilenames.value.add(filename)
  }
}

function handleScroll(e: Event): void {
  const el = e.target as HTMLElement
  if (el.scrollHeight - el.scrollTop <= el.clientHeight + 150) {
    if (!isLoadingMore.value && hasMore.value) {
      loadMedia(false)
    }
  }
}

onMounted(() => {
  loadMedia(true)
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <!-- Top Action Bar -->
    <header class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Gallery') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Lossless Stream Compression, 4K Lightbox Deck & Instant Clipboard Bridge</p>
      </div>

      <div class="flex items-center gap-3">
        <button
          @click="openNativeFolder"
          class="kip-btn-ghost px-4 py-2.5 text-xs font-bold uppercase tracking-wider flex items-center gap-2"
        >
          <FolderOpen class="w-4 h-4 text-indigo-400" />
          <span>{{ t('Open Folder') }}</span>
        </button>

        <button
          @click="runLosslessCompression"
          :disabled="isCompressing"
          class="kip-btn-primary px-6 py-2.5 text-xs font-black uppercase tracking-wider bg-indigo-500 hover:bg-indigo-400 text-white shadow-[0_0_20px_rgba(99,102,241,0.3)] flex items-center gap-2"
        >
          <Loader v-if="isCompressing" class="w-4 h-4 animate-spin" />
          <Minimize v-else class="w-4 h-4" />
          <span>{{ isCompressing ? 'Compressing...' : 'Compress to JPG' }}</span>
        </button>
      </div>
    </header>

    <!-- Optimization Banner (If uncompressed PNGs exist) -->
    <section
      v-if="Number(uncompressedPngSavingsMb) > 10"
      class="mb-6 kip-card p-4 border border-indigo-500/30 bg-gradient-to-r from-indigo-950/40 to-black/60 flex items-center justify-between shadow-xl shrink-0"
    >
      <div class="flex items-center gap-3">
        <div class="p-2 rounded-xl bg-indigo-500/20 text-indigo-400 border border-indigo-500/30">
          <Sparkles class="w-4 h-4" />
        </div>
        <div>
          <h4 class="text-xs font-bold text-white uppercase tracking-wider">Disk Storage Optimization Available</h4>
          <p class="text-[11px] text-white/60 font-mono mt-0.5">
            You can reclaim approximately ~{{ uncompressedPngSavingsMb }} MB by converting raw PNG screenshots to compressed JPEG.
          </p>
        </div>
      </div>

      <button
        @click="runLosslessCompression"
        :disabled="isCompressing"
        class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase text-indigo-300 border-indigo-500/40 hover:bg-indigo-500/10"
      >
        Optimize Now
      </button>
    </section>

    <!-- Streaming Progress Indicator Bar -->
    <div v-if="isCompressing" class="mb-6 kip-card p-4 bg-black/60 border border-indigo-500/40 flex flex-col gap-2 shrink-0 shadow-2xl">
      <div class="flex justify-between text-xs font-mono text-white/70">
        <span class="truncate">Processing: {{ compressionProgress.currentFile }}</span>
        <span class="text-indigo-400 font-bold">{{ Math.round(compressionProgress.percent) }}% ({{ compressionProgress.savedMb }} MB saved)</span>
      </div>
      <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden">
        <div class="h-full bg-indigo-500 transition-all duration-150" :style="{ width: `${compressionProgress.percent}%` }"></div>
      </div>
    </div>

    <!-- Filters & Search Toolbar -->
    <div class="flex justify-between items-center gap-4 mb-6 shrink-0 z-10">
      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          v-for="f in (['all', 'png', 'jpg'] as const)"
          :key="f"
          @click="setFilter(f)"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition"
          :class="activeFormatFilter === f ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'"
        >
          {{ f === 'all' ? 'All Captures' : f === 'png' ? 'Raw PNG' : 'Optimized JPG' }}
        </button>
      </div>

      <div class="relative w-64">
        <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30 w-3.5 h-3.5" />
        <input
          v-model="searchQuery"
          type="text"
          placeholder="Filter by name or date..."
          class="kip-input pl-8 py-2 text-xs font-mono"
        >
      </div>
    </div>

    <!-- Media Grid -->
    <main class="grid grid-cols-3 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10" @scroll="handleScroll">
      <div v-if="isLoading && filteredMedia.length === 0" class="col-span-3 text-center py-24 flex flex-col items-center justify-center">
        <Loader class="w-10 h-10 animate-spin text-indigo-400 mb-3" />
        <span class="text-xs font-mono uppercase text-white/40">Scanning screenshot archives and decoding thumbs...</span>
      </div>

      <div v-else-if="filteredMedia.length === 0" class="col-span-3 text-center py-24 flex flex-col items-center gap-4">
        <ImageOff class="w-16 h-16 text-white/10" />
        <span class="text-white/40 font-mono text-xs">{{ searchQuery ? 'No captures matching filter.' : t('No screenshots found.') }}</span>
      </div>

      <article
        v-for="item in filteredMedia"
        :key="item.filename"
        class="kip-card p-0 overflow-hidden group cursor-pointer border-white/5 hover:border-indigo-500/30 transition-all duration-300 bg-black/40 flex flex-col"
        @click="openLightbox(item)"
      >
        <div class="h-48 overflow-hidden relative bg-black/60">
          <img
            :src="item.thumbnail"
            class="w-full h-full object-cover group-hover:scale-105 transition-transform duration-500 opacity-80 group-hover:opacity-100"
          >

          <!-- Hover Gradient Overlay -->
          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity"></div>

          <!-- Selection Checkbox -->
          <div
            @click.stop="toggleSelect(item.filename)"
            class="absolute top-3 left-3 w-5 h-5 rounded-lg border flex items-center justify-center transition z-20"
            :class="selectedFilenames.has(item.filename) ? 'bg-indigo-500 border-indigo-400 text-white' : 'border-white/30 bg-black/60 opacity-0 group-hover:opacity-100'"
          >
            <Check v-if="selectedFilenames.has(item.filename)" class="w-3.5 h-3.5 stroke-[3]" />
          </div>

          <!-- Format & Size Badges -->
          <div class="absolute top-3 right-3 flex gap-1.5 z-20">
            <span
              class="px-2 py-0.5 text-[8px] rounded-md font-bold uppercase tracking-wider border font-mono"
              :class="item.isPng ? 'bg-indigo-500/20 text-indigo-300 border-indigo-500/30' : 'bg-emerald-500/20 text-emerald-300 border-emerald-500/30'"
            >
              {{ item.isPng ? 'PNG' : 'JPG' }}
            </span>
            <span class="px-2 py-0.5 bg-black/70 border border-white/10 backdrop-blur text-white/80 text-[8px] rounded-md font-bold font-mono">
              {{ item.sizeMb }} MB
            </span>
          </div>

          <!-- Centered Hover Trigger -->
          <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
            <div class="p-3 bg-black/50 backdrop-blur-md rounded-2xl border border-white/10 shadow-2xl">
              <Maximize class="w-5 h-5 text-white" />
            </div>
          </div>
        </div>

        <!-- Footer Data -->
        <footer class="p-3.5 bg-black/40 border-t border-white/5 flex items-center justify-between text-xs">
          <p class="text-white/70 truncate font-mono text-[11px] flex-1 pr-2">{{ item.filename }}</p>
          <span class="text-white/30 font-mono text-[10px] shrink-0">{{ item.width }}x{{ item.height }}</span>
        </footer>
      </article>

      <div v-if="isLoadingMore" class="col-span-3 flex justify-center py-6">
        <Loader class="w-8 h-8 animate-spin text-indigo-400" />
      </div>
    </main>

    <!-- Floating Batch Selection Deck -->
    <div
      v-if="selectedFilenames.size > 0"
      class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-black/85 backdrop-blur-2xl border border-indigo-500/40 px-6 py-3 rounded-2xl flex items-center gap-4 z-40 shadow-2xl"
    >
      <span class="text-xs font-mono font-bold text-white">{{ selectedFilenames.size }} selected</span>
      <div class="w-px h-4 bg-white/20"></div>
      <button @click="batchDeleteSelected" class="kip-btn-danger px-4 py-1.5 text-xs uppercase font-bold">
        <Trash2 class="w-3.5 h-3.5" />
        <span>Delete Selected</span>
      </button>
      <button @click="selectedFilenames.clear()" class="text-white/40 hover:text-white text-xs font-mono">
        Cancel
      </button>
    </div>

    <!-- Lightbox Pro Subcomponent -->
    <MediaLightboxModal
      v-model="isLightboxOpen"
      :item="activeLightboxItem"
      :full-image="fullImageBase64"
      :loading="isFullImageLoading"
      @next="nextImage"
      @prev="prevImage"
      @delete="deleteSingle($event)"
      @copy="copyActiveImageToClipboard"
    />
  </div>
</template>