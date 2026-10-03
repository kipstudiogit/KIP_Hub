<template>
  <div class="h-full flex flex-col min-h-0 select-none relative pr-2 pb-10">
    <div class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Settings') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Control Center • HotSpot JVM Matrix, AI Routing & Core Diagnostics</p>
      </div>

      <div class="flex gap-2">
        <button
          @click="confirmResetSettings"
          class="kip-btn-ghost px-4 py-2 text-xs font-bold uppercase tracking-wider text-red-400 hover:text-white border-red-500/20 hover:bg-red-500/20"
        >
          <RotateCcw class="w-3.5 h-3.5" />
          <span>{{ t('Factory Reset') }}</span>
        </button>
      </div>
    </div>

    <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl mb-6 shrink-0 z-10">
      <button
        v-for="tab in settingsTabs"
        :key="tab.id"
        @click="activeCategory = tab.id"
        class="flex-1 py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center justify-center gap-2"
        :class="activeCategory === tab.id ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-[0_0_20px_rgba(99,102,241,0.2)]' : 'text-white/40 hover:text-white'"
      >
        <component :is="tab.icon" class="w-3.5 h-3.5" />
        <span>{{ tab.label }}</span>
      </button>
    </div>

    <div class="flex-1 overflow-y-auto custom-scroll pr-1 pb-6 min-h-0 z-10">
      <div v-show="activeCategory === 'general'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 flex flex-col gap-5 border border-white/5">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Globe class="w-4 h-4 text-indigo-400" /> Localization & System Interface
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">User Interface Language</label>
            <select
              v-model="state.settings.lang"
              @change="savePropSetting('lang')"
              class="kip-input text-xs font-bold"
            >
              <option v-for="opt in langOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
            </select>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('autostart')">
            <div>
              <span class="text-xs font-bold text-white block">Run at System Startup</span>
              <span class="text-[10px] text-white/40">Launch K.I.P. Engine automatically with Windows</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.autostart" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('low_graphics')">
            <div>
              <span class="text-xs font-bold text-white block">Low Graphics Render Pipeline</span>
              <span class="text-[10px] text-white/40">Disable 3D background shaders to free GPU cycles</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.low_graphics" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 flex flex-col gap-5 border border-white/5">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Radio class="w-4 h-4 text-emerald-400" /> Daemon & Presence Integrations
          </h3>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('rpc')">
            <div>
              <span class="text-xs font-bold text-white block">Discord Rich Presence (IPC)</span>
              <span class="text-[10px] text-white/40">Broadcast game instance, playtime and activity badge</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.rpc" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('auto_backup')">
            <div>
              <span class="text-xs font-bold text-white block">Automated World Snapshot</span>
              <span class="text-[10px] text-white/40">Zip world state into archives before game ignition</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.auto_backup" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('close_on_launch')">
            <div>
              <span class="text-xs font-bold text-white block">Minimize UI on Launch</span>
              <span class="text-[10px] text-white/40">Hide launcher to system tray when game process spawns</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.close_on_launch" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>
      </div>

      <div v-show="activeCategory === 'jvm'" class="flex flex-col gap-6">
        <div class="kip-card p-6 border border-white/5 flex flex-col gap-5">
          <div class="flex justify-between items-center">
            <div>
              <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
                <Cpu class="w-4 h-4 text-purple-400" /> HotSpot JVM & Garbage Collector Matrix
              </h3>
              <p class="text-xs text-white/40 mt-0.5">Tune Heap sizing, Collector algorithms and custom runtime binaries</p>
            </div>

            <div class="flex items-center gap-2">
              <span class="text-xs font-mono font-bold text-purple-400 bg-purple-500/10 px-3 py-1 rounded-xl border border-purple-500/20">
                {{ state.settings.ram_allocation === 0 ? 'Auto Dynamic' : `${state.settings.ram_allocation} GB Heap` }}
              </span>
            </div>
          </div>

          <div class="grid grid-cols-3 gap-4">
            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Garbage Collector (GC)</label>
              <select
                v-model="state.settings.jvm_gc"
                @change="applyGcPreset"
                class="kip-input text-xs font-bold font-mono"
              >
                <option value="G1GC">G1GC (Garbage-First Default)</option>
                <option value="ZGC">ZGC (Ultra-Low Latency Sub-ms)</option>
                <option value="Shenandoah">Shenandoah (Ultra-Low Pause)</option>
                <option value="Parallel">ParallelGC (High Throughput)</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Optimization Profile</label>
              <select
                v-model="state.settings.jvm_preset"
                @change="applyOptimizationPreset"
                class="kip-input text-xs font-bold font-mono"
              >
                <option value="balanced">Balanced Standard</option>
                <option value="aikar">Aikar's Enterprise Heap Flags</option>
                <option value="esports">E-Sports Frame-Time Smoothing</option>
                <option value="low_ram">Low-Footprint Footprint</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Quick RAM Preset</label>
              <div class="flex gap-1.5">
                <button
                  v-for="ram in [0, 4, 6, 8, 12, 16]"
                  :key="ram"
                  @click="state.settings.ram_allocation = ram; savePropSetting('ram_allocation')"
                  class="flex-1 py-2 rounded-xl text-[10px] font-mono font-bold border transition"
                  :class="state.settings.ram_allocation === ram ? 'bg-purple-500/20 text-purple-400 border-purple-500/40' : 'bg-black/40 text-white/50 border-white/5 hover:border-white/20'"
                >
                  {{ ram === 0 ? 'Auto' : `${ram}G` }}
                </button>
              </div>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Custom Java Executable Binary</label>
              <span v-if="javaStatusChip" class="text-[9px] font-mono font-bold px-2 py-0.5 rounded border" :class="javaStatusChip.class">
                {{ javaStatusChip.text }}
              </span>
            </div>

            <div class="flex gap-2">
              <input
                v-model="state.settings.custom_java_path"
                @blur="savePropSetting('custom_java_path')"
                type="text"
                placeholder="Leave empty for autonomous OpenJDK provisioner"
                class="kip-input font-mono text-xs flex-1"
              >
              <button @click="pickJavaFile" class="kip-btn-ghost px-4 text-xs font-bold">
                <FolderOpen class="w-4 h-4" />
              </button>
              <button
                @click="validateJava"
                :disabled="!state.settings.custom_java_path.trim() || isValidatingJava"
                class="kip-btn-primary px-5 text-xs font-black uppercase tracking-wider"
              >
                <Loader v-if="isValidatingJava" class="w-3.5 h-3.5 animate-spin" />
                <span v-else>Validate</span>
              </button>
            </div>
          </div>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">JVM System Arguments Pipeline</label>
            <textarea
              v-model="state.settings.custom_jvm_args"
              @blur="savePropSetting('custom_jvm_args')"
              rows="3"
              placeholder="-XX:+AlwaysPreTouch -XX:+ParallelRefProcEnabled"
              class="kip-input font-mono text-xs custom-scroll resize-none leading-relaxed"
            ></textarea>
          </div>
        </div>
      </div>

      <div v-show="activeCategory === 'display'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 border border-white/5 flex flex-col gap-5">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <Monitor class="w-4 h-4 text-cyan-400" /> Canvas Resolution & Fullscreen
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Display Aspect & Resolution</label>
            <div class="grid grid-cols-3 gap-2 mb-2">
              <button
                v-for="res in ['854x480', '1280x720', '1920x1080', '2560x1440', '3840x2160']"
                :key="res"
                @click="state.settings.game_resolution = res; savePropSetting('game_resolution')"
                class="py-2 rounded-xl text-[10px] font-mono font-bold border transition"
                :class="state.settings.game_resolution === res ? 'bg-cyan-500/20 text-cyan-400 border-cyan-500/40' : 'bg-black/40 text-white/50 border-white/5 hover:border-white/20'"
              >
                {{ res }}
              </button>
            </div>
            <input
              v-model="state.settings.game_resolution"
              @blur="savePropSetting('game_resolution')"
              type="text"
              placeholder="Width x Height (e.g. 1920x1080)"
              class="kip-input font-mono text-xs text-center"
            >
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('game_fullscreen')">
            <div>
              <span class="text-xs font-bold text-white block">Launch in Native Fullscreen</span>
              <span class="text-[10px] text-white/40">Command window to maximize borderless full display</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.game_fullscreen" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 border border-white/5 flex flex-col gap-5">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <UserCheck class="w-4 h-4 text-emerald-400" /> Default Offline Identity
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Fallback Player Handle</label>
            <input
              v-model="state.settings.offline_username"
              @blur="savePropSetting('offline_username')"
              type="text"
              placeholder="Player"
              class="kip-input text-xs font-bold font-mono"
            >
            <p class="text-[10px] text-white/40 leading-relaxed font-mono mt-1">
              Active when launching outside Microsoft OAuth authentication session.
            </p>
          </div>
        </div>
      </div>

      <div v-show="activeCategory === 'ai'" class="flex flex-col gap-6">
        <div class="kip-card p-6 border border-white/5 flex flex-col gap-5">
          <div class="flex justify-between items-center">
            <div>
              <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
                <BrainCircuit class="w-4 h-4 text-amber-400" /> Neural Assistant & Provider Routing
              </h3>
              <p class="text-xs text-white/40 mt-0.5">Crash telemetry analysis, automated modpack synthesis and in-game HUD assistant</p>
            </div>

            <button
              @click="testAiLink"
              :disabled="isTestingAi"
              class="kip-btn-primary px-4 py-2 text-xs font-black uppercase tracking-wider bg-amber-500 hover:bg-amber-400 text-black shadow-[0_0_15px_rgba(245,158,11,0.3)]"
            >
              <Loader v-if="isTestingAi" class="w-3.5 h-3.5 animate-spin" />
              <Zap v-else class="w-3.5 h-3.5 fill-current" />
              <span>Test Neural Link</span>
            </button>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Active Engine Provider</label>
              <select
                v-model="state.settings.ai_provider"
                @change="savePropSetting('ai_provider')"
                class="kip-input text-xs font-bold"
              >
                <option value="google">Google Gemini (Vertex AI)</option>
                <option value="openai">OpenAI (GPT-4o Mini)</option>
                <option value="anthropic">Anthropic (Claude 3.5 Sonnet)</option>
                <option value="ollama">Ollama (Self-Hosted Local LLM)</option>
              </select>
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Model Checkpoint</label>
              <input
                v-model="state.settings.ai_model"
                @blur="savePropSetting('ai_model')"
                type="text"
                placeholder="gemini-1.5-flash"
                class="kip-input text-xs font-mono"
              >
            </div>
          </div>

          <div class="grid grid-cols-2 gap-4 pt-2 border-t border-white/5">
            <div v-if="state.settings.ai_provider === 'google'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Google Gemini API Key</label>
              <input
                v-model="state.settings.ai_api_key"
                @blur="savePropSetting('ai_api_key')"
                type="password"
                placeholder="AIzaSy..."
                class="kip-input font-mono text-xs"
              >
            </div>

            <div v-if="state.settings.ai_provider === 'openai'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">OpenAI Secret Key</label>
              <input
                v-model="state.settings.openai_api_key"
                @blur="savePropSetting('openai_api_key')"
                type="password"
                placeholder="sk-proj-..."
                class="kip-input font-mono text-xs"
              >
            </div>

            <div v-if="state.settings.ai_provider === 'anthropic'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Anthropic API Key</label>
              <input
                v-model="state.settings.anthropic_api_key"
                @blur="savePropSetting('anthropic_api_key')"
                type="password"
                placeholder="sk-ant-..."
                class="kip-input font-mono text-xs"
              >
            </div>

            <div v-if="state.settings.ai_provider === 'ollama'" class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Ollama REST Daemon Endpoint</label>
              <input
                v-model="state.settings.ollama_url"
                @blur="savePropSetting('ollama_url')"
                type="text"
                placeholder="http://localhost:11434"
                class="kip-input font-mono text-xs"
              >
            </div>

            <div class="flex flex-col gap-2">
              <label class="text-[10px] font-mono uppercase text-white/40 font-bold">CurseForge (Overwolf Core API Key)</label>
              <input
                v-model="state.settings.cf_api_key"
                @blur="savePropSetting('cf_api_key')"
                type="password"
                placeholder="$2a$10$..."
                class="kip-input font-mono text-xs"
              >
            </div>
          </div>
        </div>
      </div>

      <div v-show="activeCategory === 'storage'" class="grid grid-cols-2 gap-6">
        <div class="kip-card p-6 border border-white/5 flex flex-col gap-5">
          <h3 class="text-base font-black uppercase text-white flex items-center gap-2">
            <HardDrive class="w-4 h-4 text-emerald-400" /> Instance Root Directory
          </h3>

          <div class="flex flex-col gap-2">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold">Active File System Mount</label>
            <div class="flex gap-2">
              <input
                v-model="state.settings.mc_dir"
                readonly
                type="text"
                class="kip-input font-mono text-xs select-all flex-1"
              >
              <button @click="openActiveDir" class="kip-btn-ghost px-4 text-xs font-bold" :title="t('Open in Explorer')">
                <FolderOpen class="w-4 h-4 text-emerald-400" />
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('shield_auto_scan')">
            <div>
              <span class="text-xs font-bold text-white block">K.I.P. Shield Heuristic Scanner</span>
              <span class="text-[10px] text-white/40">Intercept dropped mods and scan bytecode on import</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.shield_auto_scan" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>

          <div class="flex items-center justify-between p-3.5 bg-black/40 rounded-xl border border-white/5 cursor-pointer" @click="togglePropSetting('voice_noise_suppression')">
            <div>
              <span class="text-xs font-bold text-white block">Hardware Voice DSP & Gate</span>
              <span class="text-[10px] text-white/40">Acoustic echo cancellation and high-pass hum filters</span>
            </div>
            <div class="relative inline-block w-10 align-middle select-none">
              <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none" :checked="state.settings.voice_noise_suppression" style="pointer-events: none;">
              <label class="toggle-label block h-5 rounded-full border border-white/10" style="pointer-events: none;"></label>
            </div>
          </div>
        </div>

        <div class="kip-card p-6 border border-white/5 flex flex-col justify-between">
          <div>
            <h3 class="text-base font-black uppercase text-white flex items-center gap-2 mb-2">
              <Database class="w-4 h-4 text-indigo-400" /> Database Maintenance
            </h3>
            <p class="text-xs text-white/50 leading-relaxed font-mono mb-6">
              Defragment internal SQLite WAL persistence file, reindex playtime metrics and purge temporary cache allocations.
            </p>
          </div>

          <button
            @click="runVacuum"
            :disabled="isVacuuming"
            class="kip-btn-ghost w-full py-3.5 text-xs font-black uppercase tracking-wider text-indigo-400 border-indigo-500/20 hover:bg-indigo-500/10"
          >
            <Loader v-if="isVacuuming" class="w-4 h-4 animate-spin" />
            <Database v-else class="w-4 h-4" />
            <span>{{ isVacuuming ? 'Optimizing Database...' : 'Run SQLite VACUUM' }}</span>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import type { Component } from 'vue'
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
  UserCheck,
  Database,
} from 'lucide-vue-next'
import { state, t, saveSetting, showToast, type SettingsState } from '@/store'
import { bridge, type JavaValidationResultDto, type AiTestResultDto } from '@/bridge'

