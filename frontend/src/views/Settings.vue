<script setup lang="ts">
import { ref, onMounted } from 'vue'
import {
  SlidersHorizontal,
  Cpu,
  Monitor,
  BrainCircuit,
  HardDrive,
  RotateCcw,
  Globe,
  Radio,
  FolderOpen,
  Loader,
  Zap,
  Database,
  Binary,
  FolderSync,
} from 'lucide-vue-next'
import { t } from '@/store'
import type { SettingsTabItem, AccentOptionItem } from '../types/settings'
import { useSettingsManager } from '../composables/useSettingsManager'
import { setLanguage } from '../composables/useI18n'

const activeCategory = ref<'general' | 'display' | 'jvm' | 'ai' | 'storage'>('general')

const {
  settings,
  isValidatingJava,
  isDetectingJava,
  isTestingAi,
  isVacuuming,
  javaValidationData,
  detectedRuntimes,
  compiledJvmPreview,
  loadSettings,
  commitSettings,
  validateJava,
  detectSystemJava,
  pickCustomInstancePath,
  testAiConnection,
  runVacuum,
  restoreFactoryDefaults,
  openActiveDir,
} = useSettingsManager()

const settingsTabs: SettingsTabItem[] = [
  { id: 'general', label: 'General', icon: SlidersHorizontal },
  { id: 'display', label: 'Display & Canvas', icon: Monitor },
  { id: 'jvm', label: 'HotSpot JVM', icon: Cpu },
  { id: 'ai', label: 'Neural Core', icon: BrainCircuit },
  { id: 'storage', label: 'Storage & Security', icon: HardDrive },
]

const langOptions = [
  { value: 'en', label: 'English (US)' },
  { value: 'ru', label: 'Русский (RU)' },
  { value: 'es', label: 'Español (ES)' },
  { value: 'de', label: 'Deutsch (DE)' },
  { value: 'zh', label: '中文 (Simplified)' },
  { value: 'fr', label: 'Français (FR)' },
  { value: 'pt', label: 'Português (BR)' },
  { value: 'ja', label: '日本語 (JA)' },
  { value: 'ko', label: '한국어 (KO)' },
]

const accentOptions: AccentOptionItem[] = [
  { id: 'indigo', label: 'Indigo', class: 'bg-[#6366F1]' },
  { id: 'emerald', label: 'Emerald', class: 'bg-[#10B981]' },
  { id: 'purple', label: 'Purple', class: 'bg-[#A855F7]' },
  { id: 'amber', label: 'Amber', class: 'bg-[#F59E0B]' },
  { id: 'cyan', label: 'Cyan', class: 'bg-[#06B6D4]' },
  { id: 'rose', label: 'Rose', class: 'bg-[#F43F5E]' },
]

async function handleLanguageChange(): Promise<void> {
  await setLanguage(settings.value.lang)
  await commitSettings()
}

function applyOptimizationPreset(): void {
  if (settings.value.jvmPreset === 'aikar') {
    settings.value.customJvmArgs =
      '-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20'
  } else if (settings.value.jvmPreset === 'esports') {
    settings.value.customJvmArgs =
      '-XX:+UseZGC -XX:+ZGenerational -XX:+AlwaysPreTouch -XX:+UnlockExperimentalVMOptions'
  } else if (settings.value.jvmPreset === 'low_ram') {
    settings.value.customJvmArgs = '-XX:+UseSerialGC -Xms128M'
  } else {
    settings.value.customJvmArgs = '-XX:+UseG1GC -XX:+ParallelRefProcEnabled'
  }
  commitSettings()
}

