<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="flex justify-between items-end mb-8 shrink-0 stagger-1">
      <div>
        <h2 class="text-3xl font-extrabold mb-1">{{ t('Gallery') }}</h2>
        <p class="text-white/50 text-sm">{{ t('View and compress your latest screenshots.') }}</p>
      </div>
      <div class="flex gap-3">
        <button @click="openFolder" class="kip-btn-ghost px-5 py-2.5">
          <FolderOpen class="w-4 h-4 text-indigo-400" />
          {{ t('Open Folder') }}
        </button>
        <button @click="compressMedia" :disabled="isCompressing" class="kip-btn-primary px-6 py-2.5 bg-indigo-500 hover:bg-indigo-400 border border-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.3)]">
          <Loader v-if="isCompressing" class="w-4 h-4 animate-spin" />
          <Minimize v-else class="w-4 h-4" />
          {{ isCompressing ? t('Compressing...') : t('Compress to JPG') }}
        </button>
      </div>
    </div>

    <div class="grid grid-cols-3 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 stagger-2 min-h-0" @scroll="handleScroll">
      <div v-if="isLoading && offset === 0" class="col-span-3 text-center text-white/30 py-10 flex justify-center">
        <Loader class="w-8 h-8 animate-spin text-indigo-500" />
      </div>
      <div v-else-if="mediaList.length === 0" class="col-span-3 text-center text-white/30 py-20 flex flex-col items-center gap-4">
        <ImageOff class="w-12 h-12 opacity-50" />
        <span class="font-medium">{{ t('No screenshots found.') }}</span>
      </div>

      <div v-for="m in mediaList" :key="m.filename" class="kip-card p-0 overflow-hidden group cursor-pointer border-white/5 hover:border-indigo-500/30 transition-all duration-500" @click="openLightbox(m)">
        <div class="h-48 overflow-hidden relative bg-black/50">
          <img :src="m.thumbnail" class="w-full h-full object-cover group-hover:scale-110 transition duration-700 opacity-80 group-hover:opacity-100">
          <div class="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
          <div class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300 scale-90 group-hover:scale-100">
            <div class="p-3 bg-black/40 backdrop-blur-md rounded-full border border-white/10 shadow-[0_0_15px_rgba(0,0,0,0.5)]">
              <Maximize class="w-5 h-5 text-white" />
            </div>
          </div>
          <div class="absolute top-3 right-3 flex gap-2">
            <span :class="m.is_png ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30' : 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30'" class="px-2 py-1 text-[9px] rounded-md font-bold uppercase tracking-wider border">
              {{ m.is_png ? 'PNG' : 'JPG' }}
            </span>
            <span class="px-2 py-1 bg-black/60 border border-white/10 backdrop-blur text-white/80 text-[9px] rounded-md font-bold tracking-wider">
              {{ m.size }} MB
            </span>
          </div>
        </div>
        <div class="p-4 bg-black/40 border-t border-white/5 flex items-center justify-between">
          <p class="text-xs text-white/70 truncate font-medium">{{ m.filename }}</p>
        </div>
      </div>

      <div v-if="isLoadingMore" class="col-span-3 flex justify-center py-6">
        <Loader class="w-8 h-8 animate-spin text-indigo-500" />
      </div>
    </div>

    <transition name="fade">
      <div v-if="lightbox.isOpen" class="fixed inset-0 bg-black/95 backdrop-blur-2xl z-[300] flex flex-col items-center justify-center p-6 cursor-default" @click.self="closeLightbox">
        <div class="w-full flex justify-between items-start shrink-0 mb-4 px-4 pointer-events-none absolute top-6 z-50">
          <div class="pointer-events-auto kip-card px-6 py-4 flex flex-col gap-1.5 shadow-2xl border-white/10 bg-black/60">
            <h3 class="font-bold text-sm text-white drop-shadow-md">{{ lightbox.item?.filename }}</h3>
            <p class="text-[10px] text-white/50 uppercase tracking-widest font-bold">{{ lightbox.item?.size }} MB • {{ lightbox.item?.is_png ? 'PNG' : 'JPG' }}</p>
          </div>
          <div class="flex gap-3 pointer-events-auto">
            <button @click="deleteMedia" class="kip-btn-danger px-4 py-3 shadow-[0_0_15px_rgba(239,68,68,0.3)]">
              <Trash2 class="w-5 h-5" />
            </button>
            <button @click="closeLightbox" class="kip-btn-ghost px-4 py-3 bg-black/40 hover:bg-white/10 border-white/10">
              <X class="w-5 h-5" />
            </button>
          </div>
        </div>

        <div class="flex-1 w-full flex items-center justify-center relative overflow-hidden" @click.self="closeLightbox">
          <Loader v-if="lightbox.loading" class="w-12 h-12 animate-spin text-indigo-500 absolute" />
          <img v-if="lightbox.fullImage" :src="lightbox.fullImage" class="max-w-[95vw] max-h-[85vh] object-contain shadow-[0_0_50px_rgba(0,0,0,0.8)] rounded-xl border border-white/5" :class="lightbox.loading ? 'opacity-0 scale-95' : 'opacity-100 scale-100 transition-all duration-500 ease-out'">
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup>
import { ref, onMounted } from 'vue'
import { Minimize, ImageOff, Loader, Maximize, X, Trash2, FolderOpen } from 'lucide-vue-next'
import { api, t, showToast } from '@/store.js'

