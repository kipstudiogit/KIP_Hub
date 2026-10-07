<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed, watch } from 'vue'
import type { Component } from 'vue'
import {
  Zap,
  Loader,
  ChevronDown,
  Cpu,
  Box,
  Hexagon,
  Component as LucideComponent,
  Layers,
  CheckCircle,
  CheckCircle2,
  AlertCircle,
  AlertTriangle,
  SlidersHorizontal,
  Flame,
  FolderOpen,
  ShieldCheck,
  Wrench,
  Stethoscope,
  Maximize2,
  FileArchive,
  Edit3,
  UserCheck,
  PackagePlus,
  Play,
  RotateCcw,
  Download,
  Search,
} from 'lucide-vue-next'
import { state, t, showToast, getAvatarUrl, saveSetting } from '@/store'
import { bridge, invokeSafe } from '@/bridge'
import { useLauncher } from '../composables/useLauncher'
import type { LoaderType, LaunchPreset } from '../types/launcher'

interface McVersionManifestEntry {
  id: string
  versionType: string
  releaseTime: string
}

const customMode = ref(false)
const isVersionDropdownOpen = ref(false)
const isLoaderVerDropdownOpen = ref(false)
const isEditingNick = ref(false)
const tempNick = ref('')
const isDraggingOver = ref(false)
const selectedVersion = ref('1.21.4')
const selectedLoader = ref<LoaderType>('vanilla')
const selectedLoaderVersion = ref('')
const selectedRam = ref<number>(state.settings.ram_allocation || 0)
const selectedResolution = ref(state.settings.game_resolution || '1920x1080')
const isFullscreen = ref(Boolean(state.settings.game_fullscreen))
const customJvmInput = ref(state.settings.custom_jvm_args || '')
const loaderVersions = ref<string[]>([])
const activePresetId = ref('vanilla_pure')

const isDownloadingVersion = ref(false)
const allManifestVersions = ref<McVersionManifestEntry[]>([])
const versionSearchQuery = ref('')
const activeVersionTypeFilter = ref<'all' | 'release' | 'snapshot' | 'historical'>('release')

const {
  isLaunching,
  isRepairing,
  isDoctorGuardActive,
  preflightReport,
  currentProgress,
  hasCriticalConflicts,
  checkPreflight,
  repairEnvironment,
  ignite,
  toggleDoctorGuard,
} = useLauncher()

const presets: LaunchPreset[] = [
  {
    id: 'vanilla_pure',
    name: 'Pure Vanilla (1.21.4)',
    desc: 'Official stable Minecraft release without modifications. Maximum stability.',
    loader: 'vanilla',
    badge: '1.21.4 Stable',
    badgeClass: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    bgClass: 'bg-emerald-500',
    icon: Box,
  },
  {
    id: 'forge_heavy',
    name: 'Forge Realm (1.21.4)',
    desc: 'Enterprise modding architecture for tech, automation and magic expansions.',
    loader: 'forge',
    badge: '1.21.4 Stable',
    badgeClass: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
    bgClass: 'bg-rose-500',
    icon: Flame,
  },
  {
    id: 'fabric_fps',
    name: 'Fabric Turbo (1.21.4)',
    desc: 'Lightweight performance core engineered for high frame rates and shaders.',
    loader: 'fabric',
    badge: '1.21.4 Stable',
    badgeClass: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
    bgClass: 'bg-amber-500',
    icon: Hexagon,
  },
  {
    id: 'neoforge_blaze',
    name: 'NeoForge Matrix (1.21.4)',
    desc: 'Official stable NeoForge next-generation loader for Minecraft 1.21.4.',
    loader: 'neoforge',
    badge: '1.21.4 Stable',
    badgeClass: 'bg-orange-500/10 text-orange-400 border-orange-500/20',
    bgClass: 'bg-orange-500',
    icon: LucideComponent,
  },
  {
    id: 'quilt_flow',
    name: 'Quilt Modular (1.21.4)',
    desc: 'Open modular ecosystem supporting Fabric ecosystem and next-gen tools.',
    loader: 'quilt',
    badge: '1.21.4 Stable',
    badgeClass: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
    bgClass: 'bg-purple-500',
    icon: Layers,
  },
]

