import { ref } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import type {
  StoreProvider,
  StoreProjectType,
  StoreItemRecordDto,
  StoreSearchResultDto,
  StoreDetailsResponseDto,
  StoreDownloadProgressDto,
  StoreInstallResultDto,
  StoreItemVersionDto,
} from '../types/store'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type { BackendAppError } from '../types/launcher'

export function useStoreManager() {
  const provider = ref<StoreProvider>('modrinth')
  const projectType = ref<StoreProjectType>('mod')
  const loader = ref<string>('')
  const gameVersion = ref<string>('')
  const searchQuery = ref<string>('')
  const sortIndex = ref<string>('relevance')
  const category = ref<string>('')
  const offset = ref<number>(0)

  const isStoreLoading = ref<boolean>(false)
  const isStoreLoadingMore = ref<boolean>(false)
  const hasMoreStoreItems = ref<boolean>(true)
  const storeResults = ref<StoreItemRecordDto[]>([])

  async function searchCatalog(reset = true): Promise<void> {
    if (reset) {
      isStoreLoading.value = true
      offset.value = 0
      hasMoreStoreItems.value = true
    } else {
      isStoreLoadingMore.value = true
    }

    try {
      const res = await invoke<StoreSearchResultDto>('search_store_catalog', {
        provider: provider.value,
        query: searchQuery.value.trim() || null,
        projectType: projectType.value || null,
        loader: loader.value || null,
        gameVersion: gameVersion.value || null,
        category: category.value || null,
        sortIndex: sortIndex.value || null,
        offset: offset.value,
      })

      if (res && res.success) {
        const mapped = (res.hits || []).map((item) => ({
          ...item,
          downloading: false,
          progress: 0,
          statusText: '',
        }))

        if (reset) {
          storeResults.value = mapped
        } else {
          storeResults.value.push(...mapped)
        }

        if (mapped.length < 16) {
          hasMoreStoreItems.value = false
        }
      } else {
        if (reset) storeResults.value = []
        hasMoreStoreItems.value = false
      }
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Store Notice'), error.message || String(err), 'danger')
      if (reset) storeResults.value = []
      hasMoreStoreItems.value = false
    } finally {
      isStoreLoading.value = false
      isStoreLoadingMore.value = false
    }
  }

  async function loadMore(): Promise<void> {
    if (isStoreLoading.value || isStoreLoadingMore.value || !hasMoreStoreItems.value) return
    offset.value += 16
    await searchCatalog(false)
  }

  async function fetchProjectDetails(projectId: string): Promise<StoreDetailsResponseDto | null> {
    try {
      return await invoke<StoreDetailsResponseDto>('get_store_project_details', {
        provider: provider.value,
        projectId,
        loader: loader.value || null,
        gameVersion: gameVersion.value || null,
      })
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Metadata Fault'), error.message || String(err), 'danger')
      return null
    }
  }

  async function installItem(
    item: StoreItemRecordDto,
    versionOverride?: StoreItemVersionDto
  ): Promise<boolean> {
    if (item.downloading) return false
    item.downloading = true
    item.progress = 5
    item.statusText = 'Resolving'

    let targetVersion = versionOverride
    if (!targetVersion) {
      const details = await fetchProjectDetails(item.projectId)
      if (!details || !details.success || details.versions.length === 0) {
        showToast(t('Error'), 'No compatible releases found for target runtime.', 'danger')
        item.downloading = false
        return false
      }
      targetVersion = details.versions[0]
    }

    if (!targetVersion || targetVersion.files.length === 0) {
      item.downloading = false
      return false
    }

    const targetFile = targetVersion.files.find((f) => f.primary) || targetVersion.files[0]
    if (!targetFile) {
      item.downloading = false
      return false
    }

    const channel = new Channel<StoreDownloadProgressDto>()
    channel.onmessage = (progress) => {
      item.progress = progress.progress
      item.statusText = progress.status
    }

    try {
      const res = await invoke<StoreInstallResultDto>('install_store_item_stream', {
        payload: {
          provider: item.provider,
          projectId: item.projectId,
          versionId: targetVersion.id,
          url: targetFile.url,
          filename: targetFile.filename,
          projectType: item.projectType,
          loader: loader.value || null,
          gameVersion: gameVersion.value || null,
        },
        progressChannel: channel,
      })

      if (res && res.success) {
        item.isInstalled = true
        item.installedFilename = res.filename
        showToast(t('Installed'), res.message, 'success')
        if (res.installedDependencies.length > 0) {
          showToast(
            t('Dependencies Synced'),
            `Integrated ${res.installedDependencies.length} companion libraries automatically.`,
            'info'
          )
        }
        return true
      }
      return false
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Install Error'), error.message || String(err), 'danger')
      return false
    } finally {
      item.downloading = false
      item.progress = 0
    }
  }

  async function unpinItem(item: StoreItemRecordDto): Promise<boolean> {
    if (!item.installedFilename) return false
    try {
      const success = await invoke<boolean>('uninstall_store_item', {
        projectType: item.projectType,
        filename: item.installedFilename,
      })
      if (success) {
        item.isInstalled = false
        item.installedFilename = null
        showToast(t('Removed'), `Purged ${item.title} from instance.`, 'success')
        return true
      }
      return false
    } catch (err) {
      const error = err as BackendAppError
      showToast(t('Uninstall Error'), error.message || String(err), 'danger')
      return false
    }
  }

  return {
    provider,
    projectType,
    loader,
    gameVersion,
    searchQuery,
    sortIndex,
    category,
    offset,
    isStoreLoading,
    isStoreLoadingMore,
    hasMoreStoreItems,
    storeResults,
    searchCatalog,
    loadMore,
    fetchProjectDetails,
    installItem,
    unpinItem,
  }
}