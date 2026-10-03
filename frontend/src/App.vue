<template>
  <div class="flex flex-col h-full bg-[#050505] overflow-hidden relative" :class="[state.isBigPicture ? 'big-picture-mode' : '']">
    <div class="fixed inset-0 pointer-events-none z-0 overflow-hidden bg-[#050505] transition-opacity duration-700" :class="state.isOverlayActive ? 'opacity-0' : 'opacity-100'">
      <div class="absolute -top-[10%] -left-[10%] w-[55vw] h-[55vw] bg-indigo-600/25 blur-[130px] rounded-full animate-blob mix-blend-screen"></div>
      <div class="absolute top-[20%] -right-[10%] w-[45vw] h-[45vw] bg-purple-600/25 blur-[130px] rounded-full animate-blob animation-delay-2000 mix-blend-screen"></div>
      <div class="absolute -bottom-[20%] left-[20%] w-[65vw] h-[65vw] bg-emerald-600/20 blur-[130px] rounded-full animate-blob animation-delay-4000 mix-blend-screen"></div>
      <div class="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.03)_1px,transparent_1px)] bg-[size:60px_60px] [transform:perspective(1000px)_rotateX(60deg)_translateY(-100px)_scale(2.5)] opacity-40"></div>
      <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_20%,#050505_95%)]"></div>
    </div>

    <transition name="fade" mode="out-in">
      <div v-if="state.showBoot" class="fixed inset-0 bg-[#030305] z-[9999] flex flex-col items-center justify-center overflow-hidden">
        <div class="absolute inset-0 bg-[linear-gradient(rgba(99,102,241,0.03)_1px,transparent_1px),linear-gradient(90deg,rgba(99,102,241,0.03)_1px,transparent_1px)] bg-[size:40px_40px] [mask-image:radial-gradient(ellipse_60%_60%_at_50%_50%,#000_10%,transparent_100%)]"></div>
        <div class="absolute top-[-10%] left-0 w-full h-[20%] bg-gradient-to-b from-transparent via-indigo-500/10 to-transparent blur-md animate-[pulse_4s_ease-in-out_infinite]"></div>
        <div class="absolute top-1/4 left-1/4 w-[400px] h-[400px] bg-indigo-600/10 blur-[100px] rounded-full animate-pulse pointer-events-none"></div>
        <div class="absolute bottom-1/4 right-1/4 w-[400px] h-[400px] bg-emerald-600/10 blur-[100px] rounded-full animate-pulse pointer-events-none" style="animation-delay: 1s;"></div>

        <div class="relative z-10 flex flex-col items-center w-full max-w-lg">
          <div class="relative w-36 h-36 mb-12 flex items-center justify-center">
            <div class="absolute inset-0 border-t-2 border-b-2 border-indigo-500/50 rounded-full animate-[spin_3s_linear_infinite] shadow-[0_0_15px_rgba(99,102,241,0.5)]"></div>
            <div class="absolute inset-2 border-r-2 border-l-2 border-emerald-500/50 rounded-full animate-[spin_4s_linear_infinite_reverse] shadow-[0_0_15px_rgba(16,185,129,0.5)]"></div>
            <div class="absolute inset-4 border-2 border-dashed border-purple-500/40 rounded-full animate-[spin_10s_linear_infinite]"></div>
            <div class="absolute inset-8 border border-white/10 rounded-full animate-ping opacity-30" style="animation-duration: 2s;"></div>
            <div class="absolute inset-0 bg-indigo-500/10 blur-xl rounded-full animate-pulse"></div>
            <Hexagon class="text-white w-10 h-10 relative z-10 drop-shadow-[0_0_15px_rgba(255,255,255,0.8)]" />
          </div>

          <h1 class="text-5xl font-extrabold tracking-[0.3em] mb-4 uppercase flex items-center gap-2 drop-shadow-[0_0_10px_rgba(255,255,255,0.3)]">
            {{ state.appName }}<span class="text-transparent bg-clip-text bg-gradient-to-r from-indigo-400 to-emerald-400">{{ state.appAccent }}</span>
          </h1>
          <p class="text-white/30 font-mono text-[10px] tracking-[0.4em] mb-12 uppercase border border-white/10 px-4 py-1.5 rounded-full bg-white/5 shadow-inner">
            {{ t('Engine Version') }} {{ state.version || '1.5.8' }}
          </p>

          <div class="w-full flex flex-col gap-3 relative">
            <div class="flex justify-between items-end px-2">
              <span class="font-mono text-[10px] tracking-[0.2em] text-white/50 uppercase flex items-center gap-2">
                <Activity class="w-3.5 h-3.5 text-indigo-400 animate-pulse" /> {{ t('System Status') }}
              </span>
              <span class="font-mono text-sm tracking-widest font-bold" :class="state.bootProgress >= 100 ? 'text-emerald-400 drop-shadow-[0_0_10px_rgba(52,211,153,0.8)]' : 'text-indigo-400 drop-shadow-[0_0_10px_rgba(99,102,241,0.8)]'">
                {{ Math.floor(state.bootProgress) }}%
              </span>
            </div>

            <div class="w-full h-1.5 bg-black/50 border border-white/10 rounded-full overflow-hidden relative shadow-[inset_0_0_10px_rgba(0,0,0,0.8)]">
              <div class="absolute top-0 left-0 h-full bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-500 transition-all duration-75 ease-linear" :style="{ width: state.bootProgress + '%' }">
                <div class="absolute inset-0 bg-white/20" style="background-image: repeating-linear-gradient(45deg, transparent, transparent 5px, rgba(0,0,0,0.2) 5px, rgba(0,0,0,0.2) 10px);"></div>
              </div>
            </div>

            <div class="flex items-center justify-between mt-2 px-2">
              <div class="flex items-center gap-2">
                <Loader v-if="state.bootProgress < 100" class="w-3.5 h-3.5 text-indigo-400 animate-spin" />
                <CheckCircle v-else class="w-3.5 h-3.5 text-emerald-400 drop-shadow-[0_0_8px_rgba(52,211,153,0.8)]" />
                <p class="font-mono text-[10px] tracking-[0.1em] uppercase transition-colors duration-300" :class="state.bootProgress >= 100 ? 'text-emerald-400 font-bold' : 'text-white/70'">
                  {{ t(state.bootText) }}
                </p>
              </div>
            </div>

            <div class="h-24 mt-4 bg-black/40 border border-white/5 rounded-xl p-3 overflow-hidden relative shadow-inner flex flex-col justify-end">
              <div class="absolute inset-0 bg-gradient-to-b from-[#030305] via-transparent to-transparent pointer-events-none z-10"></div>
              <div class="font-mono text-[9px] text-emerald-400/60 tracking-wider flex flex-col gap-1">
                <div v-for="(log, idx) in bootLogs" :key="idx" class="truncate" :class="{'text-white/90 font-bold drop-shadow-[0_0_5px_rgba(255,255,255,0.5)]': idx === bootLogs.length - 1, 'opacity-40': idx < bootLogs.length - 2}">
                  <span class="text-indigo-400/50">></span> {{ log }}
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="!state.showBoot && !state.settings.eula_accepted" class="fixed inset-0 bg-black/95 backdrop-blur-2xl z-[99999] flex items-center justify-center p-10 cursor-default">
        <div class="kip-card max-w-2xl w-full p-8 border-indigo-500/30 shadow-[0_0_50px_rgba(99,102,241,0.2)] relative overflow-hidden flex flex-col">
          <div class="absolute -top-32 -left-32 w-64 h-64 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none"></div>
          <h2 class="text-3xl font-extrabold mb-4 flex items-center gap-3 relative z-10"><ShieldCheck class="w-8 h-8 text-indigo-400" /> License Agreement</h2>
          <div class="bg-black/40 border border-white/5 p-5 rounded-xl mb-6 h-64 overflow-y-auto custom-scroll text-sm text-white/70 leading-relaxed font-medium relative z-10 shadow-inner">
            NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
            <br><br>
            This software downloads files directly from Mojang servers. A valid Minecraft license is required to play. Provided "AS IS" under the GNU General Public License v3.0 (GPL-3.0).
            <br><br>
            By clicking "I Accept", you agree to these terms and confirm you understand this is a third-party open-source manager licensed under GPL-3.0. Data such as crash logs may be sent to AI providers (Google/OpenAI/Anthropic) if you explicitly use the AI Support feature. Your Microsoft tokens are stored locally. No personal data is collected by K.I.P. Studio.
          </div>
          <button @click="acceptEula" class="kip-btn-primary w-full py-4 text-lg tracking-widest uppercase shadow-[0_0_20px_rgba(99,102,241,0.4)] relative z-10">I Accept</button>
        </div>
      </div>
    </transition>

    <div v-show="!state.isOverlayActive" class="titlebar flex justify-between items-center px-6 py-4 z-50 transition-colors duration-300 bg-black/60 border-b border-white/5 backdrop-blur-md relative" data-tauri-drag-region>
      <div class="flex items-center gap-3">
        <Hexagon class="text-indigo-500 w-5 h-5" />
        <span class="font-bold tracking-[0.1em] text-sm uppercase">{{ state.appName }}<span class="text-indigo-400">{{ state.appAccent }}</span></span>
      </div>
      <div class="flex gap-4 titlebar-btn items-center">
        <button @click="toggleBigPicture" class="text-white/50 hover:text-amber-400 transition" :title="t('Big Picture Mode')">
          <Gamepad2 class="w-4 h-4" />
        </button>
        <div class="w-px h-4 bg-white/10 mx-1"></div>
        <button @click="state.isNexusOpen = !state.isNexusOpen" class="text-white/50 hover:text-emerald-400 transition relative flex items-center">
          <Users class="w-4 h-4" />
          <span class="absolute -top-1 -right-1 w-2 h-2 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]"></span>
        </button>
        <div class="w-px h-4 bg-white/10 mx-1"></div>
        <button @click="windowMinimize" class="text-white/50 hover:text-indigo-400 transition"><Minus class="w-4 h-4" /></button>
        <button @click="windowMaximize" class="text-white/50 hover:text-indigo-400 transition"><Square class="w-3.5 h-3.5" /></button>
        <button @click="windowClose" class="text-white/50 hover:text-red-500 transition"><X class="w-4 h-4" /></button>
      </div>
    </div>

    <div v-show="!state.isOverlayActive" class="flex flex-1 overflow-hidden relative z-10">
      <div v-show="!state.isMiniMode" class="border-r border-white/5 flex flex-col py-6 gap-2 z-40 transition-all duration-300 bg-black/40 backdrop-blur-xl shrink-0" :class="isSidebarCollapsed ? 'w-20 px-3' : 'w-64 px-4'">
        <div class="flex items-center justify-between mb-4 px-2" :class="isSidebarCollapsed ? 'justify-center' : ''">
          <button @click="isSidebarCollapsed = !isSidebarCollapsed" class="text-white/40 hover:text-white transition">
            <Menu v-if="isSidebarCollapsed" class="w-5 h-5" />
            <AlignLeft v-else class="w-5 h-5" />
          </button>
        </div>

        <div v-if="!isSidebarCollapsed" class="px-2 mb-4">
          <select v-model="state.settings.mc_dir" @change="changeInstance" class="kip-input appearance-none cursor-pointer text-xs py-2 truncate" :title="state.settings.mc_dir">
            <option v-for="inst in state.settings.instances" :key="inst" :value="inst">{{ formatInstanceName(inst) }}</option>
          </select>
        </div>

        <div v-for="nav in state.navItems" :key="nav.id" @click="state.currentView = nav.id" class="flex items-center rounded-xl cursor-pointer transition-all duration-200" :class="[state.currentView === nav.id ? 'bg-indigo-500/10 text-indigo-400' : 'text-white/60 hover:bg-white/5 hover:text-white', isSidebarCollapsed ? 'p-3 justify-center' : 'px-4 py-3.5 gap-3']" :title="isSidebarCollapsed ? t(nav.label) : ''">
          <component :is="getIcon(nav.icon)" class="w-5 h-5 shrink-0" />
          <span v-if="!isSidebarCollapsed" class="font-medium text-sm">{{ t(nav.label) }}</span>
        </div>

        <div class="mt-auto"></div>

        <div class="px-4 py-3 mx-2 mb-4 rounded-xl border transition-colors duration-300 flex items-center gap-3" :class="[state.isMcRunning ? 'bg-emerald-500/10 border-emerald-500/20' : 'bg-black/40 border-white/5', isSidebarCollapsed ? 'justify-center p-3' : '']" :title="isSidebarCollapsed ? t(state.mcStatusText) : ''">
          <div class="w-2.5 h-2.5 rounded-full shrink-0" :class="state.isMcRunning ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)] animate-pulse' : 'bg-white/30'"></div>
          <span v-if="!isSidebarCollapsed" class="text-xs font-bold truncate" :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/50'">{{ t(state.mcStatusText) }}</span>
        </div>

        <button v-if="hasUpdate" @click="performUpdate" :disabled="isUpdating" class="kip-btn-primary py-3" :class="isSidebarCollapsed ? 'p-3' : 'w-full mx-2'" :title="isSidebarCollapsed ? t('Update App') : ''">
          <Loader v-if="isUpdating" class="w-4 h-4 animate-spin" />
          <Download v-else class="w-4 h-4" />
          <span v-if="!isSidebarCollapsed">{{ isUpdating ? t('Updating...') : t('Update App') }}</span>
        </button>
      </div>

      <div class="flex-1 overflow-y-auto custom-scroll relative flex flex-col min-h-0 transition-all duration-500 p-10" :class="state.isBigPicture ? 'p-16 pb-48' : ''" id="main-content">
        <transition name="fade" mode="out-in">
          <component :is="currentViewComponent" />
        </transition>
      </div>

      <transition name="slide">
        <Nexus v-if="state.isNexusOpen" />
      </transition>
    </div>

    <transition name="fade">
      <div v-if="state.partyInvite" class="fixed top-8 left-1/2 -translate-x-1/2 z-[10000] kip-card p-5 border border-indigo-500/50 shadow-[0_0_50px_rgba(99,102,241,0.3)] flex items-center gap-6 animate-[bounce_2s_infinite]">
        <div class="relative shrink-0">
          <img :src="getAvatarUrl(state.partyInvite.senderName)" class="w-12 h-12 rounded-xl object-cover border-2 border-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.8)]">
          <div class="absolute -bottom-2 -right-2 p-1 bg-black rounded-lg">
            <Gamepad2 class="w-4 h-4 text-emerald-400 animate-pulse" />
          </div>
        </div>
        <div class="flex-1">
          <h4 class="font-extrabold text-white text-lg tracking-wider mb-0.5">K.I.P. Party Invite</h4>
          <p class="text-xs text-white/70">From <span class="font-bold text-indigo-400">{{ state.partyInvite.senderName }}</span> • {{ state.partyInvite.mods.length }} mods</p>
        </div>
        <div class="flex gap-2 shrink-0 ml-4">
          <button @click="acceptPartyInvite" class="px-5 py-2.5 bg-emerald-500 hover:bg-emerald-400 text-black font-extrabold rounded-xl transition shadow-[0_0_15px_rgba(16,185,129,0.4)]">Accept</button>
          <button @click="declinePartyInvite" class="px-5 py-2.5 bg-red-500/20 hover:bg-red-500 text-red-400 hover:text-white font-bold rounded-xl transition border border-red-500/30">Decline</button>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="state.isBigPicture" class="fixed bottom-0 left-0 right-0 h-16 bg-black/80 backdrop-blur-2xl border-t border-white/10 z-[8000] flex items-center justify-between px-10 select-none pointer-events-none">
        <div class="flex items-center gap-6">
          <div class="flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-emerald-500 text-black font-black text-xs flex items-center justify-center shadow-[0_0_10px_rgba(16,185,129,0.5)]">A</span>
            <span class="text-xs font-bold text-white/80 uppercase tracking-wider">{{ t('Select') }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="w-6 h-6 rounded-full bg-red-500 text-white font-black text-xs flex items-center justify-center shadow-[0_0_10px_rgba(239,68,68,0.5)]">B</span>
            <span class="text-xs font-bold text-white/80 uppercase tracking-wider">{{ t('Back / Close') }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="px-2 py-0.5 rounded bg-white/20 text-white font-mono font-bold text-xs border border-white/20">LB / RB</span>
            <span class="text-xs font-bold text-white/80 uppercase tracking-wider">{{ t('Switch Tab') }}</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="px-2 py-0.5 rounded bg-white/20 text-white font-mono font-bold text-xs border border-white/20">D-PAD</span>
            <span class="text-xs font-bold text-white/80 uppercase tracking-wider">{{ t('Navigate') }}</span>
          </div>
        </div>
        <div class="flex items-center gap-4 pointer-events-auto">
          <button @click="toggleBigPicture" class="kip-btn-danger px-4 py-1.5 text-xs rounded-xl flex items-center gap-2 shadow-lg">
            <X class="w-4 h-4" /> {{ t('Exit Big Picture') }}
          </button>
        </div>
      </div>
    </transition>

    <div class="fixed bottom-5 right-5 z-[300] flex flex-col gap-3 pointer-events-none">
      <transition-group name="toast">
        <div v-for="toast in toasts" :key="toast.id" class="kip-card p-4 flex items-start gap-3 border-l-4 w-80 shadow-2xl pointer-events-auto" :class="toast.type === 'success' ? 'border-emerald-500' : toast.type === 'danger' ? 'border-red-500' : 'border-indigo-500'">
          <component :is="getIcon(toast.icon)" class="w-6 h-6 mt-0.5 shrink-0" :class="toast.type === 'success' ? 'text-emerald-400' : toast.type === 'danger' ? 'text-red-400' : 'text-indigo-400'" />
          <div class="overflow-hidden">
            <h4 class="font-bold text-white text-sm truncate">{{ toast.title }}</h4>
            <p class="text-white/60 text-xs mt-1 break-words">{{ toast.message }}</p>
          </div>
        </div>
      </transition-group>
    </div>

    <transition name="fade">
      <Overlay v-if="state.isOverlayActive" />
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import type { Component } from 'vue'
import {
  Hexagon,
  Gamepad2,
  Users,
  Minus,
  Square,
  X,
  Loader,
  Download,
  Zap,
  LayoutDashboard,
  Wand2,
  Puzzle,
  ShoppingCart,
  Globe,
  Wifi,
  Image as ImageIcon,
  Terminal,
  LifeBuoy,
  Settings,
  CheckCircle,
  XCircle,
  Info,
  Activity,
  Menu,
  AlignLeft,
  ShieldCheck,
} from 'lucide-vue-next'

import Dashboard from '@/views/Dashboard.vue'
import Launcher from '@/views/Launcher.vue'
import Builder from '@/views/Builder.vue'
import Mods from '@/views/Mods.vue'
import Store from '@/views/Store.vue'
import Worlds from '@/views/Worlds.vue'
import Network from '@/views/Network.vue'
import Tools from '@/views/Tools.vue'
import Media from '@/views/Media.vue'
import Console from '@/views/Console.vue'
import Support from '@/views/Support.vue'
import AppSettings from '@/views/Settings.vue'
import Overlay from '@/views/Overlay.vue'
import Nexus from '@/views/Nexus.vue'

import {
  state,
  t,
  showToast,
  toasts,
  loadSettings,
  loadTranslations,
  loadDashboardStats,
  windowMinimize,
  windowMaximize,
  windowClose,
  toggleBigPicture,
  sanitizeHTML,
  getAvatarUrl,
  leaveVoiceChannel,
  acceptPartyInvite,
  declinePartyInvite,
  initGamepadMode,
  acceptEula,
  setupTauriListeners,
} from '@/store'
import {
  bridge,
  type ImportDroppedModsResult,
} from '@/bridge'

const isSidebarCollapsed = ref<boolean>(false)
const isUpdating = ref<boolean>(false)
const hasUpdate = ref<boolean>(false)

let mouseIdleTimer: ReturnType<typeof setTimeout> | null = null

const bootLogs = ref<string[]>([
  'INITIALIZING NEURAL CORE...',
  'ALLOCATING MEMORY BLOCKS...',
  'ESTABLISHING SECURE CONNECTION...',
])

const viewsMap: Record<string, Component> = {
  dashboard: Dashboard,
  launcher: Launcher,
  builder: Builder,
  mods: Mods,
  store: Store,
  worlds: Worlds,
  network: Network,
  tools: Tools,
  media: Media,
  console: Console,
  support: Support,
  settings: AppSettings,
}

const currentViewComponent = computed<Component>(() => {
  return viewsMap[state.currentView] || Dashboard
})

const iconsMap: Record<string, Component> = {
  'layout-dashboard': LayoutDashboard,
  'gamepad-2': Gamepad2,
  'wand-2': Wand2,
  puzzle: Puzzle,
  'shopping-cart': ShoppingCart,
  globe: Globe,
  wifi: Wifi,
  zap: Zap,
  image: ImageIcon,
  terminal: Terminal,
  'life-buoy': LifeBuoy,
  settings: Settings,
  'check-circle': CheckCircle,
  'x-circle': XCircle,
  info: Info,
}

const getIcon = (name: string): Component => {
  return iconsMap[name] || Info
}

const formatInstanceName = (path: string): string => {
  if (!path) return 'Instance'
  const p = path.toLowerCase().replace(/\\/g, '/')
  if (p.endsWith('.minecraft') || p.endsWith('.minecraft/')) {
    return 'Default (.minecraft)'
  }
  const clean = path.replace(/[\\/]+$/, '')
  const name = clean.split(/[\\/]/).pop()
  return name || path
}

const handleMouseMove = (): void => {
  if (!state.isBigPicture) {
    document.body.classList.remove('cursor-none')
    return
  }
  document.body.classList.remove('cursor-none')
  if (mouseIdleTimer) clearTimeout(mouseIdleTimer)
  mouseIdleTimer = setTimeout(() => {
    if (state.isBigPicture) {
      document.body.classList.add('cursor-none')
    }
  }, 3000)
}

const handleGlobalKeyDown = (e: KeyboardEvent): void => {
  if (e.shiftKey && (e.key === 'Tab' || e.keyCode === 9)) {
    e.preventDefault()
    e.stopPropagation()
    toggleOverlayState()
  }
}

const toggleOverlayState = (): void => {
  state.isOverlayActive = !state.isOverlayActive
  if (state.isOverlayActive) {
    document.body.classList.add('in-game-overlay')
    document.documentElement.classList.add('in-game-overlay')
  } else {
    document.body.classList.remove('in-game-overlay')
    document.documentElement.classList.remove('in-game-overlay')
  }
}

const checkForAppUpdates = async (): Promise<void> => {
  try {
    const res = await bridge.checkAppUpdate()
    if (res && res.has_update) {
      hasUpdate.value = true
      showToast(t('Update Available'), `Version ${res.version} is ready to install.`, 'info')
    }
  } catch {
  }
}

const performUpdate = async (): Promise<void> => {
  isUpdating.value = true
  try {
    const res = await bridge.performAppUpdate()
    if (!res) {
      isUpdating.value = false
    }
  } catch (err: unknown) {
    isUpdating.value = false
    const errorMessage = err instanceof Error ? err.message : String(err)
    showToast(t('Update Error'), errorMessage || t('Failed to download or install update.'), 'danger')
  }
}

const initApp = async (): Promise<void> => {
  try {
    const initData = await bridge.getInitData()
    if (initData) {
      state.appName = initData.appName
      state.appAccent = initData.appAccent
      state.greeting = initData.greeting
      state.version = initData.version
    }
  } catch {
  }

  await loadSettings()
  await loadTranslations()
  await loadDashboardStats()
  await checkForAppUpdates()

  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('keydown', handleGlobalKeyDown)

  const phases = [
    { target: 15, text: 'MOUNTING VIRTUAL FILE SYSTEMS...', log: 'VFS: Mounted successfully.' },
    { target: 35, text: 'LOADING MACHINE LEARNING MODELS...', log: 'AI: Models initialized.' },
    { target: 60, text: 'ESTABLISHING P2P NODES...', log: 'NETWORK: Swarm nodes active.' },
    { target: 85, text: 'CALIBRATING ENGINE INTERFACE...', log: 'UI: Render pipeline ready.' },
    { target: 100, text: 'SYSTEM NOMINAL. WELCOME.', log: 'BOOT: Sequence complete.' },
  ]

  let currentPhase = 0

  const animateProgress = (): void => {
    if (currentPhase >= phases.length) {
      setTimeout(() => {
        state.showBoot = false
      }, 1000)
      return
    }

    const phase = phases[currentPhase]
    if (!phase) return
    const target = phase.target
    state.bootText = phase.text

    const step = (): void => {
      if (state.bootProgress < target) {
        state.bootProgress += Math.random() * 3 + 1
        if (state.bootProgress > target) state.bootProgress = target
        requestAnimationFrame(step)
      } else {
        bootLogs.value.push(phase.log)
        if (bootLogs.value.length > 5) bootLogs.value.shift()
        currentPhase++
        setTimeout(animateProgress, 250)
      }
    }
    requestAnimationFrame(step)
  }

  setTimeout(animateProgress, 300)
}

const changeInstance = async (): Promise<void> => {
  if (state.settings.mc_dir) {
    try {
      const res = await bridge.saveSetting('mc_dir', state.settings.mc_dir)
      if (res) {
        showToast(t('Instance Changed'), t('Switched game directory'), 'success')
        await loadDashboardStats()
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), errorMessage || t('Failed to switch instance directory.'), 'danger')
    }
  }
}

const handleFileDrop = async (e: DragEvent): Promise<void> => {
  e.preventDefault()
  e.stopPropagation()
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const filePaths = Array.from(e.dataTransfer.files)
      .map((f) => (f as unknown as { path?: string }).path)
      .filter((p): p is string => typeof p === 'string' && p.length > 0)

    if (filePaths.length > 0) {
      try {
        const res: ImportDroppedModsResult = await bridge.importDroppedMods(filePaths)
        if (res.success) {
          showToast(
            t('Imported'),
            `${t('Successfully imported')} ${res.count} ${t('items.')}`,
            'success'
          )
        } else {
          showToast(t('Import Error'), res.msg || t('Failed to import files.'), 'danger')
        }
      } catch (err: unknown) {
        const errorMessage = err instanceof Error ? err.message : String(err)
        showToast(t('Error'), errorMessage || t('Failed to import files.'), 'danger')
      }
    }
  }
}