const loaders: { id: LoaderType; name: string; icon: Component }[] = [
  { id: 'vanilla', name: 'Vanilla', icon: Box },
  { id: 'forge', name: 'Forge', icon: Flame },
  { id: 'fabric', name: 'Fabric', icon: Hexagon },
  { id: 'neoforge', name: 'NeoForge', icon: LucideComponent },
  { id: 'quilt', name: 'Quilt', icon: Layers },
]

const currentUserDisplay = computed(() => {
  return state.settings.ms_name || state.settings.offline_username || 'Operator'
})

const activeLoaderIcon = computed(() => {
  const found = loaders.find((l) => l.id === selectedLoader.value)
  return found ? found.icon : Box
})

const activeLoaderGlow = computed(() => {
  switch (selectedLoader.value) {
    case 'vanilla':
      return 'from-emerald-500/20 to-transparent'
    case 'forge':
      return 'from-rose-500/20 to-transparent'
    case 'fabric':
      return 'from-amber-500/20 to-transparent'
    case 'neoforge':
      return 'from-orange-500/20 to-transparent'
    case 'quilt':
      return 'from-purple-500/20 to-transparent'
    default:
      return 'from-indigo-500/20 to-transparent'
  }
})

const activeColorText = computed(() => {
  switch (selectedLoader.value) {
    case 'vanilla':
      return 'text-emerald-400'
    case 'forge':
      return 'text-rose-400'
    case 'fabric':
      return 'text-amber-400'
    case 'neoforge':
      return 'text-orange-400'
    case 'quilt':
      return 'text-purple-400'
    default:
      return 'text-indigo-400'
  }
})

const activeColorBtn = computed(() => {
  if (isLaunching.value) return 'bg-white/10 text-white/40 border-white/10 cursor-wait'
  switch (selectedLoader.value) {
    case 'vanilla':
      return 'bg-emerald-500 text-black border-emerald-400 hover:shadow-[0_0_40px_rgba(16,185,129,0.6)]'
    case 'forge':
      return 'bg-rose-500 text-white border-rose-400 hover:shadow-[0_0_40px_rgba(244,63,94,0.6)]'
    case 'fabric':
      return 'bg-amber-500 text-black border-amber-400 hover:shadow-[0_0_40px_rgba(245,158,11,0.6)]'
    case 'neoforge':
      return 'bg-orange-500 text-black border-orange-400 hover:shadow-[0_0_40px_rgba(249,115,22,0.6)]'
    case 'quilt':
      return 'bg-purple-500 text-white border-purple-400 hover:shadow-[0_0_40px_rgba(168,85,247,0.6)]'
    default:
      return 'bg-indigo-500 text-white border-indigo-400'
  }
})

const filteredManifestVersions = computed(() => {
  const query = versionSearchQuery.value.trim().toLowerCase()
  const filter = activeVersionTypeFilter.value

  return allManifestVersions.value.filter((v) => {
    if (filter === 'release' && v.versionType !== 'release') return false
    if (filter === 'snapshot' && v.versionType !== 'snapshot') return false
    if (filter === 'historical' && (v.versionType !== 'old_beta' && v.versionType !== 'old_alpha')) return false
    if (query && !v.id.toLowerCase().includes(query)) return false
    return true
  })
})

async function fetchAllMojangVersions(): Promise<void> {
  try {
    const list = await invokeSafe<McVersionManifestEntry[]>('get_all_minecraft_versions')
    if (list && list.length > 0) {
      allManifestVersions.value = list
    }
  } catch {
    allManifestVersions.value = [
      { id: '1.21.4', versionType: 'release', releaseTime: '2024-12-03' },
      { id: '1.21.1', versionType: 'release', releaseTime: '2024-08-08' },
      { id: '1.20.1', versionType: 'release', releaseTime: '2023-06-12' },
      { id: '1.16.5', versionType: 'release', releaseTime: '2021-01-15' },
    ]
  }
}

