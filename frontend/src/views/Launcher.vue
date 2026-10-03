<template>
  <div
    class="h-full flex flex-col justify-between p-8 relative overflow-hidden select-none"
    @dragover.prevent="isDraggingOver = true"
    @dragleave.prevent="isDraggingOver = false"
    @drop.prevent="handleDirectDrop"
  >
    <div
      class="absolute -top-48 -left-48 w-[640px] h-[640px] blur-[150px] rounded-full pointer-events-none transition-colors duration-1000 opacity-20"
      :class="activeColorBg"
    ></div>
    <div
      class="absolute -bottom-48 -right-48 w-[640px] h-[640px] blur-[150px] rounded-full pointer-events-none transition-colors duration-1000 opacity-20"
      :class="activeColorBg"
    ></div>

    <transition name="fade">
      <div
        v-if="isDraggingOver"
        class="absolute inset-0 bg-indigo-950/85 backdrop-blur-md z-50 flex flex-col items-center justify-center border-4 border-dashed border-indigo-400 m-4 rounded-3xl pointer-events-none shadow-[0_0_80px_rgba(99,102,241,0.5)]"
      >
        <PackagePlus class="w-20 h-20 text-indigo-400 animate-bounce mb-4" />
        <h3 class="text-3xl font-black text-white uppercase tracking-wider">{{ t('Drop Package Here') }}</h3>
        <p class="text-sm text-indigo-200 mt-2 font-mono">{{ t('Modpack ZIP, .mrpack or mod JARs will be unpacked and auto-configured') }}</p>
      </div>
    </transition>

    <div class="flex justify-between items-center z-10 shrink-0">
      <div class="flex items-center gap-4">
        <div class="p-3.5 rounded-2xl bg-black/40 border border-white/10 backdrop-blur-xl shadow-2xl relative group">
          <div class="absolute inset-0 rounded-2xl blur-md opacity-40 transition-colors duration-500" :class="activeColorBg"></div>
          <component :is="activeLoaderIcon" class="w-8 h-8 relative z-10 transition-colors duration-500" :class="activeColorText" />
        </div>
        <div>
          <div class="flex items-center gap-2 mb-1">
            <span class="text-[9px] font-black uppercase tracking-[0.25em] text-white/40">{{ t('Autonomous Engine') }}</span>
            <span
              class="px-2 py-0.5 rounded-full text-[8px] font-black uppercase tracking-wider"
              :class="state.isMcRunning ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30' : 'bg-white/5 text-white/50 border border-white/10'"
            >
              {{ state.isMcRunning ? t('Running') : t('Standby') }}
            </span>
            <span
              v-if="contentAnalysis && contentAnalysis.mod_count > 0"
              class="px-2 py-0.5 rounded-full text-[8px] font-mono font-bold bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 flex items-center gap-1"
            >
              <Cpu class="w-3 h-3 text-indigo-400" /> {{ contentAnalysis.mod_count }} {{ t('mods') }}
            </span>
          </div>
          <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
            {{ activePresetName }}
          </h2>
        </div>
      </div>

      <div class="flex items-center gap-3">
        <div class="kip-card px-4 py-2 flex items-center gap-3 bg-black/40 border border-white/10 shadow-lg">
          <div class="relative w-8 h-8 rounded-lg overflow-hidden border border-white/10 shrink-0">
            <img :src="getAvatarUrl(currentUserDisplay)" class="w-full h-full object-cover">
          </div>
          <div class="flex flex-col min-w-0 pr-1">
            <span class="text-[8px] font-black text-white/40 uppercase tracking-widest truncate">{{ t('Profile') }}</span>
            <input
              v-if="isEditingNick"
              v-model="tempNick"
              @keyup.enter="saveGuestNick"
              @blur="saveGuestNick"
              class="bg-transparent border-b border-indigo-500 text-xs font-black text-white focus:outline-none w-24 font-mono"
            >
            <span
              v-else
              @click="isEditingNick = true"
              class="text-xs font-black text-white truncate cursor-pointer hover:text-indigo-300 transition"
              :title="t('Click to edit username')"
            >
              {{ currentUserDisplay }}
            </span>
          </div>
          <button @click="isEditingNick = !isEditingNick" class="text-white/40 hover:text-white transition">
            <UserCheck v-if="isEditingNick" class="w-3.5 h-3.5 text-emerald-400" />
            <Edit3 v-else class="w-3.5 h-3.5" />
          </button>
        </div>

        <button @click="openInstanceFolder" class="kip-btn-ghost p-2.5 border-white/10 text-white/60 hover:text-white" :title="t('Open game folder')">
          <FolderOpen class="w-4 h-4" />
        </button>

        <button @click="pickAndImportPackage" class="kip-btn-ghost px-3.5 py-2.5 text-xs tracking-wider uppercase border-white/10 text-indigo-400 hover:text-indigo-300 hover:border-indigo-500/40" :title="t('Import ZIP/MRPACK/JAR archive')">
          <FileArchive class="w-4 h-4" />
          <span>{{ t('Install Pack') }}</span>
        </button>

        <button
          @click="customMode = !customMode"
          class="kip-btn-ghost px-4 py-2.5 text-xs tracking-wider uppercase border-white/10"
          :class="customMode ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/40' : 'text-white/60'"
        >
          <SlidersHorizontal class="w-4 h-4" />
          <span>{{ customMode ? t('Presets') : t('Matrix Tuning') }}</span>
        </button>
      </div>
    </div>

    <transition name="fade">
      <div
        v-if="preflight && !preflight.ready_to_launch"
        class="z-10 mt-3 p-4 rounded-2xl bg-amber-500/10 border border-amber-500/30 flex items-center justify-between shadow-2xl backdrop-blur-md"
      >
        <div class="flex items-center gap-3">
          <AlertTriangle class="w-5 h-5 text-amber-400 shrink-0 animate-pulse" />
          <div>
            <h4 class="text-xs font-black uppercase text-amber-400">{{ t('Compatibility Conflicts Detected') }}</h4>
            <p class="text-[11px] text-white/70">
              {{ t('Conflicting files detected or compatible Java Runtime is missing.') }}
            </p>
          </div>
        </div>
        <button
          @click="runAutoRepair"
          :disabled="isRepairing"
          class="kip-btn-primary px-5 py-2 text-xs bg-amber-500 hover:bg-amber-400 text-black font-black uppercase tracking-wider shrink-0 flex items-center gap-2"
        >
          <Wrench v-if="!isRepairing" class="w-3.5 h-3.5" />
          <Loader v-else class="w-3.5 h-3.5 animate-spin" />
          <span>{{ isRepairing ? t('Repairing...') : t('Auto-Repair 1-Click') }}</span>
        </button>
      </div>
    </transition>

    <div class="flex-1 flex flex-col justify-center max-w-5xl mx-auto w-full z-10 my-4 min-h-0">
      <div v-if="!customMode" class="grid grid-cols-4 gap-4">
        <div
          v-for="preset in presets"
          :key="preset.id"
          @click="selectPreset(preset)"
          class="kip-card p-6 flex flex-col justify-between cursor-pointer relative overflow-hidden group transition-all duration-300 border"
          :class="activePresetId === preset.id ? 'border-white/40 bg-white/10 shadow-[0_0_30px_rgba(255,255,255,0.08)] scale-[1.02]' : 'border-white/5 hover:border-white/20 hover:bg-black/60'"
        >
          <div class="absolute -top-12 -right-12 w-28 h-28 rounded-full blur-2xl opacity-20 transition-all duration-500" :class="preset.bgClass"></div>
          <div class="flex justify-between items-start mb-6">
            <div
              class="p-3 rounded-xl border transition-colors duration-300"
              :class="activePresetId === preset.id ? 'bg-white/20 border-white/30 text-white' : 'bg-black/40 border-white/5 text-white/50'"
            >
              <component :is="preset.icon" class="w-6 h-6" />
            </div>
            <span class="text-[9px] font-black uppercase px-2.5 py-0.5 rounded border tracking-widest" :class="preset.badgeClass">{{ preset.badge }}</span>
          </div>

          <div>
            <h3 class="text-xl font-black uppercase tracking-tight text-white mb-1.5">{{ t(preset.name) }}</h3>
            <p class="text-xs text-white/50 font-medium leading-relaxed mb-4">{{ t(preset.desc) }}</p>
          </div>

          <div class="pt-3 border-t border-white/5 flex items-center justify-between text-[11px] font-mono text-white/40">
            <span>MC {{ selectedVersionDisplay }}</span>
            <span :class="activePresetId === preset.id ? 'text-emerald-400 font-bold' : ''">{{ activePresetId === preset.id ? t('ACTIVE') : t('SELECT') }}</span>
          </div>
        </div>
      </div>

      <div v-else class="kip-card p-8 flex flex-col gap-6 max-w-2xl mx-auto w-full border-white/10 shadow-2xl">
        <div class="flex gap-2">
          <button
            v-for="l in loaders"
            :key="l.id"
            @click="launchData.loader = l.id"
            class="flex-1 py-3 rounded-xl font-bold text-xs uppercase tracking-wider transition-all duration-300 relative z-10 flex flex-col items-center gap-1.5"
            :class="launchData.loader === l.id ? 'bg-white/10 text-white shadow-lg border border-white/20' : 'text-white/40 hover:text-white hover:bg-white/5'">
            <component :is="l.icon" class="w-5 h-5 transition-colors duration-300" :class="launchData.loader === l.id ? activeColorText : ''" />
            {{ l.name }}
          </button>
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div class="relative custom-dropdown">
            <label class="block text-[9px] text-white/40 uppercase tracking-widest font-black mb-1.5">{{ t('Game Release') }}</label>
            <div
              @click="isVersionDropdownOpen = !isVersionDropdownOpen"
              class="bg-black/60 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition hover:border-white/30"
              :class="isVersionDropdownOpen ? 'border-indigo-500' : 'border-white/10'"
            >
              <span class="font-extrabold text-sm">{{ launchData.version || t('Auto') }}</span>
              <ChevronDown class="w-4 h-4 text-white/40 transition-transform" :class="{'rotate-180': isVersionDropdownOpen}" />
            </div>
            <transition name="fade">
              <div v-if="isVersionDropdownOpen" class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-2 z-50 max-h-56 overflow-y-auto custom-scroll">
                <div
                  v-for="v in state.mcVersions"
                  :key="v"
                  @click="launchData.version = v; isVersionDropdownOpen = false"
                  class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-sm font-bold flex justify-between items-center"
                  :class="launchData.version === v ? activeColorText : 'text-white/70'"
                >
                  <span>{{ v }}</span>
                  <CheckCircle v-if="launchData.version === v" class="w-3.5 h-3.5" />
                </div>
              </div>
            </transition>
          </div>

          <div class="relative custom-dropdown" :class="launchData.loader === 'vanilla' ? 'opacity-30 pointer-events-none' : ''">
            <label class="block text-[9px] text-white/40 uppercase tracking-widest font-black mb-1.5">{{ t('Loader Build') }}</label>
            <div
              @click="isLoaderVerDropdownOpen = !isLoaderVerDropdownOpen"
              class="bg-black/60 border rounded-xl px-4 py-3 flex justify-between items-center cursor-pointer transition hover:border-white/30"
              :class="isLoaderVerDropdownOpen ? 'border-indigo-500' : 'border-white/10'"
            >
              <span class="font-extrabold text-sm truncate">{{ launchData.loader_version || t('Auto-Recommended') }}</span>
              <ChevronDown class="w-4 h-4 text-white/40 transition-transform" :class="{'rotate-180': isLoaderVerDropdownOpen}" />
            </div>
            <transition name="fade">
              <div v-if="isLoaderVerDropdownOpen" class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-2 z-50 max-h-56 overflow-y-auto custom-scroll">
                <div @click="launchData.loader_version = ''; isLoaderVerDropdownOpen = false" class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-sm font-bold flex justify-between items-center border-b border-white/5">
                  <span>{{ t('Auto-Recommended') }}</span>
                  <CheckCircle v-if="!launchData.loader_version" class="w-3.5 h-3.5 text-emerald-400" />
                </div>
                <div
                  v-for="lv in loaderVersions"
                  :key="lv"
                  @click="launchData.loader_version = lv; isLoaderVerDropdownOpen = false"
                  class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-sm font-bold flex justify-between items-center"
                  :class="launchData.loader_version === lv ? activeColorText : 'text-white/70'"
                >
                  <span>{{ lv }}</span>
                  <CheckCircle v-if="launchData.loader_version === lv" class="w-3.5 h-3.5" />
                </div>
              </div>
            </transition>
          </div>
        </div>

        <div class="flex flex-col gap-2">
          <label class="text-[9px] text-white/40 uppercase tracking-widest font-black">{{ t('RAM Allocation Express') }}</label>
          <div class="flex gap-2">
            <button
              v-for="ram in [0, 4, 6, 8, 12, 16]"
              :key="ram"
              @click="setQuickRam(ram)"
              class="flex-1 py-2 rounded-xl font-mono text-xs font-bold transition border"
              :class="state.settings.ram_allocation === ram ? 'bg-indigo-500 text-white border-indigo-400 shadow-[0_0_12px_rgba(99,102,241,0.5)]' : 'bg-black/40 text-white/60 border-white/5 hover:border-white/20'"
            >
              {{ ram === 0 ? t('Auto') : `${ram}G` }}
            </button>
          </div>
        </div>
      </div>
    </div>

    <div class="max-w-2xl mx-auto w-full z-10 flex flex-col gap-3 shrink-0">
      <div class="grid grid-cols-4 gap-2">
        <div class="bg-black/50 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <CheckCircle2 v-if="preflight?.java_compatible" class="w-4 h-4 text-emerald-400 shrink-0" />
          <AlertCircle v-else class="w-4 h-4 text-amber-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">{{ t('Java Runtime') }}</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">{{ preflight?.java_version || 'Resolved' }}</span>
          </div>
        </div>

        <div class="bg-black/50 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <ShieldCheck class="w-4 h-4 text-indigo-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">{{ t('Modules') }}</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">
              {{ contentAnalysis?.is_clean ? t('Clean') : t('Fix Required') }}
            </span>
          </div>
        </div>

        <div class="bg-black/50 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <Layers class="w-4 h-4 text-purple-400 shrink-0" />
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">{{ t('Memory (RAM)') }}</span>
            <span class="text-[11px] font-bold text-white truncate block mt-0.5">{{ detectedMemoryDisplay }}</span>
          </div>
        </div>

        <div class="bg-black/50 border border-white/5 p-2.5 rounded-xl flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full shrink-0" :class="state.isMcRunning ? 'bg-emerald-400 animate-pulse' : 'bg-white/30'"></span>
          <div class="min-w-0">
            <span class="text-[8px] font-mono text-white/40 uppercase block font-bold leading-none">{{ t('Core Status') }}</span>
            <span class="text-[11px] font-bold truncate block mt-0.5" :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/70'">
              {{ state.isMcRunning ? t('In-Game') : t('Ready') }}
            </span>
          </div>
        </div>
      </div>

      <div class="relative group">
        <div class="absolute -inset-1 rounded-2xl blur-xl opacity-30 group-hover:opacity-75 transition duration-500" :class="activeColorBg"></div>
        <button
          @click="igniteEngine"
          :disabled="isLaunching"
          class="w-full py-6 rounded-2xl font-black text-2xl tracking-[0.25em] uppercase transition-all duration-300 relative overflow-hidden border shadow-2xl flex items-center justify-center gap-3 cursor-pointer"
          :class="activeColorBtn"
        >
          <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out pointer-events-none"></div>
          <template v-if="state.isMcRunning">
            <CheckCircle class="w-8 h-8 fill-current text-emerald-300" />
            <span>{{ t('GAME RUNNING • RE-IGNITE') }}</span>
          </template>
          <template v-else-if="!isLaunching">
            <Zap class="w-8 h-8 fill-current" />
            <span>{{ t('IGNITE ENGINE') }}</span>
          </template>
          <template v-else>
            <Loader class="w-8 h-8 animate-spin" />
            <span>{{ t('PREPARING & LAUNCHING...') }}</span>
          </template>
        </button>
      </div>

      <transition name="fade">
        <div v-if="isLaunching" class="bg-black/60 p-4 rounded-xl border border-white/10 backdrop-blur-md shadow-2xl">
          <div class="flex justify-between text-xs text-white/70 mb-2.5 font-mono font-bold tracking-wider uppercase">
            <span class="truncate pr-4 flex items-center gap-2">
              <Cpu class="w-4 h-4" :class="activeColorText" />
              {{ currentStatusText }}
            </span>
            <span class="shrink-0 font-mono" :class="activeColorText">{{ Math.round(currentProgressVal) }}%</span>
          </div>
          <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden shadow-inner">
            <div class="h-full transition-all duration-300 ease-out shadow-[0_0_10px_rgba(255,255,255,0.5)]" :class="activeColorProgress" :style="{ width: currentProgressVal + '%' }"></div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, onBeforeUnmount } from 'vue'
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
  Edit3,
  UserCheck,
  Flame,
  FolderOpen,
  ShieldCheck,
  Wrench,
  PackagePlus,
  FileArchive,
} from 'lucide-vue-next'
import { state, t, showToast, getAvatarUrl, saveSetting } from '@/store'
import {
  bridge,
  invokeSafe,
  type LaunchPayload,
  type LaunchResult,
  type ContentInspectionDto,
  type PreflightReportDto,
  type AutoRepairResultDto,
  type ModpackImportResultDto,
  type ImportDroppedModsResult,
} from '@/bridge'

