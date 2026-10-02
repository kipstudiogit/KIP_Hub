<template>
  <div class="h-full overflow-y-auto custom-scroll pr-2 pb-10">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-extrabold mb-1">{{ t('System Tools') }}</h2>
      <p class="text-white/50 text-sm">{{ t('Maintain, clean, and optimize your instance.') }}</p>
    </div>

    <div class="grid grid-cols-3 gap-6 mb-8 stagger-2">
      <div @click="runTool({ id: 'mod_doctor' })" class="kip-card kip-card-hover p-6 relative overflow-hidden border-pink-500/20 hover:border-pink-500/40 cursor-pointer flex flex-col justify-center group">
        <div class="absolute -right-10 -top-10 w-48 h-48 bg-pink-500/10 blur-3xl rounded-full pointer-events-none group-hover:bg-pink-500/20 transition-all duration-700"></div>
        <div class="flex items-center gap-4 relative z-10">
          <div class="p-4 bg-pink-500/10 rounded-2xl border border-pink-500/20 shadow-[0_0_15px_rgba(236,72,153,0.2)] group-hover:scale-110 transition-transform duration-500 shrink-0">
            <Stethoscope class="w-8 h-8 text-pink-400" />
          </div>
          <div>
            <h3 class="text-lg font-extrabold text-white mb-1 flex items-center gap-2 leading-tight">
              {{ t('Mod Doctor') }}
            </h3>
            <p class="text-white/50 text-[10px] uppercase tracking-wider font-bold">{{ t('AI Assist') }}</p>
          </div>
        </div>
      </div>

      <div @click="runTool({ id: 'shield_scan' })" class="kip-card kip-card-hover p-6 relative overflow-hidden border-rose-500/20 hover:border-rose-500/40 cursor-pointer flex flex-col justify-center group">
        <div class="absolute -left-10 -bottom-10 w-48 h-48 bg-rose-500/10 blur-3xl rounded-full pointer-events-none group-hover:bg-rose-500/20 transition-all duration-700"></div>
        <div class="flex items-center gap-4 relative z-10">
          <div class="p-4 bg-rose-500/10 rounded-2xl border border-rose-500/20 shadow-[0_0_15px_rgba(244,63,94,0.2)] group-hover:scale-110 transition-transform duration-500 shrink-0">
            <ShieldAlert class="w-8 h-8 text-rose-400" />
          </div>
          <div>
            <h3 class="text-lg font-extrabold text-white mb-1 flex items-center gap-2 leading-tight">
              K.I.P. Shield
            </h3>
            <p class="text-white/50 text-[10px] uppercase tracking-wider font-bold">Malware Scanner</p>
          </div>
        </div>
      </div>

      <div class="kip-card p-6 relative overflow-hidden flex flex-col justify-center group transition-colors duration-500" :class="state.settings.safe_mode ? 'border-emerald-500/40 bg-emerald-900/10' : 'kip-card-hover border-white/5'">
        <div class="absolute -right-10 -top-10 w-40 h-40 blur-2xl rounded-full pointer-events-none transition-colors duration-500" :class="state.settings.safe_mode ? 'bg-emerald-500/20' : 'bg-white/5'"></div>
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-3 rounded-xl transition duration-300 shrink-0 border" :class="state.settings.safe_mode ? 'bg-emerald-500/20 border-emerald-500/30' : 'bg-white/5 border-white/10'">
            <Shield class="w-6 h-6 transition duration-300" :class="state.settings.safe_mode ? 'text-emerald-400' : 'text-white/50'" />
          </div>
          <div>
            <h4 class="font-bold text-base leading-tight">{{ t('Safe Mode') }}</h4>
            <p class="text-[9px] text-white/50 uppercase tracking-widest">{{ t('Temporarily disable all mods') }}</p>
          </div>
        </div>
        <button @click="toggleSafeMode" class="w-full py-2.5 rounded-xl text-xs font-bold transition-all duration-300 relative z-10 shadow-lg" :class="state.settings.safe_mode ? 'bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)]' : 'bg-white/10 hover:bg-white/20 text-white'">
          {{ state.settings.safe_mode ? t('ENABLED') : t('ENABLE') }}
        </button>
      </div>
    </div>

    <div class="mb-8 stagger-3">
      <h3 class="text-sm font-bold text-white/50 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Zap class="w-4 h-4 text-amber-400" /> {{ t('Performance & Network') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div v-for="tool in perfTools" :key="tool.id" @click="runTool(tool)" class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5">
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="mb-4 stagger-3">
      <h3 class="text-sm font-bold text-white/50 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Trash2 class="w-4 h-4 text-red-400" /> {{ t('Maintenance & Cleanup') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div v-for="tool in cleanupTools" :key="tool.id" @click="runTool(tool)" class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5">
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="doctorModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="doctorModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[80vh] overflow-hidden relative shadow-[0_0_50px_rgba(236,72,153,0.15)] border-pink-500/20">
          <div class="absolute -top-32 -right-32 w-64 h-64 bg-pink-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="p-8 border-b border-white/5 bg-black/40 flex justify-between items-center shrink-0 relative z-10">
            <h3 class="text-2xl font-extrabold flex items-center gap-3 text-white">
              <div class="p-2 bg-pink-500/10 rounded-xl border border-pink-500/20"><Stethoscope class="w-6 h-6 text-pink-400" /></div>
              {{ t('Mod Doctor Analysis') }}
            </h3>
            <button @click="doctorModal.isOpen = false" class="kip-btn-ghost p-2 text-white/70 hover:text-red-400 transition">
              <X class="w-6 h-6" />
            </button>
          </div>

          <div v-if="doctorModal.loading" class="flex-1 flex flex-col items-center justify-center text-pink-500 py-16 relative z-10">
            <Loader class="w-12 h-12 animate-spin mb-4" />
            <span class="font-bold tracking-wider animate-pulse">{{ t('Applying fixes and downloading missing libraries...') }}</span>
          </div>

          <template v-else>
            <div v-if="doctorModal.isClean" class="flex-1 flex flex-col items-center justify-center text-emerald-400 py-16 relative z-10">
              <div class="p-4 bg-emerald-500/10 rounded-full mb-5 shadow-[0_0_30px_rgba(16,185,129,0.3)] border border-emerald-500/20">
                <CheckCircle class="w-16 h-16" />
              </div>
              <h4 class="text-3xl font-extrabold">{{ t('No issues found!') }}</h4>
              <p class="text-emerald-400/60 text-sm mt-3 font-medium">{{ t('Your modpack is healthy and ready to launch.') }}</p>
            </div>

            <div v-else class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#050505]/80 flex flex-col gap-3 min-h-0 relative z-10">
              <div v-for="issue in doctorModal.issues" :key="issue.id" @click="issue.action !== 'NONE' && (issue.selected = !issue.selected)" class="p-4 bg-black/40 border rounded-2xl flex items-start gap-4 transition" :class="[issue.type === 'CRITICAL' ? 'border-red-500/50 hover:bg-red-500/10' : issue.type === 'WARNING' ? 'border-amber-500/50 hover:bg-amber-500/10' : 'border-pink-500/30 hover:bg-pink-500/10', issue.action !== 'NONE' ? 'cursor-pointer hover:shadow-lg' : '']">
                <div class="mt-1 shrink-0 p-2 rounded-xl border" :class="issue.type === 'CRITICAL' ? 'bg-red-500/10 border-red-500/20' : issue.type === 'WARNING' ? 'bg-amber-500/10 border-amber-500/20' : 'bg-pink-500/10 border-pink-500/20'">
                  <AlertTriangle v-if="issue.type === 'CRITICAL'" class="w-5 h-5 text-red-400" />
                  <AlertCircle v-else-if="issue.type === 'WARNING'" class="w-5 h-5 text-amber-400" />
                  <DownloadCloud v-else-if="issue.action === 'DOWNLOAD'" class="w-5 h-5 text-blue-400" />
                  <Trash2 v-else-if="issue.action === 'DELETE'" class="w-5 h-5 text-red-400" />
                  <Info v-else class="w-5 h-5 text-indigo-400" />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-2">
                    <span class="text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded" :class="issue.type === 'CRITICAL' ? 'bg-red-500/20 text-red-400 border border-red-500/30' : issue.type === 'WARNING' ? 'bg-amber-500/20 text-amber-400 border border-amber-500/30' : 'bg-pink-500/20 text-pink-400 border border-pink-500/30'">{{ issue.type }}</span>
                    <span v-if="issue.action !== 'NONE'" class="text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded bg-white/10 text-white/70 border border-white/10">{{ issue.action }}: {{ issue.target }}</span>
                  </div>
                  <p class="text-sm text-white/80 leading-relaxed font-medium">{{ issue.text }}</p>
                </div>
                <div v-if="issue.action !== 'NONE'" class="relative inline-block w-10 shrink-0 mt-2">
                  <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none transition-transform" :checked="issue.selected" style="pointer-events: none;">
                  <label class="toggle-label block h-5 rounded-full transition-colors border border-white/10" style="pointer-events: none;"></label>
                </div>
              </div>
            </div>

            <div v-if="!doctorModal.isClean" class="p-6 border-t border-white/5 bg-black/40 shrink-0 flex justify-between items-center relative z-10">
              <span class="text-[10px] font-bold text-white/50 uppercase tracking-widest">{{ doctorModal.issues.filter(i => i.selected).length }} {{ t('actions selected') }}</span>
              <div class="flex gap-3">
                <button @click="doctorModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5">{{ t('Cancel') }}</button>
                <button @click="applyDoctorFixes" :disabled="doctorModal.issues.filter(i => i.selected).length === 0" class="kip-btn-primary px-8 py-2.5 bg-pink-500 hover:bg-pink-400 text-white border-pink-400 shadow-[0_0_15px_rgba(236,72,153,0.3)]">
                  {{ t('Apply Fixes') }}
                </button>
              </div>
            </div>
          </template>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="shieldModal.isOpen" class="fixed inset-0 bg-black/80 backdrop-blur-xl z-[200] flex items-center justify-center p-10 cursor-default" @click.self="shieldModal.isOpen = false">
        <div class="kip-card p-0 w-full max-w-2xl border flex flex-col max-h-[80vh] overflow-hidden relative shadow-[0_0_50px_rgba(244,63,94,0.15)] border-rose-500/30">
          <div class="absolute top-0 right-0 w-80 h-80 bg-rose-500/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div class="p-8 border-b border-white/5 bg-black/40 flex justify-between items-center shrink-0 relative z-10">
            <h3 class="text-2xl font-extrabold flex items-center gap-3 text-white">
              <div class="p-2 bg-rose-500/10 rounded-xl border border-rose-500/20"><ShieldAlert class="w-6 h-6 text-rose-400" /></div>
              K.I.P. Shield
            </h3>
            <button @click="shieldModal.isOpen = false" class="kip-btn-ghost p-2 text-white/70 hover:text-red-400 transition">
              <X class="w-6 h-6" />
            </button>
          </div>

          <div v-if="shieldModal.loading" class="flex-1 flex flex-col items-center justify-center text-rose-400 py-16 relative z-10">
            <ScanSearch class="w-16 h-16 animate-pulse mb-6 drop-shadow-[0_0_15px_rgba(244,63,94,0.8)]" />
            <span class="font-extrabold tracking-widest uppercase text-lg">{{ t('Decompiling & Scanning...') }}</span>
            <span class="text-rose-400/50 text-xs font-mono mt-2">Checking constant pools and signatures</span>
          </div>

          <template v-else>
            <div v-if="shieldModal.isClean" class="flex-1 flex flex-col items-center justify-center text-emerald-400 py-16 relative z-10">
              <div class="p-5 bg-emerald-500/10 rounded-full mb-5 shadow-[0_0_30px_rgba(16,185,129,0.3)] border border-emerald-500/20">
                <ShieldCheck class="w-16 h-16" />
              </div>
              <h4 class="text-3xl font-extrabold tracking-wide uppercase">{{ t('System Secure') }}</h4>
              <p class="text-emerald-400/60 text-sm mt-3 font-medium">{{ t('No malware or suspicious signatures detected.') }}</p>
            </div>

            <div v-else class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#050505]/80 flex flex-col gap-4 min-h-0 relative z-10">
              <div class="bg-red-500/10 border border-red-500/30 p-5 rounded-2xl flex items-start gap-4 mb-2 shadow-inner">
                <BugOff class="w-8 h-8 text-red-400 shrink-0 mt-1" />
                <div>
                  <h4 class="font-extrabold text-red-400 text-lg mb-1">{{ t('Threats Detected!') }}</h4>
                  <p class="text-xs text-red-400/80 leading-relaxed">{{ t('K.I.P. Shield found malicious signatures in the following files. It is highly recommended to quarantine them immediately.') }}</p>
                </div>
              </div>

              <div v-for="threat in shieldModal.threats" :key="threat.file" class="p-5 bg-black/60 border border-rose-500/30 rounded-2xl flex flex-col gap-4 shadow-lg hover:border-rose-500/50 transition-colors">
                <div class="flex justify-between items-start">
                  <div class="flex-1 min-w-0">
                    <h4 class="font-bold text-rose-400 truncate text-lg mb-1">{{ threat.file }}</h4>
                    <p class="text-[10px] font-mono text-white/40 truncate bg-white/5 inline-block px-2 py-0.5 rounded border border-white/5">SHA256: {{ threat.hash }}</p>
                  </div>
                  <button @click="deleteThreat(threat.file)" class="kip-btn-danger px-4 py-2 text-xs ml-4 border-rose-400">
                    <Trash2 class="w-4 h-4" /> Quarantine
                  </button>
                </div>

                <div class="bg-rose-900/10 rounded-xl p-4 border border-rose-500/10 shadow-inner">
                  <ul class="list-disc pl-4 space-y-2">
                    <li v-for="(t_desc, idx) in threat.threats" :key="idx" class="text-xs text-rose-300/90 font-medium">{{ t_desc }}</li>
                  </ul>
                </div>
              </div>
            </div>

            <div v-if="!shieldModal.isClean" class="p-6 border-t border-white/5 bg-black/40 shrink-0 flex justify-end relative z-10">
              <button @click="shieldModal.isOpen = false" class="kip-btn-ghost px-8 py-2.5">{{ t('Close') }}</button>
            </div>
          </template>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import type { Component } from 'vue'
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
  X,
  Loader,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  DownloadCloud,
  Info,
  ShieldAlert,
  ScanSearch,
  BugOff,
  ShieldCheck,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type ToolExecutionResult,
  type GenericActionResult,
} from '@/bridge'