onMounted(() => {
  setupTauriListeners({
    onDaemonStatus: (isRunning: boolean, status: string) => {
      state.isMcRunning = isRunning
      state.mcStatusText = status
    },
    onConsoleLine: (line: string) => {
      let safeLine = sanitizeHTML(line)
      if (safeLine.includes('ERROR') || safeLine.includes('Exception')) {
        safeLine = `<span class="text-red-400">${safeLine}</span>`
      } else if (safeLine.includes('WARN')) {
        safeLine = `<span class="text-amber-400">${safeLine}</span>`
      }
      state._consoleBuffer.push(safeLine)
      if (state._consoleBuffer.length > 300) state._consoleBuffer.shift()
      state.consoleHtml = state._consoleBuffer.join('<br>')
    },
    onCrashAlert: (logData: string) => {
      showToast(t('CRASH DETECTED'), t('Minecraft exited abnormally.'), 'danger')
      state.aiInputText = logData
      state.currentView = 'support'
    },
    onLaunchStatus: (launchMessage: string) => {
      state.launchStatus = launchMessage
    },
    onLaunchProgress: (progress: number) => {
      state.launchProgress = progress
    },
    onTunnelStatus: (tunnelMessage: string) => {
      if (tunnelMessage.includes('Error') || tunnelMessage.includes('NO_SSH')) {
        showToast(t('Tunnel Error'), tunnelMessage, 'danger')
      } else {
        showToast(t('Tunnel Online'), `TCP: ${tunnelMessage}`, 'success')
      }
    },
    onToggleOverlay: () => {
      toggleOverlayState()
    },
  })

  document.addEventListener('dragover', (e: DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
  })
  document.addEventListener('drop', handleFileDrop)

  initApp()
  initGamepadMode()
})

onBeforeUnmount(() => {
  window.removeEventListener('mousemove', handleMouseMove)
  window.removeEventListener('keydown', handleGlobalKeyDown)
  document.removeEventListener('drop', handleFileDrop)
  if (mouseIdleTimer) clearTimeout(mouseIdleTimer)
  leaveVoiceChannel()
})
</script>