<template>
  <div class="h-full overflow-y-auto custom-scroll pr-2 pb-10 select-none relative">
    <div class="mb-8 shrink-0 stagger-1">
      <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
        {{ t('System Tools') }}
      </h2>
      <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Automated Heuristic Engine, Forensic Crash Diagnostics & Quantum Instance Maintenance</p>
    </div>

    <div class="grid grid-cols-3 gap-6 mb-8 stagger-2">
      <div @click="openDoctorDeck" class="kip-card kip-card-hover p-6 relative overflow-hidden border-pink-500/30 hover:border-pink-500/60 cursor-pointer flex flex-col justify-between group shadow-[0_0_40px_rgba(236,72,153,0.15)] bg-pink-950/10">
        <div class="absolute -right-12 -top-12 w-48 h-48 bg-pink-500/20 blur-3xl rounded-full pointer-events-none group-hover:scale-125 transition-transform duration-700"></div>
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-4 bg-pink-500/20 rounded-2xl border border-pink-500/40 shadow-[0_0_20px_rgba(236,72,153,0.3)] group-hover:scale-110 transition-transform duration-500 shrink-0 text-pink-400">
            <Stethoscope class="w-8 h-8 animate-pulse" />
          </div>
          <div>
            <div class="flex items-center gap-2 mb-1">
              <span class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider bg-pink-500/20 text-pink-300 border border-pink-500/30">AI Diagnostic</span>
            </div>
            <h3 class="text-xl font-black text-white leading-tight">Mod Doctor Core</h3>
            <p class="text-white/50 text-[10px] font-mono mt-0.5 uppercase tracking-wider">Dependency & Conflict Solver</p>
          </div>
        </div>

        <div class="pt-4 border-t border-pink-500/20 flex items-center justify-between text-xs font-mono text-pink-300/80 relative z-10">
          <span>{{ isAnalyzingDoctor ? 'Auditing Jars...' : 'Run Topology Scan' }}</span>
          <ArrowRight class="w-4 h-4 group-hover:translate-x-1 transition-transform" />
        </div>
      </div>

      <div @click="openShieldCenter" class="kip-card kip-card-hover p-6 relative overflow-hidden border-rose-500/30 hover:border-rose-500/60 cursor-pointer flex flex-col justify-between group shadow-[0_0_40px_rgba(244,63,94,0.15)] bg-rose-950/10">
        <div class="absolute -right-12 -top-12 w-48 h-48 bg-rose-500/20 blur-3xl rounded-full pointer-events-none group-hover:scale-125 transition-transform duration-700"></div>
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-4 bg-rose-500/20 rounded-2xl border border-rose-500/40 shadow-[0_0_20px_rgba(244,63,94,0.3)] group-hover:scale-110 transition-transform duration-500 shrink-0 text-rose-400">
            <ShieldAlert class="w-8 h-8" />
          </div>
          <div>
            <div class="flex items-center gap-2 mb-1">
              <span class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider bg-rose-500/20 text-rose-300 border border-rose-500/30">Resident Guard</span>
            </div>
            <h3 class="text-xl font-black text-white leading-tight">K.I.P. Shield Flagship</h3>
            <p class="text-white/50 text-[10px] font-mono mt-0.5 uppercase tracking-wider">Antivirus & Bytecode Analyzer</p>
          </div>
        </div>
        <div class="pt-4 border-t border-rose-500/20 flex items-center justify-between text-xs font-mono text-rose-300/80 relative z-10">
          <span>Security Center</span>
          <ArrowRight class="w-4 h-4 group-hover:translate-x-1 transition-transform" />
        </div>
      </div>

      <div class="kip-card p-6 relative overflow-hidden flex flex-col justify-between group transition-colors duration-500" :class="state.settings.safe_mode ? 'border-emerald-500/40 bg-emerald-900/10' : 'kip-card-hover border-white/5'">
        <div class="flex items-center gap-4 relative z-10 mb-4">
          <div class="p-3.5 rounded-xl border shrink-0" :class="state.settings.safe_mode ? 'bg-emerald-500/20 border-emerald-500/30 text-emerald-400' : 'bg-white/5 border-white/10 text-white/50'">
            <Shield class="w-6 h-6" />
          </div>
          <div>
            <h4 class="font-black text-base text-white leading-tight">{{ t('Safe Mode') }}</h4>
            <p class="text-[9px] text-white/50 uppercase tracking-widest font-mono">{{ t('Temporarily disable all mods') }}</p>
          </div>
        </div>
        <button @click="toggleSafeMode" class="w-full py-2.5 rounded-xl text-xs font-black uppercase tracking-wider transition-all duration-300 relative z-10 shadow-lg" :class="state.settings.safe_mode ? 'bg-emerald-500 hover:bg-emerald-400 text-black shadow-[0_0_15px_rgba(16,185,129,0.3)]' : 'bg-white/10 hover:bg-white/20 text-white'">
          {{ state.settings.safe_mode ? t('ENABLED') : t('ENABLE') }}
        </button>
      </div>
    </div>

    <div class="mb-8 stagger-3">
      <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Zap class="w-3.5 h-3.5 text-amber-400" /> {{ t('Performance & Network') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div v-for="tool in perfTools" :key="tool.id" @click="runTool(tool)" class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5">
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm text-white">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider font-mono">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </div>

    <div class="mb-4 stagger-3">
      <h3 class="text-xs font-mono font-bold text-white/40 uppercase tracking-widest mb-4 flex items-center gap-2">
        <Trash2 class="w-3.5 h-3.5 text-red-400" /> {{ t('Maintenance & Cleanup') }}
      </h3>
      <div class="grid grid-cols-4 gap-4">
        <div v-for="tool in cleanupTools" :key="tool.id" @click="runTool(tool)" class="kip-card kip-card-hover p-5 cursor-pointer flex flex-col items-center text-center gap-3 group border-white/5">
          <div :class="['p-4 rounded-2xl transition-transform duration-300 group-hover:scale-110 group-hover:shadow-lg', tool.bgClass, tool.shadowClass]">
            <component :is="tool.icon" :class="['w-6 h-6', tool.textClass]" />
          </div>
          <div>
            <h4 class="font-bold text-sm text-white">{{ t(tool.name) }}</h4>
            <p class="text-[10px] text-white/40 mt-1 uppercase tracking-wider font-mono">{{ t(tool.desc) }}</p>
          </div>
        </div>
      </div>
    </div>

    <transition name="fade">
      <div v-if="doctorModal.isOpen" class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default" @click.self="doctorModal.isOpen = false">
        <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-5xl shadow-[0_0_70px_rgba(236,72,153,0.25)] overflow-hidden border border-pink-500/30">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
            <div class="absolute -top-32 -right-32 w-80 h-80 bg-pink-500/10 blur-[100px] rounded-full pointer-events-none"></div>

            <div class="flex items-center gap-4 relative z-10">
              <div class="p-3 bg-pink-500/20 border border-pink-500/40 rounded-2xl text-pink-400">
                <Stethoscope class="w-7 h-7" />
              </div>
              <div>
                <h3 class="text-2xl font-black uppercase text-white tracking-wider flex items-center gap-3">
                  Mod Doctor • Diagnostic & Remediation Engine
                </h3>
                <span class="text-xs font-mono text-pink-400">Automated Dependency Synthesis, Pipeline Collision Resolver & JiJ Inspector</span>
              </div>
            </div>

            <div class="flex items-center gap-3 relative z-10">
              <button @click="triggerDoctorAnalysis" :disabled="doctorModal.loading" class="kip-btn-primary px-5 py-2.5 text-xs uppercase font-black tracking-wider bg-pink-500 hover:bg-pink-400 text-white shadow-[0_0_20px_rgba(236,72,153,0.4)]">
                <Loader v-if="doctorModal.loading" class="w-4 h-4 animate-spin" />
                <RefreshCw v-else class="w-4 h-4" />
                <span>Re-Audit</span>
              </button>
              <button @click="doctorModal.isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition">
                <X class="w-6 h-6" />
              </button>
            </div>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 min-h-0 relative">
            <div v-if="doctorModal.loading" class="py-24 text-center flex flex-col items-center justify-center">
              <div class="w-20 h-20 border-4 border-pink-500/20 border-t-pink-500 rounded-full animate-spin mb-4"></div>
              <span class="text-xs font-mono font-bold uppercase tracking-widest text-pink-400 animate-pulse">Decompiling Descriptors, Parsing JiJ Nodes & Validating Mod Matrix...</span>
            </div>

            <div v-else-if="doctorModal.isClean" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-emerald-500/20 rounded-3xl bg-emerald-950/5">
              <CheckCircle class="w-16 h-16 text-emerald-400 mb-3" />
              <h4 class="text-xl font-black text-white uppercase">Instance 100% Healthy</h4>
              <p class="text-xs text-white/50 font-mono mt-1">Zero missing dependencies, cross-loader violations or pipeline collisions detected across {{ doctorModal.totalChecked }} modules.</p>
            </div>

            <div v-else class="flex flex-col gap-4">
              <div class="flex justify-between items-center bg-black/40 p-3.5 rounded-2xl border border-white/5">
                <span class="text-xs font-mono font-bold text-white/60">Identified {{ doctorModal.issues.length }} anomalies across {{ doctorModal.totalChecked }} inspected archives</span>
                <div class="flex gap-2">
                  <button @click="selectAllIssues(true)" class="kip-btn-ghost px-3 py-1.5 text-xs text-pink-400">Select All</button>
                  <button @click="selectAllIssues(false)" class="kip-btn-ghost px-3 py-1.5 text-xs text-white/40">Deselect</button>
                </div>
              </div>

              <div v-for="issue in doctorModal.issues" :key="issue.id" @click="toggleIssueSelection(issue)" class="p-5 bg-black/60 border rounded-2xl flex items-start gap-4 transition cursor-pointer hover:bg-black/80" :class="[issue.type === 'CRITICAL' ? 'border-red-500/50' : issue.type === 'WARNING' ? 'border-amber-500/50' : 'border-cyan-500/40', issue.selected ? 'bg-pink-500/5' : '']">
                <div class="mt-1 p-2.5 rounded-xl border shrink-0" :class="issue.type === 'CRITICAL' ? 'bg-red-500/10 border-red-500/20 text-red-400' : issue.type === 'WARNING' ? 'bg-amber-500/10 border-amber-500/20 text-amber-400' : 'bg-cyan-500/10 border-cyan-500/20 text-cyan-400'">
                  <AlertTriangle v-if="issue.type === 'CRITICAL'" class="w-5 h-5" />
                  <AlertCircle v-else-if="issue.type === 'WARNING'" class="w-5 h-5" />
                  <Sparkles v-else class="w-5 h-5" />
                </div>

                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2 mb-1.5">
                    <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-black uppercase tracking-wider border" :class="issue.type === 'CRITICAL' ? 'bg-red-500/20 text-red-400 border-red-500/30' : issue.type === 'WARNING' ? 'bg-amber-500/20 text-amber-400 border-amber-500/30' : 'bg-cyan-500/20 text-cyan-400 border-cyan-500/30'">
                      {{ issue.type }}
                    </span>
                    <h4 class="font-black text-sm text-white">{{ issue.title || 'Anomalous Manifest State' }}</h4>
                  </div>
                  <p class="text-xs text-white/70 leading-relaxed font-medium mb-2">{{ issue.text }}</p>

                  <div v-if="issue.action !== 'NONE'" class="inline-flex items-center gap-2 px-2.5 py-1 rounded-lg bg-black/40 border border-white/10 text-[10px] font-mono text-white/60">
                    <span class="font-bold text-pink-400 uppercase">{{ issue.action }}:</span>
                    <span class="truncate max-w-sm">{{ issue.target }}</span>
                  </div>
                </div>

                <div v-if="issue.action !== 'NONE'" class="relative inline-block w-10 shrink-0 mt-2" @click.stop>
                  <input type="checkbox" class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none transition-transform" :checked="issue.selected" @change="issue.selected = !issue.selected">
                  <label class="toggle-label block h-5 rounded-full transition-colors border border-white/10"></label>
                </div>
              </div>
            </div>
          </div>

          <div v-if="!doctorModal.isClean" class="p-6 border-t border-white/5 bg-black/60 shrink-0 flex justify-between items-center relative z-20">
            <span class="text-xs font-mono font-bold text-white/40 uppercase">{{ doctorModal.issues.filter(i => i.selected).length }} remediation actions primed</span>
            <div class="flex gap-3">
              <button @click="doctorModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">{{ t('Close') }}</button>
              <button @click="applyDoctorRemediation" :disabled="isCuring || doctorModal.issues.filter(i => i.selected).length === 0" class="kip-btn-primary px-8 py-2.5 bg-pink-500 hover:bg-pink-400 text-white font-black text-xs uppercase tracking-wider shadow-[0_0_20px_rgba(236,72,153,0.4)]">
                <Loader v-if="isCuring" class="w-4 h-4 animate-spin" />
                <Wrench v-else class="w-4 h-4" />
                <span>{{ isCuring ? 'Applying Remediation...' : 'Cure Modpack 1-Click' }}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div v-if="shieldModal.isOpen" class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default" @click.self="shieldModal.isOpen = false">
        <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-6xl shadow-[0_0_70px_rgba(244,63,94,0.25)] overflow-hidden border border-rose-500/30">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
            <div class="absolute -top-32 -right-32 w-80 h-80 bg-rose-500/10 blur-[100px] rounded-full pointer-events-none"></div>

            <div class="flex items-center gap-4 relative z-10">
              <div class="p-3 bg-rose-500/20 border border-rose-500/40 rounded-2xl text-rose-400">
                <ShieldAlert class="w-7 h-7" />
              </div>
              <div>
                <h3 class="text-2xl font-black uppercase text-white tracking-wider flex items-center gap-3">
                  K.I.P. Shield • Enterprise Threat Center
                </h3>
                <span class="text-xs font-mono text-rose-400">Deep Heuristic Decompiler & Multi-Vector Cryptographic Quarantine Vault</span>
              </div>
            </div>

            <div class="flex items-center gap-3 relative z-10">
              <button @click="triggerFullShieldScan" :disabled="shieldModal.loading" class="kip-btn-primary px-6 py-2.5 text-xs uppercase font-black tracking-wider bg-rose-500 hover:bg-rose-400 text-white shadow-[0_0_20px_rgba(244,63,94,0.4)]">
                <Loader v-if="shieldModal.loading" class="w-4 h-4 animate-spin" />
                <ScanSearch v-else class="w-4 h-4" />
                <span>{{ shieldModal.loading ? 'Scanning Bytecode...' : 'Run Deep Audit' }}</span>
              </button>
              <button @click="shieldModal.isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition">
                <X class="w-6 h-6" />
              </button>
            </div>
          </div>

          <div class="flex border-b border-white/5 bg-black/30 shrink-0">
            <button @click="shieldActiveTab = 'scanner'" class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2" :class="shieldActiveTab === 'scanner' ? 'text-rose-400 border-rose-400 bg-rose-500/5' : 'text-white/40 border-transparent hover:text-white'">
              <ScanSearch class="w-3.5 h-3.5" /> Threat Surface Radar ({{ shieldReport?.threat_count || 0 }})
            </button>
            <button @click="shieldActiveTab = 'vault'; loadVaultRecords()" class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2" :class="shieldActiveTab === 'vault' ? 'text-rose-400 border-rose-400 bg-rose-500/5' : 'text-white/40 border-transparent hover:text-white'">
              <Lock class="w-3.5 h-3.5" /> Quarantine Vault ({{ vaultRecords.length }})
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-6 bg-[#030305]/95 min-h-0 relative">
            <div v-show="shieldActiveTab === 'scanner'" class="flex flex-col gap-6">
              <div class="grid grid-cols-4 gap-4">
                <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                  <span class="text-[9px] font-mono font-bold text-white/40 uppercase">Scanned Archives</span>
                  <span class="text-2xl font-black text-white mt-1">{{ shieldReport?.total_scanned || 0 }}</span>
                </div>
                <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                  <span class="text-[9px] font-mono font-bold text-emerald-400 uppercase">Verified Clean</span>
                  <span class="text-2xl font-black text-emerald-400 mt-1">{{ shieldReport?.clean_count || 0 }}</span>
                </div>
                <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                  <span class="text-[9px] font-mono font-bold text-amber-400 uppercase">Suspicious Packages</span>
                  <span class="text-2xl font-black text-amber-400 mt-1">{{ shieldReport?.threat_count || 0 }}</span>
                </div>
                <div class="bg-black/50 border border-white/5 p-4 rounded-2xl flex flex-col justify-between">
                  <span class="text-[9px] font-mono font-bold text-rose-500 uppercase">Critical Injections</span>
                  <span class="text-2xl font-black text-rose-500 mt-1">{{ shieldReport?.critical_count || 0 }}</span>
                </div>
              </div>

              <div v-if="shieldModal.loading" class="py-24 text-center flex flex-col items-center justify-center">
                <div class="w-20 h-20 border-4 border-rose-500/20 border-t-rose-500 rounded-full animate-spin mb-4"></div>
                <span class="text-xs font-mono font-bold uppercase tracking-widest text-rose-400 animate-pulse">Decompressing Jar Bytecode & Calculating Entropy Vectors...</span>
              </div>

              <div v-else-if="!shieldReport || shieldReport.threat_count === 0" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-emerald-500/20 rounded-3xl bg-emerald-950/5">
                <ShieldCheck class="w-16 h-16 text-emerald-400 mb-3" />
                <h4 class="text-xl font-black text-white uppercase">Instance Completely Fortified</h4>
                <p class="text-xs text-white/50 font-mono mt-1">Zero known malware signatures, droppers or exfiltration webhooks detected.</p>
              </div>

              <div v-else class="flex flex-col gap-4">
                <div v-for="threat in shieldReport.threats" :key="threat.filepath" class="p-5 bg-black/60 border rounded-2xl flex flex-col gap-4 transition-colors" :class="threat.threat_level === 'CRITICAL' ? 'border-rose-500/50 hover:border-rose-500' : 'border-amber-500/50 hover:border-amber-500'">
                  <div class="flex justify-between items-start gap-4">
                    <div>
                      <div class="flex items-center gap-3 mb-1">
                        <h4 class="font-black text-lg text-white leading-tight">{{ threat.filename }}</h4>
                        <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-black uppercase tracking-wider border" :class="threat.threat_level === 'CRITICAL' ? 'bg-rose-500/20 text-rose-400 border-rose-500/30' : 'bg-amber-500/20 text-amber-400 border-amber-500/30'">
                          {{ threat.threat_level }} (Risk Score: {{ threat.threat_score }}/100)
                        </span>
                        <span class="text-[10px] font-mono text-white/40">Entropy: {{ threat.entropy }}</span>
                      </div>
                      <p class="text-[10px] font-mono text-white/40 select-all">SHA256: {{ threat.sha256 }}</p>
                    </div>

                    <div class="flex items-center gap-2 shrink-0">
                      <button @click="quarantineThreat(threat)" class="kip-btn-primary px-4 py-2 text-xs font-black uppercase tracking-wider bg-rose-500 hover:bg-rose-400 text-white shadow-[0_0_15px_rgba(244,63,94,0.3)]">
                        <Lock class="w-3.5 h-3.5" /> Quarantine
                      </button>
                    </div>
                  </div>

                  <div class="bg-black/40 border border-white/5 rounded-xl p-3 flex flex-col gap-2">
                    <div v-for="(ind, idx) in threat.indicators" :key="idx" class="flex items-start justify-between text-xs font-mono">
                      <div class="flex items-center gap-2">
                        <Bug class="w-3.5 h-3.5 text-rose-400 shrink-0" />
                        <span class="text-white/80 font-bold">[{{ ind.category }}] {{ ind.title }}</span>
                      </div>
                      <span class="text-white/40 text-[10px] truncate max-w-md">{{ ind.location }}</span>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div v-show="shieldActiveTab === 'vault'" class="flex flex-col gap-4">
              <div v-if="vaultRecords.length === 0" class="py-24 text-center flex flex-col items-center justify-center border border-dashed border-white/10 rounded-3xl">
                <Lock class="w-12 h-12 text-white/20 mb-3" />
                <span class="text-xs font-mono text-white/40 uppercase">Quarantine Vault is Empty</span>
              </div>

              <div v-for="rec in vaultRecords" :key="rec.id" class="p-5 bg-black/60 border border-white/10 rounded-2xl flex justify-between items-center">
                <div>
                  <div class="flex items-center gap-3 mb-1">
                    <h4 class="font-black text-white text-base leading-tight">{{ rec.filename }}</h4>
                    <span class="px-2.5 py-0.5 rounded bg-rose-500/20 text-rose-400 border border-rose-500/30 text-[9px] font-mono font-bold uppercase">
                      Score: {{ rec.threat_score }}/100
                    </span>
                  </div>
                  <p class="text-[10px] font-mono text-white/40">Isolated: {{ rec.quarantined_at }} • Original: {{ rec.original_path }}</p>
                </div>

                <div class="flex items-center gap-2">
                  <button @click="restoreThreat(rec.id)" class="kip-btn-ghost px-4 py-2 text-xs font-bold uppercase text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/10">
                    <History class="w-3.5 h-3.5" /> Restore
                  </button>
                  <button @click="shredThreat(rec.id)" class="kip-btn-danger px-4 py-2 text-xs font-black uppercase tracking-wider">
                    <Trash2 class="w-3.5 h-3.5" /> DoD Shred
                  </button>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
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
  ShieldAlert,
  ShieldCheck,
  ScanSearch,
  Lock,
  History,
  Bug,
  X,
  Loader,
  ArrowRight,
  CheckCircle,
  AlertTriangle,
  AlertCircle,
  Sparkles,
  Wrench,
  RefreshCw,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type ToolExecutionResult,
  type ShieldScanReportDto,
  type FileSecurityReportDto,
  type QuarantineRecordDto,
} from '@/bridge'

interface ToolDefinition {
  id: string
  name: string
  desc: string
  bgClass: string
  textClass: string
  shadowClass: string
  icon: Component
}

interface DoctorIssueItem {
  id: string
  type: string
  title: string
  text: string
  action: string
  target: string
  selected: boolean
}

const isAnalyzingDoctor = ref<boolean>(false)
const isCuring = ref<boolean>(false)

const doctorModal = ref<{
  isOpen: boolean
  loading: boolean
  isClean: boolean
  totalChecked: number
  issues: DoctorIssueItem[]
}>({
  isOpen: false,
  loading: false,
  isClean: true,
  totalChecked: 0,
  issues: [],
})

const shieldActiveTab = ref<'scanner' | 'vault'>('scanner')
const shieldReport = ref<ShieldScanReportDto | null>(null)
const vaultRecords = ref<QuarantineRecordDto[]>([])

const shieldModal = ref({
  isOpen: false,
  loading: false,
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

const openDoctorDeck = async (): Promise<void> => {
  doctorModal.value.isOpen = true
  await triggerDoctorAnalysis()
}

const triggerDoctorAnalysis = async (): Promise<void> => {
  doctorModal.value.loading = true
  isAnalyzingDoctor.value = true
  try {
    const res: ToolExecutionResult = await invokeSafe<ToolExecutionResult>('run_tool', { toolId: 'mod_doctor' })
    if (res && res.success && res.doctor_res) {
      const data = res.doctor_res as { is_clean?: boolean; total_checked?: number; issues?: Array<Record<string, unknown>> }
      doctorModal.value.isClean = Boolean(data.is_clean)
      doctorModal.value.totalChecked = Number(data.total_checked || 0)
      doctorModal.value.issues = (data.issues || []).map((i) => ({
        id: String(i.id || Math.random()),
        type: String(i.type || 'WARNING'),
        title: String(i.title || 'Diagnostic Point'),
        text: String(i.text || ''),
        action: String(i.action || 'NONE'),
        target: String(i.target || ''),
        selected: String(i.action || 'NONE') !== 'NONE',
      }))
      showToast(t('Diagnostics Complete'), `Audited ${doctorModal.value.totalChecked} modules.`, 'success')
    } else {
      showToast(t('Error'), res?.msg || 'Doctor engine returned failure.', 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    doctorModal.value.loading = false
    isAnalyzingDoctor.value = false
  }
}

const toggleIssueSelection = (issue: DoctorIssueItem): void => {
  if (issue.action !== 'NONE') {
    issue.selected = !issue.selected
  }
}

const selectAllIssues = (select: boolean): void => {
  doctorModal.value.issues.forEach((i) => {
    if (i.action !== 'NONE') {
      i.selected = select
    }
  })
}

const applyDoctorRemediation = async (): Promise<void> => {
  const selected = doctorModal.value.issues.filter((i) => i.selected)
  if (selected.length === 0) return
  isCuring.value = true

  try {
    const res = await invokeSafe<{ success: boolean; deleted: number; downloaded: number; errors: string[] }>('apply_doctor_fixes', {
      issues: selected,
      mcVersion: state.settings.game_resolution ? null : null,
      loader: null,
    })

    if (res && res.success) {
      showToast(t('Remediation Applied'), `Pruned ${res.deleted} files, integrated ${res.downloaded} libraries.`, 'success')
      await triggerDoctorAnalysis()
    } else {
      showToast(t('Remediation Notice'), res?.errors?.[0] || 'Some actions could not be resolved.', 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isCuring.value = false
  }
}

const openShieldCenter = async (): Promise<void> => {
  shieldModal.value.isOpen = true
  if (!shieldReport.value) {
    await triggerFullShieldScan()
  }
}

const triggerFullShieldScan = async (): Promise<void> => {
  shieldModal.value.loading = true
  try {
    shieldReport.value = await bridge.shieldScanFull(null)
    showToast(t('Audit Complete'), `Audited ${shieldReport.value.total_scanned} archives.`, 'success')
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Scan Error'), msg, 'danger')
  } finally {
    shieldModal.value.loading = false
  }
}

const quarantineThreat = async (threat: FileSecurityReportDto): Promise<void> => {
  try {
    await bridge.shieldQuarantineThreat(threat.filepath)
    showToast(t('Threat Isolated'), `${threat.filename} secured in quarantine vault.`, 'success')
    await triggerFullShieldScan()
    await loadVaultRecords()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const loadVaultRecords = async (): Promise<void> => {
  try {
    vaultRecords.value = await bridge.shieldGetVault()
  } catch {
    vaultRecords.value = []
  }
}

const restoreThreat = async (id: string): Promise<void> => {
  try {
    await bridge.shieldRestoreThreat(id)
    showToast(t('Restored'), 'Restored isolated archive to original instance location.', 'info')
    await loadVaultRecords()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const shredThreat = async (id: string): Promise<void> => {
  try {
    await bridge.shieldShredThreat(id)
    showToast(t('Zero-Fill Shred Complete'), 'Payload permanently erased from vault.', 'success')
    await loadVaultRecords()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const runTool = async (tool: { id: string }): Promise<void> => {
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
    showToast(t('Error'), msg, 'danger')
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
    showToast(t('Error'), msg, 'danger')
  }
}

onMounted(() => {
  loadVaultRecords()
})
</script>