interface SettingsTab {
  id: string
  label: string
  icon: Component
}

const activeCategory = ref<string>('general')
const isValidatingJava = ref<boolean>(false)
const isTestingAi = ref<boolean>(false)
const isVacuuming = ref<boolean>(false)
const javaValidationData = ref<JavaValidationResultDto | null>(null)

const settingsTabs: SettingsTab[] = [
  { id: 'general', label: 'General', icon: SlidersHorizontal },
  { id: 'jvm', label: 'HotSpot JVM', icon: Cpu },
  { id: 'display', label: 'Display & Canvas', icon: Monitor },
  { id: 'ai', label: 'Neural Core', icon: BrainCircuit },
  { id: 'storage', label: 'Storage & Security', icon: HardDrive },
]

const langOptions = [
  { value: 'en', label: 'English (US)' },
  { value: 'ru', label: 'Русский' },
  { value: 'es', label: 'Español' },
  { value: 'de', label: 'Deutsch' },
  { value: 'zh', label: '中文' },
]

const javaStatusChip = computed(() => {
  if (!javaValidationData.value) return null
  if (javaValidationData.value.valid) {
    return {
      text: `Java ${javaValidationData.value.major} Validated`,
      class: 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20',
    }
  }
  return {
    text: 'Invalid Binary',
    class: 'bg-red-500/10 text-red-400 border-red-500/20',
  }
})