export type LoaderType = 'vanilla' | 'fabric' | 'forge' | 'neoforge' | 'quilt'

export interface LoaderItem {
  id: LoaderType
  name: string
  icon: Component
}

export interface PresetCard {
  id: string
  name: string
  desc: string
  loader: LoaderType
  badge: string
  badgeClass: string
  bgClass: string
  icon: Component
}

const customMode = ref<boolean>(false)
const isVersionDropdownOpen = ref<boolean>(false)
const isLoaderVerDropdownOpen = ref<boolean>(false)
const isEditingNick = ref<boolean>(false)
const tempNick = ref<string>('')
const isLaunching = ref<boolean>(false)
const isRepairing = ref<boolean>(false)
const isDraggingOver = ref<boolean>(false)
const loaderVersions = ref<string[]>([])

const contentAnalysis = ref<ContentInspectionDto | null>(null)
const preflight = ref<PreflightReportDto | null>(null)

const activePresetId = ref<string>('fabric_fps')

const launchData = ref<LaunchPayload>({
  loader: 'fabric',
  version: '',
  loader_version: '',
})

const loaders: LoaderItem[] = [
  { id: 'vanilla', name: 'Vanilla', icon: Box },
  { id: 'fabric', name: 'Fabric', icon: Hexagon },
  { id: 'forge', name: 'Forge', icon: Flame },
  { id: 'neoforge', name: 'NeoForge', icon: LucideComponent },
  { id: 'quilt', name: 'Quilt', icon: Layers },
]