async function triggerDownloadVersion(): Promise<void> {
  const target = selectedVersion.value.trim()
  if (!target || isDownloadingVersion.value) return

  isDownloadingVersion.value = true
  try {
    const res = await invokeSafe<{ success: boolean; version: string; message: string }>(
      'download_minecraft_version',
      { version: target }
    )
    if (res.success) {
      showToast(t('Downloaded'), res.message, 'success')
      await checkPreflight(selectedVersion.value, selectedLoader.value)
    }
  } catch (err: unknown) {
    showToast(t('Download Error'), String(err), 'danger')
  } finally {
    isDownloadingVersion.value = false
  }
}

function selectPreset(preset: LaunchPreset): void {
  activePresetId.value = preset.id
  selectedLoader.value = preset.loader
  selectedLoaderVersion.value = ''
  selectedVersion.value = '1.21.4'
  checkPreflight(selectedVersion.value, selectedLoader.value)
}

function handleIgnition(bypass = false): void {
  ignite(
    selectedVersion.value,
    selectedLoader.value,
    selectedLoaderVersion.value || null,
    selectedRam.value,
    customJvmInput.value,
    selectedResolution.value,
    isFullscreen.value,
    bypass
  )
}

async function saveGuestNick(): Promise<void> {
  isEditingNick.value = false
  const clean = tempNick.value.trim()
  if (clean && clean.length >= 3) {
    state.settings.offline_username = clean
    await saveSetting('offline_username', clean)
    showToast(t('Operator Handle Updated'), clean, 'success')
  }
}

async function fetchLoaderVersions(): Promise<void> {
  if (selectedLoader.value === 'vanilla' || !selectedVersion.value) {
    loaderVersions.value = []
    selectedLoaderVersion.value = ''
    return
  }
  selectedLoaderVersion.value = ''
  try {
    const list = await bridge.getLoaderVersions(selectedLoader.value, selectedVersion.value)
    loaderVersions.value = list || []
  } catch {
    loaderVersions.value = []
  }
}

async function handlePackageDrop(e: DragEvent): Promise<void> {
  isDraggingOver.value = false
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const files = Array.from(e.dataTransfer.files)
    const pkg = files.find((f) => {
      const n = f.name.toLowerCase()
      return n.endsWith('.zip') || n.endsWith('.mrpack') || n.endsWith('.jar')
    })

    if (pkg) {
      const p = (pkg as unknown as { path?: string }).path
      if (p) {
        showToast(t('Mounting Package'), `Injecting ${pkg.name}...`, 'info')
        try {
          const res = await invokeSafe<{ success: boolean; mc_version?: string; loader?: string; message: string }>(
            'import_modpack_or_archive',
            { filePath: p }
          )
          if (res.success) {
            showToast(t('Integrated'), res.message, 'success')
            if (res.mc_version) selectedVersion.value = res.mc_version
            if (res.loader) selectedLoader.value = res.loader as LoaderType
            await checkPreflight(selectedVersion.value, selectedLoader.value)
          }
        } catch (err: unknown) {
          showToast(t('Error'), String(err), 'danger')
        }
      }
    }
  }
}

async function pickInstallPackage(): Promise<void> {
  try {
    const file = await bridge.pickFile()
    if (file && file.trim()) {
      showToast(t('Mounting Package'), 'Integrating package container...', 'info')
      const res = await invokeSafe<{ success: boolean; mc_version?: string; loader?: string; message: string }>(
        'import_modpack_or_archive',
        { filePath: file.trim() }
      )
      if (res.success) {
        showToast(t('Integrated'), res.message, 'success')
        if (res.mc_version) selectedVersion.value = res.mc_version
        if (res.loader) selectedLoader.value = res.loader as LoaderType
        await checkPreflight(selectedVersion.value, selectedLoader.value)
      }
    }
  } catch (err: unknown) {
    showToast(t('Import Notice'), String(err), 'danger')
  }
}

function openGameFolder(): void {
  bridge.openContentFolder('mods').catch(() => {
    showToast(t('Error'), 'Could not launch explorer', 'danger')
  })
}

function closeDropdowns(): void {
  isVersionDropdownOpen.value = false
  isLoaderVerDropdownOpen.value = false
}

