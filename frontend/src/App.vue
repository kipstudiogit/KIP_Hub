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
  Menu,
  AlignLeft,
  ShieldCheck,
  Pin,
  Copy,
  Activity,
  Minimize2,
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
import BigPictureDeck from '@/components/bigpicture/BigPictureDeck.vue'
import MiniWidgetDeck from '@/components/mini/MiniWidgetDeck.vue'

import {
  state,
  t,
  showToast,
  toasts,
  loadSettings,
  loadTranslations,
  loadDashboardStats,
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
  isMaximized,
  isPinned,
  minimizeWindow,
  toggleMaximize,
  closeWindow,
  togglePinWindow,
  toggleBigPicture,
  toggleMiniMode,
  syncWindowState,
} from '@/composables/useWindow'
import { useOverlay } from '@/composables/useOverlay'
import {
  bridge,
  invokeSafe,
  type ContentActionResultDto,
} from '@/bridge'

interface InitialAppPayload {
  appName?: string
  app_name?: string
  appAccent?: string
  app_accent?: string
  version?: string
  greeting?: string
  detectedJava?: string
  activeInstance?: string
  systemMemoryGb?: number
}

interface AppSettingsExtension {
  scale?: number
  mica?: boolean
}

const isSidebarCollapsed = ref<boolean>(false)
const isUpdating = ref<boolean>(false)
const hasUpdate = ref<boolean>(false)

let mouseIdleTimer: ReturnType<typeof setTimeout> | null = null

const { toggleOverlayState, exitOverlayMode } = useOverlay()

const bootLogs = ref<string[]>([
  'INITIALIZING QUANTUM HARDWARE MATRIX...',
  'AUDITING ISOLATED WAL PERSISTENCE...',
  'CALIBRATING ACTIVE LAUNCHPAD CONTAINER...',
])

const resolvedAppName = computed<string>(() => {
  return state.appName || 'K.I.P.'
})

const resolvedAppAccent = computed<string>(() => {
  return state.appAccent || ' Hub'
})

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
  if (!path) return 'Default (.minecraft)'
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
  if (e.key === 'Escape') {
    if (state.showBoot) {
      skipBootSequence()
    } else if (state.isOverlayActive) {
      toggleOverlayState()
    }
  } else if (e.shiftKey && (e.key === 'Tab' || e.keyCode === 9)) {
    e.preventDefault()
    e.stopPropagation()
    toggleOverlayState()
  }
}

const skipBootSequence = (): void => {
  state.bootProgress = 100
  state.showBoot = false
}

