<script setup lang="ts">
import { onMounted } from 'vue'
import {
  Trash2,
  Skull,
  Archive,
  Map as MapIcon,
  WifiOff,
  Cpu,
  Stethoscope,
  FileWarning,
  UploadCloud,
  MonitorX,
  Zap,
  Shield,
  ShieldAlert,
  ArrowRight,
  Flame,
  Bot,
} from 'lucide-vue-next'
import { state, t } from '@/store'
import type { ToolItem } from '../types/tools'
import { useToolsManager } from '../composables/useToolsManager'
import ModDoctorModal from '../components/tools/ModDoctorModal.vue'
import ShieldModal from '../components/tools/ShieldModal.vue'

const {
  isSafeModeActive,
  crashReport,
  isDoctorModalOpen,
  isDoctorAnalyzing,
  isDoctorRemediating,
  doctorReport,
  isShieldModalOpen,
  isShieldScanning,
  shieldReport,
  shieldProgress,
  vaultRecords,
  executeTool,
  checkLatestCrash,
  fetchSafeModeStatus,
  toggleSafeMode,
  analyzeDoctor,
  remediateDoctorIssues,
  scanShieldStream,
  quarantineThreat,
  loadVaultRecords,
  restoreVaultItem,
  shredVaultItem,
} = useToolsManager()

const perfTools: ToolItem[] = [
  { id: 'generate_jvm', name: 'RAM Optimizer', desc: 'Copy JVM arguments', bgClass: 'bg-purple-500/10', textClass: 'text-purple-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(168,85,247,0.4)]', icon: Cpu },
  { id: 'ai_fps', name: 'AI FPS Optimizer', desc: 'Auto-tune graphics', bgClass: 'bg-emerald-500/10', textClass: 'text-emerald-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(16,185,129,0.4)]', icon: Zap },
  { id: 'flush_dns', name: 'Flush DNS', desc: 'Fix network connection', bgClass: 'bg-cyan-500/10', textClass: 'text-cyan-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(6,182,212,0.4)]', icon: WifiOff },
  { id: 'mclogs', name: 'Cloud Logs', desc: 'Upload to mclo.gs', bgClass: 'bg-blue-500/10', textClass: 'text-blue-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(59,130,246,0.4)]', icon: UploadCloud },
]

const cleanupTools: ToolItem[] = [
  { id: 'clean_logs', name: 'Clean Logs', desc: 'Delete old gz/log files', bgClass: 'bg-red-500/10', textClass: 'text-red-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(239,68,68,0.4)]', icon: Trash2 },
  { id: 'kill_java', name: 'Kill Zombies', desc: 'Force stop Java processes', bgClass: 'bg-amber-500/10', textClass: 'text-amber-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(245,158,11,0.4)]', icon: Skull },
  { id: 'backup', name: 'Create Backup', desc: 'Zip all world saves', bgClass: 'bg-emerald-500/10', textClass: 'text-emerald-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(16,185,129,0.4)]', icon: Archive },
  { id: 'clean_worlds', name: 'Clean Worlds', desc: 'Remove minimap caches', bgClass: 'bg-indigo-500/10', textClass: 'text-indigo-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(99,102,241,0.4)]', icon: MapIcon },
  { id: 'wipe_configs', name: 'Wipe Configs', desc: 'Delete config folder', bgClass: 'bg-rose-500/10', textClass: 'text-rose-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(244,63,94,0.4)]', icon: FileWarning },
  { id: 'reset_video', name: 'Reset Video', desc: 'Delete options.txt', bgClass: 'bg-slate-500/10', textClass: 'text-slate-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(148,163,184,0.4)]', icon: MonitorX },
]

function openDoctor(): void {
  isDoctorModalOpen.value = true
  analyzeDoctor('26.3', 'fabric')
}

function openShield(): void {
  isShieldModalOpen.value = true
  scanShieldStream()
  loadVaultRecords()
}

function sendCrashToAi(snippet: string): void {
  state.aiInputText = snippet
  state.currentView = 'support'
}

onMounted(() => {
  fetchSafeModeStatus()
  checkLatestCrash()
})
</script>

