<script setup lang="ts">
import {
  X,
  FileText,
  Layers,
  Clock,
  Trash2,
} from 'lucide-vue-next'
import type { LocalModRecord } from '@/bridge'
import { t } from '../../composables/useI18n'

const isOpen = defineModel<boolean>({ required: true })

defineProps<{
  item: LocalModRecord | null
}>()

const emit = defineEmits<{
  (e: 'toggle', item: LocalModRecord): void
  (e: 'delete', item: LocalModRecord): void
}>()

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen && item"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="kip-card p-0 w-full max-w-2xl flex flex-col max-h-[85vh] overflow-hidden relative border border-white/10 shadow-[0_0_60px_rgba(0,0,0,0.8)]">
        <header class="flex justify-between items-start p-6 border-b border-white/5 bg-black/60 shrink-0">
          <div class="flex items-center gap-4">
            <img
              :src="item.icon || fallbackModIcon"
              class="w-16 h-16 rounded-2xl bg-black/60 p-1 object-cover border border-white/10 shadow-2xl"
            >
            <div>
              <h3 class="text-2xl font-black text-white leading-tight mb-1">{{ item.name }}</h3>
              <div class="flex items-center gap-3">
                <span class="text-xs font-mono text-indigo-400 bg-indigo-500/10 px-2.5 py-0.5 rounded border border-indigo-500/20 font-bold">
                  {{ item.version }}
                </span>
                <span class="text-xs text-white/50">Author: <span class="text-white font-bold">{{ item.author }}</span></span>
              </div>
            </div>
          </div>
          <button @click="isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition">
            <X class="w-5 h-5" />
          </button>
        </header>

        <main class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 flex flex-col gap-4 min-h-0">
          <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
            <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-1.5 flex items-center gap-2">
              <FileText class="w-3.5 h-3.5 text-indigo-400" /> Package Description
            </h4>
            <p class="text-xs text-white/80 leading-relaxed font-medium">
              {{ item.description || 'No descriptive payload provided inside manifest.' }}
            </p>
          </div>

          <div class="grid grid-cols-2 gap-4">
            <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
              <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-2 flex items-center gap-2">
                <Layers class="w-3.5 h-3.5 text-amber-400" /> Target Loaders
              </h4>
              <div class="flex gap-2 flex-wrap">
                <span
                  v-for="ldr in item.loaders"
                  :key="ldr"
                  class="px-2.5 py-1 rounded text-[10px] font-black uppercase tracking-wider border bg-white/5 text-white/70 border-white/10"
                >
                  {{ ldr }}
                </span>
              </div>
            </div>

            <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
              <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-2 flex items-center gap-2">
                <Clock class="w-3.5 h-3.5 text-cyan-400" /> File System Data
              </h4>
              <p class="font-mono text-xs text-white/70">{{ formatBytes(item.size_bytes) }} • {{ item.date_modified }}</p>
            </div>
          </div>
        </main>

        <footer class="p-6 border-t border-white/5 bg-black/60 shrink-0 flex justify-between items-center">
          <button
            @click="emit('toggle', item)"
            class="px-6 py-2.5 rounded-xl font-black text-xs uppercase tracking-wider transition border"
            :class="item.disabled ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30' : 'bg-amber-500/10 text-amber-400 border-amber-500/30'"
          >
            {{ item.disabled ? t('ENABLE') : t('Disable') }}
          </button>

          <div class="flex gap-3">
            <button @click="isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">
              {{ t('Close') }}
            </button>
            <button
              @click="emit('delete', item); isOpen = false"
              class="kip-btn-danger px-6 py-2.5 text-xs font-black uppercase tracking-wider"
            >
              <Trash2 class="w-4 h-4" /> {{ t('Delete') }}
            </button>
          </div>
        </footer>
      </div>
    </div>
  </transition>
</template>