const presets: PresetCard[] = [
  {
    id: 'fabric_fps',
    name: 'Fabric Turbo',
    desc: 'High-performance modern core engineered for maximum frames and fluid rendering.',
    loader: 'fabric',
    badge: 'FPS Boost',
    badgeClass: 'bg-amber-500/10 text-amber-400 border-amber-500/20',
    bgClass: 'bg-amber-500',
    icon: Hexagon,
  },
  {
    id: 'vanilla_latest',
    name: 'Pure Vanilla',
    desc: 'Original Minecraft runtime without modifications. Absolute stability and purity.',
    loader: 'vanilla',
    badge: 'Vanilla',
    badgeClass: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    bgClass: 'bg-emerald-500',
    icon: Box,
  },
  {
    id: 'forge_heavy',
    name: 'Forge Realm',
    desc: 'Legacy and enterprise-grade modular stack for heavy mechanical and magic expansions.',
    loader: 'forge',
    badge: 'Modded',
    badgeClass: 'bg-rose-500/10 text-rose-400 border-rose-500/20',
    bgClass: 'bg-rose-500',
    icon: Flame,
  },
  {
    id: 'quilt_flow',
    name: 'Quilt Matrix',
    desc: 'Next-generation ecosystem compatible with Fabric modules and bleeding-edge APIs.',
    loader: 'quilt',
    badge: 'Next-Gen',
    badgeClass: 'bg-purple-500/10 text-purple-400 border-purple-500/20',
    bgClass: 'bg-purple-500',
    icon: Layers,
  },
]

