<template>
  <div class="h-full flex items-center justify-center p-8 relative overflow-hidden">
    <div class="absolute -top-40 -left-40 w-[600px] h-[600px] blur-[120px] rounded-full pointer-events-none transition-colors duration-1000 opacity-20" :class="activeColorBg"></div>
    <div class="absolute -bottom-40 -right-40 w-[600px] h-[600px] blur-[120px] rounded-full pointer-events-none transition-colors duration-1000 opacity-20" :class="activeColorBg"></div>

    <div class="w-full max-w-6xl flex gap-12 items-center relative z-10">
      <div class="flex-1 flex flex-col justify-center stagger-1">
        <div class="relative w-40 h-40 mb-8">
          <div class="absolute inset-0 blur-2xl opacity-40 transition-colors duration-700" :class="activeColorBg"></div>
          <component :is="activeLoaderIcon" class="w-full h-full relative z-10 transition-colors duration-700 drop-shadow-[0_0_20px_rgba(255,255,255,0.2)]" :class="activeColorText" />
        </div>

        <h1 class="text-7xl font-black uppercase tracking-tighter mb-4 text-white drop-shadow-lg">
          {{ activeLoaderName }}
        </h1>
        <p class="text-white/50 text-xl tracking-wide leading-relaxed max-w-lg">
          {{ t('Select your preferred configuration and ignite the engine.') }}
        </p>
      </div>

      <div class="w-[450px] flex flex-col gap-6 stagger-2">
        <div class="kip-card p-6 flex justify-between items-center">
          <div class="flex flex-col">
            <span class="text-[10px] text-white/50 uppercase tracking-widest font-bold mb-1">{{ t('Account') }}</span>
            <span class="font-bold text-white text-lg" :class="!state.settings.has_ms_token ? 'text-amber-400' : 'text-emerald-400'">
              {{ state.settings.ms_name || state.settings.offline_username || t('Player (Offline)') }}
            </span>
          </div>
          <div class="relative">
            <div class="absolute inset-0 bg-indigo-500/20 blur-md rounded-full"></div>
            <img v-if="state.settings.has_ms_token" :src="getAvatarUrl(state.settings.ms_name)" class="w-14 h-14 rounded-full object-cover border-2 border-white/10 relative z-10 shadow-lg">
            <div v-else class="w-14 h-14 rounded-full bg-black/60 border-2 border-amber-500/30 flex items-center justify-center relative z-10">
              <User class="w-6 h-6 text-amber-400" />
            </div>
          </div>
        </div>

        <div class="kip-card p-2 flex gap-1 relative overflow-hidden">
          <div class="absolute inset-0 opacity-10 transition-colors duration-700 pointer-events-none" :class="activeColorBg"></div>
          <button v-for="l in loaders" :key="l.id" @click="launchData.loader = l.id" class="flex-1 py-3 rounded-xl font-bold text-xs uppercase tracking-wider transition-all duration-300 relative z-10 flex flex-col items-center gap-1.5" :class="launchData.loader === l.id ? 'bg-white/10 text-white shadow-lg border border-white/10' : 'text-white/40 hover:text-white hover:bg-white/5'">
            <component :is="l.icon" class="w-5 h-5 transition-colors duration-300" :class="launchData.loader === l.id ? activeColorText : ''" />
            {{ l.name }}
          </button>
        </div>

        <div class="kip-card p-2 flex gap-2">
          <div class="relative flex-1 custom-dropdown">
            <div @click="isVersionDropdownOpen = !isVersionDropdownOpen" class="bg-black/40 border rounded-xl px-5 py-4 flex justify-between items-center cursor-pointer transition hover:bg-black/60" :class="isVersionDropdownOpen ? 'border-indigo-500 shadow-[0_0_20px_rgba(255,255,255,0.1)]' : 'border-white/5'">
              <div class="flex flex-col min-w-0">
                <span class="text-[9px] text-white/50 uppercase tracking-widest font-bold mb-0.5 truncate">{{ t('Game Version') }}</span>
                <span class="font-extrabold text-base tracking-wide truncate">{{ launchData.version || t('Select') }}</span>
              </div>
              <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0" :class="{'rotate-180': isVersionDropdownOpen}" />
            </div>
            <transition name="fade">
              <div v-if="isVersionDropdownOpen" class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_30px_rgba(0,0,0,0.8)] overflow-hidden py-2 z-50 max-h-72 overflow-y-auto custom-scroll">
                <div v-if="state.mcVersions.length === 0" class="px-5 py-6 text-center text-white/50 flex justify-center">
                  <Loader class="w-6 h-6 animate-spin" />
                </div>
                <div v-else v-for="v in state.mcVersions" :key="v" @click="launchData.version = v; isVersionDropdownOpen = false" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition flex items-center justify-between group">
                  <span class="font-bold text-sm transition-colors" :class="launchData.version === v ? activeColorText : 'text-white/70 group-hover:text-white'">{{ v }}</span>
                  <CheckCircle v-if="launchData.version === v" class="w-3.5 h-3.5" :class="activeColorText" />
                </div>
              </div>
            </transition>
          </div>

          <div v-if="launchData.loader !== 'vanilla'" class="relative flex-1 custom-dropdown">
            <div @click="isLoaderVerDropdownOpen = !isLoaderVerDropdownOpen" class="bg-black/40 border rounded-xl px-5 py-4 flex justify-between items-center cursor-pointer transition hover:bg-black/60" :class="isLoaderVerDropdownOpen ? 'border-indigo-500 shadow-[0_0_20px_rgba(255,255,255,0.1)]' : 'border-white/5'">
              <div class="flex flex-col min-w-0">
                <span class="text-[9px] text-white/50 uppercase tracking-widest font-bold mb-0.5 truncate">Loader Build</span>
                <span v-if="isLoadingVersions" class="font-extrabold text-base tracking-wide text-white/30 truncate flex items-center gap-2"><Loader class="w-3 h-3 animate-spin" /></span>
                <span v-else class="font-extrabold text-base tracking-wide truncate">{{ launchData.loader_version || 'Latest' }}</span>
              </div>
              <ChevronDown class="w-4 h-4 text-white/50 transition-transform shrink-0" :class="{'rotate-180': isLoaderVerDropdownOpen}" />
            </div>
            <transition name="fade">
              <div v-if="isLoaderVerDropdownOpen" class="absolute bottom-full mb-2 left-0 w-full bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_30px_rgba(0,0,0,0.8)] overflow-hidden py-2 z-50 max-h-72 overflow-y-auto custom-scroll">
                <div @click="launchData.loader_version = ''; isLoaderVerDropdownOpen = false" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition flex items-center justify-between group border-b border-white/5">
                  <span class="font-bold text-sm transition-colors" :class="!launchData.loader_version ? activeColorText : 'text-white/70 group-hover:text-white'">Latest Stable</span>
                  <CheckCircle v-if="!launchData.loader_version" class="w-3.5 h-3.5" :class="activeColorText" />
                </div>
                <div v-if="loaderVersions.length === 0 && !isLoadingVersions" class="px-5 py-4 text-center text-xs text-white/40">Auto-resolved by Engine</div>
                <div v-else v-for="lv in loaderVersions" :key="lv" @click="launchData.loader_version = lv; isLoaderVerDropdownOpen = false" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition flex items-center justify-between group">
                  <span class="font-bold text-sm transition-colors" :class="launchData.loader_version === lv ? activeColorText : 'text-white/70 group-hover:text-white'">{{ lv }}</span>
                  <CheckCircle v-if="launchData.loader_version === lv" class="w-3.5 h-3.5" :class="activeColorText" />
                </div>
              </div>
            </transition>
          </div>
        </div>

        <div class="kip-card p-2 flex flex-col gap-2 relative overflow-hidden">
          <div class="absolute inset-0 opacity-20 pointer-events-none transition-colors duration-1000" :class="isLaunching ? activeColorBg : 'bg-transparent'"></div>

          <button @click="startGame" :disabled="isLaunching || (!state.settings.has_ms_token && !state.settings.offline_username)" class="w-full py-6 rounded-xl font-black text-2xl tracking-[0.2em] uppercase transition-all duration-300 relative group overflow-hidden border shadow-xl disabled:opacity-50 disabled:cursor-not-allowed z-10" :class="activeColorBtn">
            <div class="absolute inset-0 bg-white/20 translate-y-full group-hover:translate-y-0 transition-transform duration-300 ease-out pointer-events-none"></div>
            <div class="relative z-10 flex items-center justify-center gap-3">
              <template v-if="!state.settings.has_ms_token && !state.settings.offline_username">
                <UserX class="w-7 h-7" /> {{ t('LOGIN REQUIRED') }}
              </template>
              <template v-else-if="!isLaunching">
                <Play class="w-7 h-7 fill-current" /> {{ t('PLAY') }}
              </template>
              <template v-else>
                <Loader class="w-7 h-7 animate-spin" /> {{ t('INITIALIZING') }}
              </template>
            </div>
          </button>

          <transition name="fade">
            <div v-if="isLaunching" class="bg-black/60 p-4 rounded-xl border border-white/5 relative z-10 backdrop-blur-md">
              <div class="flex justify-between text-xs text-white/70 mb-2.5 font-bold tracking-wider uppercase">
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
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, watch, onBeforeUnmount } from 'vue'
import type { Component } from 'vue'
import {
  Play,
  Loader,
  ChevronDown,
  User,
  UserX,
  Cpu,
  Box,
  Hexagon,
  Component as LucideComponent,
  Layers,
  Zap,
  CheckCircle,
} from 'lucide-vue-next'
import { state, t, showToast, getAvatarUrl } from '@/store.js'
import { bridge, type LaunchPayload } from '@/bridge'

