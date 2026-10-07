<script setup lang="ts">
import { ref } from 'vue'
import {
  Globe,
  X,
  Search,
  UploadCloud,
  DownloadCloud,
  Loader,
  CloudOff,
} from 'lucide-vue-next'
import type { HubPreset } from '../../types/content'
import { bridge, invokeSafe } from '@/bridge'
import { showToast } from '../../composables/useToasts'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

const props = defineProps<{
  presets: HubPreset[]
  loading: boolean
}>()

const activeTab = ref<'browse' | 'publish'>('browse')
const isPublishing = ref(false)

const publishForm = ref({
  title: '',
  author: 'Operator',
  desc: '',
})

async function publishPreset(): Promise<void> {
  if (!publishForm.value.title.trim()) return
  isPublishing.value = true

  try {
    const mods = await bridge.getLocalMods('mods')
    const activeMods = mods.filter((m) => !m.disabled).map((m) => m.name || m.filename)

    if (activeMods.length === 0) {
      showToast(t('Error'), 'No active mods found to publish in preset.', 'danger')
      isPublishing.value = false
      return
    }

    const success = await bridge.publishHub(
      publishForm.value.title,
      publishForm.value.author,
      publishForm.value.desc,
      activeMods
    )

    if (success) {
      showToast(t('Success'), 'Profile published to Global Hub!', 'success')
      publishForm.value.title = ''
      publishForm.value.desc = ''
      activeTab.value = 'browse'
    } else {
      showToast(t('Error'), 'Failed to publish preset profile.', 'danger')
    }
  } catch (err: unknown) {
    showToast(t('Error'), String(err), 'danger')
  } finally {
    isPublishing.value = false
  }
}

async function applyPreset(preset: HubPreset): Promise<void> {
  isOpen.value = false
  showToast(t('Applying'), `Downloading modules for preset: ${preset.title}`, 'info')

  try {
    const issues = preset.preset.map((modName) => ({
      action: 'DOWNLOAD',
      target: modName,
      selected: true,
    }))

    const res = await invokeSafe<{ success: boolean; downloaded: number }>('apply_doctor_fixes', {
      issues,
    })

    if (res && (res.success || res.downloaded > 0)) {
      showToast(t('Success'), `Acquired ${res.downloaded || 0} preset modules.`, 'success')
    }
  } catch (err: unknown) {
    showToast(t('Error'), String(err), 'danger')
  }
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="kip-card p-0 w-full max-w-4xl border-blue-500/30 flex flex-col h-[85vh] relative overflow-hidden shadow-[0_0_60px_rgba(59,130,246,0.25)]">
        <!-- Header -->
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
          <div class="flex items-center gap-4">
            <div class="p-3 bg-blue-500/20 rounded-2xl border border-blue-500/30 text-blue-400">
              <Globe class="w-6 h-6" />
            </div>
            <div>
              <h3 class="text-2xl font-black uppercase text-white tracking-wider">Community Hub Registry</h3>
              <span class="text-xs font-mono text-blue-400">Cloudflare Edge KV Presets & Distributed Configuration Profiles</span>
            </div>
          </div>
          <button @click="isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition">
            <X class="w-5 h-5" />
          </button>
        </header>

        <!-- Navigation Tabs -->
        <div class="flex border-b border-white/5 bg-black/30 shrink-0">
          <button
            @click="activeTab = 'browse'"
            :class="activeTab === 'browse' ? 'text-blue-400 border-b-2 border-blue-400 bg-blue-500/5' : 'text-white/40 hover:text-white'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider"
          >
            <Search class="w-4 h-4" /> Shared Profiles ({{ presets.length }})
          </button>
          <button
            @click="activeTab = 'publish'"
            :class="activeTab === 'publish' ? 'text-blue-400 border-b-2 border-blue-400 bg-blue-500/5' : 'text-white/40 hover:text-white'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider"
          >
            <UploadCloud class="w-4 h-4" /> Publish Active Profile
          </button>
        </div>

        <!-- Body -->
        <main class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#030305]/95 min-h-0">
          <div v-if="activeTab === 'browse'" class="h-full">
            <div v-if="loading" class="flex flex-col justify-center items-center h-full gap-3 py-20">
              <Loader class="w-8 h-8 animate-spin text-blue-500" />
              <span class="font-mono text-xs text-white/40 uppercase">Connecting to Cloudflare KV Edge...</span>
            </div>

            <div v-else-if="presets.length === 0" class="flex flex-col justify-center items-center h-full text-white/40 py-20 border border-dashed border-white/10 rounded-3xl">
              <CloudOff class="w-14 h-14 mb-3 opacity-40" />
              <p class="font-mono text-xs">Community preset repository is empty. Be the first to publish.</p>
            </div>

            <div v-else class="grid grid-cols-2 gap-5">
              <div
                v-for="preset in presets"
                :key="preset.id"
                class="kip-card p-6 kip-card-hover group border border-white/5 hover:border-blue-500/30 flex flex-col justify-between"
              >
                <div>
                  <div class="flex items-center justify-between gap-2 mb-1.5">
                    <h4 class="font-black text-lg text-white leading-tight">{{ preset.title }}</h4>
                    <span class="px-2 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 text-[9px] font-mono font-bold">
                      {{ preset.preset.length }} mods
                    </span>
                  </div>

                  <p class="text-xs text-white/40 mb-3 font-mono">Curated by <span class="text-white font-bold">{{ preset.author }}</span></p>
                  <p class="text-xs text-white/70 line-clamp-3 leading-relaxed mb-5">{{ preset.description }}</p>
                </div>

                <button
                  @click="applyPreset(preset)"
                  class="w-full py-2.5 bg-blue-500/10 hover:bg-blue-500 hover:text-white text-blue-400 font-black text-xs uppercase tracking-wider rounded-xl transition flex items-center justify-center gap-2 border border-blue-500/30 shadow-lg"
                >
                  <DownloadCloud class="w-4 h-4" />
                  <span>Apply Preset to Instance</span>
                </button>
              </div>
            </div>
          </div>

          <div v-if="activeTab === 'publish'" class="max-w-xl mx-auto h-full flex flex-col justify-center">
            <div class="kip-card p-8 flex flex-col gap-5 border border-white/10 bg-black/60 shadow-2xl">
              <div>
                <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Profile Title</label>
                <input v-model="publishForm.title" type="text" placeholder="Medieval Exploration Stack" class="kip-input text-xs font-bold">
              </div>

              <div>
                <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Author Handle</label>
                <input v-model="publishForm.author" type="text" class="kip-input text-xs font-mono">
              </div>

              <div>
                <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Description & Highlights</label>
                <textarea v-model="publishForm.desc" rows="4" placeholder="Overview of features, shaders, keybindings..." class="kip-input resize-none custom-scroll text-xs leading-relaxed"></textarea>
              </div>

              <button
                @click="publishPreset"
                :disabled="isPublishing || !publishForm.title.trim()"
                class="kip-btn-primary py-3.5 bg-blue-500 hover:bg-blue-400 text-white font-black text-xs uppercase tracking-wider shadow-[0_0_20px_rgba(59,130,246,0.3)]"
              >
                <Loader v-if="isPublishing" class="w-4 h-4 animate-spin" />
                <UploadCloud v-else class="w-4 h-4" />
                <span>Publish to Global Hub</span>
              </button>
            </div>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>