const currentUserDisplay = computed<string>(() => {
  if (state.settings.has_ms_token && state.settings.ms_name) {
    return state.settings.ms_name
  }
  if (state.settings.offline_username && state.settings.offline_username.trim()) {
    return state.settings.offline_username.trim()
  }
  return 'Player_Guest'
})

const selectedVersionDisplay = computed<string>(() => {
  if (launchData.value.version) return launchData.value.version
  if (state.mcVersions.length > 0 && state.mcVersions[0]) return state.mcVersions[0]
  return '1.21.1'
})

const detectedMemoryDisplay = computed<string>(() => {
  if (state.settings.ram_allocation && state.settings.ram_allocation > 0) {
    return `${state.settings.ram_allocation} GB RAM`
  }
  return `4 GB (${t('Auto')})`
})

const activePresetName = computed<string>(() => {
  if (customMode.value) return t('Custom Tuning Matrix')
  const p = presets.find((x) => x.id === activePresetId.value)
  return p ? t(p.name) : t('Ignition Engine')
})

const activeLoaderIcon = computed<Component>(() => {
  const l = loaders.find((x) => x.id === launchData.value.loader)
  return l ? l.icon : Box
})

const currentStatusText = computed<string>(() => {
  return state.launchStatus || t('Engine Ready')
})

