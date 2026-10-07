<script setup lang="ts">
import { onMounted } from 'vue'
import {
  Sparkles,
  Send,
  Loader,
  Bug,
  BrainCircuit,
  FileTerminal,
  Cpu,
  AlertCircle,
  Bot,
  Copy,
  ShieldCheck,
  Lock,
  RefreshCw,
  Flame,
  CheckCircle2,
} from 'lucide-vue-next'
import { t } from '@/store'
import { useSupportManager } from '../composables/useSupportManager'

const {
  telemetry,
  isFetchingTelemetry,
  isAiDiagnosing,
  isSubmittingBug,
  isLoadingLog,
  rawLogInput,
  aiResult,
  formattedAnalysisHtml,
  bugReportText,
  includeTelemetryInReport,
  lastSubmittedReportId,
  loadTelemetry,
  loadLatestLogSnippet,
  runAiDiagnosis,
  submitBugReport,
  copyTelemetryDossier,
} = useSupportManager()

onMounted(() => {
  loadTelemetry()
})
</script>

<template>
  <div class="h-full flex flex-col min-h-0 select-none relative pb-12 overflow-y-auto custom-scroll pr-2">
    <!-- Header -->
    <header class="mb-6 shrink-0">
      <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
        {{ t('Support') }}
      </h2>
      <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Automated Neural Diagnosis, Crash Diagnostics & Remote Telemetry</p>
    </header>

    <!-- Master Workspace -->
    <div class="grid grid-cols-12 gap-6 flex-1 min-h-0">
      <!-- Left Column: Neural Diagnostic Oracle -->
      <section class="col-span-7 flex flex-col gap-5">
        <div class="kip-card p-6 border border-purple-500/20 bg-black/60 shadow-2xl relative overflow-hidden flex flex-col justify-between">
          <div class="absolute -top-32 -right-32 w-80 h-80 bg-purple-600/10 blur-[100px] rounded-full pointer-events-none"></div>

          <div>
            <div class="flex justify-between items-center mb-4 relative z-10">
              <div class="flex items-center gap-3">
                <div class="p-2.5 bg-purple-500/10 rounded-2xl border border-purple-500/20 text-purple-400">
                  <BrainCircuit class="w-6 h-6" />
                </div>
                <div>
                  <h3 class="text-xl font-black uppercase text-white tracking-wider">Neural Crash Oracle</h3>
                  <span class="text-[9px] font-mono text-purple-300">Heuristic Extraction + LLM Multi-Provider Synthesis</span>
                </div>
              </div>

              <div class="flex items-center gap-2">
                <button
                  @click="loadLatestLogSnippet"
                  :disabled="isLoadingLog"
                  class="kip-btn-ghost px-3.5 py-1.5 text-xs font-mono font-bold uppercase text-purple-300 border-purple-500/30 hover:bg-purple-500/10 flex items-center gap-1.5"
                >
                  <Loader v-if="isLoadingLog" class="w-3.5 h-3.5 animate-spin" />
                  <FileTerminal v-else class="w-3.5 h-3.5" />
                  <span>Ingest Latest Log</span>
                </button>
              </div>
            </div>

            <textarea
              v-model="rawLogInput"
              rows="7"
              placeholder="Paste crash report stacktrace or latest.log error lines here..."
              class="kip-input text-xs font-mono resize-none leading-relaxed p-4 h-48 mb-3 shadow-inner focus:border-purple-500 select-text"
            ></textarea>

            <div class="flex justify-between items-center text-[10px] font-mono text-white/40 mb-4 px-1">
              <span class="flex items-center gap-1 text-emerald-400">
                <ShieldCheck class="w-3.5 h-3.5" />
                <span>Personal data scrubbed automatically (&lt;USER&gt;, &lt;IPV4&gt;, tokens)</span>
              </span>
              <span>{{ rawLogInput.length }} bytes</span>
            </div>
          </div>

          <button
            @click="runAiDiagnosis"
            :disabled="isAiDiagnosing || !rawLogInput.trim()"
            class="kip-btn-primary py-3.5 text-xs font-black uppercase tracking-widest bg-purple-500 hover:bg-purple-400 text-white shadow-[0_0_20px_rgba(168,85,247,0.4)] flex items-center justify-center gap-2"
          >
            <Loader v-if="isAiDiagnosing" class="w-4 h-4 animate-spin" />
            <Sparkles v-else class="w-4 h-4" />
            <span>{{ isAiDiagnosing ? 'Consulting Neural Oracle...' : 'Analyze Crash Stacktrace' }}</span>
          </button>
        </div>

        <!-- Diagnostic Output Terminal -->
        <div class="kip-card p-6 border border-white/10 bg-black/80 flex-1 flex flex-col shadow-2xl relative min-h-[300px]">
          <!-- Culprit Mod Badge (If found) -->
          <div
            v-if="aiResult?.culpritMod"
            class="mb-4 p-3 bg-rose-500/15 border border-rose-500/40 rounded-xl flex items-center justify-between"
          >
            <div class="flex items-center gap-2">
              <Flame class="w-4 h-4 text-rose-400 animate-pulse" />
              <span class="text-xs font-mono text-white">
                Suspected Culprit Module: <strong class="text-rose-400">{{ aiResult.culpritMod }}</strong>
              </span>
            </div>
            <span class="px-2 py-0.5 rounded text-[8px] font-mono uppercase bg-rose-500/20 text-rose-300 border border-rose-500/40 font-bold">
              Isolated
            </span>
          </div>

          <!-- Local Hints Pills -->
          <div v-if="aiResult?.localHints && aiResult.localHints.length > 0" class="mb-4 space-y-1">
            <span class="text-[9px] font-mono font-bold uppercase text-white/40 block">Offline Diagnostic Rules:</span>
            <div v-for="(hint, hIdx) in aiResult.localHints" :key="hIdx" class="text-[11px] font-mono text-amber-300/80 bg-amber-500/10 px-2.5 py-1 rounded-lg border border-amber-500/20">
              • {{ hint }}
            </div>
          </div>

          <!-- Analysis Render Area -->
          <div v-if="isAiDiagnosing" class="py-24 text-center flex flex-col items-center justify-center">
            <Loader class="w-8 h-8 animate-spin text-purple-400 mb-3" />
            <span class="text-xs font-mono text-purple-300 uppercase tracking-widest animate-pulse">Decompiling bytecode trace and querying neural core...</span>
          </div>

          <div v-else-if="formattedAnalysisHtml" class="markdown-body text-xs text-white/80 leading-relaxed font-sans select-text overflow-y-auto custom-scroll pr-1" v-html="formattedAnalysisHtml"></div>

          <div v-else class="py-24 text-center text-white/30 font-mono text-xs flex flex-col items-center gap-3">
            <Bot class="w-12 h-12 opacity-30" />
            <span>Awaiting crash log ingestion or diagnostic prompt...</span>
          </div>
        </div>
      </section>

      <!-- Right Column: Hardware Dossier & Bug Dispatcher -->
      <section class="col-span-5 flex flex-col gap-5">
        <!-- Hardware Telemetry Card -->
        <div class="kip-card p-6 border border-white/10 bg-black/60 shadow-2xl relative flex flex-col justify-between">
          <div class="flex justify-between items-center mb-4">
            <div class="flex items-center gap-2">
              <Cpu class="w-4 h-4 text-indigo-400" />
              <h4 class="font-black text-xs font-mono uppercase tracking-widest text-indigo-400">System Telemetry Metrics</h4>
            </div>

            <button
              @click="loadTelemetry"
              :disabled="isFetchingTelemetry"
              class="text-white/40 hover:text-white transition"
              title="Refresh telemetry"
            >
              <RefreshCw class="w-3.5 h-3.5" :class="isFetchingTelemetry ? 'animate-spin text-indigo-400' : ''" />
            </button>
          </div>

          <div class="grid grid-cols-2 gap-3 mb-4">
            <div class="bg-black/50 p-3 rounded-xl border border-white/5">
              <span class="text-[8px] font-mono text-white/40 uppercase block font-bold">OS Distribution</span>
              <span class="text-xs font-mono font-bold text-white truncate block mt-0.5">{{ telemetry.osName }} {{ telemetry.osVersion }}</span>
            </div>

            <div class="bg-black/50 p-3 rounded-xl border border-white/5">
              <span class="text-[8px] font-mono text-white/40 uppercase block font-bold">Host Processor</span>
              <span class="text-xs font-mono font-bold text-white truncate block mt-0.5">{{ telemetry.cpuBrand }}</span>
            </div>

            <div class="bg-black/50 p-3 rounded-xl border border-white/5">
              <span class="text-[8px] font-mono text-white/40 uppercase block font-bold">Physical RAM</span>
              <span class="text-xs font-mono font-bold text-white truncate block mt-0.5">{{ telemetry.totalRamGb }} GB ({{ telemetry.availableRamGb }} GB free)</span>
            </div>

            <div class="bg-black/50 p-3 rounded-xl border border-white/5">
              <span class="text-[8px] font-mono text-white/40 uppercase block font-bold">HotSpot Java</span>
              <span class="text-xs font-mono font-bold text-white truncate block mt-0.5">{{ telemetry.javaVersion }}</span>
            </div>
          </div>

          <button
            @click="copyTelemetryDossier"
            class="kip-btn-ghost w-full py-2 text-xs font-mono font-bold uppercase tracking-wider text-white/60 hover:text-white border-white/10 flex items-center justify-center gap-2"
          >
            <Copy class="w-3.5 h-3.5" />
            <span>Copy Hardware Dossier</span>
          </button>
        </div>

        <!-- Encrypted Bug Report Dispatcher -->
        <div class="kip-card p-6 border border-white/10 bg-black/60 shadow-2xl flex-1 flex flex-col justify-between">
          <div>
            <div class="flex items-center gap-2.5 mb-2">
              <div class="p-2 bg-indigo-500/10 rounded-xl border border-indigo-500/20 text-indigo-400">
                <Bug class="w-5 h-5" />
              </div>
              <div>
                <h3 class="font-black text-sm uppercase text-white tracking-wider">Bug Report Dispatcher</h3>
                <span class="text-[9px] font-mono text-white/40">Secure transmission to Cloudflare Edge</span>
              </div>
            </div>

            <p class="text-[10px] text-white/50 leading-relaxed font-mono mb-3">
              Describe steps to reproduce, unexpected behaviors or glitches. Sensitive paths and credentials will be scrubbed before transmission.
            </p>

            <textarea
              v-model="bugReportText"
              rows="5"
              placeholder="Detailed reproduction steps, unexpected exceptions or launcher faults..."
              class="kip-input text-xs font-mono resize-none leading-relaxed p-3 h-36 mb-3 select-text"
            ></textarea>

            <div class="flex items-center justify-between text-xs mb-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  v-model="includeTelemetryInReport"
                  type="checkbox"
                  class="accent-indigo-500 rounded"
                >
                <span class="text-[11px] font-mono text-white/70">Attach sanitized hardware telemetry</span>
              </label>
            </div>

            <div
              v-if="lastSubmittedReportId"
              class="p-3 bg-emerald-500/10 border border-emerald-500/30 rounded-xl flex items-center gap-2 text-xs text-emerald-400 font-mono mb-4"
            >
              <CheckCircle2 class="w-4 h-4 shrink-0" />
              <span>Report accepted! Ref ID: <strong>{{ lastSubmittedReportId }}</strong></span>
            </div>
          </div>

          <button
            @click="submitBugReport"
            :disabled="isSubmittingBug || !bugReportText.trim()"
            class="kip-btn-primary w-full py-3.5 text-xs font-black uppercase tracking-widest shadow-[0_0_20px_rgba(99,102,241,0.4)] flex items-center justify-center gap-2"
          >
            <Loader v-if="isSubmittingBug" class="w-4 h-4 animate-spin" />
            <Send v-else class="w-4 h-4" />
            <span>{{ isSubmittingBug ? 'Transmitting Encrypted Frame...' : 'Submit Encrypted Report' }}</span>
          </button>
        </div>
      </section>
    </div>
  </div>
</template>