watch([selectedVersion, selectedLoader], () => {
  fetchLoaderVersions()
  checkPreflight(selectedVersion.value, selectedLoader.value)
})

onMounted(() => {
  tempNick.value = currentUserDisplay.value
  fetchAllMojangVersions()
  fetchLoaderVersions()
  checkPreflight(selectedVersion.value, selectedLoader.value)
  window.addEventListener('click', closeDropdowns)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>

<template>
  <div
    class="h-full flex flex-col justify-between p-8 relative overflow-hidden select-none"
    @dragover.prevent="isDraggingOver = true"
    @dragleave.prevent="isDraggingOver = false"
    @drop.prevent="handlePackageDrop"
    @click="closeDropdowns"
  >
    <div
      class="absolute -top-48 -left-48 w-[640px] h-[640px] blur-[150px] rounded-full pointer-events-none transition-colors duration-1000 opacity-20 bg-gradient-to-br"
      :class="activeLoaderGlow"
    ></div>

    <transition name="fade">
      <div
        v-if="isDraggingOver"
        class="absolute inset-0 bg-black/90 backdrop-blur-md z-50 flex flex-col items-center justify-center border-4 border-dashed border-indigo-400 m-4 rounded-3xl pointer-events-none shadow-[0_0_80px_rgba(99,102,241,0.5)]"
      >
        <PackagePlus class="w-20 h-20 text-indigo-400 animate-bounce mb-4" />
        <h3 class="text-3xl font-black text-white uppercase tracking-wider">Drop Package Container</h3>
        <p class="text-xs text-indigo-300 font-mono mt-1">Directly injects .mrpack, .zip or standalone .jar into active profile</p>
      </div>
    </transition>

    <header class="flex justify-between items-center z-10 shrink-0">
      <div class="flex items-center gap-4">
        <div class="p-4 rounded-2xl bg-black/60 border border-white/10 backdrop-blur-xl shadow-2xl relative group">
          <component :is="activeLoaderIcon" class="w-8 h-8 relative z-10 transition-colors duration-500" :class="activeColorText" />
        </div>
        <div>
          <div class="flex items-center gap-2 mb-1">
            <span class="text-[9px] font-black uppercase tracking-[0.25em] text-white/40">Launchpad Core Matrix</span>
            <span
              class="px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider border"
              :class="state.isMcRunning ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' : 'bg-white/5 text-white/50 border-white/10'"
            >
              {{ state.isMcRunning ? 'Running' : 'Ready' }}
            </span>
          </div>
          <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-2">
            <span>{{ selectedLoader }}</span>
            <span class="text-white/40">•</span>
            <span :class="activeColorText">{{ selectedVersion }}</span>
          </h2>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <button
          @click="toggleDoctorGuard(selectedVersion, selectedLoader)"
          class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase tracking-wider border transition-all duration-300 flex items-center gap-2 shadow-lg"
          :class="isDoctorGuardActive ? 'bg-pink-500/20 text-pink-300 border-pink-500/40 shadow-[0_0_15px_rgba(236,72,153,0.3)]' : 'bg-black/40 text-white/40 border-white/10'"
        >
          <Stethoscope class="w-4 h-4" :class="isDoctorGuardActive ? 'animate-pulse text-pink-400' : ''" />
          <span>{{ isDoctorGuardActive ? 'Guard: Active' : 'Guard: Off' }}</span>
        </button>

        <div class="kip-card px-4 py-2 flex items-center gap-3 bg-black/40 border border-white/10">
          <img :src="getAvatarUrl(currentUserDisplay)" class="w-7 h-7 rounded-lg object-cover border border-white/10">
          <div class="flex flex-col min-w-0 pr-1">
            <span class="text-[8px] font-black text-white/40 uppercase tracking-widest leading-none">Handle</span>
            <input
              v-if="isEditingNick"
              v-model="tempNick"
              @keyup.enter="saveGuestNick"
              @blur="saveGuestNick"
              class="bg-transparent border-b border-indigo-500 text-xs font-black text-white focus:outline-none w-24 font-mono mt-0.5"
            >
            <span
              v-else
              @click="isEditingNick = true"
              class="text-xs font-black text-white truncate cursor-pointer hover:text-indigo-300 transition mt-0.5"
            >
              {{ currentUserDisplay }}
            </span>
          </div>
          <button @click="isEditingNick = !isEditingNick" class="text-white/40 hover:text-white transition">
            <UserCheck v-if="isEditingNick" class="w-3.5 h-3.5 text-emerald-400" />
            <Edit3 v-else class="w-3.5 h-3.5" />
          </button>
        </div>

        <button @click="openGameFolder" class="kip-btn-ghost p-2.5 border-white/10 text-white/60 hover:text-white" :title="t('Open instance directory')">
          <FolderOpen class="w-4 h-4" />
        </button>

        <button
          @click="pickInstallPackage"
          class="kip-btn-ghost px-3.5 py-2.5 text-xs font-bold uppercase tracking-wider text-indigo-400 border-indigo-500/30 hover:bg-indigo-500/10 flex items-center gap-2"
        >
          <FileArchive class="w-4 h-4" />
          <span>Import Pack</span>
        </button>

        <button
          @click="customMode = !customMode"
          class="kip-btn-ghost px-4 py-2 text-xs tracking-wider uppercase border-white/10 flex items-center gap-2"
          :class="customMode ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/40' : 'text-white/60'"
        >
          <SlidersHorizontal class="w-4 h-4" />
          <span>{{ customMode ? 'Presets' : 'Matrix Tuning' }}</span>
        </button>
      </div>
    </header>

    <transition name="fade">
      <section
        v-if="isDoctorGuardActive && hasCriticalConflicts"
        class="z-10 mt-3 p-4 rounded-2xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-between shadow-2xl backdrop-blur-md"
      >
        <div class="flex items-center gap-3">
          <AlertTriangle class="w-5 h-5 text-amber-400 shrink-0 animate-pulse" />
          <div>
            <h4 class="text-xs font-black uppercase text-amber-400 tracking-wider">Critical Mod Collisions Detected</h4>
            <p class="text-[11px] text-white/70 font-mono mt-0.5">
              Mod Doctor detected incompatibilities targeting Minecraft {{ selectedVersion }} ({{ selectedLoader }}).
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <button
            @click="handleIgnition(true)"
            class="kip-btn-ghost px-4 py-2 text-xs font-mono font-bold uppercase text-white/70 hover:text-white border-white/20"
          >
            Bypass & Ignite
          </button>
          <button
            @click="repairEnvironment(selectedVersion, selectedLoader)"
            :disabled="isRepairing"
            class="kip-btn-primary px-5 py-2 text-xs bg-amber-500 hover:bg-amber-400 text-black font-black uppercase tracking-wider flex items-center gap-2"
          >
            <Loader v-if="isRepairing" class="w-3.5 h-3.5 animate-spin" />
            <Wrench v-else class="w-3.5 h-3.5" />
            <span>{{ isRepairing ? 'Healing...' : 'Auto-Repair (1-Click)' }}</span>
          </button>
        </div>
      </section>
    </transition>

    <main class="flex-1 flex flex-col justify-center max-w-6xl mx-auto w-full z-10 my-4 min-h-0">
      <div v-if="!customMode" class="grid grid-cols-5 gap-3.5">
        <article
          v-for="preset in presets"
          :key="preset.id"
          @click="selectPreset(preset)"
          class="kip-card p-5 flex flex-col justify-between cursor-pointer relative overflow-hidden group transition-all duration-300 border bg-black/40 hover:bg-black/60"
          :class="activePresetId === preset.id ? 'border-white/40 shadow-[0_0_30px_rgba(255,255,255,0.08)] scale-[1.02]' : 'border-white/5 hover:border-white/20'"
        >
          <div class="flex justify-between items-start mb-5">
            <div
              class="p-3 rounded-xl border transition-colors duration-300"
              :class="activePresetId === preset.id ? 'bg-white/20 border-white/30 text-white' : 'bg-black/40 border-white/5 text-white/50'"
            >
              <component :is="preset.icon" class="w-5 h-5" />
            </div>
            <span class="text-[8px] font-black uppercase px-2 py-0.5 rounded border tracking-widest" :class="preset.badgeClass">
              {{ preset.badge }}
            </span>
          </div>

          <div>
            <h3 class="text-base font-black uppercase tracking-tight text-white mb-1 leading-snug">{{ preset.name }}</h3>
            <p class="text-[11px] text-white/50 font-medium leading-relaxed mb-3 line-clamp-3">{{ preset.desc }}</p>
          </div>

          <div class="pt-2.5 border-t border-white/5 flex items-center justify-between text-[10px] font-mono text-white/40">
            <span>MC {{ selectedVersion }}</span>
            <span :class="activePresetId === preset.id ? 'text-emerald-400 font-bold' : ''">
              {{ activePresetId === preset.id ? 'ACTIVE' : 'SELECT' }}
            </span>
          </div>
        </article>
      </div>

      <div v-else class="kip-card p-8 flex flex-col gap-6 max-w-3xl mx-auto w-full border-white/10 shadow-2xl bg-black/60 backdrop-blur-xl">
        <div class="flex gap-2 p-1.5 bg-black/60 rounded-2xl border border-white/10">
          <button
            v-for="l in loaders"
            :key="l.id"
            @click="selectedLoader = l.id"
            class="flex-1 py-3 rounded-xl font-bold text-xs uppercase tracking-wider transition-all duration-300 flex flex-col items-center gap-1.5 cursor-pointer"
            :class="selectedLoader === l.id ? 'bg-white/10 text-white shadow-lg border border-white/20' : 'text-white/40 hover:text-white'"
          >
            <component :is="l.icon" class="w-5 h-5 transition-colors" :class="selectedLoader === l.id ? activeColorText : ''" />
            <span>{{ l.name }}</span>
          </button>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="relative" @click.stop>
            <div class="flex justify-between items-center mb-1.5">
              <label class="text-[9px] text-white/40 uppercase tracking-widest font-black font-mono">Game Release Registry</label>
              <button
                @click="triggerDownloadVersion"
                :disabled="isDownloadingVersion"
                class="text-[9px] font-mono text-emerald-400 hover:text-emerald-300 uppercase font-black flex items-center gap-1"
                title="Download this version jar and assets"
              >
                <Loader v-if="isDownloadingVersion" class="w-2.5 h-2.5 animate-spin" />
                <Download v-else class="w-2.5 h-2.5" />
                <span>{{ isDownloadingVersion ? 'Fetching...' : 'Acquire JAR' }}</span>
              </button>
            </div>

            <div
              @click="isVersionDropdownOpen = !isVersionDropdownOpen"
              class="bg-black/60 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer hover:border-white/30"
              :class="isVersionDropdownOpen ? 'border-indigo-500' : 'border-white/10'"
            >
              <div class="flex items-center gap-2">
                <input
                  v-model="selectedVersion"
                  type="text"
                  class="bg-transparent font-extrabold text-sm text-white focus:outline-none font-mono w-40"
                  placeholder="e.g. 1.21.4, b1.7.3"
                  @click.stop
                >
              </div>
              <ChevronDown class="w-4 h-4 text-white/40 transition-transform" :class="{'rotate-180': isVersionDropdownOpen}" />
            </div>

            <div
              v-if="isVersionDropdownOpen"
              class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/98 backdrop-blur-2xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-3 z-50 max-h-72 flex flex-col gap-2"
            >
              <div class="px-3 flex gap-1">
                <div class="relative flex-1">
                  <Search class="w-3 h-3 text-white/40 absolute left-2.5 top-1/2 -translate-y-1/2" />
                  <input
                    v-model="versionSearchQuery"
                    type="text"
                    placeholder="Search all versions..."
                    class="w-full bg-black/60 border border-white/10 rounded-lg pl-8 pr-2 py-1 text-xs text-white font-mono"
                    @click.stop
                  >
                </div>
              </div>

              <div class="px-3 flex gap-1 font-mono text-[9px] uppercase">
                <button
                  @click.stop="activeVersionTypeFilter = 'release'"
                  class="px-2 py-0.5 rounded border"
                  :class="activeVersionTypeFilter === 'release' ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/40' : 'text-white/40 border-transparent'"
                >
                  Releases
                </button>
                <button
                  @click.stop="activeVersionTypeFilter = 'snapshot'"
                  class="px-2 py-0.5 rounded border"
                  :class="activeVersionTypeFilter === 'snapshot' ? 'bg-amber-500/20 text-amber-400 border-amber-500/40' : 'text-white/40 border-transparent'"
                >
                  Snapshots
                </button>
                <button
                  @click.stop="activeVersionTypeFilter = 'historical'"
                  class="px-2 py-0.5 rounded border"
                  :class="activeVersionTypeFilter === 'historical' ? 'bg-purple-500/20 text-purple-400 border-purple-500/40' : 'text-white/40 border-transparent'"
                >
                  Beta / Alpha
                </button>
                <button
                  @click.stop="activeVersionTypeFilter = 'all'"
                  class="px-2 py-0.5 rounded border"
                  :class="activeVersionTypeFilter === 'all' ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/40' : 'text-white/40 border-transparent'"
                >
                  All ({{ allManifestVersions.length }})
                </button>
              </div>

              <div class="flex-1 overflow-y-auto custom-scroll max-h-48 px-1">
                <div
                  v-for="v in filteredManifestVersions"
                  :key="v.id"
                  @click="selectedVersion = v.id; isVersionDropdownOpen = false"
                  class="px-3 py-1.5 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold flex justify-between items-center text-white/70 rounded-lg mx-1"
                >
                  <div class="flex items-center gap-2">
                    <span :class="v.versionType === 'release' ? 'text-white' : 'text-white/60'">{{ v.id }}</span>
                    <span class="text-[8px] uppercase px-1 rounded border border-white/10 text-white/30">{{ v.versionType }}</span>
                  </div>
                  <CheckCircle v-if="selectedVersion === v.id" class="w-3.5 h-3.5 text-emerald-400" />
                </div>
              </div>
            </div>
          </div>

          <div class="relative" :class="selectedLoader === 'vanilla' ? 'opacity-30 pointer-events-none' : ''" @click.stop>
            <label class="block text-[9px] text-white/40 uppercase tracking-widest font-black mb-1.5 font-mono">Loader Build</label>
            <div
              @click="isLoaderVerDropdownOpen = !isLoaderVerDropdownOpen"
              class="bg-black/60 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer hover:border-white/30"
              :class="isLoaderVerDropdownOpen ? 'border-indigo-500' : 'border-white/10'"
            >
              <span class="font-extrabold text-sm truncate text-white">{{ selectedLoaderVersion || 'Auto-Recommended' }}</span>
              <ChevronDown class="w-4 h-4 text-white/40 transition-transform" :class="{'rotate-180': isLoaderVerDropdownOpen}" />
            </div>

            <div
              v-if="isLoaderVerDropdownOpen"
              class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/98 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-2 z-50 max-h-56 overflow-y-auto custom-scroll"
            >
              <div
                @click="selectedLoaderVersion = ''; isLoaderVerDropdownOpen = false"
                class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-sm font-bold flex justify-between items-center text-white/70 border-b border-white/5"
              >
                <span>Auto-Recommended</span>
                <CheckCircle v-if="!selectedLoaderVersion" class="w-3.5 h-3.5 text-emerald-400" />
              </div>
              <div
                v-for="lv in loaderVersions"
                :key="lv"
                @click="selectedLoaderVersion = lv; isLoaderVerDropdownOpen = false"
                class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-sm font-bold flex justify-between items-center text-white/70"
              >
                <span>{{ lv }}</span>
                <CheckCircle v-if="selectedLoaderVersion === lv" class="w-3.5 h-3.5 text-emerald-400" />
              </div>
            </div>
          </div>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label class="block text-[9px] text-white/40 uppercase tracking-widest font-black mb-1.5 font-mono">Heap RAM Allocation</label>
            <div class="flex gap-1.5">
              <button
                v-for="ram in [0, 4, 6, 8, 12, 16]"
                :key="ram"
                @click="selectedRam = ram"
                class="flex-1 py-2 rounded-xl text-xs font-mono font-bold border transition cursor-pointer"
                :class="selectedRam === ram ? 'bg-indigo-500 text-white border-indigo-400 shadow-md' : 'bg-black/40 text-white/50 border-white/5 hover:border-white/20'"
              >
                {{ ram === 0 ? 'Auto' : `${ram}G` }}
              </button>
            </div>
          </div>

          <div>
            <label class="block text-[9px] text-white/40 uppercase tracking-widest font-black mb-1.5 font-mono">Display Resolution</label>
            <div class="flex gap-2">
              <select v-model="selectedResolution" class="kip-input py-2 text-xs font-mono flex-1">
                <option value="1280x720">1280x720 (HD 16:9)</option>
                <option value="1920x1080">1920x1080 (FHD 16:9)</option>
                <option value="2560x1440">2560x1440 (2K 16:9)</option>
                <option value="3840x2160">3840x2160 (4K UHD)</option>
                <option value="2560x1080">2560x1080 (UltraWide 21:9)</option>
              </select>
              <button
                @click="isFullscreen = !isFullscreen"
                class="px-3 py-2 rounded-xl border text-xs font-mono font-bold transition flex items-center gap-1 cursor-pointer"
                :class="isFullscreen ? 'bg-emerald-500/20 text-emerald-400 border-emerald-500/30' : 'bg-black/40 text-white/40 border-white/10'"
              >
                <Maximize2 class="w-3.5 h-3.5" />
                <span>Full</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </main>

    <footer class="max-w-2xl mx-auto w-full z-10 flex flex-col gap-3 shrink-0">
      <div class="grid grid-cols-4 gap-2">
        <div class="bg-black/60 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <CheckCircle2 v-if="preflightReport?.javaCompatible" class="w-4 h-4 text-emerald-400 shrink-0" />
          <AlertCircle v-else class="w-4 h-4 text-amber-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">Runtime</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">{{ preflightReport?.javaVersion || 'JDK Resolving' }}</span>
          </div>
        </div>

        <div class="bg-black/60 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <ShieldCheck class="w-4 h-4 text-indigo-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">Integrity</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">
              {{ hasCriticalConflicts ? 'Conflicts' : 'Clean' }}
            </span>
          </div>
        </div>

        <div class="bg-black/60 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <Layers class="w-4 h-4 text-purple-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">Assigned Heap</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">{{ selectedRam === 0 ? '4 GB (Auto)' : `${selectedRam} GB` }}</span>
          </div>
        </div>

        <div class="bg-black/60 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full shrink-0" :class="state.isMcRunning ? 'bg-emerald-400 animate-pulse' : 'bg-white/30'"></span>
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">Process</span>
            <span class="text-[11px] font-bold truncate block mt-0.5" :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/70'">
              {{ state.isMcRunning ? 'Active' : 'Standby' }}
            </span>
          </div>
        </div>
      </div>

      <button
        @click="handleIgnition(false)"
        :disabled="isLaunching"
        class="w-full py-6 rounded-2xl font-black text-2xl tracking-[0.25em] uppercase transition-all duration-300 border shadow-2xl flex items-center justify-center gap-3 cursor-pointer"
        :class="activeColorBtn"
      >
        <template v-if="!isLaunching">
          <Zap class="w-8 h-8 fill-current" />
          <span>IGNITE ENGINE</span>
        </template>
        <template v-else>
          <Loader class="w-8 h-8 animate-spin" />
          <span>{{ currentProgress.message }}</span>
        </template>
      </button>

      <div v-if="isLaunching" class="bg-black/60 p-4 rounded-xl border border-white/10 backdrop-blur-md shadow-2xl">
        <div class="flex justify-between text-xs text-white/70 mb-2 font-mono font-bold tracking-wider uppercase">
          <span class="truncate pr-4 flex items-center gap-2">
            <Cpu class="w-4 h-4 text-indigo-400" />
            <span>{{ currentProgress.message }}</span>
          </span>
          <span class="text-indigo-400 font-mono">{{ Math.round(currentProgress.percent) }}%</span>
        </div>
        <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden">
          <div class="h-full bg-indigo-500 transition-all duration-200" :style="{ width: `${currentProgress.percent}%` }"></div>
        </div>
      </div>
    </footer>
  </div>
</template>