const currentProgressVal = computed<number>(() => {
  return state.launchProgress || 0
})

const buttonColorMap: Record<LoaderType, string> = {
  vanilla: 'bg-emerald-500 text-black border-emerald-400 hover:shadow-[0_0_35px_rgba(16,185,129,0.5)]',
  fabric: 'bg-amber-500 text-black border-amber-400 hover:shadow-[0_0_35px_rgba(245,158,11,0.5)]',
  forge: 'bg-rose-500 text-white border-rose-400 hover:shadow-[0_0_35px_rgba(244,63,94,0.5)]',
  neoforge: 'bg-orange-500 text-black border-orange-400 hover:shadow-[0_0_35px_rgba(249,115,22,0.5)]',
  quilt: 'bg-purple-500 text-white border-purple-400 hover:shadow-[0_0_35px_rgba(168,85,247,0.5)]',
}

const textColorMap: Record<LoaderType, string> = {
  vanilla: 'text-emerald-400',
  fabric: 'text-amber-400',
  forge: 'text-rose-400',
  neoforge: 'text-orange-400',
  quilt: 'text-purple-400',
}

const progressColorMap: Record<LoaderType, string> = {
  vanilla: 'bg-emerald-400',
  fabric: 'bg-amber-400',
  forge: 'bg-rose-500',
  neoforge: 'bg-orange-400',
  quilt: 'bg-purple-400',
}

