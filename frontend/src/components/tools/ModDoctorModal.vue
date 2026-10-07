<script setup lang="ts">
import { ref, computed } from 'vue'
import {
  Stethoscope,
  X,
  RefreshCw,
  Zap,
  Loader,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  FileWarning,
  Sparkles,
} from 'lucide-vue-next'
import type { DoctorIssueDto, DoctorAnalysisReportDto } from '../../types/tools'

const isOpen = defineModel<boolean>({ required: true })

const props = defineProps<{
  report: DoctorAnalysisReportDto | null
  loading: boolean
  remediating: boolean
}>()

const emit = defineEmits<{
  (e: 're-audit', version: string, loader: string): void
  (e: 'auto-cure', issues: DoctorIssueDto[], version: string, loader: string): void
}>()

const targetVersion = ref('26.3')
const targetLoader = ref<'fabric' | 'forge' | 'neoforge' | 'quilt'>('fabric')
const activeFilter = ref<'ALL' | 'CRITICAL' | 'ERROR' | 'WARNING' | 'OPTIMIZATION'>('ALL')

const filteredIssues = computed(() => {
  if (!props.report) return []
  if (activeFilter.value === 'ALL') return props.report.issues
  return props.report.issues.filter((i) => i.type === activeFilter.value)
})

function getIssueBorderClass(type: string): string {
  if (type === 'CRITICAL') return 'border-rose-500/50'
  if (type === 'ERROR') return 'border-red-500/40'
  if (type === 'WARNING') return 'border-amber-500/40'
  return 'border-cyan-500/30'
}

function getIssueBadgeClass(type: string): string {
  if (type === 'CRITICAL') return 'bg-rose-500/20 text-rose-300 border-rose-500/30'
  if (type === 'ERROR') return 'bg-red-500/20 text-red-300 border-red-500/30'
  if (type === 'WARNING') return 'bg-amber-500/20 text-amber-300 border-amber-500/30'
  return 'bg-cyan-500/20 text-cyan-300 border-cyan-500/30'
}

function getActionColorClass(action: string): string {
  if (action === 'DELETE') return 'text-rose-400'
  if (action === 'DISABLE') return 'text-amber-400'
  if (action === 'UPGRADE') return 'text-purple-400'
  return 'text-emerald-400'
}

function handleReAudit(): void {
  emit('re-audit', targetVersion.value, targetLoader.value)
}