const savePropSetting = async (key: keyof SettingsState): Promise<void> => {
  const result = await saveSetting(key, state.settings[key])
  if (result) {
    showToast(t('Saved'), 'Settings updated successfully.', 'success')
  } else {
    showToast(t('Error'), 'Failed to commit setting change.', 'danger')
  }
}

const togglePropSetting = async (key: keyof SettingsState): Promise<void> => {
  const current = Boolean(state.settings[key])
  const next = !current
  const res = await saveSetting(key, next)
  if (res) {
    (state.settings as Record<string, unknown>)[key] = next
  }
}

const applyGcPreset = (): void => {
  savePropSetting('jvm_gc')
  if (state.settings.jvm_gc === 'ZGC') {
    state.settings.custom_jvm_args = '-XX:+UseZGC -XX:+ZGenerational'
  } else if (state.settings.jvm_gc === 'Shenandoah') {
    state.settings.custom_jvm_args = '-XX:+UseShenandoahGC -XX:ShenandoahGCMode=iu'
  } else {
    state.settings.custom_jvm_args = '-XX:+UseG1GC -XX:+ParallelRefProcEnabled'
  }
  savePropSetting('custom_jvm_args')
}

const applyOptimizationPreset = (): void => {
  savePropSetting('jvm_preset')
  if (state.settings.jvm_preset === 'aikar') {
    state.settings.custom_jvm_args = '-XX:+UseG1GC -XX:+ParallelRefProcEnabled -XX:MaxGCPauseMillis=200 -XX:+UnlockExperimentalVMOptions -XX:+DisableExplicitGC -XX:+AlwaysPreTouch -XX:G1NewSizePercent=30 -XX:G1MaxNewSizePercent=40 -XX:G1ReservePercent=20'
  } else if (state.settings.jvm_preset === 'esports') {
    state.settings.custom_jvm_args = '-XX:+UseZGC -XX:+AlwaysPreTouch -XX:+UnlockExperimentalVMOptions'
  } else if (state.settings.jvm_preset === 'low_ram') {
    state.settings.custom_jvm_args = '-XX:+UseSerialGC -Xms128M'
  } else {
    state.settings.custom_jvm_args = '-XX:+UseG1GC -XX:+ParallelRefProcEnabled'
  }
  savePropSetting('custom_jvm_args')
}