const bgColorMap: Record<LoaderType, string> = {
  vanilla: 'bg-emerald-600',
  fabric: 'bg-amber-600',
  forge: 'bg-rose-600',
  neoforge: 'bg-orange-600',
  quilt: 'bg-purple-600',
}

const activeColorBtn = computed<string>(() => {
  if (isLaunching.value) return 'bg-white/10 text-white/40 border-white/10 cursor-wait'
  return buttonColorMap[launchData.value.loader as LoaderType] || buttonColorMap.fabric
})

const activeColorText = computed<string>(() => {
  return textColorMap[launchData.value.loader as LoaderType] || 'text-amber-400'
})

const activeColorProgress = computed<string>(() => {
  return progressColorMap[launchData.value.loader as LoaderType] || 'bg-amber-400'
})

const activeColorBg = computed<string>(() => {
  return bgColorMap[launchData.value.loader as LoaderType] || 'bg-amber-600'
})

const selectPreset = (preset: PresetCard): void => {
  activePresetId.value = preset.id
  launchData.value.loader = preset.loader
  launchData.value.loader_version = ''
  if (!launchData.value.version && state.mcVersions.length > 0 && state.mcVersions[0]) {
    launchData.value.version = state.mcVersions[0]
  }
  runPreflightCheck()
}