export interface DoctorIssue {
  id: string
  type: 'CRITICAL' | 'WARNING' | 'INFO'
  text: string
  action: 'DOWNLOAD' | 'DELETE' | 'NONE'
  target: string
  selected?: boolean
}

export interface DoctorAnalysisResult {
  is_clean: boolean
  issues: DoctorIssue[]
}

export interface ShieldThreat {
  file: string
  hash: string
  threats: string[]
  safe?: boolean
  error?: string | null
}

export interface ToolDefinition {
  id: string
  name: string
  desc: string
  bgClass: string
  textClass: string
  shadowClass: string
  icon: Component
}

const doctorModal = ref<{
  isOpen: boolean
  loading: boolean
  isClean: boolean
  issues: DoctorIssue[]
}>({
  isOpen: false,
  loading: false,
  isClean: false,
  issues: [],
})

const shieldModal = ref<{
  isOpen: boolean
  loading: boolean
  isClean: boolean
  threats: ShieldThreat[]
}>({
  isOpen: false,
  loading: false,
  isClean: false,
  threats: [],
})

const perfTools: ToolDefinition[] = [
  { id: 'generate_jvm', name: 'RAM Optimizer', desc: 'Copy JVM arguments', bgClass: 'bg-purple-500/10', textClass: 'text-purple-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(168,85,247,0.4)]', icon: Cpu },
  { id: 'ai_fps', name: 'AI FPS Optimizer', desc: 'Auto-tune graphics', bgClass: 'bg-emerald-500/10', textClass: 'text-emerald-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(16,185,129,0.4)]', icon: Zap },
  { id: 'flush_dns', name: 'Flush DNS', desc: 'Fix network connection', bgClass: 'bg-cyan-500/10', textClass: 'text-cyan-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(6,182,212,0.4)]', icon: WifiOff },
  { id: 'mclogs', name: 'Cloud Logs', desc: 'Upload to mclo.gs', bgClass: 'bg-blue-500/10', textClass: 'text-blue-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(59,130,246,0.4)]', icon: UploadCloud },
]

