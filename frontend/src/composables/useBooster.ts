import { ref, computed } from 'vue'
import { invoke, Channel } from '@tauri-apps/api/core'
import { showToast } from './useToasts'
import { t } from './useI18n'
import type {
  ChunkAcceleratorStatusDto,
  ChunkPrebakeProgressDto,
  ChunkPrebakeResultDto,
  ChunkRingFlushResultDto,
  MemoryMatrixStatusDto,
  MemoryPrefaultResultDto,
  PingMasterConfigDto,
  VpnDetectionDto,
  PingMasterMetricDto,
  PingMasterStatusDto,
  HardwareBoosterConfigDto,
  HardwareBoosterStatusDto,
  TranscodeProgressDto,
  TranscodeResultDto,
  PageCacheWarmupResultDto,
} from '../types/booster'

export interface ServerQuickPreset {
  id: string
  name: string
  host: string
  port: number
  badge: string
}

const chunkAcceleratorStatus = ref<ChunkAcceleratorStatusDto>({
  isActive: true,
  simdCapabilities: {
    avx512Supported: false,
    avx2Supported: true,
    fmaSupported: true,
    vectorWidthBits: 256,
    simdTier: 'AVX2 SIMD 8-Way Parallel',
  },
  ringBufferPendingChunks: 0,
  totalChunksBuffered: 0,
  totalBytesStreamedMb: 0,
  predictiveConeRadius: 16,
  estimatedTpsGain: 4.2,
  engineSignature: 'AVX2/AVX-512 Matrix Ring 4.2 (Zero-GC)',
  arenaMetrics: {
    totalAllocatedMb: 128,
    usedMemoryMb: 0,
    activeChunksInFlight: 0,
    allocationBumpsCount: 0,
    zeroGcEfficiencyPercent: 99,
  },
  topologyMetrics: {
    currentPhase: 0,
    phaseName: 'Phase 0 [Even, Even] - Primary Slabs',
    queuedChunks: 0,
    resolvedCascadesCount: 0,
    threadConcurrencyTier: 'Zero-Lock 2D Cellular Automaton',
  },
})

const isPrebakingChunks = ref<boolean>(false)
const isFlushingRing = ref<boolean>(false)
const chunkPrebakeProgress = ref<ChunkPrebakeProgressDto>({
  currentChunk: '[0, 0]',
  generatedCount: 0,
  totalTargetChunks: 0,
  percent: 0,
  speedChunksPerSec: 0,
})

const memoryMatrixStatus = ref<MemoryMatrixStatusDto>({
  largePagesSupported: true,
  largePagesActive: false,
  seLockPrivilegeGranted: false,
  compactHeadersSupported: true,
  compactHeadersActive: true,
  standbyPrefaultFiles: 0,
  standbyPrefaultMb: 0,
  estimatedTlbMissReductionPercent: 95,
  savedMemoryMb: 1850,
  statusText: 'Probing Virtual Memory Subsystem...',
})

const isPrefaulting = ref<boolean>(false)
const isGrantingPrivilege = ref<boolean>(false)

const isPingMasterRunning = ref<boolean>(false)
const targetHostInput = ref<string>('mc.hypixel.net')
const targetPortInput = ref<number>(25565)

const pingMasterConfig = ref<PingMasterConfigDto>({
  routingMode: 'adaptive',
  upstreamSocks5Host: '127.0.0.1',
  upstreamSocks5Port: 7890,
  antiBufferbloat: true,
  dscpQosEnabled: true,
})

const vpnStatus = ref<VpnDetectionDto>({
  vpnActive: false,
  activeAdapterName: null,
  adapterList: [],
})

const currentMetric = ref<PingMasterMetricDto>({
  localProxyPort: 0,
  targetHost: 'mc.hypixel.net',
  targetPort: 25565,
  currentPingMs: 0,
  jitterMs: 0,
  minPingMs: 0,
  maxPingMs: 0,
  packetsOptimized: 0,
  bytesTransferred: 0,
  tcpNodelayActive: true,
  bufferbloatReduced: true,
  activeRoute: 'Resolving optimal gateway...',
  vpnDetected: false,
  activeVpnAdapter: null,
  routingMode: 'adaptive',
  hitRegQuality: 'Calibrating Socket...',
  packetLossPercent: 0,
})

const pingHistory = ref<number[]>([24, 22, 23, 21, 23, 22, 20, 21, 22, 21])

