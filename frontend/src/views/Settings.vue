<template>
  <div class="h-full pb-10 overflow-y-auto custom-scroll pr-2 relative">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-extrabold mb-1">{{ t('Settings') }}</h2>
      <p class="text-white/50 text-sm">{{ t('Configure your launcher experience.') }}</p>
    </div>

    <div class="grid grid-cols-2 gap-6">
      <div class="flex flex-col gap-6 stagger-2">
        <div class="kip-card p-8 relative overflow-hidden group hover:border-purple-500/30 transition-colors duration-500">
          <div class="absolute top-0 right-0 w-64 h-64 bg-purple-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-purple-500/20 transition-all duration-500"></div>

          <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-purple-500/10 rounded-xl border border-purple-500/20"><Sparkles class="text-purple-400 w-5 h-5" /></div>
            {{ t('API Integrations') }}
          </h3>

          <div class="flex flex-col gap-5 relative z-10">
            <div class="relative custom-dropdown mb-2">
              <label class="block text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2">Primary AI Provider</label>
              <div @click="isProviderDropdownOpen = !isProviderDropdownOpen" class="kip-input cursor-pointer flex justify-between items-center hover:bg-black/60 hover:border-purple-500/50">
                <span class="font-bold flex items-center gap-2">
                  <Bot class="w-4 h-4 text-purple-400" />
                  {{ providerOptions.find(o => o.value === state.settings.ai_provider)?.label || 'Google Gemini' }}
                </span>
                <ChevronDown class="w-4 h-4 text-white/50 transition-transform" :class="{'rotate-180': isProviderDropdownOpen}" />
              </div>
              <transition name="fade">
                <div v-if="isProviderDropdownOpen" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_30px_rgba(0,0,0,0.8)] overflow-hidden py-2 z-50">
                  <div v-for="opt in providerOptions" :key="opt.value" @click="selectProvider(opt.value)" class="px-5 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-bold flex items-center gap-3" :class="state.settings.ai_provider === opt.value ? 'text-purple-400 bg-purple-500/10' : 'text-white/70'">
                    {{ opt.label }}
                  </div>
                </div>
              </transition>
            </div>

            <div v-if="state.settings.ai_provider === 'google'" class="bg-black/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-colors shadow-inner">
              <label class="flex items-center gap-2 text-[10px] font-bold text-white/50 uppercase tracking-wider mb-3">
                <Cpu class="w-4 h-4 text-purple-400" /> Gemini API Key
              </label>
              <input v-model="state.settings.ai_api_key" @blur="saveTextSetting('ai_api_key')" type="password" :placeholder="t('API Key')" class="kip-input font-mono focus:border-purple-500">
            </div>

            <div v-if="state.settings.ai_provider === 'openai'" class="bg-black/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-colors shadow-inner">
              <label class="flex items-center gap-2 text-[10px] font-bold text-white/50 uppercase tracking-wider mb-3">
                <Cpu class="w-4 h-4 text-green-400" /> OpenAI API Key
              </label>
              <input v-model="state.settings.openai_api_key" @blur="saveTextSetting('openai_api_key')" type="password" :placeholder="t('API Key')" class="kip-input font-mono focus:border-green-500">
            </div>

            <div v-if="state.settings.ai_provider === 'anthropic'" class="bg-black/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-colors shadow-inner">
              <label class="flex items-center gap-2 text-[10px] font-bold text-white/50 uppercase tracking-wider mb-3">
                <Cpu class="w-4 h-4 text-amber-400" /> Anthropic API Key
              </label>
              <input v-model="state.settings.anthropic_api_key" @blur="saveTextSetting('anthropic_api_key')" type="password" :placeholder="t('API Key')" class="kip-input font-mono focus:border-amber-500">
            </div>

            <div v-if="state.settings.ai_provider === 'ollama'" class="bg-black/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-colors shadow-inner">
              <label class="flex items-center gap-2 text-[10px] font-bold text-white/50 uppercase tracking-wider mb-3">
                <Server class="w-4 h-4 text-blue-400" /> Ollama Local URL
              </label>
              <input v-model="state.settings.ollama_url" @blur="saveTextSetting('ollama_url')" type="text" placeholder="http://localhost:11434" class="kip-input font-mono focus:border-blue-500">
            </div>

            <div class="bg-black/40 border border-white/5 rounded-2xl p-5 hover:border-white/10 transition-colors mt-2 shadow-inner">
              <label class="flex items-center gap-2 text-[10px] font-bold text-white/50 uppercase tracking-wider mb-3">
                <Box class="w-4 h-4 text-orange-400" /> CurseForge (Overwolf)
              </label>
              <input v-model="state.settings.cf_api_key" @blur="saveTextSetting('cf_api_key')" type="password" :placeholder="t('CurseForge API Key')" class="kip-input font-mono focus:border-orange-500">
              <p class="text-[10px] text-white/40 mt-3 font-medium uppercase tracking-wider">{{ t('Required to browse and download mods from CurseForge.') }}</p>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 relative overflow-hidden group hover:border-blue-500/30 transition-colors duration-500">
          <div class="absolute -bottom-20 -right-20 w-64 h-64 bg-blue-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-blue-500/20 transition-all duration-500"></div>

          <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-blue-500/10 rounded-xl border border-blue-500/20"><Rocket class="text-blue-400 w-5 h-5" /></div>
            {{ t('Game Launch & Java') }}
          </h3>

          <div class="relative z-10 flex flex-col divide-y divide-white/5">
            <div class="py-5 first:pt-0">
              <label class="block text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2">Offline Username</label>
              <input v-model="state.settings.offline_username" @blur="saveTextSetting('offline_username')" type="text" placeholder="Player" class="kip-input">
            </div>

            <div class="py-5 flex gap-4">
              <div class="flex-1">
                <label class="block text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2">Game Resolution</label>
                <input v-model="state.settings.game_resolution" @blur="saveTextSetting('game_resolution')" type="text" placeholder="854x480" class="kip-input font-mono text-center">
              </div>
              <div class="flex-1 flex flex-col justify-center">
                <div class="flex items-center justify-between group/item cursor-pointer mt-4" @click="toggleSetting('game_fullscreen')">
                  <span class="font-bold text-white group-hover/item:text-blue-400 transition-colors text-sm">Start Fullscreen</span>
                  <div class="relative inline-block w-12 shrink-0">
                    <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.game_fullscreen" style="pointer-events: none;">
                    <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
                  </div>
                </div>
              </div>
            </div>

            <div class="py-5 flex items-center justify-between">
              <div>
                <span class="font-bold text-white block mb-1">{{ t('RAM Allocation') }}</span>
                <span class="text-xs text-white/50">{{ t('Maximum memory (GB). 0 = Auto') }}</span>
              </div>
              <div class="relative">
                <select v-model.number="state.settings.ram_allocation" @change="saveTextSetting('ram_allocation')" class="bg-black/60 border border-white/10 rounded-xl px-4 py-2 focus:outline-none focus:border-blue-500 font-bold cursor-pointer transition hover:bg-black/80 text-sm appearance-none pr-10 shadow-inner">
                  <option :value="0">{{ t('Auto') }}</option>
                  <option :value="2">2 GB</option>
                  <option :value="4">4 GB</option>
                  <option :value="6">6 GB</option>
                  <option :value="8">8 GB</option>
                  <option :value="12">12 GB</option>
                  <option :value="16">16 GB</option>
                </select>
                <ChevronDown class="w-4 h-4 text-white/50 absolute right-3 top-1/2 -translate-y-1/2 pointer-events-none" />
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('close_on_launch')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-blue-400 transition-colors">{{ t('Close on Launch') }}</span>
                <span class="text-xs text-white/50">{{ t('Minimize launcher when game starts.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.close_on_launch" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="py-5">
              <label class="block text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2">Custom Java Executable Path</label>
              <div class="flex gap-2">
                <input v-model="state.settings.custom_java_path" @blur="saveTextSetting('custom_java_path')" type="text" placeholder="Leave empty for auto-download" class="kip-input flex-1 font-mono text-xs">
                <button @click="pickJavaPath" class="kip-btn-ghost px-4 bg-white/5 hover:bg-white/10">
                  <FolderOpen class="w-4 h-4" />
                </button>
              </div>
            </div>

            <div class="py-5">
              <label class="block text-[10px] font-bold text-white/50 uppercase tracking-wider mb-2">Custom JVM Arguments</label>
              <textarea v-model="state.settings.custom_jvm_args" @blur="saveTextSetting('custom_jvm_args')" placeholder="Leave empty for auto-generated ZGC/G1GC flags" class="kip-input font-mono text-xs h-24 resize-none custom-scroll leading-relaxed"></textarea>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 relative overflow-hidden group hover:border-indigo-500/30 transition-colors duration-500">
          <div class="absolute -bottom-20 -left-20 w-64 h-64 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-indigo-500/20 transition-all duration-500"></div>

          <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20"><Info class="text-indigo-400 w-5 h-5" /></div>
            System Info
          </h3>

          <div class="relative z-10 flex flex-col gap-4">
            <div class="flex items-center justify-between p-5 bg-black/40 rounded-2xl border border-white/5 shadow-inner">
              <div class="flex items-center gap-4">
                <div class="p-3 bg-indigo-500/10 rounded-xl border border-indigo-500/20">
                  <Hexagon class="w-6 h-6 text-indigo-500" />
                </div>
                <div>
                  <h4 class="font-extrabold text-white text-lg tracking-wider">K.I.P. <span class="text-indigo-400">Engine</span></h4>
                  <p class="text-xs text-white/50 font-mono mt-1">v{{ state.version || '1.5.8' }} • by K.I.P. Studio</p>
                </div>
              </div>
            </div>

            <div class="flex items-center justify-between py-2 px-1">
              <div>
                <span class="font-bold text-white block mb-1">{{ t('About') }}</span>
                <span class="text-xs text-white/50">{{ t('License and Disclaimer') }}</span>
              </div>
              <button @click="legalModal.isOpen = true" class="kip-btn-ghost px-5 py-2.5 text-sm text-indigo-400 border-indigo-500/20 hover:bg-indigo-500/10 hover:text-indigo-300">
                <FileText class="w-4 h-4" /> {{ t('Legal') }}
              </button>
            </div>
          </div>
        </div>
      </div>

      <div class="flex flex-col gap-6 stagger-3">
        <div class="kip-card p-8 relative overflow-hidden group hover:border-emerald-500/30 transition-colors duration-500">
          <div class="absolute -right-20 -top-20 w-64 h-64 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-emerald-500/20 transition-all duration-500"></div>

          <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-emerald-500/10 rounded-xl border border-emerald-500/20"><SettingsIcon class="text-emerald-400 w-5 h-5" /></div>
            {{ t('App Preferences') }}
          </h3>

          <div class="relative z-10 flex flex-col divide-y divide-white/5">
            <div class="py-5 first:pt-0">
              <div class="flex items-center justify-between mb-3">
                <div>
                  <span class="font-bold text-white block mb-1">{{ t('Language') }}</span>
                  <span class="text-xs text-white/50">{{ t('Select interface language (Requires restart).') }}</span>
                </div>
              </div>
              <div class="relative custom-dropdown">
                <div @click="isLangDropdownOpen = !isLangDropdownOpen" class="kip-input cursor-pointer flex justify-between items-center hover:bg-black/60 hover:border-emerald-500/50">
                  <span class="font-bold text-sm">{{ langOptions.find(o => o.value === state.settings.lang)?.label || 'English' }}</span>
                  <ChevronDown class="w-4 h-4 text-white/50 transition-transform" :class="{'rotate-180': isLangDropdownOpen}" />
                </div>
                <transition name="fade">
                  <div v-if="isLangDropdownOpen" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-[0_0_30px_rgba(0,0,0,0.8)] overflow-hidden py-2 z-50">
                    <div v-for="opt in langOptions" :key="opt.value" @click="selectLang(opt.value)" class="px-5 py-3 hover:bg-white/5 cursor-pointer text-sm transition font-bold" :class="state.settings.lang === opt.value ? 'text-emerald-400 bg-emerald-500/10' : 'text-white/70'">
                      {{ opt.label }}
                    </div>
                  </div>
                </transition>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('auto_backup')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-emerald-400 transition-colors">{{ t('Auto-Backup Worlds') }}</span>
                <span class="text-xs text-white/50">{{ t('Create a zip archive before launching.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.auto_backup" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('rpc')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-emerald-400 transition-colors">{{ t('Discord RPC') }}</span>
                <span class="text-xs text-white/50">{{ t('Show your game status on Discord.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.rpc" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('low_graphics')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-emerald-400 transition-colors">{{ t('Low Graphics Mode') }}</span>
                <span class="text-xs text-white/50">{{ t('Disable 3D background to save GPU resources.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.low_graphics" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('autostart')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-emerald-400 transition-colors">{{ t('Run at Startup') }}</span>
                <span class="text-xs text-white/50">{{ t('Start KIP Hub with Windows.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.autostart" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>
          </div>
        </div>

        <div class="kip-card p-8 relative overflow-hidden group hover:border-rose-500/30 transition-colors duration-500">
          <div class="absolute -right-20 -bottom-20 w-64 h-64 bg-rose-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-rose-500/20 transition-all duration-500"></div>

          <h3 class="text-xl font-bold mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-rose-500/10 rounded-xl border border-rose-500/20"><Shield class="text-rose-400 w-5 h-5" /></div>
            {{ t('Security & Network') }}
          </h3>

          <div class="relative z-10 flex flex-col divide-y divide-white/5">
            <div class="flex items-center justify-between py-5 first:pt-0 group/item cursor-pointer" @click="toggleSetting('shield_auto_scan')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-rose-400 transition-colors">{{ t('K.I.P. Shield Auto-Scan') }}</span>
                <span class="text-xs text-white/50">{{ t('Scan mods for malware on import.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.shield_auto_scan" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('voice_noise_suppression')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-rose-400 transition-colors">{{ t('Voice Noise Suppression') }}</span>
                <span class="text-xs text-white/50">{{ t('Hardware level noise gate and echo cancellation.') }}</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.voice_noise_suppression" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>

            <div class="flex items-center justify-between py-5 group/item cursor-pointer" @click="toggleSetting('telemetry_opt_in')">
              <div>
                <span class="font-bold text-white block mb-1 group-hover/item:text-rose-400 transition-colors">Telemetry & AI Logs</span>
                <span class="text-xs text-white/50">Allow crash logs parsing via chosen AI provider.</span>
              </div>
              <div class="relative inline-block w-12 shrink-0">
                <input type="checkbox" class="toggle-checkbox absolute block w-6 h-6 rounded-full bg-white border-4 appearance-none" :checked="state.settings.telemetry_opt_in" style="pointer-events: none;">
                <label class="toggle-label block h-6 rounded-full" style="pointer-events: none;"></label>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="legalModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="legalModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[80vh] overflow-hidden relative border-indigo-500/30">
          <div class="absolute top-0 right-0 w-96 h-96 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="p-8 border-b border-white/5 flex justify-between items-center bg-black/40 shrink-0 relative z-10">
            <h3 class="text-2xl font-extrabold flex items-center gap-3">
              <ShieldCheck class="w-7 h-7 text-indigo-400" /> {{ t('License and Disclaimer') }}
            </h3>
            <button @click="legalModal.isOpen = false" class="kip-btn-ghost p-2 hover:text-red-400 border-transparent hover:bg-white/10">
              <X class="w-6 h-6" />
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 relative z-10 text-sm text-white/80 leading-relaxed whitespace-pre-wrap bg-[#050505]/80 font-medium">{{ t("NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.\n\nThis software downloads files directly from Mojang servers. A valid Minecraft license is required to play. Provided 'AS IS' under the GNU General Public License v3.0 (GPL-3.0).") }}</div>

          <div class="p-6 border-t border-white/5 bg-black/40 shrink-0 flex justify-end relative z-10">
            <button @click="legalModal.isOpen = false" class="kip-btn-primary px-8 py-3">{{ t('Close') }}</button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import {
  Sparkles,
  Info,
  X,
  Cpu,
  Box,
  Settings as SettingsIcon,
  FileText,
  Hexagon,
  ShieldCheck,
  ChevronDown,
  Bot,
  Server,
  Rocket,
  Shield,
  FolderOpen,
} from 'lucide-vue-next'
import { state, t, saveSetting, showToast, type SettingsState } from '@/store'
import { bridge } from '@/bridge'

interface SelectOption<T = string> {
  value: T
  label: string
}

const legalModal = ref<{ isOpen: boolean }>({ isOpen: false })
const isLangDropdownOpen = ref<boolean>(false)
const isProviderDropdownOpen = ref<boolean>(false)

const langOptions: SelectOption[] = [
  { value: 'en', label: 'English' },
  { value: 'ru', label: 'Русский' },
  { value: 'es', label: 'Español' },
  { value: 'de', label: 'Deutsch' },
  { value: 'zh', label: '中文' },
  { value: 'ja', label: '日本語' },
  { value: 'ko', label: '한국어' },
  { value: 'fr', label: 'Français' },
  { value: 'pt', label: 'Português' },
  { value: 'it', label: 'Italiano' },
  { value: 'pl', label: 'Polski' },
  { value: 'tr', label: 'Türkçe' },
]

const providerOptions: SelectOption[] = [
  { value: 'google', label: 'Google Gemini' },
  { value: 'openai', label: 'OpenAI ChatGPT' },
  { value: 'anthropic', label: 'Anthropic Claude' },
  { value: 'ollama', label: 'Ollama (Local AI)' },
]

const saveTextSetting = async (key: keyof SettingsState): Promise<void> => {
  const result = await saveSetting(key, state.settings[key])
  if (result) {
    showToast(t('Saved'), t('Settings updated successfully.'), 'success')
  } else {
    showToast(t('Error'), t('Failed to save setting.'), 'danger')
  }
}

const toggleSetting = async (key: keyof SettingsState): Promise<void> => {
  const currentValue = Boolean(state.settings[key])
  const newValue = !currentValue
  const result = await saveSetting(key, newValue)
  if (!result) {
    (state.settings as Record<string, unknown>)[key] = currentValue
    showToast(t('Error'), t('Failed to save setting.'), 'danger')
  }
}

const selectLang = (val: string): void => {
  state.settings.lang = val
  saveTextSetting('lang')
  isLangDropdownOpen.value = false
}

const selectProvider = (val: string): void => {
  state.settings.ai_provider = val
  saveTextSetting('ai_provider')
  isProviderDropdownOpen.value = false
}

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    isLangDropdownOpen.value = false
    isProviderDropdownOpen.value = false
  }
}

const pickJavaPath = async (): Promise<void> => {
  try {
    const path = await bridge.pickFile()
    if (path && typeof path === 'string' && path.trim().length > 0) {
      state.settings.custom_java_path = path.trim()
      await saveTextSetting('custom_java_path')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Failed to pick Java path.'), 'danger')
  }
}

onMounted(() => {
  window.addEventListener('click', closeDropdowns)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>