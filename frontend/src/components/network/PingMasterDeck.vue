<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue'
import {
  Zap,
  Play,
  Square,
  Copy,
  FolderSync,
  Sparkles,
  Cpu,
  Clock,
  ShieldAlert,
  FastForward,
  Loader,
  Minimize2,
  MemoryStick,
  Binary,
  Layers3,
  KeyRound,
  Compass,
  Database,
  Shield,
  Sliders,
  Network,
} from 'lucide-vue-next'
import { showToast, t } from '@/store'
import { useBooster } from '../../composables/useBooster'

const targetPrebakeRadius = ref<number>(16)
const prebakeCenterX = ref<number>(0)
const prebakeCenterZ = ref<number>(0)
const showAdvancedNetworkModal = ref<boolean>(false)

const {
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
} = useBooster()

const sparklineSvgPoints = computed(() => {
  const data = pingHistory.value
  const step = 280 / Math.max(data.length - 1, 1)
  const maxVal = Math.max(...data, 60)
  return data
    .map((val, idx) => {
      const x = idx * step
      const y = 45 - (val / maxVal) * 38
      return `${x},${y}`
    })
    .join(' ')
})

async function copyOptimizedLocalAddress(): Promise<void> {
  if (!currentMetric.value.localProxyPort) return
  const addr = `127.0.0.1:${currentMetric.value.localProxyPort}`
  await navigator.clipboard.writeText(addr)
  showToast(t('Copied to clipboard!'), `${t('Connect to endpoint')}: ${addr}`, 'success')
}

onMounted(async () => {
  await detectVpnStatus()
  await fetchChunkAcceleratorStatus()
  await fetchMemoryMatrixStatus()
  await fetchHardwareBoosterData()
})

onBeforeUnmount(() => {
  if (isPingMasterRunning.value) {
    stopPingMaster()
  }
})
</script>