const cleanupTools: ToolDefinition[] = [
  { id: 'clean_logs', name: 'Clean Logs', desc: 'Delete old gz/log files', bgClass: 'bg-red-500/10', textClass: 'text-red-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(239,68,68,0.4)]', icon: Trash2 },
  { id: 'kill_java', name: 'Kill Zombies', desc: 'Force stop Java processes', bgClass: 'bg-amber-500/10', textClass: 'text-amber-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(245,158,11,0.4)]', icon: Skull },
  { id: 'backup', name: 'Create Backup', desc: 'Zip all world saves', bgClass: 'bg-emerald-500/10', textClass: 'text-emerald-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(16,185,129,0.4)]', icon: Archive },
  { id: 'clean_worlds', name: 'Clean Worlds', desc: 'Remove minimap caches', bgClass: 'bg-indigo-500/10', textClass: 'text-indigo-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(99,102,241,0.4)]', icon: MapIcon },
  { id: 'wipe_configs', name: 'Wipe Configs', desc: 'Delete config folder', bgClass: 'bg-rose-500/10', textClass: 'text-rose-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(244,63,94,0.4)]', icon: FileWarning },
  { id: 'reset_video', name: 'Reset Video', desc: 'Delete options.txt', bgClass: 'bg-slate-500/10', textClass: 'text-slate-400', shadowClass: 'group-hover:shadow-[0_0_15px_rgba(148,163,184,0.4)]', icon: MonitorX },
]

