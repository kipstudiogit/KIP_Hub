import { ref, computed } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import type {
  MediaItemDto,
  MediaCompressProgressDto,
  MediaCompressResultDto,
  MediaBatchActionResultDto,
  MediaFormatFilter,
} from '../types/media'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useMediaManager() {
  const mediaList = ref<MediaItemDto[]>([])
  const isLoading = ref(false)
  const isLoadingMore = ref(false)
  const hasMore = ref(true)
  const offset = ref(0)
  const limit = 12
  const activeFormatFilter = ref<MediaFormatFilter>('all')
  const searchQuery = ref('')
  const selectedFilenames = ref<Set<string>>(new Set())

  // Streaming compression state
  const isCompressing = ref(false)
  const compressionProgress = ref<MediaCompressProgressDto>({
    currentFile: '',
    processedCount: 0,
    totalFiles: 0,
    savedMb: 0,
    percent: 0,
  })

  // Lightbox modal state
  const isLightboxOpen = ref(false)
  const activeLightboxIndex = ref<number>(-1)
  const fullImageBase64 = ref('')
  const isFullImageLoading = ref(false)

  const activeLightboxItem = computed<MediaItemDto | null>(() => {
    if (activeLightboxIndex.value >= 0 && activeLightboxIndex.value < filteredMedia.value.length) {
      return filteredMedia.value[activeLightboxIndex.value] || null
    }
    return null
  })

  const filteredMedia = computed(() => {
    let list = [...mediaList.value]
    const q = searchQuery.value.trim().toLowerCase()
    if (q) {
      list = list.filter(
        (m) => m.filename.toLowerCase().includes(q) || m.dateModified.toLowerCase().includes(q)
      )
    }
    return list
  })

  const uncompressedPngSavingsMb = computed(() => {
    return mediaList.value
      .filter((m) => m.isPng)
      .reduce((acc, curr) => acc + curr.sizeMb * 0.65, 0)
      .toFixed(1)
  })

  async function loadMedia(reset = true): Promise<void> {
    if (!hasMore.value && !reset) return
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
      const data = await invoke<MediaItemDto[]>('get_media_catalog', {
        offset: offset.value,
        limit,
        formatFilter: activeFormatFilter.value,
      })

      if (data && data.length > 0) {
        mediaList.value.push(...data)
        offset.value += limit
        if (data.length < limit) {
          hasMore.value = false
        }
      } else {
        hasMore.value = false
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    } finally {
      isLoading.value = false
      isLoadingMore.value = false
    }
  }

  async function openLightbox(item: MediaItemDto): Promise<void> {
    const idx = filteredMedia.value.findIndex((m) => m.filename === item.filename)
    activeLightboxIndex.value = idx >= 0 ? idx : 0
    isLightboxOpen.value = true
    await loadActiveFullImage()
  }

  async function loadActiveFullImage(): Promise<void> {
    if (!activeLightboxItem.value) return
    fullImageBase64.value = ''
    isFullImageLoading.value = true

    try {
      const b64 = await invoke<string>('get_media_full_image', {
        filename: activeLightboxItem.value.filename,
      })
      fullImageBase64.value = b64 || activeLightboxItem.value.thumbnail
    } catch {
      fullImageBase64.value = activeLightboxItem.value.thumbnail
    } finally {
      isFullImageLoading.value = false
    }
  }

  function nextImage(): void {
    if (activeLightboxIndex.value < filteredMedia.value.length - 1) {
      activeLightboxIndex.value++
      loadActiveFullImage()
    }
  }

  function prevImage(): void {
    if (activeLightboxIndex.value > 0) {
      activeLightboxIndex.value--
      loadActiveFullImage()
    }
  }

  async function copyActiveImageToClipboard(): Promise<void> {
    if (!fullImageBase64.value) return
    try {
      const res = await fetch(fullImageBase64.value)
      const blob = await res.blob()
      await navigator.clipboard.write([new ClipboardItem({ [blob.type]: blob })])
      showToast(t('Copied'), 'Image copied to system clipboard.', 'success')
    } catch {
      showToast(t('Error'), 'Could not copy image binary directly.', 'danger')
    }
  }

  async function deleteSingle(filename: string): Promise<void> {
    try {
      const success = await invoke<boolean>('delete_media_file', { filename })
      if (success) {
        showToast(t('Deleted'), `Removed ${filename}.`, 'success')
        mediaList.value = mediaList.value.filter((m) => m.filename !== filename)
        selectedFilenames.value.delete(filename)
        if (isLightboxOpen.value) {
          if (filteredMedia.value.length === 0) {
            isLightboxOpen.value = false
          } else {
            activeLightboxIndex.value = Math.min(activeLightboxIndex.value, filteredMedia.value.length - 1)
            await loadActiveFullImage()
          }
        }
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  async function batchDeleteSelected(): Promise<void> {
    if (selectedFilenames.value.size === 0) return
    const targets = Array.from(selectedFilenames.value)
    try {
      const res = await invoke<MediaBatchActionResultDto>('batch_delete_media_files', {
        filenames: targets,
      })
      showToast(t('Batch Delete'), res.msg, 'success')
      mediaList.value = mediaList.value.filter((m) => !selectedFilenames.value.has(m.filename))
      selectedFilenames.value.clear()
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Error'), error.message || String(err), 'danger')
    }
  }

  async function runLosslessCompression(): Promise<void> {
    if (isCompressing.value) return
    isCompressing.value = true
    compressionProgress.value = {
      currentFile: 'Starting compression pipeline...',
      processedCount: 0,
      totalFiles: 0,
      savedMb: 0,
      percent: 0,
    }

    const channel = new Channel<MediaCompressProgressDto>()
    channel.onmessage = (progress) => {
      compressionProgress.value = progress
    }

    try {
      const res = await invoke<MediaCompressResultDto>('compress_media_stream', {
        progressChannel: channel,
      })
      if (res.success) {
        showToast(t('Compression Complete'), res.msg, 'success')
        await loadMedia(true)
      } else {
        showToast(t('Notice'), res.msg, 'info')
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Compression Error'), error.message || String(err), 'danger')
    } finally {
      isCompressing.value = false
    }
  }

  async function openNativeFolder(): Promise<void> {
    try {
      await invoke('open_screenshots_directory')
    } catch {
      showToast(t('Error'), 'Could not launch explorer.', 'danger')
    }
  }

  return {
    mediaList,
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
  }
}