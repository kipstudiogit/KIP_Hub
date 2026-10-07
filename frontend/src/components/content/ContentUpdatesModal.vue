<script setup lang="ts">
import {
  X,
  ArrowUpCircle,
  Loader,
} from 'lucide-vue-next'
import type { ModUpdateItemDto } from '../../types/content'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

defineProps<{
  updates: ModUpdateItemDto[]
  isApplying: boolean
  progress: number
  currentFile: string
}>()

const emit = defineEmits<{
  (e: 'apply'): void
}>()
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="kip-card p-0 w-full max-w-2xl border border-emerald-500/40 flex flex-col max-h-[85vh] shadow-[0_0_60px_rgba(16,185,129,0.25)]">
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
          <div class="flex items-center gap-3">
            <div class="p-2.5 rounded-xl bg-emerald-500/20 border border-emerald-500/30 text-emerald-400">
              <ArrowUpCircle class="w-6 h-6" />
            </div>
            <div>
              <h3 class="text-2xl font-black uppercase text-white tracking-wider">Modrinth Hash Updates</h3>
              <span class="text-xs font-mono text-emerald-400">{{ updates.length }} upgrades verified against API</span>
            </div>
          </div>
          <button @click="isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition">
            <X class="w-5 h-5" />
          </button>
        </header>

        <main class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 flex flex-col gap-3 min-h-0">
          <div
            v-for="item in updates"
            :key="item.filename"
            class="p-4 bg-black/50 border border-white/5 rounded-2xl flex items-center justify-between"
          >
            <div>
              <h4 class="font-black text-sm text-white">{{ item.name }}</h4>
              <p class="text-[10px] font-mono text-white/40">{{ item.filename }} &rarr; <span class="text-emerald-400 font-bold">{{ item.newFilename }}</span></p>
            </div>
            <span class="px-2.5 py-1 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-xs font-mono font-bold">
              {{ item.latestVersion }}
            </span>
          </div>
        </main>

        <div v-if="isApplying" class="px-6 py-3 bg-black/40 border-t border-white/5">
          <div class="flex justify-between text-xs font-mono text-white/70 mb-1">
            <span>Upgrading: {{ currentFile }}</span>
            <span>{{ Math.round(progress) }}%</span>
          </div>
          <div class="w-full h-1.5 bg-black/80 rounded-full overflow-hidden">
            <div class="h-full bg-emerald-400 transition-all duration-150" :style="{ width: progress + '%' }"></div>
          </div>
        </div>

        <footer class="p-6 border-t border-white/5 bg-black/60 shrink-0 flex justify-end gap-3">
          <button @click="isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">
            {{ t('Close') }}
          </button>
          <button
            @click="emit('apply')"
            :disabled="isApplying"
            class="kip-btn-primary px-8 py-2.5 bg-emerald-500 hover:bg-emerald-400 text-black font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(16,185,129,0.3)]"
          >
            <Loader v-if="isApplying" class="w-4 h-4 animate-spin" />
            <ArrowUpCircle v-else class="w-4 h-4 fill-current" />
            <span>{{ isApplying ? 'Applying All...' : 'Apply Verified Upgrades' }}</span>
          </button>
        </footer>
      </div>
    </div>
  </transition>
</template>