const pickJavaFile = async (): Promise<void> => {
  try {
    const picked = await bridge.pickFile()
    if (picked && picked.trim()) {
      state.settings.custom_java_path = picked.trim()
      await savePropSetting('custom_java_path')
      await validateJava()
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const validateJava = async (): Promise<void> => {
  if (!state.settings.custom_java_path.trim()) return
  isValidatingJava.value = true

  try {
    const res: JavaValidationResultDto = await bridge.validateJavaBinary(state.settings.custom_java_path.trim())
    javaValidationData.value = res
    if (res.valid) {
      showToast(t('Java Verified'), res.message, 'success')
    } else {
      showToast(t('Validation Error'), res.message, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isValidatingJava.value = false
  }
}

const testAiLink = async (): Promise<void> => {
  isTestingAi.value = true
  try {
    const res: AiTestResultDto = await bridge.testAiConnection(
      state.settings.ai_provider,
      state.settings.ai_model,
      state.settings.ollama_url
    )
    if (res.success) {
      showToast(t('Neural Link Online'), `${res.message} (${res.latency_ms} ms)`, 'success')
    } else {
      showToast(t('Connection Failed'), res.message, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isTestingAi.value = false
  }
}

const openActiveDir = async (): Promise<void> => {
  try {
    await bridge.openInstanceFolder()
  } catch {
    showToast(t('Error'), 'Could not launch system file manager.', 'danger')
  }
}

const runVacuum = async (): Promise<void> => {
  isVacuuming.value = true
  try {
    const msg = await bridge.vacuumDatabase()
    showToast(t('Database Optimized'), msg, 'success')
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isVacuuming.value = false
  }
}

const confirmResetSettings = async (): Promise<void> => {
  try {
    const def = await bridge.resetSettingsToDefault()
    Object.assign(state.settings, def)
    showToast(t('Factory Reset'), 'Engine settings restored to default baseline.', 'info')
  } catch {
    showToast(t('Error'), 'Failed to restore default configuration.', 'danger')
  }
}
</script>