const checkForAppUpdates = async (): Promise<void> => {
  try {
    const res = await bridge.checkAppUpdate()
    if (res && res.has_update) {
      hasUpdate.value = true
      showToast(t('Update Available'), `Version ${res.version} is ready to install.`, 'info')
    }
  } catch {}
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
    const initData = await invokeSafe<InitialAppPayload>('get_init_data')
    if (initData) {
      state.appName = initData.appName || initData.app_name || 'K.I.P.'
      state.appAccent = initData.appAccent || initData.app_accent || ' Hub'
      state.greeting = initData.greeting || 'Welcome'
      state.version = initData.version || '1.7.0'
    }
  } catch {
    state.appName = 'K.I.P.'
    state.appAccent = ' Hub'
  }

  await loadSettings()
  await loadTranslations()
  await loadDashboardStats()
  await checkForAppUpdates()
  await syncWindowState()

  const extSettings = state.settings as typeof state.settings & AppSettingsExtension

  if (state.settings.low_graphics) {
    document.documentElement.classList.add('low-graphics-mode')
    document.body.classList.add('low-graphics-mode')
  }

  if (state.settings.mica === false) {
    document.documentElement.classList.add('no-blur-mode')
    document.body.classList.add('no-blur-mode')
  }

  const accent = state.settings.theme_accent || 'indigo'
  document.documentElement.setAttribute('data-accent', accent)
  document.body.setAttribute('data-accent', accent)

  if (extSettings.scale && extSettings.scale !== 1.0) {
    document.documentElement.style.zoom = String(extSettings.scale)
    document.body.style.zoom = String(extSettings.scale)
  }

  window.addEventListener('mousemove', handleMouseMove)
  window.addEventListener('keydown', handleGlobalKeyDown)

  const phases = [
    { target: 20, text: 'AUDITING HOST COMPILERS & RUNTIMES...', log: 'JVM: OpenJDK matrix synchronized.' },
    { target: 45, text: 'MOUNTING MINECRAFT 26.x REPOSITORIES...', log: 'STORAGE: Virtual instance linked.' },
    { target: 70, text: 'CALIBRATING K.I.P. SHIELD HEURISTICS...', log: 'SECURITY: Real-time integrity primed.' },
    { target: 90, text: 'SYNCHRONIZING SECURE WEBRTC MESH...', log: 'NETWORK: Signaling gate online.' },
    { target: 100, text: 'SYSTEM OPERATIONAL • LAUNCHPAD READY', log: 'CORE: Launch sequence initialized.' },
  ]

  let currentPhase = 0

  const animateProgress = (): void => {
    if (!state.showBoot) return

    if (currentPhase >= phases.length) {
      setTimeout(() => {
        state.showBoot = false
      }, 700)
      return
    }

    const phase = phases[currentPhase]
    if (!phase) return
    const target = phase.target
    state.bootText = phase.text

    const step = (): void => {
      if (!state.showBoot) return

      if (state.bootProgress < target) {
        state.bootProgress += Math.random() * 4 + 2
        if (state.bootProgress > target) state.bootProgress = target
        requestAnimationFrame(step)
      } else {
        bootLogs.value.push(phase.log)
        if (bootLogs.value.length > 5) bootLogs.value.shift()
        currentPhase++
        setTimeout(animateProgress, 200)
      }
    }
    requestAnimationFrame(step)
  }

  setTimeout(animateProgress, 200)
}