export type LoaderType = 'vanilla' | 'fabric' | 'forge' | 'neoforge' | 'quilt'

export interface LoaderItem {
  id: LoaderType
  name: string
  icon: Component
}

const isVersionDropdownOpen = ref<boolean>(false)
const isLoaderVerDropdownOpen = ref<boolean>(false)
const launchData = ref<LaunchPayload>({
  loader: 'vanilla',
  version: '',
  loader_version: '',
})

const isLaunching = ref<boolean>(false)
const loaderVersions = ref<string[]>([])
const isLoadingVersions = ref<boolean>(false)

const loaders: LoaderItem[] = [
  { id: 'vanilla', name: 'Vanilla', icon: Box },
  { id: 'fabric', name: 'Fabric', icon: Hexagon },
  { id: 'forge', name: 'Forge', icon: Zap },
  { id: 'neoforge', name: 'NeoForge', icon: LucideComponent },
  { id: 'quilt', name: 'Quilt', icon: Layers },
]

const activeLoaderName = computed<string>(() => {
  const l = loaders.find((x) => x.id === launchData.value.loader)
  return l ? l.name : 'Engine'
})

const activeLoaderIcon = computed<Component>(() => {
  const l = loaders.find((x) => x.id === launchData.value.loader)
  return l ? l.icon : Box
})