const saveGuestNick = async (): Promise<void> => {
  isEditingNick.value = false
  const clean = tempNick.value.trim()
  if (clean && clean.length >= 3) {
    state.settings.offline_username = clean
    await saveSetting('offline_username', clean)
    showToast(t('Operator Saved'), clean, 'success')
  }
}

const openInstanceFolder = async (): Promise<void> => {
  try {
    await invokeSafe('open_media_folder')
  } catch {
    showToast(t('Folder Notice'), t('Opening root instance location'), 'info')
  }
}

const setQuickRam = async (ram: number): Promise<void> => {
  try {
    await bridge.autoTuneRam(ram)
    state.settings.ram_allocation = ram
    showToast(t('Memory Adjusted'), `${ram === 0 ? t('Auto') : `${ram} GB`} ${t('assigned')}`, 'success')
    runPreflightCheck()
  } catch {
    showToast(t('Error'), t('Failed to update RAM allocation.'), 'danger')
  }
}

const pickAndImportPackage = async (): Promise<void> => {
  try {
    const selectedFile = await bridge.pickFile()
    if (selectedFile && selectedFile.trim().length > 0) {
      await processImportFile(selectedFile.trim())
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Import Error'), msg, 'danger')
  }
}

const processImportFile = async (filePath: string): Promise<void> => {
  try {
    showToast(t('Package Inspection'), t('Decompressing and adapting environment...'), 'info')
    const res: ModpackImportResultDto = await bridge.importModpackOrArchive(filePath)
    if (res.success) {
      showToast(t('Package Synchronized'), `${res.pack_name} (${res.loader.toUpperCase()} ${res.mc_version})`, 'success')
      launchData.value.version = res.mc_version
      launchData.value.loader = res.loader as LoaderType
      await refreshContentAnalysis()
      await runPreflightCheck()
    } else {
      showToast(t('Import Notice'), res.message, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Import Error'), msg, 'danger')
  }
}

const runPreflightCheck = async (): Promise<void> => {
  try {
    const targetVersion = launchData.value.version || selectedVersionDisplay.value || '1.21.1'
    const targetLoader = launchData.value.loader || 'fabric'
    preflight.value = await bridge.preflightInspection(targetVersion, targetLoader)
  } catch {
    preflight.value = null
  }
}

const runAutoRepair = async (): Promise<void> => {
  if (isRepairing.value) return
  isRepairing.value = true

  try {
    const targetVersion = launchData.value.version || selectedVersionDisplay.value || '1.21.1'
    const targetLoader = launchData.value.loader || 'fabric'
    const res: AutoRepairResultDto = await bridge.autoRepairInstance(targetVersion, targetLoader)

    if (res.success) {
      showToast(t('Auto-Repair'), res.message, 'success')
      await refreshContentAnalysis()
      await runPreflightCheck()
    } else {
      showToast(t('Repair Notice'), res.message, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isRepairing.value = false
  }
}

const refreshContentAnalysis = async (): Promise<void> => {
  try {
    contentAnalysis.value = await bridge.inspectInstalledContent()
    if (contentAnalysis.value && contentAnalysis.value.mod_count > 0) {
      if (!launchData.value.version && contentAnalysis.value.detected_version) {
        launchData.value.version = contentAnalysis.value.detected_version
      }
      if (launchData.value.loader === 'vanilla' && contentAnalysis.value.detected_loader !== 'vanilla') {
        launchData.value.loader = contentAnalysis.value.detected_loader as LoaderType
      }
    }
  } catch {
    contentAnalysis.value = null
  }
}

const handleDirectDrop = async (e: DragEvent): Promise<void> => {
  isDraggingOver.value = false
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const files = Array.from(e.dataTransfer.files)
    const firstPackage = files.find((f) => {
      const name = f.name.toLowerCase()
      return name.endsWith('.zip') || name.endsWith('.mrpack')
    })

    if (firstPackage) {
      const path = (firstPackage as unknown as { path?: string }).path
      if (path) {
        await processImportFile(path)
        return
      }
    }

    const filePaths = files
      .map((f) => (f as unknown as { path?: string }).path)
      .filter((p): p is string => typeof p === 'string' && p.length > 0)

    if (filePaths.length > 0) {
      try {
        const res: ImportDroppedModsResult = await bridge.importDroppedMods(filePaths)
        if (res.success) {
          showToast(t('Imported'), `${t('Successfully imported')} ${res.count} ${t('items.')}`, 'success')
          await refreshContentAnalysis()
          await runPreflightCheck()
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err)
        showToast(t('Import Error'), msg, 'danger')
      }
    }
  }
}

const fetchLoaderVersions = async (): Promise<void> => {
  if (launchData.value.loader === 'vanilla' || !launchData.value.version) {
    loaderVersions.value = []
    launchData.value.loader_version = ''
    return
  }

  launchData.value.loader_version = ''

  try {
    const versions = await bridge.getLoaderVersions(launchData.value.loader, launchData.value.version)
    loaderVersions.value = versions || []
  } catch {
    loaderVersions.value = []
  }
}

watch(() => launchData.value.loader, () => {
  fetchLoaderVersions()
  runPreflightCheck()
})

watch(() => launchData.value.version, () => {
  fetchLoaderVersions()
  runPreflightCheck()
})

const igniteEngine = async (): Promise<void> => {
  if (isLaunching.value) return

  if (!state.settings.has_ms_token && (!state.settings.offline_username || !state.settings.offline_username.trim())) {
    const autoNick = 'Player_' + Math.random().toString(36).substring(2, 6)
    state.settings.offline_username = autoNick
    await saveSetting('offline_username', autoNick)
  }

  const targetVersion = launchData.value.version || selectedVersionDisplay.value || '1.21.1'
  const targetLoader = launchData.value.loader || 'fabric'
  const targetLoaderBuild = launchData.value.loader_version || null

  isLaunching.value = true
  state.launchStatus = t('Automated calibration and integrity verification...')
  state.launchProgress = 10

  try {
    if (preflight.value && !preflight.value.ready_to_launch) {
      state.launchStatus = t('Resolving conflicts and terminating zombie processes...')
      state.launchProgress = 25
      await bridge.autoRepairInstance(targetVersion, targetLoader)
    }

    state.launchStatus = t('Starting Minecraft game process...')
    state.launchProgress = 50

    const res: LaunchResult = await bridge.launchGame(targetVersion, targetLoader, targetLoaderBuild)
    if (!res.success) {
      showToast(t('Ignition Notice'), res.message, 'danger')
    } else {
      showToast(t('Engine Ignited'), `${targetLoader.toUpperCase()} ${targetVersion} ${t('launched successfully')}`, 'success')
      state.isMcRunning = true
      state.mcStatusText = 'Minecraft running'
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Ignition Error'), msg || t('Automated launch sequence failed.'), 'danger')
  } finally {
    setTimeout(() => {
      isLaunching.value = false
      state.launchProgress = 0
      state.launchStatus = 'Ready'
    }, 3500)
  }
}

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    isVersionDropdownOpen.value = false
    isLoaderVerDropdownOpen.value = false
  }
}

onMounted(async () => {
  window.addEventListener('click', closeDropdowns)
  tempNick.value = currentUserDisplay.value

  if (state.mcVersions.length === 0) {
    try {
      const versions = await bridge.getMcVersions()
      if (versions && versions.length > 0) {
        state.mcVersions = versions
        if (!launchData.value.version && versions[0]) {
          launchData.value.version = versions[0]
        }
      }
    } catch {
      launchData.value.version = '1.21.1'
    }
  } else if (!launchData.value.version && state.mcVersions[0]) {
    launchData.value.version = state.mcVersions[0]
  }

  await refreshContentAnalysis()
  await runPreflightCheck()
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>

<style scoped>
.kip-card {
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.6);
}
</style>