const changeInstance = async (): Promise<void> => {
  if (state.settings.mc_dir) {
    try {
      const res = await invokeSafe<boolean>('change_instance', { newDir: state.settings.mc_dir })
      if (res) {
        showToast(t('Instance Changed'), t('Switched game directory'), 'success')
        await loadSettings()
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
        const res: ContentActionResultDto = await bridge.importDroppedMods(filePaths)
        if (res.success) {
          showToast(
            t('Imported'),
            `${t('Successfully imported')} ${res.count || 0} ${t('items.')}`,
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
      if (!isRunning && state.isOverlayActive) {
        exitOverlayMode()
      }
    },
    onConsoleLine: (line: string) => {
      let safeLine = sanitizeHTML(line)
      if (safeLine.includes('ERROR') || safeLine.includes('Exception')) {
        safeLine = `<span class="text-rose-400">${safeLine}</span>`
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

<template>
  <div
    class="flex flex-col h-full overflow-hidden relative border transition-all duration-700 ease-out"
    :class="[
      state.isBigPicture ? 'big-picture-mode' : '',
      state.isMiniMode ? 'border-none rounded-2xl' : '',
      state.isOverlayActive ? 'border-none rounded-none' : '',
      state.isMcRunning
        ? 'border-emerald-500/50 shadow-[0_0_60px_rgba(16,185,129,0.25)]'
        : state.settings.safe_mode
        ? 'border-rose-500/50 shadow-[0_0_60px_rgba(244,63,94,0.25)]'
        : 'border-[var(--accent-border)] shadow-[0_0_50px_var(--accent-glow)]'
    ]"
    :style="{ backgroundColor: state.isOverlayActive ? 'transparent' : (state.settings.mica && !state.settings.low_graphics ? 'rgba(5, 5, 8, 0.78)' : '#07070a') }"
  >
    <BigPictureDeck v-if="state.isBigPicture" @exit="toggleBigPicture" />

    <MiniWidgetDeck v-else-if="state.isMiniMode" @expand="toggleMiniMode" />

    <template v-else>
      <div
        v-if="!state.settings.low_graphics"
        class="fixed inset-0 pointer-events-none z-0 overflow-hidden transition-opacity duration-700"
        :class="state.isOverlayActive ? 'opacity-0' : 'opacity-100'"
      >
        <div class="absolute -top-[10%] -left-[10%] w-[55vw] h-[55vw] bg-[var(--accent-color)] opacity-20 blur-[140px] rounded-full animate-blob mix-blend-screen"></div>
        <div class="absolute top-[20%] -right-[10%] w-[45vw] h-[45vw] bg-purple-600/20 blur-[140px] rounded-full animate-blob animation-delay-2000 mix-blend-screen"></div>
        <div class="absolute -bottom-[20%] left-[20%] w-[65vw] h-[65vw] bg-emerald-600/15 blur-[140px] rounded-full animate-blob animation-delay-4000 mix-blend-screen"></div>
        <div class="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:60px_60px] [transform:perspective(1000px)_rotateX(60deg)_translateY(-100px)_scale(2.5)] opacity-30"></div>
        <div class="absolute inset-0 bg-[radial-gradient(ellipse_at_center,transparent_20%,rgba(5,5,8,0.92)_95%)]"></div>
      </div>

      <transition name="fade" mode="out-in">
        <div v-if="state.showBoot" class="fixed inset-0 bg-[#020204] z-[9999] flex flex-col items-center justify-center overflow-hidden select-none">
          <div class="absolute inset-0 bg-[radial-gradient(circle_at_center,var(--accent-subtle)_0%,transparent_70%)]"></div>
          <div class="absolute inset-0 bg-[linear-gradient(rgba(255,255,255,0.02)_1px,transparent_1px),linear-gradient(90deg,rgba(255,255,255,0.02)_1px,transparent_1px)] bg-[size:32px_32px]"></div>

          <button
            @click="skipBootSequence"
            class="absolute top-6 right-6 px-4 py-2 rounded-xl bg-white/5 hover:bg-white/10 border border-white/10 text-white/40 hover:text-white text-xs font-mono font-bold uppercase tracking-wider transition-all z-20 flex items-center gap-2 cursor-pointer"
          >
            <span>Skip Boot [ESC]</span>
          </button>

          <div class="relative z-10 flex flex-col items-center w-full max-w-xl px-6">
            <div class="relative w-44 h-44 mb-8 flex items-center justify-center">
              <div class="absolute inset-0 rounded-full border border-[var(--accent-border)] border-dashed animate-[spin_12s_linear_infinite]"></div>
              <div class="absolute inset-3 rounded-full border-2 border-t-[var(--accent-color)] border-b-emerald-400 border-l-transparent border-r-transparent animate-[spin_4s_cubic-bezier(0.4,0,0.2,1)_infinite] shadow-[0_0_30px_var(--accent-glow)]"></div>
              <div class="absolute inset-6 rounded-full border border-purple-500/30 animate-[spin_8s_linear_infinite_reverse]"></div>
              <div class="absolute inset-10 rounded-full bg-[var(--accent-subtle)] blur-xl animate-pulse"></div>
              <div class="relative z-10 p-5 rounded-3xl bg-black/60 border border-white/15 backdrop-blur-xl shadow-2xl flex items-center justify-center">
                <Hexagon class="text-white w-12 h-12 stroke-[1.5] drop-shadow-[0_0_20px_var(--accent-glow)]" />
              </div>
            </div>

            <div class="flex items-center gap-3 mb-2">
              <h1 class="text-5xl font-black tracking-[0.2em] uppercase text-white drop-shadow-[0_0_25px_rgba(255,255,255,0.4)]">
                {{ resolvedAppName }}
              </h1>
              <span class="px-3 py-1 rounded-xl text-xs font-black uppercase tracking-widest bg-gradient-to-r from-[var(--accent-subtle)] to-emerald-500/20 text-emerald-300 border border-emerald-500/40 shadow-[0_0_15px_rgba(16,185,129,0.3)]">
                {{ resolvedAppAccent }}
              </span>
            </div>

            <p class="text-white/40 font-mono text-[10px] tracking-[0.35em] uppercase mb-8">
              Neural Gaming Fabric • Minecraft 26.x Engine Core
            </p>

            <div class="w-full flex flex-col gap-3">
              <div class="flex justify-between items-center px-1 font-mono text-xs">
                <span class="text-white/50 uppercase tracking-widest flex items-center gap-2 font-bold">
                  <span class="w-2 h-2 rounded-full bg-emerald-400 animate-ping"></span>
                  <span>{{ t(state.bootText) }}</span>
                </span>
                <span class="font-black text-[var(--accent-color)] drop-shadow-[0_0_10px_var(--accent-glow)]">
                  {{ Math.floor(state.bootProgress) }}%
                </span>
              </div>

              <div class="w-full h-2 bg-black/80 border border-white/10 rounded-full overflow-hidden p-0.5 shadow-inner">
                <div
                  class="h-full bg-gradient-to-r from-[var(--accent-color)] via-purple-500 to-emerald-400 rounded-full transition-all duration-150 ease-out shadow-[0_0_15px_var(--accent-glow)]"
                  :style="{ width: state.bootProgress + '%' }"
                ></div>
              </div>

              <div class="h-28 mt-4 bg-black/60 border border-white/5 rounded-2xl p-4 overflow-hidden relative shadow-inner flex flex-col justify-end">
                <div class="absolute inset-0 bg-gradient-to-b from-[#020204] via-transparent to-transparent pointer-events-none z-10"></div>
                <div class="font-mono text-[10px] text-emerald-400/80 tracking-wider flex flex-col gap-1.5">
                  <div v-for="(log, idx) in bootLogs" :key="idx" class="truncate" :class="{'text-white font-bold': idx === bootLogs.length - 1, 'opacity-40': idx < bootLogs.length - 2}">
                    <span class="text-[var(--accent-color)]">></span> {{ log }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </transition>

      <transition name="fade">
        <div v-if="!state.showBoot && !state.settings.eula_accepted" class="fixed inset-0 bg-black/95 backdrop-blur-2xl z-[99999] flex items-center justify-center p-10 cursor-default">
          <div class="kip-card max-w-2xl w-full p-8 border-[var(--accent-border)] shadow-[0_0_50px_var(--accent-glow)] relative overflow-hidden flex flex-col">
            <div class="absolute -top-32 -left-32 w-64 h-64 bg-[var(--accent-subtle)] blur-[100px] rounded-full pointer-events-none"></div>
            <h2 class="text-3xl font-extrabold mb-4 flex items-center gap-3 relative z-10"><ShieldCheck class="w-8 h-8 text-[var(--accent-color)]" /> License Agreement</h2>
            <div class="bg-black/40 border border-white/5 p-5 rounded-xl mb-6 h-64 overflow-y-auto custom-scroll text-sm text-white/70 leading-relaxed font-medium relative z-10 shadow-inner">
              NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
              <br><br>
              This software downloads files directly from Mojang servers. A valid Minecraft license is required to play. Provided "AS IS" under the GNU General Public License v3.0 (GPL-3.0).
              <br><br>
              By clicking "I Accept", you agree to these terms and confirm you understand this is a third-party open-source manager licensed under GPL-3.0. Data such as crash logs may be sent to AI providers (Google/OpenAI/Anthropic) if you explicitly use the AI Support feature. Your Microsoft tokens are stored locally. No personal data is collected by K.I.P. Studio.
            </div>
            <button @click="acceptEula" class="kip-btn-primary w-full py-4 text-lg tracking-widest uppercase relative z-10">I Accept</button>
          </div>
        </div>
      </transition>

      <header
        v-show="!state.isOverlayActive"
        class="titlebar flex justify-between items-center px-5 py-2.5 z-50 transition-colors duration-300 bg-black/70 border-b border-white/10 backdrop-blur-2xl relative select-none"
        data-tauri-drag-region
        @dblclick="toggleMaximize"
      >
        <div class="flex items-center gap-4 pointer-events-none">
          <div class="flex items-center gap-2.5">
            <div class="p-1.5 rounded-xl bg-[var(--accent-subtle)] border border-[var(--accent-border)] text-[var(--accent-color)]">
              <Hexagon class="w-4 h-4 animate-pulse stroke-[2.2]" />
            </div>
            <div class="flex flex-col">
              <span class="font-black tracking-[0.15em] text-xs uppercase leading-none text-white">
                {{ resolvedAppName }}<span class="text-[var(--accent-color)]">{{ resolvedAppAccent }}</span>
              </span>
              <span class="text-[8px] font-mono text-white/30 uppercase tracking-widest mt-0.5">Matrix v{{ state.version }}</span>
            </div>
          </div>

          <div class="w-px h-5 bg-white/10"></div>

          <div
            class="flex items-center gap-2 px-3 py-1 rounded-full border text-[10px] font-mono font-bold uppercase tracking-wider backdrop-blur-md"
            :class="state.isMcRunning ? 'bg-emerald-500/10 text-emerald-300 border-emerald-500/30' : 'bg-white/5 text-white/50 border-white/10'"
          >
            <span
              class="w-2 h-2 rounded-full"
              :class="state.isMcRunning ? 'bg-emerald-400 shadow-[0_0_10px_rgba(52,211,153,0.9)] animate-pulse' : 'bg-white/30'"
            ></span>
            <span>{{ state.isMcRunning ? t(state.mcStatusText) : 'Standby Matrix' }}</span>
          </div>
        </div>

        <div class="flex items-center gap-3 text-xs font-mono text-white/40 pointer-events-none">
          <span class="text-[9px] uppercase tracking-widest flex items-center gap-1.5">
            <Activity class="w-3 h-3 text-[var(--accent-color)]" />
            <span>Profile:</span>
            <strong class="text-white/80 font-bold">{{ formatInstanceName(state.settings.mc_dir) }}</strong>
          </span>
        </div>

        <div class="flex items-center gap-1.5 titlebar-btn pointer-events-auto">
          <button
            @click="toggleOverlayState"
            class="p-2 text-white/40 hover:text-indigo-400 hover:bg-white/5 rounded-xl transition cursor-pointer"
            title="Toggle In-Game HUD Overlay [Shift + Tab]"
          >
            <Zap class="w-3.5 h-3.5" />
          </button>

          <button
            @click="toggleMiniMode"
            class="p-2 text-white/40 hover:text-[var(--accent-color)] hover:bg-white/5 rounded-xl transition cursor-pointer"
            title="Switch to Mini Companion Widget"
          >
            <Minimize2 class="w-3.5 h-3.5" />
          </button>

          <button
            @click="togglePinWindow"
            class="p-2 rounded-xl transition flex items-center justify-center cursor-pointer"
            :class="isPinned ? 'bg-[var(--accent-subtle)] text-[var(--accent-color)] border border-[var(--accent-border)] shadow-[0_0_12px_var(--accent-glow)]' : 'text-white/40 hover:text-white hover:bg-white/5'"
            :title="isPinned ? 'Unpin Window' : 'Pin Always on Top'"
          >
            <Pin class="w-3.5 h-3.5" :class="isPinned ? 'fill-current' : ''" />
          </button>

          <button
            @click="toggleBigPicture"
            class="p-2 text-white/40 hover:text-amber-400 hover:bg-white/5 rounded-xl transition cursor-pointer"
            :title="t('Big Picture Mode')"
          >
            <Gamepad2 class="w-3.5 h-3.5" />
          </button>

          <button
            @click="state.isNexusOpen = !state.isNexusOpen"
            class="p-2 text-white/40 hover:text-emerald-400 hover:bg-white/5 rounded-xl transition relative cursor-pointer"
            :title="t('K.I.P. Nexus')"
          >
            <Users class="w-3.5 h-3.5" />
            <span class="absolute top-1.5 right-1.5 w-1.5 h-1.5 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(16,185,129,0.9)] animate-pulse"></span>
          </button>

          <div class="w-px h-4 bg-white/10 mx-1"></div>

          <button
            @click="minimizeWindow"
            class="p-2 text-white/40 hover:text-white hover:bg-white/5 rounded-xl transition cursor-pointer"
            title="Minimize"
          >
            <Minus class="w-3.5 h-3.5" />
          </button>

          <button
            @click="toggleMaximize"
            class="p-2 text-white/40 hover:text-[var(--accent-color)] hover:bg-white/5 rounded-xl transition cursor-pointer"
            :title="isMaximized ? 'Restore' : 'Maximize'"
          >
            <Copy v-if="isMaximized" class="w-3 h-3 stroke-[2.2]" />
            <Square v-else class="w-3 h-3 stroke-[2.2]" />
          </button>

          <button
            @click="closeWindow"
            class="p-2 text-white/40 hover:text-white hover:bg-rose-600 rounded-xl transition duration-150 cursor-pointer"
            title="Close [ESC]"
          >
            <X class="w-3.5 h-3.5" />
          </button>
        </div>
      </header>

      <div v-show="!state.isOverlayActive" class="flex flex-1 overflow-hidden relative z-10">
        <div
          v-show="!state.isMiniMode"
          class="border-r border-white/5 flex flex-col py-5 gap-1.5 z-40 transition-all duration-300 bg-black/45 backdrop-blur-xl shrink-0"
          :class="isSidebarCollapsed ? 'w-20 px-3' : 'w-64 px-4'"
        >
          <div class="flex items-center justify-between mb-3 px-2" :class="isSidebarCollapsed ? 'justify-center' : ''">
            <button @click="isSidebarCollapsed = !isSidebarCollapsed" class="text-white/40 hover:text-white transition p-1 cursor-pointer">
              <Menu v-if="isSidebarCollapsed" class="w-5 h-5" />
              <AlignLeft v-else class="w-5 h-5" />
            </button>
          </div>

          <div v-if="!isSidebarCollapsed" class="px-2 mb-3">
            <label class="text-[8px] font-mono text-white/30 uppercase font-black tracking-widest block mb-1">Active Universe</label>
            <select
              v-model="state.settings.mc_dir"
              @change="changeInstance"
              class="kip-input appearance-none cursor-pointer text-xs py-2 truncate font-mono bg-black/60 border-white/10"
              :title="state.settings.mc_dir"
            >
              <option v-for="inst in state.settings.instances" :key="inst" :value="inst">{{ formatInstanceName(inst) }}</option>
            </select>
          </div>

          <div class="flex flex-col gap-1 overflow-y-auto custom-scroll pr-1 flex-1">
            <div
              v-for="nav in state.navItems"
              :key="nav.id"
              @click="state.currentView = nav.id"
              class="flex items-center rounded-xl cursor-pointer transition-all duration-200"
              :class="[
                state.currentView === nav.id
                  ? 'bg-[var(--accent-subtle)] text-[var(--accent-color)] font-bold border border-[var(--accent-border)] shadow-[0_0_15px_var(--accent-glow)]'
                  : 'text-white/60 hover:bg-white/5 hover:text-white border border-transparent',
                isSidebarCollapsed ? 'p-3 justify-center' : 'px-4 py-2.5 gap-3'
              ]"
              :title="isSidebarCollapsed ? t(nav.label) : ''"
            >
              <component :is="getIcon(nav.icon)" class="w-4 h-4 shrink-0" />
              <span v-if="!isSidebarCollapsed" class="text-xs font-semibold tracking-wide">{{ t(nav.label) }}</span>
            </div>
          </div>

          <div class="mt-auto pt-2"></div>

          <div
            class="px-3.5 py-2.5 rounded-xl border transition-colors duration-300 flex items-center gap-2.5"
            :class="[
              state.isMcRunning ? 'bg-emerald-500/10 border-emerald-500/30' : 'bg-black/40 border-white/5',
              isSidebarCollapsed ? 'justify-center p-2.5' : 'mx-1 mb-2'
            ]"
            :title="isSidebarCollapsed ? t(state.mcStatusText) : ''"
          >
            <div
              class="w-2 h-2 rounded-full shrink-0"
              :class="state.isMcRunning ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.9)] animate-pulse' : 'bg-white/30'"
            ></div>
            <span
              v-if="!isSidebarCollapsed"
              class="text-[11px] font-mono font-bold truncate"
              :class="state.isMcRunning ? 'text-emerald-400' : 'text-white/40'"
            >
              {{ t(state.mcStatusText) }}
            </span>
          </div>

          <button
            v-if="hasUpdate"
            @click="performUpdate"
            :disabled="isUpdating"
            class="kip-btn-primary py-2.5 text-xs font-black uppercase tracking-wider cursor-pointer"
            :class="isSidebarCollapsed ? 'p-2.5' : 'w-full mx-1'"
            :title="isSidebarCollapsed ? t('Update App') : ''"
          >
            <Loader v-if="isUpdating" class="w-3.5 h-3.5 animate-spin" />
            <Download v-else class="w-3.5 h-3.5" />
            <span v-if="!isSidebarCollapsed">{{ isUpdating ? t('Updating...') : t('Update App') }}</span>
          </button>
        </div>

        <div
          class="flex-1 overflow-y-auto custom-scroll relative flex flex-col min-h-0 transition-all duration-500 p-8"
          id="main-content"
        >
          <transition name="fade" mode="out-in">
            <component :is="currentViewComponent" />
          </transition>
        </div>

        <transition name="slide">
          <Nexus v-if="state.isNexusOpen" />
        </transition>
      </div>
    </template>

    <transition name="fade">
      <div
        v-if="state.partyInvite"
        class="fixed top-12 left-1/2 -translate-x-1/2 z-[10000] kip-card p-5 border border-[var(--accent-border)] shadow-[0_0_50px_var(--accent-glow)] flex items-center gap-6 animate-[bounce_2s_infinite] bg-black/90 backdrop-blur-2xl"
      >
        <div class="relative shrink-0">
          <img :src="getAvatarUrl(state.partyInvite.senderName)" class="w-12 h-12 rounded-xl object-cover border-2 border-[var(--accent-color)] shadow-[0_0_15px_var(--accent-glow)]">
          <div class="absolute -bottom-2 -right-2 p-1 bg-black rounded-lg border border-white/10">
            <Gamepad2 class="w-4 h-4 text-emerald-400 animate-pulse" />
          </div>
        </div>
        <div class="flex-1 min-w-0">
          <h4 class="font-black text-white text-base tracking-wider leading-none mb-1">K.I.P. Party Beacon</h4>
          <p class="text-xs text-white/70 font-mono">From <strong class="text-[var(--accent-color)]">{{ state.partyInvite.senderName }}</strong> • {{ state.partyInvite.mods.length }} sync packages</p>
        </div>
        <div class="flex gap-2 shrink-0 ml-4">
          <button @click="acceptPartyInvite" class="px-5 py-2.5 bg-emerald-500 hover:bg-emerald-400 text-black font-black text-xs uppercase tracking-wider rounded-xl transition shadow-[0_0_15px_rgba(16,185,129,0.4)] cursor-pointer">Accept</button>
          <button @click="declinePartyInvite" class="px-5 py-2.5 bg-rose-500/20 hover:bg-rose-500 text-rose-400 hover:text-white font-bold text-xs uppercase tracking-wider rounded-xl transition border border-rose-500/30 cursor-pointer">Decline</button>
        </div>
      </div>
    </transition>

    <div class="fixed bottom-5 right-5 z-[300] flex flex-col gap-3 pointer-events-none">
      <transition-group name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          class="kip-card p-4 flex items-start gap-3 border-l-4 w-80 shadow-2xl pointer-events-auto"
          :class="toast.type === 'success' ? 'border-emerald-500' : toast.type === 'danger' ? 'border-rose-500' : 'border-[var(--accent-color)]'"
        >
          <component
            :is="getIcon(toast.icon)"
            class="w-5 h-5 mt-0.5 shrink-0"
            :class="toast.type === 'success' ? 'text-emerald-400' : toast.type === 'danger' ? 'text-rose-400' : 'text-[var(--accent-color)]'"
          />
          <div class="overflow-hidden">
            <h4 class="font-black text-white text-xs uppercase tracking-wider">{{ toast.title }}</h4>
            <p class="text-white/70 text-xs mt-0.5 break-words font-medium leading-relaxed">{{ toast.message }}</p>
          </div>
        </div>
      </transition-group>
    </div>

    <Overlay v-if="state.isOverlayActive" />
  </div>
</template>