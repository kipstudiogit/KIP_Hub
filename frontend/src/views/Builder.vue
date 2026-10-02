<template>
  <div class="h-full flex flex-col min-h-0 relative">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-extrabold mb-1 flex items-center gap-3">
        <Wand2 class="text-indigo-400 w-8 h-8" /> {{ t('Auto-Builder') }}
      </h2>
      <p class="text-white/50 text-sm">{{ t('Describe your dream modpack and let AI build it. Foundations and libraries are added automatically.') }}</p>
    </div>

    <div class="grid grid-cols-3 gap-6 flex-1 min-h-0 stagger-2">
      <div class="col-span-2 kip-card p-8 flex flex-col relative h-full group hover:border-indigo-500/30 transition-colors duration-500">
        <div class="absolute inset-0 overflow-hidden rounded-3xl pointer-events-none">
          <div class="absolute -top-32 -left-32 w-96 h-96 bg-indigo-600/10 blur-[100px] rounded-full transition-all duration-700 group-hover:bg-indigo-600/20"></div>
        </div>

        <div class="flex gap-4 mb-6 shrink-0 relative z-30">
          <div class="relative flex-1 custom-dropdown">
            <div @click="activeDropdown = activeDropdown === 'loader' ? null : 'loader'" class="kip-input cursor-pointer flex justify-between items-center hover:bg-black/60 transition" :class="{'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]': activeDropdown === 'loader'}">
              <span class="font-bold text-sm uppercase tracking-wider">{{ builderData.loader }}</span>
              <ChevronDown class="w-4 h-4 text-white/50 transition-transform" :class="{'rotate-180': activeDropdown === 'loader'}" />
            </div>
            <transition name="fade">
              <div v-if="activeDropdown === 'loader'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-2 z-50">
                <div v-for="l in (['fabric', 'forge', 'neoforge', 'quilt'] as const)" :key="l" @click="builderData.loader = l; activeDropdown = null" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition font-bold text-sm uppercase tracking-wider" :class="builderData.loader === l ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'">
                  {{ l }}
                </div>
              </div>
            </transition>
          </div>

          <div class="relative flex-1 custom-dropdown">
            <div @click="activeDropdown = activeDropdown === 'version' ? null : 'version'" class="kip-input cursor-pointer flex justify-between items-center hover:bg-black/60 transition" :class="{'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]': activeDropdown === 'version'}">
              <span class="font-bold text-sm">{{ builderData.mc_version || t('Select Version') }}</span>
              <ChevronDown class="w-4 h-4 text-white/50 transition-transform" :class="{'rotate-180': activeDropdown === 'version'}" />
            </div>
            <transition name="fade">
              <div v-if="activeDropdown === 'version'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-2xl shadow-2xl overflow-hidden py-2 z-50 max-h-60 overflow-y-auto custom-scroll">
                <div v-for="v in state.mcVersions" :key="v" @click="builderData.mc_version = v; activeDropdown = null" class="px-5 py-3 hover:bg-white/5 cursor-pointer transition font-bold text-sm" :class="builderData.mc_version === v ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'">
                  {{ v }}
                </div>
              </div>
            </transition>
          </div>
        </div>

        <textarea v-model="builderData.prompt" :placeholder="t('E.g., I want a medieval RPG modpack with dragons, deep dungeons, magic spells, and beautiful world generation.')" class="kip-input flex-1 resize-none custom-scroll relative z-10 mb-6 transition shadow-inner leading-relaxed text-sm p-6"></textarea>

        <div v-if="builderData.isBuilding" class="mb-6 shrink-0 relative z-10 bg-black/60 backdrop-blur border border-indigo-500/30 rounded-2xl p-5 shadow-[0_0_20px_rgba(99,102,241,0.1)]">
          <div class="flex justify-between items-center mb-3">
            <span class="text-xs font-bold text-indigo-400 animate-pulse flex items-center gap-2 uppercase tracking-wider">
              <Cpu class="w-4 h-4" /> {{ builderData.status }}
            </span>
            <span class="text-xs font-mono text-white/70 bg-white/10 px-2 py-1 rounded">{{ Math.round(builderData.progress) }}%</span>
          </div>
          <div class="w-full h-1.5 bg-black/50 border border-white/5 rounded-full overflow-hidden shadow-inner">
            <div class="h-full bg-gradient-to-r from-indigo-500 to-purple-500 transition-all duration-300 shadow-[0_0_10px_rgba(99,102,241,0.8)]" :style="{ width: builderData.progress + '%' }"></div>
          </div>
        </div>

        <button @click="generateAutoBuild" :disabled="builderData.isBuilding || !builderData.prompt.trim()" class="kip-btn-primary py-5 text-lg w-full relative z-10">
          <span v-if="!builderData.isBuilding" class="flex items-center justify-center gap-2"><Sparkles class="w-6 h-6" /> {{ t('Generate Modpack') }}</span>
          <span v-else class="flex items-center justify-center gap-2"><Loader class="w-6 h-6 animate-spin" /> {{ t('Synthesizing...') }}</span>
        </button>
      </div>

      <div class="col-span-1 flex flex-col gap-6 h-full stagger-3">
        <div class="kip-card p-8 relative overflow-hidden group hover:border-emerald-500/30 transition duration-500">
          <div class="absolute -right-20 -bottom-20 w-64 h-64 bg-emerald-500/10 blur-[100px] rounded-full pointer-events-none group-hover:bg-emerald-500/20 transition-all duration-500"></div>
          <h3 class="font-extrabold text-xl text-emerald-400 mb-6 flex items-center gap-3 relative z-10">
            <div class="p-2 bg-emerald-500/10 rounded-xl border border-emerald-500/20"><Keyboard class="w-5 h-5" /></div>
            {{ t('Auto-Keybinds') }}
          </h3>
          <p class="text-sm text-white/60 mb-8 relative z-10 leading-relaxed font-medium">{{ t('Resolves button conflicts in options.txt automatically by remapping duplicates to free keys (Numpad, Brackets, etc).') }}</p>
          <button @click="resolveKeybinds" :disabled="isResolvingKeybinds" class="w-full py-4 bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)] rounded-xl font-extrabold transition text-sm flex items-center justify-center gap-2 relative z-10 disabled:opacity-50">
            <Loader v-if="isResolvingKeybinds" class="w-5 h-5 animate-spin" />
            <Wrench v-else class="w-5 h-5" />
            {{ isResolvingKeybinds ? t('Resolving...') : t('Resolve Conflicts') }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { Wand2, Cpu, Sparkles, Loader, Keyboard, ChevronDown, Wrench } from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type AutoBuildResult,
  type KeybindResolveResult,
  type StoreDetailsResponse,
  type StoreItemFile,
} from '@/bridge'