const runTool = async (tool: { id: string }): Promise<void> => {
  if (tool.id === 'mod_doctor') {
    doctorModal.value.loading = true
    doctorModal.value.isOpen = true
    try {
      const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: tool.id })
      if (res && res.success && res.doctor_res) {
        const doc = res.doctor_res as unknown as DoctorAnalysisResult
        doctorModal.value.isClean = doc.is_clean || false
        doctorModal.value.issues = (doc.issues || []).map((i) => ({
          ...i,
          selected: i.action !== 'NONE',
        }))
      } else {
        showToast(t('Error'), res?.msg || t('Analysis failed.'), 'danger')
        doctorModal.value.isOpen = false
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
      doctorModal.value.isOpen = false
    } finally {
      doctorModal.value.loading = false
    }
    return
  }

  if (tool.id === 'shield_scan') {
    shieldModal.value.loading = true
    shieldModal.value.isOpen = true
    try {
      const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: tool.id })
      if (res && res.success) {
        if (res.threats && res.threats.length > 0) {
          shieldModal.value.isClean = false
          shieldModal.value.threats = res.threats as unknown as ShieldThreat[]
        } else {
          shieldModal.value.isClean = true
          shieldModal.value.threats = []
        }
      } else {
        showToast(t('Error'), res?.msg || t('Scan failed.'), 'danger')
        shieldModal.value.isOpen = false
      }
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : String(err)
      showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
      shieldModal.value.isOpen = false
    } finally {
      shieldModal.value.loading = false
    }
    return
  }

  try {
    const res = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: tool.id })
    if (res && res.success) {
      showToast(t('Success'), res.msg, 'success')
      if (res.clipboard) {
        navigator.clipboard.writeText(res.clipboard).catch(() => {})
      }
    } else {
      showToast(t('Error'), res?.msg || t('Action failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  }
}

const toggleSafeMode = async (): Promise<void> => {
  try {
    const res = await invokeSafe<{ success: boolean; state?: boolean; msg?: string }>('toggle_safe_mode')
    if (res && res.success && typeof res.state === 'boolean') {
      state.settings.safe_mode = res.state
      showToast(t('Safe Mode'), res.state ? t('Enabled') : t('Disabled'), 'success')
    } else {
      showToast(t('Error'), res?.msg || t('Failed to toggle safe mode.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  }
}

const applyDoctorFixes = async (): Promise<void> => {
  doctorModal.value.loading = true
  const selectedIssues = doctorModal.value.issues.filter((i) => i.selected)

  try {
    const res = await invokeSafe<{
      success: boolean
      deleted?: number
      downloaded?: number
      errors?: string[]
    }>('apply_doctor_fixes', {
      issues: selectedIssues,
    })

    if (res && (res.success || (res.deleted ?? 0) > 0 || (res.downloaded ?? 0) > 0)) {
      showToast(
        t('Fixed'),
        `${t('Deleted')} ${res.deleted || 0}, ${t('Downloaded')} ${res.downloaded || 0}.`,
        'success'
      )
      if (res.errors && res.errors.length > 0) {
        res.errors.forEach((e) => showToast(t('Warning'), e, 'danger'))
      }
      doctorModal.value.isOpen = false
    } else {
      showToast(t('Error'), res?.errors?.[0] || t('Failed to apply fixes.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  } finally {
    doctorModal.value.loading = false
  }
}

const deleteThreat = async (filename: string): Promise<void> => {
  try {
    const res: GenericActionResult = await bridge.deleteMod(filename)
    if (res && res.success) {
      showToast(t('Quarantined'), t('Threat removed successfully.'), 'success')
      shieldModal.value.threats = shieldModal.value.threats.filter((t) => t.file !== filename)
      if (shieldModal.value.threats.length === 0) {
        shieldModal.value.isClean = true
      }
    } else {
      showToast(t('Error'), res?.msg || t('Failed to delete threat file.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg || t('Backend communication failed.'), 'danger')
  }
}
</script>