const boosterConfig = ref<HardwareBoosterConfigDto>({
  timerResolutionEnabled: true,
  processPriorityBoost: true,
  trimLauncherMemory: true,
  defenderBypassEnabled: true,
  cracAccelerationEnabled: false,
  appCdsEnabled: true,
  directVramTranscode: true,
  shaderPrewarmingEnabled: true,
  pageCacheWarmupEnabled: true,
})

const boosterStatus = ref<HardwareBoosterStatusDto>({
  timerActive: false,
  timerResolutionMs: 15.6,
  trimmedMemoryMb: 0,
  defenderExcludedPaths: [],
  appCdsArchiveExists: false,
  appCdsArchiveSizeMb: 0,
  cracCheckpointExists: false,
  shaderCachePath: '',
  pageCacheWarmFiles: 0,
  pageCacheWarmMb: 0,
})

const isTranscoding = ref<boolean>(false)
const transcodeProgress = ref<TranscodeProgressDto>({
  currentFile: '',
  processedCount: 0,
  totalFiles: 0,
  percent: 0,
  vramSavedMb: 0,
})

const isWarmingPageCache = ref<boolean>(false)

const serverPresets: ServerQuickPreset[] = [
  { id: 'hypixel', name: 'Hypixel Network', host: 'mc.hypixel.net', port: 25565, badge: 'PvP & BedWars' },
  { id: '2b2t', name: '2b2t Anarchy', host: '2b2t.org', port: 25565, badge: 'Anarchy' },
  { id: 'gomme', name: 'GommeHD', host: 'gommehd.net', port: 25565, badge: 'EU Hub' },
  { id: 'cubecraft', name: 'CubeCraft', host: 'play.cubecraft.net', port: 25565, badge: 'Minigames' },
  { id: 'local_lan', name: 'Local Server', host: '127.0.0.1', port: 25565, badge: 'LAN / Tunnel' },
]