const mediaList = ref([])
const isLoading = ref(false)
const isLoadingMore = ref(false)
const isCompressing = ref(false)
const hasMore = ref(true)
const offset = ref(0)
const limit = 12

const lightbox = ref({
  isOpen: false,
  item: null,
  fullImage: '',
  loading: false
})

const loadMedia = async (reset = true) => {
  if (!api.value || (!hasMore.value && !reset)) return
  if (isLoading.value || isLoadingMore.value) return

  if (reset) {
    isLoading.value = true
    offset.value = 0
    mediaList.value = []
    hasMore.value = true
  } else {
    isLoadingMore.value = true
  }

  try {
    const data = await api.value.get_media(offset.value, limit)
    if (data && data.length > 0) {
      mediaList.value.push(...data)
      offset.value += limit
      if (data.length < limit) {
        hasMore.value = false
      }
    } else {
      hasMore.value = false
    }
  } catch (e) {
    showToast(t("Error"), t("Failed to load gallery."), "danger")
  }

  isLoading.value = false
  isLoadingMore.value = false
}

const handleScroll = (e) => {
  const el = e.target
  if (el.scrollHeight - el.scrollTop <= el.clientHeight + 100) {
    if (!isLoadingMore.value && hasMore.value) {
      loadMedia(false)
    }
  }
}

const compressMedia = async () => {
  if (!api.value || isCompressing.value) return
  isCompressing.value = true
  try {
    const res = await api.value.compress_media()
    if (res && res.success) {
      showToast(t("Success"), res.msg, "success")
      loadMedia(true)
    } else {
      showToast(t("Error"), res?.msg || t("Failed to compress media."), "danger")
    }
  } catch (e) {
    showToast(t("Error"), t("Backend communication failed."), "danger")
  }
  isCompressing.value = false
}

const openFolder = async () => {
  if (!api.value) return
  try {
    await api.value.open_media_folder()
  } catch (e) {}
}

const openLightbox = async (item) => {
  lightbox.value.item = item
  lightbox.value.isOpen = true
  lightbox.value.fullImage = ''
  lightbox.value.loading = true

  if (api.value) {
    try {
      const b64 = await api.value.get_media_full(item.filename)
      if (b64 && b64.length > 0) {
        lightbox.value.fullImage = b64
      } else {
        showToast(t("Error"), t("Failed to load full image."), "danger")
        closeLightbox()
      }
    } catch (e) {
      showToast(t("Error"), t("Network error."), "danger")
      closeLightbox()
    }
  }
  lightbox.value.loading = false
}

const closeLightbox = () => {
  lightbox.value.isOpen = false
  lightbox.value.fullImage = ''
  lightbox.value.item = null
}

const deleteMedia = async () => {
  if (!api.value || !lightbox.value.item) return
  try {
    const filename = lightbox.value.item.filename
    const res = await api.value.delete_media(filename)
    if (res && res.success) {
      showToast(t("Deleted"), res.msg, "success")
      mediaList.value = mediaList.value.filter(m => m.filename !== filename)
      closeLightbox()
    } else {
      showToast(t("Error"), res?.msg || t("Failed to delete media."), "danger")
    }
  } catch (e) {
    showToast(t("Error"), t("Backend communication failed."), "danger")
  }
}

onMounted(() => {
  loadMedia(true)
})
</script>