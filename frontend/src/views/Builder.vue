<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, nextTick } from 'vue'
import {
  Wand2,
  Sparkles,
  Loader,
  ChevronDown,
  Wrench,
  CheckCircle,
  Terminal,
  ShieldCheck,
  Zap,
  Swords,
  Cog,
  Eye,
  Flame,
  Compass,
  AlertTriangle,
  Play,
  RotateCcw,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import { useAutoBuilder } from '../composables/useAutoBuilder'
import type { SynthPreset } from '../types/builder'

const activeDropdown = ref<'loader' | 'version' | null>(null)
const logContainer = ref<HTMLElement | null>(null)

const builderForm = ref({
  prompt: '',
  mcVersion: '26.3',
  loader: 'fabric',
  maxMods: 25,
  includePerformance: true,
  resolveKeybindsAfter: true,
})

const {
  isBuilding,
  isResolvingKeybinds,
  terminalLogs,
  currentProgress,
  lastBuildResult,
  keybindReport,
  startBuild,
  resolveKeybinds,
} = useAutoBuilder()

const presets: SynthPreset[] = [
  {
    id: 'rpg_dragons',
    name: 'Medieval RPG & Dragons',
    desc: 'Deep dungeons, boss fights, mythical creatures, questlines and magic lore.',
    icon: Swords,
    loader: 'fabric',
    maxMods: 30,
    prompt: 'I want an immersive medieval fantasy RPG modpack with dragons, sprawling underground dungeons, bosses, custom loot, and atmospheric world generation.',
  },
  {
    id: 'tech_automation',
    name: 'Industrial Automation',
    desc: 'Complex logistics, power networks, pipes, machines and factory setups.',
    icon: Cog,
    loader: 'neoforge',
    maxMods: 35,
    prompt: 'A comprehensive modern technical modpack with machinery, electrical grids, automated farming, item logistics, and ore processing pipelines.',
  },
  {
    id: 'vanilla_ultra_fps',
    name: 'Vanilla+ Visuals & 300 FPS',
    desc: 'Maximum optimization, fluid physics, audio overhaul and shader pipelines.',
    icon: Eye,
    loader: 'fabric',
    maxMods: 18,
    prompt: 'Essential vanilla enhancement pack focused on achieving 300+ FPS, beautiful shaders support, ambient dynamic sounds, appleskin, and smooth camera physics.',
  },
  {
    id: 'arcane_magic',
    name: 'Arcane Sorcery & Spells',
    desc: 'Spell crafting, celestial altars, wizard towers and ancient artifacts.',
    icon: Sparkles,
    loader: 'fabric',
    maxMods: 25,
    prompt: 'An enchanting magic modpack featuring custom spellcraft, wizard robes, celestial rituals, magical artifacts, and dangerous arcane dimensions.',
  },
  {
    id: 'eldritch_horror',
    name: 'Eldritch Horror & Hardcore',
    desc: 'Terrifying ambient monsters, sanity meters, pitch-black caves and dread.',
    icon: Flame,
    loader: 'forge',
    maxMods: 22,
    prompt: 'A dark, psychological horror survival modpack featuring horrifying stalker entities, sanity mechanics, realistic body damage, and unnerving cave ambiance.',
  },
  {
    id: 'dimension_voyager',
    name: 'Dimensional Exploration',
    desc: 'New planets, twilight dimensions, space rockets and endless frontiers.',
    icon: Compass,
    loader: 'fabric',
    maxMods: 28,
    prompt: 'An exploration-heavy modpack with unique dimensional portals, alien planets, custom biomes, waystones, and glider flight systems.',
  },
]

function applyPreset(preset: SynthPreset): void {
  builderForm.value.prompt = preset.prompt
  builderForm.value.loader = preset.loader
  builderForm.value.maxMods = preset.maxMods
  showToast(t('Preset Applied'), preset.name, 'info')
}

async function handleStartSynthesis(): Promise<void> {
  const cleanPrompt = builderForm.value.prompt.trim()
  if (!cleanPrompt) return

  if (
    !state.settings.ai_api_key &&
    !state.settings.openai_api_key &&
    !state.settings.anthropic_api_key &&
    state.settings.ai_provider !== 'ollama'
  ) {
    showToast(t('Error'), 'AI API Key is required in Settings to synthesize modpacks.', 'danger')
    return
  }

  await startBuild({
    prompt: cleanPrompt,
    mcVersion: builderForm.value.mcVersion,
    loader: builderForm.value.loader,
    maxMods: builderForm.value.maxMods,
    includePerformance: builderForm.value.includePerformance,
    resolveKeybindsAfter: builderForm.value.resolveKeybindsAfter,
  })

  nextTick(() => {
    if (logContainer.value) {
      logContainer.value.scrollTop = logContainer.value.scrollHeight
    }
  })
}

function launchSynthesizedInstance(): void {
  state.currentView = 'launcher'
}

function closeDropdowns(): void {
  activeDropdown.value = null
}

onMounted(() => {
  window.addEventListener('click', closeDropdowns)
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 select-none relative overflow-y-auto custom-scroll pr-2 pb-12" @click="closeDropdowns">
    <!-- Header Matrix -->
    <header class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          <Wand2 class="text-indigo-400 w-8 h-8" />
          <span>{{ t('Auto-Builder') }}</span>
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Autonomous Neural Modpack Compiler, Dependency Graph Resolver & Keybind Balancer</p>
      </div>

      <div class="flex items-center gap-3">
        <button
          @click="resolveKeybinds"
          :disabled="isResolvingKeybinds || isBuilding"
          class="kip-btn-ghost px-4 py-2.5 text-xs font-mono font-bold uppercase tracking-wider text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10 flex items-center gap-2"
        >
          <Loader v-if="isResolvingKeybinds" class="w-4 h-4 animate-spin" />
          <Wrench v-else class="w-4 h-4" />
          <span>Fix Keybinds</span>
        </button>

        <button
          v-if="lastBuildResult?.success"
          @click="launchSynthesizedInstance"
          class="kip-btn-primary px-6 py-2.5 text-xs font-black uppercase tracking-widest bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_20px_rgba(16,185,129,0.4)] flex items-center gap-2"
        >
          <Play class="w-4 h-4 fill-current" />
          <span>Launch Instance</span>
        </button>
      </div>
    </header>

    <!-- Quick Preset Synth Chips -->
    <section class="mb-6 shrink-0 z-10">
      <div class="flex items-center gap-2 mb-2 text-[10px] font-mono font-bold uppercase text-white/40">
        <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
        <span>Neural Concept Templates</span>
      </div>

      <div class="grid grid-cols-6 gap-3">
        <button
          v-for="p in presets"
          :key="p.id"
          @click="applyPreset(p)"
          class="kip-card p-3 text-left border border-white/5 hover:border-indigo-500/40 bg-black/40 hover:bg-black/60 transition group flex flex-col justify-between h-24"
        >
          <div class="flex items-center justify-between mb-1">
            <component :is="p.icon" class="w-4 h-4 text-indigo-400 group-hover:scale-110 transition-transform" />
            <span class="text-[8px] font-mono uppercase px-1.5 py-0.5 rounded bg-white/5 text-white/40 border border-white/10 font-bold">
              {{ p.loader }}
            </span>
          </div>
          <span class="font-bold text-xs text-white leading-tight line-clamp-2">{{ p.name }}</span>
        </button>
      </div>
    </section>

    <!-- Main Workspace -->
    <div class="grid grid-cols-12 gap-6 flex-1 min-h-0 z-10">
      <!-- Left Config Deck -->
      <section class="col-span-7 flex flex-col gap-4">
        <div class="kip-card p-6 border border-white/10 flex flex-col gap-5 bg-black/60 shadow-2xl">
          <!-- Parameter Bar -->
          <div class="grid grid-cols-3 gap-3">
            <!-- Loader Selector -->
            <div class="relative" @click.stop>
              <label class="text-[9px] font-mono uppercase text-white/40 font-bold block mb-1">Mod Loader</label>
              <div
                @click="activeDropdown = activeDropdown === 'loader' ? null : 'loader'"
                class="kip-input py-2 px-3 text-xs font-bold uppercase flex justify-between items-center cursor-pointer hover:border-indigo-500"
                :class="activeDropdown === 'loader' ? 'border-indigo-500' : ''"
              >
                <span>{{ builderForm.loader }}</span>
                <ChevronDown class="w-3.5 h-3.5 text-white/40" />
              </div>
              <div
                v-if="activeDropdown === 'loader'"
                class="absolute top-full left-0 w-full mt-1.5 bg-[#121214]/98 border border-white/10 rounded-xl shadow-2xl py-1 z-50 backdrop-blur-xl"
              >
                <div
                  v-for="l in (['fabric', 'forge', 'neoforge', 'quilt'] as const)"
                  :key="l"
                  @click="builderForm.loader = l; activeDropdown = null"
                  class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase"
                  :class="builderForm.loader === l ? 'text-indigo-400' : 'text-white/70'"
                >
                  {{ l }}
                </div>
              </div>
            </div>

            <!-- Version Selector -->
            <div class="relative" @click.stop>
              <label class="text-[9px] font-mono uppercase text-white/40 font-bold block mb-1">Target Version</label>
              <div
                @click="activeDropdown = activeDropdown === 'version' ? null : 'version'"
                class="kip-input py-2 px-3 text-xs font-mono font-bold flex justify-between items-center cursor-pointer hover:border-indigo-500"
                :class="activeDropdown === 'version' ? 'border-indigo-500' : ''"
              >
                <span>MC {{ builderForm.mcVersion }}</span>
                <ChevronDown class="w-3.5 h-3.5 text-white/40" />
              </div>
              <div
                v-if="activeDropdown === 'version'"
                class="absolute top-full left-0 w-full mt-1.5 bg-[#121214]/98 border border-white/10 rounded-xl shadow-2xl py-1 z-50 backdrop-blur-xl max-h-48 overflow-y-auto custom-scroll"
              >
                <div
                  v-for="v in ['26.3', '26.2', '26.1', '1.21.4', '1.21.1', '1.20.1', '1.16.5']"
                  :key="v"
                  @click="builderForm.mcVersion = v; activeDropdown = null"
                  class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold"
                  :class="builderForm.mcVersion === v ? 'text-indigo-400' : 'text-white/70'"
                >
                  {{ v }}
                </div>
              </div>
            </div>

            <!-- Mod Limit Slider -->
            <div>
              <div class="flex justify-between items-center mb-1">
                <label class="text-[9px] font-mono uppercase text-white/40 font-bold">Scope Limit</label>
                <span class="text-xs font-mono font-bold text-indigo-400">{{ builderForm.maxMods }} mods</span>
              </div>
              <input
                v-model.number="builderForm.maxMods"
                type="range"
                min="10"
                max="60"
                step="5"
                class="w-full accent-indigo-500 bg-white/10 h-1.5 rounded-lg appearance-none cursor-pointer"
              >
            </div>
          </div>

          <!-- Prompt Area -->
          <div class="flex flex-col gap-2">
            <label class="text-[9px] font-mono uppercase text-white/40 font-bold">Neural Concept Specifications</label>
            <textarea
              v-model="builderForm.prompt"
              rows="6"
              placeholder="Describe your desired gameplay experience in natural language... Include preferred biomes, mechanical tiers, magic schools, visual enhancements or specific mods."
              class="kip-input text-xs font-mono resize-none leading-relaxed p-4 h-44 select-text"
            ></textarea>
          </div>

          <!-- Switches -->
          <div class="grid grid-cols-2 gap-3 pt-2 border-t border-white/5">
            <div
              @click="builderForm.includePerformance = !builderForm.includePerformance"
              class="p-3 bg-black/40 rounded-xl border border-white/5 flex items-center justify-between cursor-pointer"
            >
              <div>
                <span class="text-xs font-bold text-white block">Foundation Optimization</span>
                <span class="text-[9px] font-mono text-white/40">Inject Sodium, Lithium, FerriteCore</span>
              </div>
              <input type="checkbox" :checked="builderForm.includePerformance" class="accent-indigo-500 w-4 h-4 pointer-events-none">
            </div>

            <div
              @click="builderForm.resolveKeybindsAfter = !builderForm.resolveKeybindsAfter"
              class="p-3 bg-black/40 rounded-xl border border-white/5 flex items-center justify-between cursor-pointer"
            >
              <div>
                <span class="text-xs font-bold text-white block">Auto-Remap Keybinds</span>
                <span class="text-[9px] font-mono text-white/40">Resolve duplicates in options.txt</span>
              </div>
              <input type="checkbox" :checked="builderForm.resolveKeybindsAfter" class="accent-indigo-500 w-4 h-4 pointer-events-none">
            </div>
          </div>

          <!-- Action Button -->
          <button
            @click="handleStartSynthesis"
            :disabled="isBuilding || !builderForm.prompt.trim()"
            class="kip-btn-primary py-4 text-xs font-black uppercase tracking-widest shadow-[0_0_25px_rgba(99,102,241,0.4)] flex items-center justify-center gap-2"
          >
            <Loader v-if="isBuilding" class="w-4 h-4 animate-spin" />
            <Sparkles v-else class="w-4 h-4" />
            <span>{{ isBuilding ? 'Synthesizing Architecture...' : 'Ignite Modpack Synthesis' }}</span>
          </button>
        </div>
      </section>

      <!-- Right Synth Progress & Terminal Deck -->
      <section class="col-span-5 flex flex-col gap-4">
        <!-- Live Synthesis Status Card -->
        <div class="kip-card p-6 border border-white/10 flex flex-col justify-between bg-black/60 shadow-2xl">
          <div class="flex justify-between items-start mb-4">
            <div>
              <span class="text-[9px] font-mono font-bold uppercase text-white/40 block">Compilation Phase</span>
              <span class="text-sm font-black text-indigo-400 font-mono uppercase mt-0.5 block">
                {{ currentProgress.phase }}
              </span>
            </div>
            <span class="text-xs font-mono font-black text-white bg-white/10 px-2.5 py-1 rounded-lg">
              {{ Math.round(currentProgress.percent) }}%
            </span>
          </div>

          <!-- Progress Bar -->
          <div class="w-full h-2 bg-black/80 rounded-full overflow-hidden border border-white/5 mb-3">
            <div
              class="h-full bg-gradient-to-r from-indigo-500 via-purple-500 to-emerald-400 transition-all duration-300"
              :style="{ width: `${currentProgress.percent}%` }"
            ></div>
          </div>

          <p class="text-[11px] font-mono text-white/60 truncate">{{ currentProgress.currentStep }}</p>
        </div>

        <!-- Terminal Output -->
        <div class="kip-card p-4 border border-white/10 flex-1 flex flex-col bg-black/80 font-mono min-h-[260px] shadow-inner">
          <div class="flex justify-between items-center pb-2 mb-2 border-b border-white/5 text-[10px] text-white/40">
            <span class="flex items-center gap-1.5"><Terminal class="w-3.5 h-3.5 text-indigo-400" /> Synthesis Telemetry Log</span>
            <span>{{ terminalLogs.length }} lines</span>
          </div>

          <div ref="logContainer" class="flex-1 overflow-y-auto custom-scroll text-[11px] text-white/70 space-y-1 select-text leading-relaxed">
            <div v-for="(log, idx) in terminalLogs" :key="idx" class="truncate" :class="log.includes('[ERROR]') ? 'text-rose-400' : log.includes('[DONE]') ? 'text-emerald-400 font-bold' : ''">
              {{ log }}
            </div>
          </div>
        </div>

        <!-- Keybind Remap Report (If available) -->
        <div v-if="keybindReport && keybindReport.conflictsResolved > 0" class="kip-card p-4 border border-emerald-500/30 bg-emerald-950/10">
          <div class="flex items-center gap-2 mb-2 text-xs font-mono font-bold text-emerald-400">
            <CheckCircle class="w-4 h-4" />
            <span>Remapped {{ keybindReport.conflictsResolved }} Collisions</span>
          </div>
          <div class="max-h-24 overflow-y-auto custom-scroll space-y-1 text-[10px] font-mono text-white/60">
            <div v-for="d in keybindReport.details" :key="d.keyId" class="flex justify-between">
              <span class="text-white/80 truncate max-w-[140px]">{{ d.keyId }}</span>
              <span>{{ d.oldBinding }} &rarr; <strong class="text-emerald-400">{{ d.newBinding }}</strong></span>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>