<template>
  <div class="flex flex-col gap-6">
    <section class="kip-card p-7 border border-emerald-500/40 bg-gradient-to-br from-emerald-950/40 via-black/80 to-[#020205] shadow-[0_0_60px_rgba(16,185,129,0.15)] relative overflow-hidden">
      <div class="absolute -right-32 -top-32 w-96 h-96 bg-emerald-500/10 blur-[130px] rounded-full pointer-events-none"></div>

      <div class="flex justify-between items-start mb-6 relative z-10">
        <div class="flex items-center gap-4">
          <div class="p-3.5 bg-emerald-500/20 rounded-2xl border border-emerald-500/40 text-emerald-400 shadow-[0_0_25px_rgba(16,185,129,0.4)]">
            <Compass class="w-8 h-8 stroke-[2.2] animate-pulse" />
          </div>
          <div>
            <div class="flex items-center gap-2 mb-1">
              <span class="text-[9px] font-mono uppercase tracking-[0.25em] text-emerald-400 font-black">Zero-GC Native Geometry Engine</span>
              <span class="px-2 py-0.5 rounded-full text-[9px] font-mono font-bold bg-emerald-500/10 text-emerald-300 border border-emerald-500/30">
                {{ chunkAcceleratorStatus.engineSignature }}
              </span>
            </div>
            <h3 class="text-2xl font-black uppercase text-white tracking-wider flex items-center gap-3">
              <span>Quantum Chunk Matrix</span>
              <span class="text-xs px-2.5 py-0.5 rounded-xl bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-mono">
                {{ chunkAcceleratorStatus.simdCapabilities.simdTier }}
              </span>
            </h3>
          </div>
        </div>

        <div class="flex items-center gap-3">
          <button
            @click="flushRingBuffer"
            :disabled="isFlushingRing"
            class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/10 flex items-center gap-2 cursor-pointer"
          >
            <Loader v-if="isFlushingRing" class="w-3.5 h-3.5 animate-spin" />
            <Database v-else class="w-3.5 h-3.5" />
            <span>Flush MCA Ring ({{ chunkAcceleratorStatus.ringBufferPendingChunks }})</span>
          </button>

          <input
            type="checkbox"
            :checked="chunkAcceleratorStatus.isActive"
            @change="toggleChunkAccelerator(!chunkAcceleratorStatus.isActive)"
            class="accent-emerald-400 w-5 h-5 cursor-pointer ml-2"
          >
        </div>
      </div>

      <div class="grid grid-cols-4 gap-4 mb-6 relative z-10">
        <div class="bg-black/60 p-4 rounded-2xl border border-white/5 flex flex-col justify-between">
          <span class="text-[9px] font-mono uppercase text-white/40 font-bold">SIMD Vector Processing</span>
          <span class="text-xl font-black text-emerald-400 mt-1 font-mono">
            {{ chunkAcceleratorStatus.simdCapabilities.vectorWidthBits }}-bit AVX Math
          </span>
          <span class="text-[9px] font-mono text-white/50 mt-1">Parallel Voxel Columns</span>
        </div>

        <div class="bg-black/60 p-4 rounded-2xl border border-white/5 flex flex-col justify-between">
          <span class="text-[9px] font-mono uppercase text-white/40 font-bold">2D Checkerboard Topology</span>
          <span class="text-sm font-black text-cyan-300 mt-1 font-mono truncate">
            {{ chunkAcceleratorStatus.topologyMetrics.phaseName }}
          </span>
          <span class="text-[9px] font-mono text-cyan-400/80 mt-1">
            {{ chunkAcceleratorStatus.topologyMetrics.resolvedCascadesCount }} Cascades Erased
          </span>
        </div>

        <div class="bg-black/60 p-4 rounded-2xl border border-white/5 flex flex-col justify-between">
          <span class="text-[9px] font-mono uppercase text-white/40 font-bold">Off-Heap Voxel Arena</span>
          <span class="text-xl font-black text-purple-300 mt-1 font-mono">
            {{ chunkAcceleratorStatus.arenaMetrics.usedMemoryMb }}M / {{ chunkAcceleratorStatus.arenaMetrics.totalAllocatedMb }}M
          </span>
          <span class="text-[9px] font-mono text-purple-400 mt-1">
            Zero-GC Heap Bypass: {{ chunkAcceleratorStatus.arenaMetrics.zeroGcEfficiencyPercent }}%
          </span>
        </div>

        <div class="bg-black/60 p-4 rounded-2xl border border-white/5 flex flex-col justify-between">
          <span class="text-[9px] font-mono uppercase text-white/40 font-bold">Lock-Free MCA Ring Stream</span>
          <span class="text-xl font-black text-amber-400 mt-1 font-mono">
            {{ chunkAcceleratorStatus.totalBytesStreamedMb }} MB
          </span>
          <span class="text-[9px] font-mono text-white/50 mt-1">
            {{ chunkAcceleratorStatus.totalChunksBuffered }} Chunks Dispatched
          </span>
        </div>
      </div>

      <div class="p-5 bg-black/70 rounded-2xl border border-emerald-500/30 flex items-center justify-between gap-5 relative z-10">
        <div class="flex items-center gap-4">
          <div class="flex flex-col">
            <span class="text-[9px] font-mono uppercase text-white/40 font-bold">Target Center Coordinates</span>
            <div class="flex gap-2 mt-1">
              <input v-model.number="prebakeCenterX" type="number" placeholder="X" class="kip-input py-1.5 px-3 text-xs font-mono w-20 text-center">
              <input v-model.number="prebakeCenterZ" type="number" placeholder="Z" class="kip-input py-1.5 px-3 text-xs font-mono w-20 text-center">
            </div>
          </div>

          <div class="flex flex-col">
            <span class="text-[9px] font-mono uppercase text-white/40 font-bold">Cone Radius</span>
            <div class="flex gap-1.5 mt-1">
              <button
                v-for="r in [8, 16, 24, 32]"
                :key="r"
                @click="targetPrebakeRadius = r"
                class="px-2.5 py-1.5 rounded-lg text-xs font-mono font-bold transition border cursor-pointer"
                :class="targetPrebakeRadius === r ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/50' : 'bg-black/40 text-white/50 border-white/10 hover:border-white/30'"
              >
                {{ r }}c
              </button>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-4 flex-1 justify-end max-w-md">
          <div v-if="isPrebakingChunks" class="flex flex-col gap-1 flex-1">
            <div class="flex justify-between text-xs font-mono">
              <span class="text-emerald-400 font-bold">Synthesizing: {{ chunkPrebakeProgress.currentChunk }}</span>
              <span class="text-white font-bold">{{ Math.round(chunkPrebakeProgress.percent) }}% ({{ chunkPrebakeProgress.speedChunksPerSec }} c/s)</span>
            </div>
            <div class="w-full h-2 bg-black/80 rounded-full overflow-hidden border border-white/10">
              <div class="h-full bg-gradient-to-r from-emerald-500 to-cyan-400 transition-all duration-75" :style="{ width: `${chunkPrebakeProgress.percent}%` }"></div>
            </div>
          </div>

          <button
            @click="triggerChunkPrebake(prebakeCenterX, prebakeCenterZ, targetPrebakeRadius)"
            :disabled="isPrebakingChunks"
            class="kip-btn-primary px-6 py-3.5 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(16,185,129,0.4)] cursor-pointer flex items-center gap-2"
          >
            <Loader v-if="isPrebakingChunks" class="w-4 h-4 animate-spin" />
            <Zap v-else class="w-4 h-4 fill-current" />
            <span>{{ isPrebakingChunks ? 'Vector Synthesizing...' : 'Pre-Bake Directional Cone' }}</span>
          </button>
        </div>
      </div>
    </section>

    <section class="grid grid-cols-3 gap-5">
      <article class="kip-card p-6 border border-cyan-500/30 bg-gradient-to-br from-cyan-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-cyan-500/20 rounded-2xl border border-cyan-500/40 text-cyan-400 shadow-[0_0_20px_rgba(6,182,212,0.35)]">
                <Clock class="w-6 h-6 stroke-[2.2]" />
              </div>
              <div>
                <h4 class="text-base font-black uppercase text-white tracking-wider">0.5ms Timer Precision</h4>
                <span class="text-[10px] text-white/50 block font-mono">Win32 Multimedia API timeBeginPeriod</span>
              </div>
            </div>
            <input
              type="checkbox"
              v-model="boosterConfig.timerResolutionEnabled"
              @change="updateBoosterConfig"
              class="accent-cyan-400 w-4 h-4 cursor-pointer"
            >
          </div>
          <p class="text-xs text-white/70 leading-relaxed font-sans">
            Bypasses the standard 15.6ms Windows kernel scheduler jitter. Synchronizes frame intervals and eliminates micro-stutters during un-synced display refresh.
          </p>
        </div>
        <div class="pt-4 border-t border-white/5 mt-4 flex items-center justify-between text-xs font-mono">
          <span class="text-white/40 uppercase text-[9px] font-bold">Scheduler Resolution:</span>
          <span class="text-cyan-400 font-black">{{ boosterStatus.timerActive ? '0.50 ms' : '15.60 ms' }}</span>
        </div>
      </article>

      <article class="kip-card p-6 border border-rose-500/30 bg-gradient-to-br from-rose-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-rose-500/20 rounded-2xl border border-rose-500/40 text-rose-400 shadow-[0_0_20px_rgba(244,63,94,0.35)]">
                <ShieldAlert class="w-6 h-6 stroke-[2.2]" />
              </div>
              <div>
                <h4 class="text-base font-black uppercase text-white tracking-wider">Defender I/O Bypass</h4>
                <span class="text-[10px] text-white/50 block font-mono">Real-time Exclusion Injection</span>
              </div>
            </div>
            <input
              type="checkbox"
              v-model="boosterConfig.defenderBypassEnabled"
              @change="updateBoosterConfig"
              class="accent-rose-400 w-4 h-4 cursor-pointer"
            >
          </div>
          <p class="text-xs text-white/70 leading-relaxed font-sans">
            Prevents Windows Defender from inspecting 80,000+ class files on startup. Cold modpack load time accelerated up to +100% without security trade-offs.
          </p>
        </div>
        <div class="pt-4 border-t border-white/5 mt-4 flex items-center justify-between text-xs font-mono">
          <button
            @click="triggerDefenderToggle(true)"
            class="text-[10px] uppercase font-bold text-rose-400 hover:underline cursor-pointer"
          >
            Apply Exclusions
          </button>
          <span class="text-white/40 text-[9px]">Autocleans on exit</span>
        </div>
      </article>

      <article class="kip-card p-6 border border-purple-500/30 bg-gradient-to-br from-purple-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-purple-500/20 rounded-2xl border border-purple-500/40 text-purple-400 shadow-[0_0_20px_rgba(168,85,247,0.35)]">
                <Cpu class="w-6 h-6 stroke-[2.2]" />
              </div>
              <div>
                <h4 class="text-base font-black uppercase text-white tracking-wider">Priority & Sleep Mode</h4>
                <span class="text-[10px] text-white/50 block font-mono">Process Sleep</span>
              </div>
            </div>
            <input
              type="checkbox"
              v-model="boosterConfig.processPriorityBoost"
              @change="updateBoosterConfig"
              class="accent-purple-400 w-4 h-4 cursor-pointer"
            >
          </div>
          <p class="text-xs text-white/70 leading-relaxed font-sans">
            Elevates Minecraft to HIGH_PRIORITY_CLASS and evacuates launcher RAM.
          </p>
        </div>
        <div class="pt-4 border-t border-white/5 mt-4 flex items-center justify-between text-xs font-mono">
          <button @click="triggerWorkingSetTrim" class="text-[10px] text-purple-400 font-bold uppercase hover:underline cursor-pointer flex items-center gap-1">
            <Minimize2 class="w-3 h-3" />
            <span>Trim</span>
          </button>
          <span class="text-white/40 text-[9px]">0% CPU Sleep</span>
        </div>
      </article>
    </section>

    <section class="grid grid-cols-4 gap-5">
      <article class="kip-card p-6 border border-indigo-500/30 bg-gradient-to-br from-indigo-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-3">
          <div class="flex justify-between items-start">
            <div class="p-2.5 bg-indigo-500/20 rounded-2xl border border-indigo-500/40 text-indigo-400">
              <FastForward class="w-5 h-5 stroke-[2.2]" />
            </div>
            <div>
              <h4 class="text-sm font-black uppercase text-white tracking-wider">Project CRaC</h4>
              <span class="text-[9px] text-white/50 block font-mono">JVM Checkpoint</span>
            </div>
          </div>
          <input
            type="checkbox"
            v-model="boosterConfig.cracAccelerationEnabled"
            :disabled="!boosterStatus.cracSupported"
            @change="updateBoosterConfig"
            class="accent-indigo-400 w-4 h-4 cursor-pointer disabled:opacity-30 disabled:cursor-not-allowed"
          >
        </div>
        <p class="text-[11px] text-white/70 leading-relaxed font-sans">
          {{ boosterStatus.cracSupported ? 'Coordinated checkpoint restore bypassing HotSpot boot loops on Liberica/Zulu JVM.' : 'Project CRaC requires Linux CRIU kernel interfaces. Unavailable on standard Windows HotSpot runtimes.' }}
        </p>
        <div class="pt-3 border-t border-white/5 flex items-center justify-between text-xs font-mono">
          <span class="text-white/40 uppercase text-[9px]">Status:</span>
          <span class="font-bold text-[10px]" :class="boosterStatus.cracSupported ? 'text-indigo-400' : 'text-amber-400/80'">
            {{ boosterStatus.cracSupported ? (boosterStatus.cracCheckpointExists ? 'Active' : 'Unmounted') : 'Linux Only' }}
          </span>
        </div>
      </article>

      <article class="kip-card p-6 border border-blue-500/30 bg-gradient-to-br from-blue-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-3">
          <div class="flex justify-between items-start">
            <div class="p-2.5 bg-blue-500/20 rounded-2xl border border-blue-500/40 text-blue-400">
              <Binary class="w-5 h-5 stroke-[2.2]" />
            </div>
            <div>
              <h4 class="text-sm font-black uppercase text-white tracking-wider">AppCDS Engine</h4>
              <span class="text-[9px] text-white/50 block font-mono">Classes.jsa Archive</span>
            </div>
          </div>
          <input
            type="checkbox"
            v-model="boosterConfig.appCdsEnabled"
            @change="updateBoosterConfig"
            class="accent-blue-400 w-4 h-4 cursor-pointer"
          >
        </div>
        <p class="text-[11px] text-white/70 leading-relaxed font-sans">
          Stores loaded class bytecode metadata into an OS shared memory mapping.
        </p>
        <div class="pt-3 border-t border-white/5 flex items-center justify-between text-xs font-mono">
          <button @click="triggerAppCdsDump" class="text-[10px] text-blue-400 font-bold uppercase hover:underline cursor-pointer">Rebuild</button>
          <span class="text-white/40 text-[10px]">{{ boosterStatus.appCdsArchiveExists ? `${boosterStatus.appCdsArchiveSizeMb} MB` : 'Cold' }}</span>
        </div>
      </article>

      <article class="kip-card p-6 border border-amber-500/30 bg-gradient-to-br from-amber-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-3">
          <div class="flex justify-between items-start">
            <div class="p-2.5 bg-amber-500/20 rounded-2xl border border-amber-500/40 text-amber-400">
              <Layers3 class="w-5 h-5 stroke-[2.2]" />
            </div>
            <div>
              <h4 class="text-sm font-black uppercase text-white tracking-wider">Direct-to-VRAM</h4>
              <span class="text-[10px] text-white/50 block font-mono">BC7 Transcoder</span>
            </div>
          </div>
          <input
            type="checkbox"
            v-model="boosterConfig.directVramTranscode"
            @change="updateBoosterConfig"
            class="accent-amber-400 w-4 h-4 cursor-pointer"
          >
        </div>
        <p class="text-[11px] text-white/70 leading-relaxed font-sans">
          Transcodes PNG textures into native GPU memory blocks with mip levels.
        </p>
        <div class="pt-3 border-t border-white/5 flex items-center justify-between text-xs font-mono">
          <button @click="triggerAssetTranscode" :disabled="isTranscoding" class="text-[10px] text-amber-400 font-bold uppercase hover:underline cursor-pointer flex items-center gap-1">
            <Loader v-if="isTranscoding" class="w-3 h-3 animate-spin" />
            <span>{{ isTranscoding ? `${Math.round(transcodeProgress.percent)}%` : 'Run' }}</span>
          </button>
          <span class="text-white/40 text-[10px]">{{ transcodeProgress.vramSavedMb > 0 ? `${transcodeProgress.vramSavedMb} MB` : 'Standby' }}</span>
        </div>
      </article>

      <article class="kip-card p-6 border border-rose-500/30 bg-gradient-to-br from-rose-950/30 via-black/60 to-black/80 flex flex-col justify-between shadow-2xl relative overflow-hidden">
        <div class="flex flex-col gap-3">
          <div class="flex justify-between items-start">
            <div class="p-2.5 bg-rose-500/20 rounded-2xl border border-rose-500/40 text-rose-400">
              <MemoryStick class="w-5 h-5 stroke-[2.2]" />
            </div>
            <div>
              <h4 class="text-sm font-black uppercase text-white tracking-wider">Page Cache Pre-fetch</h4>
              <span class="text-[10px] text-white/50 block font-mono">Standby List Warming</span>
            </div>
          </div>
          <input
            type="checkbox"
            v-model="boosterConfig.pageCacheWarmupEnabled"
            @change="updateBoosterConfig"
            class="accent-rose-400 w-4 h-4 cursor-pointer"
          >
        </div>
        <p class="text-[11px] text-white/70 leading-relaxed font-sans">
          Pre-reads the initial 64KB headers of all JAR modules in parallel before process launch.
        </p>
        <div class="pt-3 border-t border-white/5 flex items-center justify-between text-xs font-mono">
          <button @click="triggerPageCacheWarmup" :disabled="isWarmingPageCache" class="text-[10px] text-rose-400 font-bold uppercase hover:underline cursor-pointer flex items-center gap-1">
            <Loader v-if="isWarmingPageCache" class="w-3 h-3 animate-spin" />
            <span>{{ isWarmingPageCache ? 'Warming...' : 'Pre-fetch' }}</span>
          </button>
          <span class="text-white/40 text-[10px]">OS Buffered</span>
        </div>
      </article>
    </section>

    <section class="grid grid-cols-2 gap-6">
      <article class="kip-card p-6 border border-emerald-500/30 bg-gradient-to-br from-emerald-950/20 via-black/60 to-black/80 shadow-2xl flex flex-col justify-between">
        <div class="flex flex-col gap-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-emerald-500/20 rounded-2xl border border-emerald-500/40 text-emerald-400 shadow-[0_0_20px_rgba(16,185,129,0.35)]">
                <MemoryStick class="w-6 h-6 stroke-[2.2]" />
              </div>
              <div>
                <h3 class="text-xl font-black uppercase text-white tracking-wider flex items-center gap-2">
                  <span>Kernel Memory Matrix</span>
                  <span class="text-[9px] font-mono px-2 py-0.5 rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/40">HugeTLB 2MB</span>
                </h3>
                <span class="text-[10px] text-white/50 block mt-0.5">Hardware Large Pages & Compact Object Headers Engine</span>
              </div>
            </div>

            <span
              class="px-3 py-1 rounded-xl text-[10px] font-mono uppercase font-black border"
              :class="memoryMatrixStatus.largePagesActive ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40 shadow-[0_0_12px_rgba(16,185,129,0.4)]' : 'bg-white/5 text-white/40 border-white/10'"
            >
              {{ memoryMatrixStatus.largePagesActive ? '2MB HugeTLB Active' : '4KB Paging Standard' }}
            </span>
          </div>

          <div class="p-3.5 rounded-xl border flex items-start gap-2.5 text-xs leading-relaxed bg-emerald-500/10 border-emerald-500/30">
            <Sparkles class="w-4 h-4 shrink-0 mt-0.5 text-emerald-400" />
            <span class="text-emerald-400 font-medium">
              {{ memoryMatrixStatus.statusText }}
            </span>
          </div>

          <div class="grid grid-cols-3 gap-2.5 p-3.5 bg-black/60 rounded-xl border border-white/5 font-mono text-xs">
            <div>
              <span class="text-[9px] uppercase text-white/40 font-bold block">Page Granularity</span>
              <span class="text-base font-black text-emerald-400 mt-0.5 block">{{ memoryMatrixStatus.largePagesActive ? '2048 KB' : '4 KB' }}</span>
            </div>
            <div>
              <span class="text-[9px] uppercase text-white/40 font-bold block">Heap Savings</span>
              <span class="text-base font-black text-white mt-0.5 block">~{{ memoryMatrixStatus.savedMemoryMb }} MB</span>
            </div>
            <div>
              <span class="text-[9px] uppercase text-white/40 font-bold block">TLB Hit Efficiency</span>
              <span class="text-base font-black text-cyan-400 mt-0.5 block">+{{ memoryMatrixStatus.estimatedTlbMissReductionPercent }}%</span>
            </div>
          </div>

          <div class="flex items-center justify-between p-3 bg-black/40 rounded-xl border border-white/5 text-xs font-mono">
            <div class="flex items-center gap-2">
              <KeyRound class="w-4 h-4" :class="memoryMatrixStatus.seLockPrivilegeGranted ? 'text-emerald-400' : 'text-amber-400'" />
              <span class="text-white/80">Kernel Privilege (SeLockMemoryPrivilege):</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="font-bold uppercase text-[10px]" :class="memoryMatrixStatus.seLockPrivilegeGranted ? 'text-emerald-400' : 'text-amber-400'">
                {{ memoryMatrixStatus.seLockPrivilegeGranted ? 'Granted' : 'Missing' }}
              </span>
              <button
                v-if="!memoryMatrixStatus.seLockPrivilegeGranted"
                @click="grantSeLockPrivilege"
                :disabled="isGrantingPrivilege"
                class="px-2.5 py-1 rounded bg-amber-500/20 text-amber-300 border border-amber-500/40 hover:bg-amber-500/30 text-[9px] font-bold uppercase transition cursor-pointer"
              >
                {{ isGrantingPrivilege ? 'Elevating...' : 'Assign Privilege (UAC)' }}
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between p-3 bg-black/40 rounded-xl border border-white/5 text-xs font-mono">
            <div class="flex items-center gap-2">
              <Binary class="w-4 h-4" :class="memoryMatrixStatus.compactHeadersActive ? 'text-emerald-400' : 'text-white/40'" />
              <span class="text-white/80">Compact Object Headers (Project Lilliput):</span>
            </div>
            <div class="flex items-center gap-2">
              <button
                @click="toggleCompactHeaders(!memoryMatrixStatus.compactHeadersActive)"
                class="px-2.5 py-1 rounded text-[9px] font-bold uppercase transition border cursor-pointer"
                :class="memoryMatrixStatus.compactHeadersActive ? 'bg-emerald-500/20 text-emerald-300 border-emerald-500/40' : 'bg-white/5 text-white/40 border-white/10'"
              >
                {{ memoryMatrixStatus.compactHeadersActive ? 'Armed' : 'Disabled' }}
              </button>
            </div>
          </div>
        </div>

        <div class="flex items-center gap-3 pt-4 border-t border-white/5 mt-4">
          <button
            @click="prefaultMemoryMatrix"
            :disabled="isPrefaulting"
            class="kip-btn-primary flex-1 py-3.5 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(16,185,129,0.4)] cursor-pointer flex items-center justify-center gap-2"
          >
            <Loader v-if="isPrefaulting" class="w-4 h-4 animate-spin" />
            <FolderSync v-else class="w-4 h-4 fill-current" />
            <span>{{ isPrefaulting ? 'Mapping Standby List...' : 'Prefault Zero-Copy Memory' }}</span>
          </button>

          <button
            @click="toggleLargePages(!memoryMatrixStatus.largePagesActive)"
            class="kip-btn-ghost px-5 py-3 text-xs font-bold uppercase cursor-pointer"
            :class="memoryMatrixStatus.largePagesActive ? 'text-emerald-400 border-emerald-500/40' : 'text-white/60 border-white/10'"
          >
            {{ memoryMatrixStatus.largePagesActive ? '2MB Active' : 'Enable 2MB' }}
          </button>
        </div>
      </article>

      <article class="kip-card p-6 border border-cyan-500/30 bg-gradient-to-br from-cyan-950/20 via-black/60 to-black/80 shadow-2xl flex flex-col justify-between">
        <div class="flex flex-col gap-4">
          <div class="flex justify-between items-start">
            <div class="flex items-center gap-3">
              <div class="p-3 bg-cyan-500/20 rounded-2xl border border-cyan-500/40 text-cyan-400 shadow-[0_0_20px_rgba(6,182,212,0.35)]">
                <Zap class="w-6 h-6 stroke-[2.2]" />
              </div>
              <div>
                <h3 class="text-xl font-black uppercase text-white tracking-wider flex items-center gap-2">
                  <span>Ping-Master Pro</span>
                  <span class="text-[9px] font-mono px-2 py-0.5 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-500/40">DSCP 46 / QoS</span>
                </h3>
                <span class="text-[10px] text-white/50 block mt-0.5">VPN/Proxy Auto-Router & Anti-Bufferbloat Socket</span>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <button
                @click="showAdvancedNetworkModal = !showAdvancedNetworkModal"
                class="p-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-cyan-300 transition cursor-pointer"
                title="VPN & SOCKS5 Routing Policy"
              >
                <Sliders class="w-3.5 h-3.5" />
              </button>

              <span
                class="px-3 py-1 rounded-xl text-[10px] font-mono uppercase font-black border"
                :class="isPingMasterRunning ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/40 shadow-[0_0_12px_rgba(6,182,212,0.4)] animate-pulse' : 'bg-white/5 text-white/40 border-white/10'"
              >
                {{ isPingMasterRunning ? t('Accelerator Active') : t('Ready to Engage') }}
              </span>
            </div>
          </div>

          <div
            v-if="vpnStatus.vpnActive"
            class="p-2.5 rounded-xl border flex items-center justify-between text-xs font-mono bg-purple-500/10 border-purple-500/30 text-purple-300"
          >
            <div class="flex items-center gap-2">
              <Shield class="w-4 h-4 text-purple-400 animate-pulse" />
              <span>VPN Detected: <strong>{{ vpnStatus.activeAdapterName || 'Virtual TUN/TAP' }}</strong></span>
            </div>
            <span class="text-[9px] uppercase font-bold text-purple-400 bg-purple-500/20 px-2 py-0.5 rounded">
              Route: {{ pingMasterConfig.routingMode }}
            </span>
          </div>

          <div v-if="showAdvancedNetworkModal" class="p-4 bg-black/80 rounded-2xl border border-cyan-500/40 flex flex-col gap-3">
            <div class="flex justify-between items-center text-xs font-mono font-bold text-cyan-300">
              <span class="flex items-center gap-1.5"><Network class="w-3.5 h-3.5" /> Upstream Route Policy</span>
              <button @click="showAdvancedNetworkModal = false" class="text-white/40 hover:text-white cursor-pointer">✕</button>
            </div>

            <div class="grid grid-cols-4 gap-1.5 font-mono text-[10px]">
              <button
                v-for="mode in (['adaptive', 'direct', 'vpn', 'socks5'] as const)"
                :key="mode"
                @click="pingMasterConfig.routingMode = mode; commitPingMasterConfig()"
                class="py-1.5 rounded-lg uppercase font-bold border transition cursor-pointer"
                :class="pingMasterConfig.routingMode === mode ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50' : 'bg-black/50 text-white/50 border-white/5'"
              >
                {{ mode }}
              </button>
            </div>

            <div v-if="pingMasterConfig.routingMode === 'socks5'" class="grid grid-cols-3 gap-2 pt-1 border-t border-white/5">
              <input v-model="pingMasterConfig.upstreamSocks5Host" @blur="commitPingMasterConfig" type="text" placeholder="127.0.0.1" class="kip-input py-1 text-[10px] font-mono col-span-2">
              <input v-model.number="pingMasterConfig.upstreamSocks5Port" @blur="commitPingMasterConfig" type="number" placeholder="7890" class="kip-input py-1 text-[10px] font-mono text-center">
            </div>

            <div class="flex justify-between items-center pt-1 border-t border-white/5 text-[10px] font-mono text-white/70">
              <label class="flex items-center gap-1.5 cursor-pointer">
                <input type="checkbox" v-model="pingMasterConfig.antiBufferbloat" @change="commitPingMasterConfig" class="accent-cyan-400">
                <span>Anti-Bufferbloat (8KB Queues)</span>
              </label>
              <label class="flex items-center gap-1.5 cursor-pointer">
                <input type="checkbox" v-model="pingMasterConfig.dscpQosEnabled" @change="commitPingMasterConfig" class="accent-cyan-400">
                <span>DSCP 46 Expedited QoS</span>
              </label>
            </div>
          </div>

          <div class="flex flex-col gap-1.5">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Quick Server Preset:') }}</label>
            <div class="flex flex-wrap gap-1.5">
              <button
                v-for="p in serverPresets"
                :key="p.id"
                @click="selectServerPreset(p)"
                :disabled="isPingMasterRunning"
                class="px-2.5 py-1 rounded-lg text-xs font-medium bg-black/60 hover:bg-cyan-500/20 hover:text-cyan-300 border border-white/10 transition cursor-pointer disabled:opacity-40"
              >
                {{ p.name }}
              </button>
            </div>
          </div>

          <div class="flex gap-2">
            <input
              v-model="targetHostInput"
              :disabled="isPingMasterRunning"
              type="text"
              placeholder="mc.hypixel.net"
              class="kip-input text-xs font-mono flex-1"
            >
            <input
              v-model.number="targetPortInput"
              :disabled="isPingMasterRunning"
              type="number"
              placeholder="25565"
              class="kip-input text-xs font-mono w-24 text-center"
            >
            <button
              v-if="!isPingMasterRunning"
              @click="startPingMaster"
              class="kip-btn-primary px-6 bg-cyan-500 hover:bg-cyan-400 text-black font-black uppercase text-xs cursor-pointer"
            >
              <Play class="w-4 h-4 fill-current" />
              <span>{{ t('Start') }}</span>
            </button>
            <button
              v-else
              @click="stopPingMaster"
              class="kip-btn-danger px-6 font-black uppercase text-xs cursor-pointer"
            >
              <Square class="w-4 h-4 fill-current" />
              <span>{{ t('Stop') }}</span>
            </button>
          </div>

          <div v-if="isPingMasterRunning" class="p-3.5 bg-black/60 rounded-xl border border-cyan-500/40 flex flex-col gap-2">
            <div class="flex justify-between items-center text-xs">
              <span class="text-white/70 font-medium">{{ t('Minecraft Direct Connect Address:') }}</span>
              <button
                @click="copyOptimizedLocalAddress"
                class="px-3 py-1 rounded-lg bg-cyan-500/20 text-cyan-300 border border-cyan-500/40 font-mono font-bold flex items-center gap-1.5 hover:bg-cyan-500/30 cursor-pointer"
              >
                <span>127.0.0.1:{{ currentMetric.localProxyPort }}</span>
                <Copy class="w-3.5 h-3.5" />
              </button>
            </div>

            <div class="grid grid-cols-2 gap-2 pt-2 border-t border-white/5 text-[11px] font-mono">
              <div>
                <span class="text-white/40 block">PvP Hit-Reg Quality:</span>
                <strong class="text-emerald-400">{{ currentMetric.hitRegQuality }}</strong>
              </div>
              <div>
                <span class="text-white/40 block">Active Gateway:</span>
                <strong class="text-cyan-300 truncate block">{{ currentMetric.activeRoute }}</strong>
              </div>
            </div>

            <div class="flex justify-between items-center text-[11px] font-mono pt-1 border-t border-white/5">
              <span class="text-white/50">{{ t('Connection Quality:') }} <strong :class="pingQuality.color">{{ pingQuality.label }}</strong></span>
              <span class="text-white/50">{{ t('Ping:') }} <strong class="text-cyan-400">{{ currentMetric.currentPingMs }} ms</strong> (Loss: {{ currentMetric.packetLossPercent }}%)</span>
            </div>
          </div>
        </div>

        <div>
          <div class="flex justify-between items-center text-[10px] font-mono text-white/40 uppercase mb-1.5">
            <span>{{ t('Real-time Network Jitter Graph') }}</span>
            <span class="text-cyan-400 font-bold">{{ t('Jitter:') }} {{ currentMetric.jitterMs }} ms</span>
          </div>
          <svg class="w-full h-12 bg-black/70 rounded-xl p-1 border border-white/5" viewBox="0 0 280 45">
            <polyline
              fill="none"
              stroke="#06b6d4"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              :points="sparklineSvgPoints"
            />
          </svg>
        </div>
      </article>
    </section>
  </div>
</template>