<script setup lang="ts">
import { ref } from 'vue'
import {
  ShieldAlert,
  ShieldCheck,
  ScanSearch,
  Lock,
  History,
  Trash2,
  Bug,
  X,
  Loader,
  Binary,
} from 'lucide-vue-next'
import type {
  ShieldScanReportDto,
  ShieldScanProgressDto,
  FileSecurityReportDto,
  QuarantineRecordDto,
} from '../../types/tools'

const isOpen = defineModel<boolean>({ required: true })

defineProps<{
  report: ShieldScanReportDto | null
  progress: ShieldScanProgressDto
  vaultRecords: QuarantineRecordDto[]
  scanning: boolean
}>()

const emit = defineEmits<{
  (e: 'scan'): void
  (e: 'quarantine', threat: FileSecurityReportDto): void
  (e: 'restore', id: string): void
  (e: 'shred', id: string): void
}>()

const activeTab = ref<'scanner' | 'vault'>('scanner')
</script>

<template>
  <transition name="fade">
    <div
      v-if="isOpen"
      class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
      @click.self="isOpen = false"
    >
      <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-6xl shadow-[0_0_70px_rgba(244,63,94,0.25)] overflow-hidden border border-rose-500/30">
        <!-- Header -->
        <header class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0">
          <div class="flex items-center gap-4">
            <div class="p-3 bg-rose-500/20 border border-rose-500/40 rounded-2xl text-rose-400">
              <ShieldAlert class="w-7 h-7" />
            </div>
            <div>
              <h3 class="text-2xl font-black uppercase text-white tracking-wider">
                K.I.P. Shield Security Matrix
              </h3>
              <span class="text-xs font-mono text-rose-400">Heuristic Decompiler & Multi-Vector Cryptographic Isolation Vault</span>
            </div>
          </div>

          <div class="flex items-center gap-3">
            <button
              @click="emit('scan')"
              :disabled="scanning"
              class="kip-btn-primary px-6 py-2.5 text-xs uppercase font-black tracking-wider bg-rose-500 hover:bg-rose-400 text-white shadow-[0_0_20px_rgba(244,63,94,0.4)] flex items-center gap-2"
            >
              <Loader v-if="scanning" class="w-4 h-4 animate-spin" />
              <ScanSearch v-else class="w-4 h-4" />
              <span>{{ scanning ? 'Auditing Bytecode...' : 'Run Deep Audit' }}</span>
            </button>
            <button @click="isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white transition">
              <X class="w-6 h-6" />
            </button>
          </div>
        </header>

        <!-- Navigation Tabs -->
        <div class="flex border-b border-white/5 bg-black/30 shrink-0">
          <button
            @click="activeTab = 'scanner'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
            :class="activeTab === 'scanner' ? 'text-rose-400 border-rose-400 bg-rose-500/5' : 'text-white/40 border-transparent hover:text-white'"
          >
            <ScanSearch class="w-3.5 h-3.5" /> Threat Radar ({{ report?.threat_count || 0 }})
          </button>
          <button
            @click="activeTab = 'vault'"
            class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
            :class="activeTab === 'vault' ? 'text-rose-400 border-rose-400 bg-rose-500/5' : 'text-white/40 border-transparent hover:text-white'"
          >
            <Lock class="w-3.5 h-3.5" /> Quarantine Vault ({{ vaultRecords.length }})
          </button>
        </div>

        <!-- Body -->
        <main class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 min-h-0">
          <div v-show="activeTab === 'scanner'" class="flex flex-col gap-6">
            <!-- Metric Cards -->
            <div class="grid grid-cols-4 gap-4">
              <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                <span class="text-[9px] font-mono font-bold text-white/40 uppercase">Scanned Archives</span>
                <span class="text-2xl font-black text-white mt-1">{{ scanning ? progress.scannedCount : (report?.total_scanned || 0) }}</span>
              </div>
              <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                <span class="text-[9px] font-mono font-bold text-emerald-400 uppercase">Verified Clean</span>
                <span class="text-2xl font-black text-emerald-400 mt-1">{{ report?.clean_count || 0 }}</span>
              </div>
              <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                <span class="text-[9px] font-mono font-bold text-amber-400 uppercase">Suspicious Packages</span>
                <span class="text-2xl font-black text-amber-400 mt-1">{{ scanning ? progress.threatsFound : (report?.threat_count || 0) }}</span>
              </div>
              <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                <span class="text-[9px] font-mono font-bold text-rose-500 uppercase">Critical Injections</span>
                <span class="text-2xl font-black text-rose-500 mt-1">{{ report?.critical_count || 0 }}</span>
              </div>
            </div>

            <!-- Live Streaming Progress Bar -->
            <div v-if="scanning" class="kip-card p-5 bg-black/60 border border-rose-500/30 flex flex-col gap-3">
              <div class="flex justify-between items-center text-xs font-mono">
                <span class="text-rose-400 flex items-center gap-2 truncate">
                  <Binary class="w-4 h-4 animate-pulse" />
                  <span>Scanning: {{ progress.currentFile }}</span>
                </span>
                <span class="text-white font-bold">{{ Math.round(progress.percent) }}%</span>
              </div>

              <div class="w-full h-2 bg-black/80 rounded-full overflow-hidden border border-white/5">
                <div
                  class="h-full bg-gradient-to-r from-rose-500 to-amber-500 transition-all duration-150"
                  :style="{ width: `${progress.percent}%` }"
                ></div>
              </div>
            </div>

            <!-- Clean Status -->
            <div v-else-if="!report || report.threat_count === 0" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-emerald-500/20 rounded-3xl bg-emerald-950/5">
              <ShieldCheck class="w-16 h-16 text-emerald-400 mb-3" />
              <h4 class="text-xl font-black text-white uppercase">Instance Completely Fortified</h4>
              <p class="text-xs text-white/50 font-mono mt-1">Zero malware signatures, webhooks or droppers discovered.</p>
            </div>

            <!-- Threats Radar List -->
            <div v-else class="flex flex-col gap-4">
              <div
                v-for="threat in report.threats"
                :key="threat.filepath"
                class="p-5 bg-black/60 border rounded-2xl flex flex-col gap-4"
                :class="threat.threat_level === 'CRITICAL' ? 'border-rose-500/50' : 'border-amber-500/50'"
              >
                <div class="flex justify-between items-start gap-4">
                  <div>
                    <div class="flex items-center gap-3 mb-1">
                      <h4 class="font-black text-lg text-white leading-tight">{{ threat.filename }}</h4>
                      <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-black uppercase border" :class="threat.threat_level === 'CRITICAL' ? 'bg-rose-500/20 text-rose-400 border-rose-500/30' : 'bg-amber-500/20 text-amber-400 border-amber-500/30'">
                        {{ threat.threat_level }} (Risk Score: {{ threat.threat_score }}/100)
                      </span>
                    </div>
                    <p class="text-[10px] font-mono text-white/40">SHA256: {{ threat.sha256 }}</p>
                  </div>

                  <button
                    @click="emit('quarantine', threat)"
                    class="kip-btn-primary px-4 py-2 text-xs font-black uppercase bg-rose-500 hover:bg-rose-400 text-white flex items-center gap-1.5"
                  >
                    <Lock class="w-3.5 h-3.5" /> Quarantine
                  </button>
                </div>

                <div class="bg-black/40 border border-white/5 rounded-xl p-3 flex flex-col gap-2">
                  <div v-for="(ind, idx) in threat.indicators" :key="idx" class="flex items-start justify-between text-xs font-mono">
                    <div class="flex items-center gap-2">
                      <Bug class="w-3.5 h-3.5 text-rose-400 shrink-0" />
                      <span class="text-white/80 font-bold">[{{ ind.category }}] {{ ind.title }}</span>
                    </div>
                    <span class="text-white/40 text-[10px]">{{ ind.location }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>

          <!-- Quarantine Vault Tab -->
          <div v-show="activeTab === 'vault'" class="flex flex-col gap-4">
            <div v-if="vaultRecords.length === 0" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-white/10 rounded-3xl">
              <Lock class="w-12 h-12 text-white/20 mb-3" />
              <span class="text-xs font-mono text-white/40 uppercase">Quarantine Vault is Empty</span>
            </div>

            <div
              v-for="rec in vaultRecords"
              :key="rec.id"
              class="p-5 bg-black/60 border border-white/10 rounded-2xl flex justify-between items-center"
            >
              <div>
                <div class="flex items-center gap-3 mb-1">
                  <h4 class="font-black text-white text-base leading-tight">{{ rec.filename }}</h4>
                  <span class="px-2.5 py-0.5 rounded bg-rose-500/20 text-rose-400 border border-rose-500/30 text-[9px] font-mono font-bold uppercase">
                    Risk: {{ rec.threat_score }}/100
                  </span>
                </div>
                <p class="text-[10px] font-mono text-white/40">Quarantined at: {{ rec.quarantined_at }}</p>
              </div>

              <div class="flex items-center gap-2">
                <button
                  @click="emit('restore', rec.id)"
                  class="kip-btn-ghost px-4 py-2 text-xs font-bold uppercase text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10 flex items-center gap-1.5"
                >
                  <History class="w-3.5 h-3.5" /> Restore
                </button>
                <button
                  @click="emit('shred', rec.id)"
                  class="kip-btn-danger px-4 py-2 text-xs font-black uppercase flex items-center gap-1.5"
                >
                  <Trash2 class="w-3.5 h-3.5" /> DoD Shred
                </button>
              </div>
            </div>
          </div>
        </main>
      </div>
    </div>
  </transition>
</template>