<template>
  <div class="h-full overflow-y-auto custom-scroll pr-2 pb-10 select-none relative">
    <header class="mb-8 shrink-0">
      <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
        {{ t('System Tools') }}
      </h2>
      <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Automated Heuristic Engine, Forensic Diagnostics & Instance Maintenance</p>
    </header>

    <!-- Forensic Crash Investigator Banner -->
    <div
      v-if="crashReport && crashReport.hasCrash"
      class="mb-8 kip-card p-6 border border-rose-500/40 bg-gradient-to-r from-rose-950/30 to-black/60 flex items-center justify-between shadow-2xl relative overflow-hidden"
    >
      <div class="flex items-center gap-4 relative z-10">
        <div class="p-3 bg-rose-500/20 border border-rose-500/40 rounded-2xl text-rose-400">
          <Flame class="w-6 h-6 animate-pulse" />
        </div>
        <div>
          <span class="text-[9px] font-mono text-rose-400 uppercase font-black tracking-widest block">Forensic Crash Detector</span>
          <h3 class="text-lg font-black text-white mt-0.5">Crash Report: {{ crashReport.filename }}</h3>
          <p class="text-xs text-white/60 font-mono mt-0.5">
            {{ crashReport.culpritMod ? crashReport.culpritMod : 'Heap or Mixin collision detected' }} • {{ crashReport.timestamp }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3 relative z-10">
        <button
          @click="sendCrashToAi(crashReport.rawSnippet)"
          class="kip-btn-primary px-5 py-2.5 text-xs font-black uppercase tracking-wider bg-rose-500 hover:bg-rose-400 text-white flex items-center gap-2 shadow-[0_0_20px_rgba(244,63,94,0.4)]"
        >
          <Bot class="w-4 h-4" />
          <span>Consult AI Support</span>
        </button>
      </div>
    </div>

    <!-- Diagnostic Featured Cards -->
    <section class="grid grid-cols-3 gap-6 mb-8">
      <!-- Mod Doctor Card -->
      <div
        @click="openDoctor"
        class="kip-card kip-card-hover p-6 relative overflow-hidden border-pink-500/30 hover:border-pink-500/60 cursor-pointer flex flex-col justify-between group shadow-[0_0_40px_rgba(236,72,153,0.15)] bg-pink-950/10"
      >
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-4 bg-pink-500/20 rounded-2xl border border-pink-500/40 text-pink-400 shrink-0">
            <Stethoscope class="w-8 h-8 animate-pulse" />
          </div>
          <div>
            <span class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider bg-pink-500/20 text-pink-300 border border-pink-500/30 block w-fit mb-1">
              Autonomous Cure
            </span>
            <h3 class="text-xl font-black text-white leading-tight">Mod Doctor</h3>
            <p class="text-white/50 text-[10px] font-mono mt-0.5 uppercase tracking-wider">Topology & Bytecode Audit</p>
          </div>
        </div>

        <div class="pt-4 border-t border-pink-500/20 flex items-center justify-between text-xs font-mono text-pink-300/80 relative z-10">
          <span>{{ isDoctorAnalyzing ? 'Analyzing...' : 'Launch Diagnostics' }}</span>
          <ArrowRight class="w-4 h-4 group-hover:translate-x-1 transition-transform" />
        </div>
      </div>

      <!-- K.I.P. Shield Card -->
      <div
        @click="openShield"
        class="kip-card kip-card-hover p-6 relative overflow-hidden border-rose-500/30 hover:border-rose-500/60 cursor-pointer flex flex-col justify-between group shadow-[0_0_40px_rgba(244,63,94,0.15)] bg-rose-950/10"
      >
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-4 bg-rose-500/20 rounded-2xl border border-rose-500/40 text-rose-400 shrink-0">
            <ShieldAlert class="w-8 h-8" />
          </div>
          <div>
            <span class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider bg-rose-500/20 text-rose-300 border border-rose-500/30 block w-fit mb-1">
              Security Matrix
            </span>
            <h3 class="text-xl font-black text-white leading-tight">K.I.P. Shield</h3>
            <p class="text-white/50 text-[10px] font-mono mt-0.5 uppercase tracking-wider">Threat Center & Vault</p>
          </div>
        </div>

        <div class="pt-4 border-t border-rose-500/20 flex items-center justify-between text-xs font-mono text-rose-300/80 relative z-10">
          <span>{{ isShieldScanning ? 'Scanning...' : 'Security Center' }}</span>
          <ArrowRight class="w-4 h-4 group-hover:translate-x-1 transition-transform" />
        </div>
      </div>

      <!-- Safe Mode Card -->
      <div
        class="kip-card p-6 relative overflow-hidden flex flex-col justify-between border"
        :class="isSafeModeActive ? 'border-emerald-500/40 bg-emerald-900/10' : 'border-white/5 bg-black/40'"
      >
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div
            class="p-3.5 rounded-xl border shrink-0"
            :class="isSafeModeActive ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-400' : 'bg-white/5 border-white/10 text-white/50'"
          >
            <Shield class="w-6 h-6" />
          </div>
          <div>
            <h4 class="font-black text-base text-white leading-tight">{{ t('Safe Mode') }}</h4>
            <p class="text-[9px] text-white/50 uppercase tracking-widest font-mono">{{ t('Temporarily isolate all mods') }}</p>
          </div>
        </div>

        <button
          @click="toggleSafeMode"
          class="w-full py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition relative z-10"
          :class="isSafeModeActive ? 'bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)]' : 'bg-white/10 hover:bg-white/20 text-white'"
        >
          {{ isSafeModeActive ? t('ENABLED') : t('ENABLE') }}
        </button>
      </div>
    </section>

    <!-- Performance & Network Tools -->
    <section class="mb-8">
      <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Zap class="w-3.5 h-3.5 text-amber-400" /> {{ t('Performance & Network') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div
          v-for="tool in perfTools"
          :key="tool.id"
          @click="executeTool(tool.id)"
          class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5 bg-black/40"
        >
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm text-white">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider font-mono">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Maintenance & Cleanup Tools -->
    <section class="mb-4">
      <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Trash2 class="w-3.5 h-3.5 text-red-400" /> {{ t('Maintenance & Cleanup') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div
          v-for="tool in cleanupTools"
          :key="tool.id"
          @click="executeTool(tool.id)"
          class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5 bg-black/40"
        >
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm text-white">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider font-mono">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </section>

    <!-- Mod Doctor Modal Component -->
    <ModDoctorModal
      v-model="isDoctorModalOpen"
      :report="doctorReport"
      :loading="isDoctorAnalyzing"
      :remediating="isDoctorRemediating"
      @re-audit="analyzeDoctor"
      @auto-cure="remediateDoctorIssues"
    />

    <!-- K.I.P. Shield Modal Component -->
    <ShieldModal
      v-model="isShieldModalOpen"
      :report="shieldReport"
      :progress="shieldProgress"
      :vault-records="vaultRecords"
      :scanning="isShieldScanning"
      @scan="scanShieldStream"
      @quarantine="quarantineThreat"
      @restore="restoreVaultItem"
      @shred="shredVaultItem"
    />
  </div>
</template>