const currentStatusText = computed<string>(() => {
  return state.launchStatus || t('Initializing...')
})

const currentProgressVal = computed<number>(() => {
  return state.launchProgress || 0
})

const buttonColorMap: Record<LoaderType, string> = {
  vanilla: 'bg-emerald-500 text-black border-emerald-400 hover:shadow-[0_0_30px_rgba(16,185,129,0.4)]',
  fabric: 'bg-amber-500 text-black border-amber-400 hover:shadow-[0_0_30px_rgba(245,158,11,0.4)]',
  forge: 'bg-rose-500 text-white border-rose-400 hover:shadow-[0_0_30px_rgba(244,63,94,0.4)]',
  neoforge: 'bg-orange-500 text-black border-orange-400 hover:shadow-[0_0_30px_rgba(249,115,22,0.4)]',
  quilt: 'bg-purple-500 text-white border-purple-400 hover:shadow-[0_0_30px_rgba(168,85,247,0.4)]',
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
  if (!state.settings.has_ms_token && !state.settings.offline_username) {
    return 'bg-red-500/20 text-red-400 border-red-500/50'
  }
  if (isLaunching.value) {
    return 'bg-white/5 text-white/50 border-white/10'
  }
  return buttonColorMap[launchData.value.loader as LoaderType] || buttonColorMap.vanilla
})

const activeColorText = computed<string>(() => {
  return textColorMap[launchData.value.loader as LoaderType] || 'text-emerald-400'
})

const activeColorProgress = computed<string>(() => {
  return progressColorMap[launchData.value.loader as LoaderType] || 'bg-emerald-400'
})

const activeColorBg = computed<string>(() => {
  return bgColorMap[launchData.value.loader as LoaderType] || 'bg-emerald-600'
})

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    isVersionDropdownOpen.value = false
    isLoaderVerDropdownOpen.value = false
  }
}

const fetchLoaderVersions = async (): Promise<void> => {
  if (launchData.value.loader === 'vanilla' || !launchData.value.version) {
    loaderVersions.value = []
    launchData.value.loader_version = ''
    return
  }

  isLoadingVersions.value = true
  launchData.value.loader_version = ''

  try {
    const versions = await bridge.getLoaderVersions(launchData.value.loader, launchData.value.version)
    loaderVersions.value = versions || []
  } catch (err) {
    loaderVersions.value = []
  } finally {
    isLoadingVersions.value = false
  }
}

watch(() => launchData.value.loader, () => {
  fetchLoaderVersions()
})

watch(() => launchData.value.version, () => {
  fetchLoaderVersions()
})

const startGame = async (): Promise<void> => {
  if (!launchData.value.version) return
  if (!state.settings.has_ms_token && !state.settings.offline_username) return

  isLaunching.value = true
  state.launchStatus = t('Authenticating...')
  state.launchProgress = 0

  try {
    const res = await bridge.launchGame(
      launchData.value.version,
      launchData.value.loader,
      launchData.value.loader_version
    )
    if (!res.success) {
      showToast(t('Launch Failed'), res.message, 'danger')
    }
  } catch (e: unknown) {
    const message = e instanceof Error ? e.message : String(e)
    showToast(t('Error'), message || t('Backend communication failed.'), 'danger')
  } finally {
    setTimeout(() => {
      isLaunching.value = false
      state.launchProgress = 0
      state.launchStatus = 'Ready'
    }, 4000)
  }
}

onMounted(async () => {
  window.addEventListener('click', closeDropdowns)

  if (state.mcVersions.length === 0) {
    try {
      state.mcVersions = await bridge.getMcVersions()
      if (state.mcVersions.length > 0 && !state.mcVersions[0].includes('Error')) {
        launchData.value.version = state.mcVersions[0]
      }
    } catch (e) {
      showToast(t('Error'), t('Failed to load Minecraft versions.'), 'danger')
    }
  } else if (!launchData.value.version && state.mcVersions.length > 0) {
    launchData.value.version = state.mcVersions[0]
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>