onMounted(() => {
  loadSettings()
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 select-none relative pr-2 pb-10">
    <header class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Settings') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">{{ t('Control Center • Display & Canvas, HotSpot Matrix & Subsystem Routing') }}</p>
      </div>

      <button
        @click="restoreFactoryDefaults"
        class="kip-btn-ghost px-4 py-2 text-xs font-bold uppercase tracking-wider text-rose-400 hover:text-white border-rose-500/20 hover:bg-rose-500/20 flex items-center gap-2 cursor-pointer"
      >
        <RotateCcw class="w-3.5 h-3.5" />
        <span>{{ t('Factory Reset') }}</span>
      </button>
    </header>

    <nav class="flex p-1 bg-black/45 rounded-2xl border border-white/10 backdrop-blur-xl mb-6 shrink-0 z-10">
      <button
        v-for="tab in settingsTabs"
        :key="tab.id"
        @click="activeCategory = tab.id"
        class="flex-1 py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center justify-center gap-2 cursor-pointer"
        :class="activeCategory === tab.id ? 'bg-[var(--accent-subtle)] text-[var(--accent-color)] border border-[var(--accent-border)] shadow-[0_0_20px_var(--accent-glow)]' : 'text-white/40 hover:text-white border border-transparent'"
      >
        <component :is="tab.icon" class="w-3.5 h-3.5" />
        <span>{{ t(tab.label) }}</span>
      </button>
    </nav>

    <main class="flex-1 overflow-y-auto custom-scroll pr-1 pb-6 min-h-0 z-10">
      <section v-show="activeCategory === 'general'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 flex flex-col gap-5 border border-white/10 bg-black/45">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Globe class="w-4 h-4 text-[var(--accent-color)]" /> {{ t('Localization & System Interface') }}
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('User Interface Language') }}</label>
            <select v-model="settings.lang" @change="handleLanguageChange" class="kip-input text-xs font-bold font-mono">
              <option v-for="opt in langOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.autostart = !settings.autostart; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Run at System Startup') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Register K.I.P. Engine in native OS registry to launch with Windows') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.autostart" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.closeOnLaunch = !settings.closeOnLaunch; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Minimize UI on Launch') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Hide launcher to system tray when game process spawns') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.closeOnLaunch" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 flex flex-col gap-5 border border-white/10 bg-black/45">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Radio class="w-4 h-4 text-emerald-400" /> {{ t('Daemon & Presence Integrations') }}
          </h3>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.rpc = !settings.rpc; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Discord Rich Presence (IPC)') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Broadcast game instance, playtime metric and activity badge') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.rpc" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.autoBackup = !settings.autoBackup; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Automated World Snapshot') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Zip world saves into differential archives before game ignition') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.autoBackup" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex flex-col gap-2 pt-2 border-t border-white/10">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Default Offline Player Handle') }}</label>
            <input v-model="settings.offlineUsername" @blur="commitSettings" type="text" class="kip-input text-xs font-bold font-mono">
          </div>
        </div>
      </section>

      <section v-show="activeCategory === 'display'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/45">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Monitor class="w-4 h-4 text-cyan-400" /> {{ t('Canvas Resolution & Window Geometry') }}
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Resolution Presets') }}</label>
            <div class="grid grid-cols-3 gap-2">
              <button
                v-for="res in ['1280x720', '1920x1080', '2560x1440', '3840x2160', '2560x1080', '3440x1440', '5120x1440']"
                :key="res"
                @click="settings.gameResolution = res; commitSettings()"
                class="py-2.5 rounded-xl text-[10px] font-mono font-bold border transition cursor-pointer"
                :class="settings.gameResolution === res ? 'bg-cyan-500/20 text-cyan-300 border-cyan-500/50 shadow-[0_0_15px_rgba(6,182,212,0.3)]' : 'bg-black/60 text-white/50 border-white/10 hover:border-white/20'"
              >
                {{ res }}
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.gameFullscreen = !settings.gameFullscreen; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Launch in Native Fullscreen') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Inject borderless full display commands') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.gameFullscreen" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/45">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Zap class="w-4 h-4 text-[var(--accent-color)]" /> {{ t('GPU Shaders, Scaling & Visual Pipelines') }}
          </h3>

          <div
            class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer hover:border-white/20 transition"
            @click="settings.lowGraphics = !settings.lowGraphics; commitSettings()"
          >
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Low Graphics Mode') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Disable 3D background shaders and intensive blur') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.lowGraphics" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div
            class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer hover:border-white/20 transition"
            @click="settings.mica = !settings.mica; commitSettings()"
          >
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Frosted Glass (Mica / Blur)') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Toggle GPU-rendered backdrop filters') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.mica" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex flex-col gap-2 pt-2 border-t border-white/10">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Theme Accent') }}</label>
            <div class="grid grid-cols-3 gap-2">
              <button
                v-for="acc in accentOptions"
                :key="acc.id"
                @click="settings.themeAccent = acc.id; commitSettings()"
                class="py-2.5 rounded-xl text-xs font-bold capitalize border transition flex items-center justify-center gap-2 cursor-pointer"
                :class="settings.themeAccent === acc.id ? 'bg-white/15 border-white/60 text-white shadow-lg' : 'bg-black/60 text-white/50 border-white/10 hover:border-white/20'"
              >
                <span class="w-2.5 h-2.5 rounded-full shadow-sm" :class="acc.class"></span>
                <span>{{ acc.label }}</span>
              </button>
            </div>
          </div>
        </div>
      </section>

      <section v-show="activeCategory === 'jvm'" class="flex flex-col gap-6">
        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/45">
          <div class="flex justify-between items-center">
            <div>
              <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
                <Cpu class="w-4 h-4 text-purple-400" /> {{ t('HotSpot JVM & Garbage Collector Matrix') }}
              </h3>
              <p class="text-xs text-white/40 mt-0.5">Heap sizing and Collector algorithms for Java 25 & 21</p>
            </div>
            <span class="text-xs font-mono font-bold text-purple-400 bg-purple-500/10 px-3 py-1 rounded-xl border border-purple-500/20">
              {{ settings.ramAllocation === 0 ? t('Auto Dynamic') : `${settings.ramAllocation} GB Heap` }}
            </span>
          </div>

          <div class="grid grid-cols-3 gap-4">
            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Garbage Collector (GC)') }}</label>
              <select v-model="settings.jvmGc" @change="commitSettings" class="kip-input text-xs font-bold font-mono">
                <option value="G1GC">G1GC (Garbage-First Default)</option>
                <option value="ZGC">ZGC (Generational ZGC Java 25)</option>
                <option value="Shenandoah">Shenandoah (Ultra-Low Pause)</option>
                <option value="Parallel">ParallelGC (High Throughput)</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Optimization Profile') }}</label>
              <select v-model="settings.jvmPreset" @change="applyOptimizationPreset" class="kip-input text-xs font-bold font-mono">
                <option value="balanced">Balanced Standard</option>
                <option value="aikar">Aikar's Enterprise Heap Flags</option>
                <option value="esports">E-Sports Low-Latency Sub-ms</option>
                <option value="low_ram">Low-Footprint Footprint</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('RAM Allocation') }}</label>
              <div class="flex gap-1.5 flex-wrap">
                <button
                  v-for="ram in [0, 2, 4, 6, 8, 12, 16, 24, 32]"
                  :key="ram"
                  @click="settings.ramAllocation = ram; commitSettings()"
                  class="flex-1 min-w-[42px] py-2 rounded-xl text-[10px] font-mono font-bold border transition cursor-pointer"
                  :class="settings.ramAllocation === ram ? 'bg-purple-500/20 text-purple-400 border-purple-500/40' : 'bg-black/60 text-white/50 border-white/10 hover:border-white/20'"
                >
                  {{ ram === 0 ? t('Auto') : `${ram}G` }}
                </button>
              </div>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Java Executable Discovery') }}</label>
              <span v-if="javaValidationData" class="text-[9px] font-mono font-bold px-2 py-0.5 rounded border" :class="javaValidationData.valid ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border-rose-500/20'">
                {{ javaValidationData.message }}
              </span>
            </div>

            <div class="flex gap-2">
              <input v-model="settings.customJavaPath" @blur="commitSettings" type="text" :placeholder="t('Leave empty for autonomous OpenJDK 25 / 21 provisioner')" class="kip-input font-mono text-xs flex-1">
              <button @click="detectSystemJava" :disabled="isDetectingJava" class="kip-btn-ghost px-4 text-xs font-bold uppercase flex items-center gap-1.5 text-purple-400 border-purple-500/30 cursor-pointer">
                <Loader v-if="isDetectingJava" class="w-3.5 h-3.5 animate-spin" />
                <Binary v-else class="w-3.5 h-3.5" />
                <span>{{ t('Auto-Detect') }}</span>
              </button>
              <button @click="validateJava" :disabled="!settings.customJavaPath.trim() || isValidatingJava" class="kip-btn-primary px-5 text-xs font-black uppercase tracking-wider cursor-pointer">
                <Loader v-if="isValidatingJava" class="w-3.5 h-3.5 animate-spin" />
                <span v-else>{{ t('Validate') }}</span>
              </button>
            </div>

            <div v-if="detectedRuntimes.length > 0" class="mt-2 p-3 bg-black/60 rounded-xl border border-white/10 flex flex-col gap-1.5">
              <span class="text-[9px] font-mono uppercase text-white/40 font-bold">{{ t('Detected Host Runtimes (Click to assign):') }}</span>
              <button
                v-for="r in detectedRuntimes"
                :key="r.path"
                @click="settings.customJavaPath = r.path; commitSettings(); validateJava()"
                class="text-left text-xs font-mono p-2 rounded-lg bg-white/5 hover:bg-purple-500/20 border border-white/5 hover:border-purple-500/40 flex justify-between items-center transition cursor-pointer"
              >
                <span class="text-white font-bold">{{ r.vendor }} (Java {{ r.major }})</span>
                <span class="text-white/40 text-[10px] truncate max-w-sm">{{ r.path }}</span>
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Compiled JVM Command Line Preview') }}</label>
            <div class="p-3 bg-black/60 rounded-xl border border-white/10 font-mono text-xs text-white/70 select-all overflow-x-auto whitespace-pre-wrap">
              {{ compiledJvmPreview }}
            </div>
          </div>
        </div>
      </section>

      <section v-show="activeCategory === 'ai'" class="flex flex-col gap-6">
        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/45">
          <div class="flex justify-between items-center">
            <div>
              <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
                <BrainCircuit class="w-4 h-4 text-amber-400" /> {{ t('Neural Assistant & Provider Routing') }}
              </h3>
              <p class="text-xs text-white/40 mt-0.5">{{ t('Crash telemetry analysis and automated modpack synthesis') }}</p>
            </div>
            <button @click="testAiConnection" :disabled="isTestingAi" class="kip-btn-primary px-4 py-2 text-xs font-black uppercase tracking-wider cursor-pointer">
              <Loader v-if="isTestingAi" class="w-3.5 h-3.5 animate-spin" />
              <Zap v-else class="w-3.5 h-3.5 fill-current" />
              <span>{{ t('Test Neural Link') }}</span>
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Active Engine Provider') }}</label>
              <select v-model="settings.aiProvider" @change="commitSettings" class="kip-input text-xs font-bold font-mono">
                <option value="google">Google Gemini (Vertex AI)</option>
                <option value="openai">OpenAI (GPT-4o Mini)</option>
                <option value="anthropic">Anthropic (Claude 3.5 Sonnet)</option>
                <option value="ollama">Ollama (Self-Hosted Local LLM)</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Model Checkpoint') }}</label>
              <input v-model="settings.aiModel" @blur="commitSettings" type="text" class="kip-input text-xs font-mono">
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4 pt-2 border-t border-white/10">
            <div v-if="settings.aiProvider === 'google'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Google Gemini API Key') }}</label>
              <input v-model="settings.aiApiKey" @blur="commitSettings" type="password" placeholder="AIzaSy..." class="kip-input font-mono text-xs">
            </div>

            <div v-if="settings.aiProvider === 'openai'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('OpenAI Secret Key') }}</label>
              <input v-model="settings.openaiApiKey" @blur="commitSettings" type="password" placeholder="sk-proj-..." class="kip-input font-mono text-xs">
            </div>

            <div v-if="settings.aiProvider === 'anthropic'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Anthropic API Key') }}</label>
              <input v-model="settings.anthropicApiKey" @blur="commitSettings" type="password" placeholder="sk-ant-..." class="kip-input font-mono text-xs">
            </div>

            <div v-if="settings.aiProvider === 'ollama'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Ollama REST Daemon Endpoint') }}</label>
              <input v-model="settings.ollamaUrl" @blur="commitSettings" type="text" placeholder="http://localhost:11434" class="kip-input font-mono text-xs">
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('CurseForge API Key') }}</label>
              <input v-model="settings.cfApiKey" @blur="commitSettings" type="password" placeholder="$2a$10$..." class="kip-input font-mono text-xs">
            </div>
          </div>
        </div>
      </section>

      <section v-show="activeCategory === 'storage'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/45">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <HardDrive class="w-4 h-4 text-emerald-400" /> {{ t('Instance Root Directory') }}
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">{{ t('Active File System Mount') }}</label>
            <div class="flex gap-2">
              <input v-model="settings.currentInstance" readonly type="text" class="kip-input font-mono text-xs select-all flex-1">
              <button @click="pickCustomInstancePath" class="kip-btn-ghost px-3 text-xs font-bold cursor-pointer" title="Browse Custom Path">
                <FolderSync class="w-4 h-4 text-[var(--accent-color)]" />
              </button>
              <button @click="openActiveDir" class="kip-btn-ghost px-3 text-xs font-bold cursor-pointer" title="Open in Explorer">
                <FolderOpen class="w-4 h-4 text-emerald-400" />
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.shieldAutoScan = !settings.shieldAutoScan; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('K.I.P. Shield Heuristic Scanner') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Intercept dropped mods and scan bytecode on import') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.shieldAutoScan" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/60 rounded-xl border border-white/10 cursor-pointer" @click="settings.voiceNoiseSuppression = !settings.voiceNoiseSuppression; commitSettings()">
            <div>
              <span class="text-xs font-bold text-white block">{{ t('Hardware Voice DSP & Gate') }}</span>
              <span class="text-[10px] text-white/40">{{ t('Acoustic echo cancellation and software high-pass hum filters') }}</span>
            </div>
            <div class="relative inline-block w-10">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="settings.voiceNoiseSuppression" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 border border-white/10 flex flex-col justify-between bg-black/45">
          <div>
            <h3 class="text-base font-black uppercase text-white flex items-center gap-2 mb-2">
              <Database class="w-4 h-4 text-[var(--accent-color)]" /> {{ t('Database Maintenance') }}
            </h3>
            <p class="text-xs text-white/50 leading-relaxed font-mono mb-4">
              {{ t('Defragment internal SQLite WAL persistence file, reindex playtime metrics and optimize cache allocations.') }}
            </p>
          </div>

          <button
            @click="runVacuum"
            :disabled="isVacuuming"
            class="kip-btn-ghost w-full py-3 text-xs font-black uppercase tracking-wider text-[var(--accent-color)] border-[var(--accent-border)] hover:bg-[var(--accent-subtle)] flex items-center justify-center gap-2 cursor-pointer"
          >
            <Loader v-if="isVacuuming" class="w-4 h-4 animate-spin" />
            <Database v-else class="w-4 h-4" />
            <span>{{ isVacuuming ? t('Optimizing...') : t('Run SQLite VACUUM') }}</span>
          </button>
        </div>
      </section>
    </main>
  </div>
</template>