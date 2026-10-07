<script setup lang="ts">
import { ref } from 'vue'
import {
  X,
  Save,
  History,
  GitCommit,
  Clock,
  Loader,
  Bookmark,
} from 'lucide-vue-next'
import type { VcsCommitDto } from '../../types/worlds'

const isOpen = defineModel<boolean>({ required: true })

defineProps<{
  worldName: string
  commits: VcsCommitDto[]
  loading: boolean
}>()

const emit = defineEmits<{
  (e: 'commit', message: string): void
  (e: 'revert', commitId: string): void
}>()

const customMessage = ref('')

function handleCapture(): void {
  emit('commit', customMessage.value.trim())
  customMessage.value = ''
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[85vh] shadow-[0_0_60px_rgba(168,85,247,0.25)] relative overflow-hidden border-purple-500/30">
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
          <div class="flex items-center gap-3">
            <div class="p-2.5 rounded-xl bg-purple-500/10 border border-purple-500/20 text-purple-400">
              <History class="w-6 h-6" />
            </div>
            <div>
              <h3 class="text-2xl font-black text-white">VCS Time Machine</h3>
              <span class="text-xs font-mono text-purple-400">Timeline Snapshots: {{ worldName }}</span>
            </div>
          </div>
          <button @click="isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition">
            <X class="w-6 h-6" />
          </button>
        </header>

        <!-- Commit Form -->
        <div class="p-5 border-b border-white/5 bg-black/40 shrink-0 flex flex-col gap-3">
          <input
            v-model="customMessage"
            @keyup.enter="handleCapture"
            type="text"
            placeholder="Snapshot name (e.g. Before Wither Boss Fight, End City Raid)..."
            class="kip-input py-2.5 text-xs font-mono"
          >

          <button
            @click="handleCapture"
            :disabled="loading"
            class="kip-btn-primary w-full py-3 bg-purple-500 hover:bg-purple-400 text-white font-black uppercase text-xs tracking-wider shadow-[0_0_20px_rgba(168,85,247,0.4)] flex items-center justify-center gap-2"
          >
            <Loader v-if="loading" class="w-4 h-4 animate-spin" />
            <Save v-else class="w-4 h-4" />
            <span>Record Differential Snapshot</span>
          </button>
        </div>

        <!-- History Timeline -->
        <main class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#050505]/90 relative z-10 min-h-0">
          <div v-if="commits.length === 0" class="py-12 text-center text-white/30 font-mono text-xs">
            No snapshot commits recorded for this universe yet.
          </div>

          <div v-else class="relative border-l-2 border-purple-500/30 ml-4 space-y-5 pb-2">
            <div
              v-for="(commit, index) in commits"
              :key="commit.id"
              class="relative pl-6 group"
            >
              <div
                class="absolute -left-[9px] top-1.5 w-4 h-4 rounded-full border-4 border-[#050505] transition-colors"
                :class="index === 0 ? 'bg-purple-400 shadow-[0_0_12px_rgba(168,85,247,0.9)]' : 'bg-purple-500/40 group-hover:bg-purple-400'"
              ></div>

              <div class="bg-black/40 border border-white/5 group-hover:border-purple-500/40 rounded-2xl p-4 transition-all duration-300 flex justify-between items-center shadow-inner">
                <div>
                  <div class="flex items-center gap-2 mb-1">
                    <GitCommit class="w-4 h-4 text-purple-400 shrink-0" />
                    <span class="font-mono text-xs font-bold text-white uppercase">{{ commit.id }}</span>
                    <span
                      v-if="index === 0"
                      class="px-2 py-0.5 bg-emerald-500/20 text-emerald-400 text-[8px] font-black uppercase tracking-wider rounded border border-emerald-500/30"
                    >
                      Head
                    </span>
                  </div>

                  <p class="text-xs text-white/80 font-medium mb-1 flex items-center gap-1.5">
                    <Bookmark class="w-3 h-3 text-purple-400/60" />
                    <span>{{ commit.message }}</span>
                  </p>

                  <div class="text-[10px] text-white/40 flex items-center gap-1.5 font-mono">
                    <Clock class="w-3 h-3" />
                    <span>{{ new Date(commit.timestamp).toLocaleString() }}</span>
                  </div>
                </div>

                <button
                  @click="emit('revert', commit.id)"
                  class="kip-btn-ghost px-4 py-2 text-xs text-purple-400 hover:text-white border-purple-500/20 hover:border-purple-500/50 hover:bg-purple-500/20 flex items-center gap-1.5 font-mono uppercase font-bold"
                >
                  <History class="w-3.5 h-3.5" /> Revert
                </button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>