export function useBooster() {
  const pingQuality = computed(() => {
    const ms = currentMetric.value.currentPingMs
    if (ms <= 0) return { label: t('Standby'), color: 'text-white/40', badge: 'bg-white/5 border-white/10' }
    if (ms < 35) return { label: t('Ideal (Esports)'), color: 'text-emerald-400', badge: 'bg-emerald-500/15 border-emerald-500/40' }
    if (ms < 80) return { label: t('Good Latency'), color: 'text-cyan-400', badge: 'bg-cyan-500/15 border-cyan-500/40' }
    if (ms < 150) return { label: t('Moderate Latency'), color: 'text-amber-400', badge: 'bg-amber-500/15 border-amber-500/40' }
    return { label: t('High Latency'), color: 'text-rose-400', badge: 'bg-rose-500/15 border-rose-500/40' }
  })

  function selectServerPreset(preset: ServerQuickPreset): void {
    targetHostInput.value = preset.host
    targetPortInput.value = preset.port
    showToast(t('Server Selected'), `${preset.name} (${preset.host})`, 'info')
  }

  async function detectVpnStatus(): Promise<void> {
    try {
      vpnStatus.value = await invoke<VpnDetectionDto>('booster_ping_master_detect_vpn')
    } catch {}
  }

  async function fetchPingMasterConfig(): Promise<void> {
    try {
      pingMasterConfig.value = await invoke<PingMasterConfigDto>('booster_ping_master_get_config')
    } catch {}
  }

  async function commitPingMasterConfig(): Promise<void> {
    try {
      await invoke('booster_ping_master_set_config', { config: pingMasterConfig.value })
      showToast(t('Ping-Master'), t('Network socket QoS policy saved.'), 'success')
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function fetchChunkAcceleratorStatus(): Promise<void> {
    try {
      chunkAcceleratorStatus.value = await invoke<ChunkAcceleratorStatusDto>('booster_chunk_accelerator_status')
    } catch {}
  }

  async function toggleChunkAccelerator(enable: boolean): Promise<void> {
    try {
      await invoke('booster_chunk_accelerator_toggle', { enable })
      await fetchChunkAcceleratorStatus()
      showToast(t('Chunk Matrix'), enable ? t('AVX2/SIMD Engine Engaged.') : t('Engine Disengaged.'), 'info')
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function triggerChunkPrebake(centerX = 0, centerZ = 0, radius = 16): Promise<void> {
    if (isPrebakingChunks.value) return
    isPrebakingChunks.value = true
    chunkPrebakeProgress.value = {
      currentChunk: 'Priming vector pipeline...',
      generatedCount: 0,
      totalTargetChunks: 0,
      percent: 0,
      speedChunksPerSec: 0,
    }

    const channel = new Channel<ChunkPrebakeProgressDto>()
    channel.onmessage = (progress) => {
      chunkPrebakeProgress.value = progress
    }

    try {
      const res = await invoke<ChunkPrebakeResultDto>('booster_chunk_accelerator_prebake_stream', {
        centerX,
        centerZ,
        radius,
        progressChannel: channel,
      })
      showToast(t('Pre-baking Finalized'), res.message, 'success')
      await fetchChunkAcceleratorStatus()
    } catch (err: unknown) {
      showToast(t('Chunk Pre-bake Fault'), String(err), 'danger')
    } finally {
      isPrebakingChunks.value = false
    }
  }

  async function flushRingBuffer(): Promise<void> {
    isFlushingRing.value = true
    try {
      const res = await invoke<ChunkRingFlushResultDto>('booster_chunk_accelerator_flush_ring')
      showToast(t('Ring Buffer Flushed'), res.message, 'success')
      await fetchChunkAcceleratorStatus()
    } catch (err: unknown) {
      showToast(t('Flush Error'), String(err), 'danger')
    } finally {
      isFlushingRing.value = false
    }
  }

  async function fetchMemoryMatrixStatus(): Promise<void> {
    try {
      memoryMatrixStatus.value = await invoke<MemoryMatrixStatusDto>('booster_memory_matrix_status')
    } catch {}
  }

  async function toggleLargePages(enable: boolean): Promise<void> {
    try {
      await invoke('booster_memory_matrix_toggle_large_pages', { enable })
      await fetchMemoryMatrixStatus()
      showToast(
        t('Large Pages'),
        enable ? t('2MB HugeTLB allocation enabled.') : t('Default 4KB paging restored.'),
        'info'
      )
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function toggleCompactHeaders(enable: boolean): Promise<void> {
    try {
      await invoke('booster_memory_matrix_toggle_compact_headers', { enable })
      await fetchMemoryMatrixStatus()
      showToast(
        t('Compact Headers'),
        enable ? t('Project Lilliput 8-byte object headers armed.') : t('Standard headers active.'),
        'info'
      )
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function grantSeLockPrivilege(): Promise<void> {
    isGrantingPrivilege.value = true
    try {
      const success = await invoke<boolean>('booster_memory_matrix_grant_privilege')
      if (success) {
        showToast(
          t('Kernel Privilege Granted'),
          t('SeLockMemoryPrivilege assigned. Policy configured via LSA.'),
          'success'
        )
        await fetchMemoryMatrixStatus()
      } else {
        showToast(t('Error'), t('Administrator privileges required to assign SeLockMemoryPrivilege.'), 'danger')
      }
    } catch (err: unknown) {
      showToast(t('Privilege Fault'), String(err), 'danger')
    } finally {
      isGrantingPrivilege.value = false
    }
  }

  async function prefaultMemoryMatrix(): Promise<void> {
    isPrefaulting.value = true
    try {
      const res = await invoke<MemoryPrefaultResultDto>('booster_memory_matrix_prefault_cache')
      showToast(t('Zero-Copy Standby Ready'), res.message, 'success')
      await fetchMemoryMatrixStatus()
    } catch (err: unknown) {
      showToast(t('Prefault Error'), String(err), 'danger')
    } finally {
      isPrefaulting.value = false
    }
  }

  async function fetchHardwareBoosterData(): Promise<void> {
    try {
      boosterConfig.value = await invoke<HardwareBoosterConfigDto>('get_hardware_booster_config')
      boosterStatus.value = await invoke<HardwareBoosterStatusDto>('get_hardware_booster_status')
    } catch {}
  }

  async function updateBoosterConfig(): Promise<void> {
    try {
      await invoke('set_hardware_booster_config', { config: boosterConfig.value })
      boosterStatus.value = await invoke<HardwareBoosterStatusDto>('get_hardware_booster_status')
      showToast(t('Hardware Booster'), t('Configuration applied to engine.'), 'success')
    } catch (err: unknown) {
      showToast(t('Error'), String(err), 'danger')
    }
  }

  async function startPingMaster(): Promise<void> {
    if (!targetHostInput.value.trim()) return

    const channel = new Channel<PingMasterMetricDto>()
    channel.onmessage = (metric: PingMasterMetricDto) => {
      currentMetric.value = metric
      if (metric.currentPingMs > 0) {
        pingHistory.value.push(metric.currentPingMs)
        if (pingHistory.value.length > 25) {
          pingHistory.value.shift()
        }
      }
    }

    try {
      const res = await invoke<PingMasterStatusDto>('booster_ping_master_start', {
        targetHost: targetHostInput.value.trim(),
        targetPort: targetPortInput.value || 25565,
        config: pingMasterConfig.value,
        channel,
      })
      isPingMasterRunning.value = res.running
      showToast(t('Zero-Latency Route Active'), res.message, 'success')
    } catch (err: unknown) {
      showToast(t('Socket Fault'), String(err), 'danger')
      isPingMasterRunning.value = false
    }
  }

  async function stopPingMaster(): Promise<void> {
    try {
      await invoke('booster_ping_master_stop')
      isPingMasterRunning.value = false
      showToast(t('Proxy Disengaged'), t('Routing restored to default adapter.'), 'info')
    } catch {}
  }

  async function triggerAssetTranscode(): Promise<void> {
    if (isTranscoding.value) return
    isTranscoding.value = true
    transcodeProgress.value = {
      currentFile: 'Starting Direct-to-VRAM texture pipeline...',
      processedCount: 0,
      totalFiles: 0,
      percent: 0,
      vramSavedMb: 0,
    }

    const channel = new Channel<TranscodeProgressDto>()
    channel.onmessage = (progress) => {
      transcodeProgress.value = progress
    }

    try {
      const res = await invoke<TranscodeResultDto>('execute_asset_transcode_stream', {
        progressChannel: channel,
      })
      showToast(t('Direct-to-VRAM Ready'), res.message, 'success')
      await fetchHardwareBoosterData()
    } catch (err: unknown) {
      showToast(t('Transcode Fault'), String(err), 'danger')
    } finally {
      isTranscoding.value = false
    }
  }

  async function triggerDefenderToggle(enable: boolean): Promise<void> {
    try {
      const success = await invoke<boolean>('trigger_manual_defender_bypass', { enable })
      if (success) {
        showToast(
          t('Defender Bypass'),
          enable ? t('Instance and Java processes exempted.') : t('Exclusions revoked.'),
          'success'
        )
        await fetchHardwareBoosterData()
      }
    } catch (err: unknown) {
      showToast(t('Security Fault'), String(err), 'danger')
    }
  }

  async function triggerWorkingSetTrim(): Promise<void> {
    try {
      const ok = await invoke<boolean>('trigger_manual_working_set_trim')
      if (ok) {
        showToast(t('Memory Trimmed'), t('Launcher working set evacuated (250MB+ freed).'), 'success')
      }
    } catch {}
  }

  async function triggerPageCacheWarmup(): Promise<void> {
    isWarmingPageCache.value = true
    try {
      const res = await invoke<PageCacheWarmupResultDto>('trigger_page_cache_warmup')
      showToast(t('Page Cache Primed'), res.message, 'success')
    } catch (err: unknown) {
      showToast(t('Page Cache Error'), String(err), 'danger')
    } finally {
      isWarmingPageCache.value = false
    }
  }

  async function triggerAppCdsDump(): Promise<void> {
    try {
      const ok = await invoke<boolean>('trigger_app_cds_dump')
      if (ok) {
        showToast(t('AppCDS Matrix'), t('AppCDS shared archive regeneration armed for next boot.'), 'success')
        await fetchHardwareBoosterData()
      }
    } catch (err: unknown) {
      showToast(t('AppCDS Error'), String(err), 'danger')
    }
  }

  return {
    chunkAcceleratorStatus,
    isPrebakingChunks,
    isFlushingRing,
    chunkPrebakeProgress,
    memoryMatrixStatus,
    isPrefaulting,
    isGrantingPrivilege,
    isPingMasterRunning,
    targetHostInput,
    targetPortInput,
    pingMasterConfig,
    vpnStatus,
    currentMetric,
    pingHistory,
    serverPresets,
    pingQuality,
    boosterConfig,
    boosterStatus,
    isTranscoding,
    transcodeProgress,
    isWarmingPageCache,
    selectServerPreset,
    detectVpnStatus,
    fetchPingMasterConfig,
    commitPingMasterConfig,
    fetchChunkAcceleratorStatus,
    toggleChunkAccelerator,
    triggerChunkPrebake,
    flushRingBuffer,
    fetchMemoryMatrixStatus,
    toggleLargePages,
    toggleCompactHeaders,
    grantSeLockPrivilege,
    prefaultMemoryMatrix,
    fetchHardwareBoosterData,
    updateBoosterConfig,
    startPingMaster,
    stopPingMaster,
    triggerAssetTranscode,
    triggerDefenderToggle,
    triggerWorkingSetTrim,
    triggerPageCacheWarmup,
    triggerAppCdsDump,
  }
}