function handleAutoCure(): void {
  const actionable = (props.report?.issues || []).filter(
    (i) => i.selected && i.action !== 'NONE'
  )
  emit('auto-cure', actionable, targetVersion.value, targetLoader.value)
}
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-5xl shadow-[0_0_70px_rgba(236,72,153,0.25)] overflow-hidden border border-pink-500/30">
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
          <div class="flex items-center gap-4 relative z-10">
            <div class="p-3 bg-pink-500/20 border border-pink-500/40 rounded-2xl text-pink-400">
              <Stethoscope class="w-7 h-7" />
            </div>
            <div>
              <h3 class="text-2xl font-black uppercase text-white tracking-wider">
                Mod Doctor Autonomous
              </h3>
              <span class="text-xs font-mono text-pink-400">
                Target Platform: MC {{ targetVersion }} • {{ targetLoader.toUpperCase() }}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-3 relative z-10">
            <button
              @click="handleReAudit"
              :disabled="loading || remediating"
              class="kip-btn-ghost px-4 py-2 text-xs uppercase font-bold tracking-wider text-pink-300 border-pink-500/30 flex items-center gap-2 cursor-pointer"
            >
              <RefreshCw class="w-4 h-4" :class="loading ? 'animate-spin' : ''" />
              <span>Re-Audit</span>
            </button>
            <button @click="isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white transition cursor-pointer">
              <X class="w-6 h-6" />
            </button>
          </div>
        </header>

        <div class="p-6 bg-black/40 border-b border-white/5 flex items-center justify-between gap-4 shrink-0">
          <div class="flex items-center gap-4">
            <div>
              <label class="text-[9px] font-mono uppercase text-white/40 font-bold block mb-1">Minecraft Release</label>
              <select v-model="targetVersion" @change="handleReAudit" class="kip-input py-2 text-xs font-mono w-48">
                <option value="26.3">26.3 (Latest)</option>
                <option value="26.2">26.2</option>
                <option value="26.1">26.1</option>
                <option value="1.21.4">1.21.4 (LTS)</option>
                <option value="1.21.3">1.21.3</option>
                <option value="1.21.2">1.21.2</option>
                <option value="1.21.1">1.21.1</option>
                <option value="1.21">1.21</option>
                <option value="1.20.6">1.20.6</option>
                <option value="1.20.4">1.20.4</option>
                <option value="1.20.2">1.20.2</option>
                <option value="1.20.1">1.20.1 (LTS)</option>
                <option value="1.19.4">1.19.4</option>
                <option value="1.19.2">1.19.2</option>
                <option value="1.18.2">1.18.2</option>
                <option value="1.16.5">1.16.5 (Legacy)</option>
                <option value="1.12.2">1.12.2 (Classic)</option>
                <option value="1.7.10">1.7.10 (Retro)</option>
              </select>
            </div>

            <div>
              <label class="text-[9px] font-mono uppercase text-white/40 font-bold block mb-1">Target Loader</label>
              <div class="flex gap-1.5 p-1 bg-black/60 rounded-xl border border-white/10">
                <button
                  v-for="l in (['fabric', 'forge', 'neoforge', 'quilt'] as const)"
                  :key="l"
                  @click="targetLoader = l; handleReAudit()"
                  class="px-3 py-1.5 rounded-lg text-xs font-mono font-bold uppercase transition cursor-pointer"
                  :class="targetLoader === l ? 'bg-pink-500/20 text-pink-300 border border-pink-500/40' : 'text-white/40 hover:text-white'"
                >
                  {{ l }}
                </button>
              </div>
            </div>
          </div>

          <button
            @click="handleAutoCure"
            :disabled="remediating || loading || !report || report.is_clean"
            class="py-3.5 px-6 rounded-2xl bg-gradient-to-r from-pink-500 to-rose-500 hover:from-pink-400 text-white font-black text-xs uppercase tracking-widest shadow-[0_0_20px_rgba(244,63,94,0.4)] transition flex items-center gap-2 disabled:opacity-40 cursor-pointer"
          >
            <Loader v-if="remediating" class="w-4 h-4 animate-spin" />
            <Zap v-else class="w-4 h-4 fill-current" />
            <span>{{ remediating ? 'Applying Cure...' : '1-Click Auto-Cure' }}</span>
          </button>
        </div>

        <div v-if="report && !loading" class="grid grid-cols-4 gap-4 p-6 border-b border-white/5 bg-black/30 shrink-0">
          <div class="p-4 bg-black/60 border border-white/10 rounded-2xl flex flex-col justify-between">
            <span class="text-[9px] font-mono font-bold uppercase text-white/50">Health Index</span>
            <span class="text-3xl font-black text-pink-400 mt-1">{{ report.health_score }}%</span>
          </div>

          <div class="p-4 bg-black/60 border border-white/10 rounded-2xl flex flex-col justify-between">
            <span class="text-[9px] font-mono font-bold uppercase text-white/50">Audited Containers</span>
            <span class="text-3xl font-black text-white mt-1">{{ report.total_checked }}</span>
          </div>

          <div class="p-4 bg-black/60 border border-white/10 rounded-2xl flex flex-col justify-between">
            <span class="text-[9px] font-mono font-bold uppercase text-white/50">Bytecode Profiles</span>
            <div class="flex gap-2 items-center mt-2 flex-wrap">
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-white/5 text-white/70">J8: {{ report.bytecode_stats.java_8 }}</span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-indigo-500/10 text-indigo-400">J17: {{ report.bytecode_stats.java_17 }}</span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-purple-500/10 text-purple-400">J21: {{ report.bytecode_stats.java_21 }}</span>
              <span class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-pink-500/10 text-pink-400">J25: {{ report.bytecode_stats.java_25 }}</span>
            </div>
          </div>

          <div class="p-4 bg-black/60 border border-white/10 rounded-2xl flex flex-col justify-between">
            <span class="text-[9px] font-mono font-bold uppercase text-white/50">Conflict Points</span>
            <span class="text-3xl font-black" :class="report.issues.length > 0 ? 'text-pink-400' : 'text-emerald-400'">
              {{ report.issues.length }}
            </span>
          </div>
        </div>

        <main class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 min-h-0">
          <div v-if="loading" class="py-24 text-center flex flex-col items-center justify-center">
            <div class="w-16 h-16 border-4 border-pink-500/20 border-t-pink-500 rounded-full animate-spin mb-4"></div>
            <span class="text-xs font-mono font-bold uppercase text-pink-400 animate-pulse">Running bytecode inspection and dependency resolution...</span>
          </div>

          <div v-else-if="report?.is_clean" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-emerald-500/20 rounded-3xl bg-emerald-950/5">
            <CheckCircle class="w-16 h-16 text-emerald-400 mb-3" />
            <h4 class="text-xl font-black text-white uppercase">Instance 100% Healthy</h4>
            <p class="text-xs text-white/50 font-mono mt-1">Zero incompatibilities, missing companion modules or bytecode anomalies detected.</p>
          </div>

          <div v-else class="flex flex-col gap-4">
            <div
              v-for="issue in filteredIssues"
              :key="issue.id"
              class="p-5 bg-black/60 border rounded-2xl flex items-start gap-4 transition"
              :class="[getIssueBorderClass(issue.type), issue.selected ? 'bg-pink-500/5' : '']"
            >
              <div class="mt-1 p-2.5 rounded-xl border shrink-0" :class="getIssueBadgeClass(issue.type)">
                <AlertTriangle v-if="issue.type === 'CRITICAL'" class="w-5 h-5" />
                <AlertCircle v-else-if="issue.type === 'ERROR'" class="w-5 h-5" />
                <FileWarning v-else-if="issue.type === 'WARNING'" class="w-5 h-5" />
                <Sparkles v-else class="w-5 h-5" />
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2 mb-1.5 flex-wrap">
                  <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-black uppercase border" :class="getIssueBadgeClass(issue.type)">
                    {{ issue.type }}
                  </span>
                  <span class="px-2 py-0.5 rounded text-[8px] font-mono font-bold bg-white/5 text-white/40 uppercase">
                    {{ issue.category }}
                  </span>
                  <h4 class="font-black text-sm text-white">{{ issue.title }}</h4>
                </div>

                <p class="text-xs text-white/80 leading-relaxed font-medium mb-2">{{ issue.text }}</p>

                <div v-if="issue.action !== 'NONE'" class="inline-flex items-center gap-2 px-2.5 py-1 rounded-lg bg-black/40 border border-white/10 text-[10px] font-mono">
                  <span class="font-bold uppercase" :class="getActionColorClass(issue.action)">{{ issue.action }}:</span>
                  <span class="text-white/70">{{ issue.target_slug || issue.target_file }}</span>
                </div>
              </div>

              <div v-if="issue.action !== 'NONE'" class="relative inline-block w-10 shrink-0 mt-2">
                <input
                  type="checkbox"
                  class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none"
                  :checked="issue.selected"
                  @change="issue.selected = !issue.selected"
                >
                <label class="toggle-label block h-5 rounded-full border border-white/10"></label>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>