type SupportedLoader = 'fabric' | 'forge' | 'neoforge' | 'quilt'

interface BuilderFormState {
  prompt: string
  mc_version: string
  loader: SupportedLoader
  isBuilding: boolean
  status: string
  progress: number
}

const activeDropdown = ref<'loader' | 'version' | null>(null)
const isResolvingKeybinds = ref<boolean>(false)

const builderData = ref<BuilderFormState>({
  prompt: '',
  mc_version: '',
  loader: 'fabric',
  isBuilding: false,
  status: '',
  progress: 0,
})

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    activeDropdown.value = null
  }
}

const generateAutoBuild = async (): Promise<void> => {
  const cleanPrompt = builderData.value.prompt.trim()
  if (!cleanPrompt || !builderData.value.mc_version) return

  if (
    !state.settings.ai_api_key &&
    !state.settings.openai_api_key &&
    !state.settings.anthropic_api_key &&
    state.settings.ai_provider !== 'ollama'
  ) {
    showToast(t('Error'), t('AI API Key is required. Set it in Settings.'), 'danger')
    return
  }

  builderData.value.isBuilding = true
  builderData.value.progress = 5
  builderData.value.status = t('Consulting Neural Core...')

  try {
    const res: AutoBuildResult = await bridge.generateAutoBuild(
      cleanPrompt,
      builderData.value.mc_version,
      builderData.value.loader
    )

    if (res && res.success) {
      const rawSlugs = [...(res.foundation || []), ...(res.mods || [])]
      const allSlugs = Array.from(new Set(rawSlugs))
        .map((s) => (typeof s === 'string' ? s.trim().toLowerCase() : ''))
        .filter((s) => s.length > 0)

      let successCount = 0

      for (let i = 0; i < allSlugs.length; i++) {
        const slug = allSlugs[i]
        if (!slug) continue

        builderData.value.status = `${t('Resolving')} ${slug}... (${i + 1}/${allSlugs.length})`
        builderData.value.progress = 10 + (i / allSlugs.length) * 80

        try {
          const versions = await invokeSafe<StoreDetailsResponse>('get_store_full_details', {
            provider: 'modrinth',
            projectId: slug,
            loader: builderData.value.loader,
            gameVersion: builderData.value.mc_version,
          })

          if (versions && versions.success && versions.versions && versions.versions.length > 0) {
            const targetVer = versions.versions[0]
            if (targetVer && targetVer.files && targetVer.files.length > 0) {
              const file = targetVer.files.find((f: StoreItemFile) => f.primary) || targetVer.files[0]
              if (file) {
                builderData.value.status = `${t('Downloading')} ${file.filename}...`
                await invokeSafe<boolean>('download_specific_file', {
                  url: file.url,
                  filename: file.filename,
                  projectType: 'mod',
                })
                successCount++
              }
            }
          }
        } catch {
          // Ignored
        }
      }

      builderData.value.progress = 100
      builderData.value.status = t('Finalizing Modpack...')
      showToast(
        t('Build Complete'),
        `${t('Successfully integrated')} ${successCount} ${t('core modules.')}`,
        'success'
      )
    } else {
      showToast(t('Build Failed'), t('AI generation failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  } finally {
    setTimeout(() => {
      builderData.value.isBuilding = false
      builderData.value.progress = 0
      builderData.value.status = ''
    }, 1000)
  }
}

const resolveKeybinds = async (): Promise<void> => {
  if (isResolvingKeybinds.value) return
  isResolvingKeybinds.value = true

  try {
    const res: KeybindResolveResult = await bridge.resolveKeybinds()
    if (res && res.success) {
      if (res.changes > 0) {
        showToast(
          t('Resolved'),
          `${t('Fixed')} ${res.changes} ${t('keybind conflicts.')}`,
          'success'
        )
      } else {
        showToast(t('Clean'), t('No keybind conflicts detected.'), 'success')
      }
    } else {
      showToast(t('Error'), t('Failed to resolve keybinds.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  } finally {
    isResolvingKeybinds.value = false
  }
}

onMounted(async () => {
  window.addEventListener('click', closeDropdowns)

  if (state.mcVersions.length === 0) {
    try {
      state.mcVersions = await bridge.getMcVersions()
      if (state.mcVersions.length > 0 && !state.mcVersions[0]?.includes('Error')) {
        builderData.value.mc_version = state.mcVersions[0] || ''
      }
    } catch {
      // Ignored
    }
  } else if (!builderData.value.mc_version && state.mcVersions.length > 0) {
    builderData.value.mc_version = state.mcVersions[0] || ''
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
})
</script>