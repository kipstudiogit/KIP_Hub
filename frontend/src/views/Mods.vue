<template>
  <div
    class="h-full flex flex-col min-h-0 relative select-none"
    @dragover.prevent="isDraggingOver = true"
    @dragleave.prevent="isDraggingOver = false"
    @drop.prevent="handleDirectDrop"
  >
    <transition name="fade">
      <div
        v-if="isDraggingOver"
        class="absolute inset-0 bg-indigo-950/90 backdrop-blur-md z-50 flex flex-col items-center justify-center border-4 border-dashed border-indigo-400 m-4 rounded-3xl pointer-events-none shadow-[0_0_80px_rgba(99,102,241,0.5)]"
      >
        <PackagePlus class="w-20 h-20 text-indigo-400 animate-bounce mb-4" />
        <h3 class="text-3xl font-black text-white uppercase tracking-wider">Drop Content Here</h3>
        <p class="text-sm text-indigo-200 mt-2 font-mono">Mod JARs, Resource Pack ZIPs, or Shader Packs will be installed automatically</p>
      </div>
    </transition>

    <div class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Content Manager') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Tri-Category Manifest Explorer, Community Hub & P2P Swarm Distribution Matrix</p>
      </div>

      <div class="flex items-center gap-2">
        <button @click="openHub" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-blue-400 border-blue-500/20 hover:border-blue-500/50 hover:bg-blue-500/20">
          <Globe class="w-4 h-4" />
          <span>{{ t('Hub') }}</span>
        </button>

        <button @click="openSwarm" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-amber-400 border-amber-500/20 hover:border-amber-500/50 hover:bg-amber-500/20">
          <Magnet class="w-4 h-4" />
          <span>{{ t('Swarm') }}</span>
        </button>

        <button @click="importContentDialog" :disabled="isImporting" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-cyan-400 border-cyan-500/20 hover:border-cyan-500/50 hover:bg-cyan-500/20">
          <Loader v-if="isImporting" class="w-4 h-4 animate-spin" />
          <Download v-else class="w-4 h-4" />
          <span>{{ t('Import') }}</span>
        </button>

        <button @click="checkModUpdates" :disabled="isCheckingUpdates" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-emerald-400 border-emerald-500/20 hover:border-emerald-500/50 hover:bg-emerald-500/20">
          <Loader v-if="isCheckingUpdates" class="w-4 h-4 animate-spin" />
          <ArrowUpCircle v-else class="w-4 h-4" />
          <span>{{ t('Update') }}</span>
        </button>

        <button @click="exportModpack" :disabled="isExporting" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-purple-400 border-purple-500/20 hover:border-purple-500/50 hover:bg-purple-500/20">
          <Loader v-if="isExporting" class="w-4 h-4 animate-spin" />
          <Package v-else class="w-4 h-4" />
          <span>{{ t('Export') }}</span>
        </button>

        <button @click="isGraphView = !isGraphView" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider" :class="isGraphView ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/30' : ''">
          <LayoutGrid v-if="isGraphView" class="w-4 h-4" />
          <GitMerge v-else class="w-4 h-4" />
          <span>{{ isGraphView ? t('Grid View') : t('Node Graph') }}</span>
        </button>

        <button @click="openCurrentFolder" class="kip-btn-ghost p-2.5" :title="t('Open active folder in OS Explorer')">
          <FolderOpen class="w-4 h-4 text-indigo-400" />
        </button>

        <button @click="loadContent" class="kip-btn-ghost p-2.5">
          <RefreshCw class="w-4 h-4" :class="isContentLoading ? 'animate-spin text-indigo-400' : ''" />
        </button>
      </div>
    </div>

    <div class="flex justify-between items-center gap-4 mb-5 shrink-0 z-10">
      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          v-for="tab in (['mods', 'resourcepacks', 'shaderpacks'] as const)"
          :key="tab"
          @click="activeContentTab = tab; selectedFilenames.clear(); loadContent()"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2"
          :class="activeContentTab === tab ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-[0_0_20px_rgba(99,102,241,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <component :is="getContentTabIcon(tab)" class="w-3.5 h-3.5" />
          <span>{{ getContentTabLabel(tab) }} ({{ contentList.length }})</span>
        </button>
      </div>

      <div class="flex items-center gap-2">
        <div class="flex p-1 bg-black/40 rounded-xl border border-white/10">
          <button
            v-for="s in (['all', 'active', 'disabled'] as const)"
            :key="s"
            @click="statusFilter = s"
            class="px-3 py-1 rounded-lg text-[10px] font-bold uppercase tracking-wider transition"
            :class="statusFilter === s ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30' : 'text-white/40 hover:text-white'"
          >
            {{ s }}
          </button>
        </div>

        <select v-model="sortBy" class="kip-input py-1.5 text-xs w-36 font-mono">
          <option value="name">Sort: Name</option>
          <option value="size">Sort: Size</option>
          <option value="date">Sort: Modified</option>
        </select>
      </div>
    </div>

    <div v-show="!isGraphView" class="flex justify-between items-center gap-3 mb-5 shrink-0 z-10">
      <div class="relative flex-1 max-w-md">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
        <input
          v-model="localSearchQuery"
          type="text"
          :placeholder="t('Search packages by name, ID or author...')"
          class="kip-input pl-11 py-2 text-xs font-mono"
        >
      </div>

      <div class="flex items-center gap-2">
        <button
          @click="toggleSelectAll"
          class="kip-btn-ghost px-3 py-2 text-xs font-mono font-bold"
          :class="isAllSelected ? 'bg-indigo-500/20 text-indigo-400 border-indigo-500/40' : 'text-white/60'"
        >
          <CheckSquare class="w-3.5 h-3.5" />
          <span>{{ isAllSelected ? 'Deselect All' : 'Select All' }}</span>
        </button>

        <transition name="fade">
          <div v-if="selectedFilenames.size > 0" class="flex items-center gap-2">
            <button @click="batchToggle(true)" class="kip-btn-ghost px-3 py-2 text-xs text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/10">
              <ToggleRight class="w-3.5 h-3.5" /> Enable ({{ selectedFilenames.size }})
            </button>
            <button @click="batchToggle(false)" class="kip-btn-ghost px-3 py-2 text-xs text-amber-400 border-amber-500/30 hover:bg-amber-500/10">
              <ToggleLeft class="w-3.5 h-3.5" /> Disable ({{ selectedFilenames.size }})
            </button>
            <button @click="batchDelete" class="kip-btn-danger px-3 py-2 text-xs">
              <Trash2 class="w-3.5 h-3.5" /> Delete ({{ selectedFilenames.size }})
            </button>
          </div>
        </transition>
      </div>
    </div>

    <div v-show="!isGraphView" class="grid grid-cols-2 gap-4 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10">
      <template v-if="isContentLoading">
        <div v-for="i in 8" :key="i" class="kip-card p-4 flex items-center gap-4">
          <div class="w-14 h-14 rounded-2xl skeleton-box shrink-0"></div>
          <div class="flex-1 space-y-2">
            <div class="h-4 w-3/4 rounded skeleton-box"></div>
            <div class="h-3 w-1/2 rounded skeleton-box"></div>
          </div>
        </div>
      </template>

      <template v-else>
        <div v-if="filteredContent.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
          <SearchX class="w-16 h-16 text-white/20" />
          <span class="text-white/40 font-mono text-xs">{{ t('No packages found matching current filter constraints.') }}</span>
        </div>

        <div
          v-for="item in filteredContent"
          :key="item.filename"
          @click="openContentDetails(item)"
          class="kip-card p-4 flex items-center gap-4 cursor-pointer relative overflow-hidden group transition-all duration-300 border border-white/5 hover:border-indigo-500/30 hover:bg-black/50"
          :class="[item.disabled ? 'opacity-40 grayscale hover:grayscale-0 hover:opacity-100' : '', selectedFilenames.has(item.filename) ? 'border-indigo-500/60 bg-indigo-500/10' : '']"
        >
          <div
            @click.stop="toggleItemSelection(item.filename)"
            class="w-5 h-5 rounded-lg border flex items-center justify-center transition shrink-0"
            :class="selectedFilenames.has(item.filename) ? 'bg-indigo-500 border-indigo-400 text-white' : 'border-white/20 bg-black/40 hover:border-white/40'"
          >
            <Check v-if="selectedFilenames.has(item.filename)" class="w-3.5 h-3.5 stroke-[3]" />
          </div>

          <img
            :src="item.icon || fallbackModIcon"
            class="w-14 h-14 rounded-2xl object-cover bg-black/60 p-1 border border-white/10 shadow-lg shrink-0 group-hover:scale-105 transition-transform duration-300"
          >

          <div class="flex-1 min-w-0">
            <div class="flex items-center justify-between gap-2 mb-0.5">
              <h3 class="font-black text-white truncate text-base leading-tight">{{ item.name }}</h3>
              <span class="text-[9px] font-mono text-white/40 shrink-0">{{ formatBytes(item.size_bytes) }}</span>
            </div>

            <p class="text-xs text-white/50 truncate font-mono mb-2">
              {{ item.version }} • {{ item.author }}
            </p>

            <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-[22px]">
              <span
                v-for="loader in item.loaders"
                :key="loader"
                class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider border"
                :class="getLoaderBadgeClass(loader)"
              >
                {{ loader }}
              </span>
              <span v-if="item.dependencies.length > 0" class="px-2 py-0.5 rounded text-[8px] font-mono font-bold bg-white/5 text-white/40 border border-white/10">
                {{ item.dependencies.length }} deps
              </span>
            </div>
          </div>

          <div class="flex flex-col items-end gap-3 shrink-0 relative z-10 pr-1">
            <div
              class="relative inline-block w-10 align-middle select-none cursor-pointer"
              @click.stop="toggleContentState(item)"
            >
              <input
                type="checkbox"
                class="toggle-checkbox absolute block w-5 h-5 rounded-full bg-white border-4 appearance-none transition-transform"
                :checked="!item.disabled"
                style="pointer-events: none;"
              >
              <label class="toggle-label block overflow-hidden h-5 rounded-full transition-colors border border-white/10" style="pointer-events: none;"></label>
            </div>

            <button
              @click.stop="deleteContentItem(item)"
              class="text-white/30 hover:text-red-400 transition p-1"
              :title="t('Delete package')"
            >
              <Trash2 class="w-4 h-4" />
            </button>
          </div>
        </div>
      </template>
    </div>

    <div v-show="isGraphView" class="w-full h-full kip-card relative overflow-hidden shrink-0 min-h-[500px] border border-white/10">
      <div ref="graphContainer" class="w-full h-full"></div>
      <div class="absolute top-4 left-4 kip-card p-4 text-xs text-white/70 z-10 flex flex-col gap-2.5 pointer-events-none bg-black/60 border border-white/10">
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-emerald-500 shadow-[0_0_8px_rgba(16,185,129,0.8)]"></span> <span class="font-bold font-mono">Requires</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.8)]"></span> <span class="font-bold font-mono">Incompatible</span></div>
      </div>
    </div>

    <transition name="fade">
      <div
        v-if="contentDetailsModal.isOpen"
        class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
        @click.self="contentDetailsModal.isOpen = false"
      >
        <div class="kip-card p-0 w-full max-w-3xl flex flex-col max-h-[85vh] overflow-hidden relative border border-white/10 shadow-[0_0_60px_rgba(0,0,0,0.8)]">
          <div class="flex justify-between items-start p-8 border-b border-white/5 bg-black/60 shrink-0 relative overflow-hidden">
            <div class="absolute -top-32 -right-32 w-96 h-96 bg-indigo-500/10 blur-[100px] rounded-full pointer-events-none"></div>

            <div class="flex items-center gap-5 relative z-10">
              <img
                :src="contentDetailsModal.item?.icon || fallbackModIcon"
                class="w-20 h-20 rounded-2xl bg-black/60 p-1 object-cover border border-white/10 shadow-2xl"
              >
              <div>
                <h3 class="text-3xl font-black text-white mb-1 leading-tight">{{ contentDetailsModal.item?.name }}</h3>
                <div class="flex items-center gap-3">
                  <span class="text-xs font-mono text-indigo-400 bg-indigo-500/10 px-2.5 py-0.5 rounded border border-indigo-500/20 font-bold">
                    {{ contentDetailsModal.item?.version }}
                  </span>
                  <span class="text-xs text-white/50">Developer: <span class="text-white font-bold">{{ contentDetailsModal.item?.author }}</span></span>
                </div>
              </div>
            </div>

            <button @click="contentDetailsModal.isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition relative z-10">
              <X class="w-5 h-5" />
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#030305]/95 flex flex-col gap-6 min-h-0">
            <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
              <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-1.5 flex items-center gap-2">
                <FileText class="w-3.5 h-3.5 text-indigo-400" /> Package Description
              </h4>
              <p class="text-xs text-white/80 leading-relaxed font-medium">
                {{ contentDetailsModal.item?.description || 'No descriptive payload provided inside manifest.' }}
              </p>
            </div>

            <div class="grid grid-cols-2 gap-4">
              <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
                <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-2 flex items-center gap-2">
                  <Layers class="w-3.5 h-3.5 text-amber-400" /> Target Loaders
                </h4>
                <div class="flex gap-2 flex-wrap">
                  <span
                    v-for="loader in contentDetailsModal.item?.loaders || []"
                    :key="loader"
                    class="px-2.5 py-1 rounded text-[10px] font-black uppercase tracking-wider border"
                    :class="getLoaderBadgeClass(loader)"
                  >
                    {{ loader }}
                  </span>
                </div>
              </div>

              <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
                <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-2 flex items-center gap-2">
                  <Clock class="w-3.5 h-3.5 text-cyan-400" /> File System Data
                </h4>
                <p class="font-mono text-xs text-white/70">{{ formatBytes(contentDetailsModal.item?.size_bytes || 0) }} • {{ contentDetailsModal.item?.date_modified }}</p>
              </div>
            </div>

            <div v-if="contentDetailsModal.item && contentDetailsModal.item.dependencies.length > 0">
              <h4 class="text-[10px] font-mono font-bold text-emerald-400 uppercase tracking-widest mb-2 flex items-center gap-2">
                <ArrowUpCircle class="w-3.5 h-3.5" /> Required Runtime Modules
              </h4>
              <div class="flex gap-2 flex-wrap">
                <span
                  v-for="dep in contentDetailsModal.item.dependencies"
                  :key="dep"
                  class="bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-3 py-1 rounded-xl text-xs font-mono font-bold"
                >
                  {{ dep }}
                </span>
              </div>
            </div>

            <div class="p-4 bg-black/40 border border-white/5 rounded-2xl">
              <h4 class="text-[10px] font-mono font-bold text-white/40 uppercase tracking-widest mb-1.5 flex items-center gap-2">
                <Box class="w-3.5 h-3.5 text-purple-400" /> Physical Location
              </h4>
              <p class="font-mono text-xs text-white/50 select-all">{{ contentDetailsModal.item?.filename }}</p>
            </div>
          </div>

          <div class="p-6 border-t border-white/5 bg-black/60 shrink-0 flex justify-between items-center relative z-20">
            <button
              v-if="contentDetailsModal.item"
              @click="toggleContentState(contentDetailsModal.item)"
              class="px-6 py-2.5 rounded-xl font-black text-xs uppercase tracking-wider transition flex items-center gap-2 border"
              :class="contentDetailsModal.item.disabled ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30 hover:bg-emerald-500/20' : 'bg-amber-500/10 text-amber-400 border-amber-500/30 hover:bg-amber-500/20'"
            >
              {{ contentDetailsModal.item.disabled ? t('ENABLE') : t('Disable') }}
            </button>

            <div class="flex gap-3">
              <button @click="contentDetailsModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">{{ t('Close') }}</button>
              <button
                v-if="contentDetailsModal.item"
                @click="deleteContentItem(contentDetailsModal.item); contentDetailsModal.isOpen = false"
                class="kip-btn-danger px-6 py-2.5 text-xs font-black uppercase tracking-wider"
              >
                <Trash2 class="w-4 h-4" /> {{ t('Delete') }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div
        v-if="hubModal.isOpen"
        class="fixed inset-0 bg-black/85 backdrop-blur-xl z-[200] flex items-center justify-center p-8 cursor-default"
        @click.self="hubModal.isOpen = false"
      >
        <div class="kip-card p-0 w-full max-w-4xl border-blue-500/30 flex flex-col h-[85vh] relative overflow-hidden shadow-[0_0_60px_rgba(59,130,246,0.2)]">
          <div class="p-8 border-b border-white/5 flex justify-between items-center bg-black/50 shrink-0 relative z-10">
            <div class="flex items-center gap-4">
              <div class="p-3 bg-blue-500/20 rounded-2xl border border-blue-500/30 text-blue-400">
                <Globe class="w-6 h-6" />
              </div>
              <div>
                <h3 class="text-2xl font-black uppercase text-white tracking-wider">Community Hub Registry</h3>
                <span class="text-xs font-mono text-blue-400">Cloudflare Edge KV Presets & Distributed Configuration Profiles</span>
              </div>
            </div>
            <button @click="hubModal.isOpen = false" class="p-2 text-white/40 hover:text-white rounded-xl transition">
              <X class="w-5 h-5" />
            </button>
          </div>

          <div class="flex border-b border-white/5 bg-black/30 shrink-0 relative z-10">
            <button
              @click="hubModal.tab = 'browse'; loadHubPresets()"
              :class="hubModal.tab === 'browse' ? 'text-blue-400 border-b-2 border-blue-400 bg-blue-500/5' : 'text-white/40 hover:text-white'"
              class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider"
            >
              <Search class="w-4 h-4" /> Browse Shared Profiles ({{ hubModal.presets.length }})
            </button>
            <button
              @click="hubModal.tab = 'publish'"
              :class="hubModal.tab === 'publish' ? 'text-blue-400 border-b-2 border-blue-400 bg-blue-500/5' : 'text-white/40 hover:text-white'"
              class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider"
            >
              <UploadCloud class="w-4 h-4" /> Publish Active Instance Profile
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll p-8 bg-[#030305]/95 relative z-10 min-h-0">
            <div v-if="hubModal.tab === 'browse'" class="h-full">
              <div v-if="hubModal.loading" class="flex flex-col justify-center items-center h-full gap-3 py-20">
                <Loader class="w-8 h-8 animate-spin text-blue-500" />
                <span class="font-mono text-xs text-white/40 uppercase">Connecting to Cloudflare KV Edge...</span>
              </div>

              <div v-else-if="hubModal.presets.length === 0" class="flex flex-col justify-center items-center h-full text-white/40 py-20 border border-dashed border-white/10 rounded-3xl">
                <CloudOff class="w-14 h-14 mb-3 opacity-40" />
                <p class="font-mono text-xs">Community preset repository is empty. Be the first to publish.</p>
              </div>

              <div v-else class="grid grid-cols-2 gap-5">
                <div v-for="preset in hubModal.presets" :key="preset.id" class="kip-card p-6 kip-card-hover group border border-white/5 hover:border-blue-500/30 flex flex-col justify-between">
                  <div>
                    <div class="flex items-center justify-between gap-2 mb-1.5">
                      <h4 class="font-black text-lg text-white leading-tight">{{ preset.title }}</h4>
                      <span class="px-2 py-0.5 rounded bg-blue-500/10 text-blue-400 border border-blue-500/20 text-[9px] font-mono font-bold">{{ preset.preset.length }} mods</span>
                    </div>

                    <p class="text-xs text-white/40 mb-3 font-mono">Curated by <span class="text-white font-bold">{{ preset.author }}</span></p>
                    <p class="text-xs text-white/70 line-clamp-3 leading-relaxed mb-5">{{ preset.description }}</p>
                  </div>

                  <button @click="applyHubPreset(preset)" class="w-full py-2.5 bg-blue-500/10 hover:bg-blue-500 hover:text-white text-blue-400 font-black text-xs uppercase tracking-wider rounded-xl transition flex items-center justify-center gap-2 border border-blue-500/30 shadow-lg">
                    <DownloadCloud class="w-4 h-4" />
                    <span>Apply Preset to Instance</span>
                  </button>
                </div>
              </div>
            </div>

            <div v-if="hubModal.tab === 'publish'" class="max-w-xl mx-auto h-full flex flex-col justify-center">
              <div class="kip-card p-8 flex flex-col gap-5 border border-white/10 bg-black/60 shadow-2xl">
                <div>
                  <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Profile Title</label>
                  <input v-model="hubModal.form.title" type="text" placeholder="Medieval Fantasy Exploration Stack" class="kip-input text-xs font-bold">
                </div>

                <div>
                  <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Author Handle</label>
                  <input v-model="hubModal.form.author" type="text" placeholder="Operator Name" class="kip-input text-xs font-mono">
                </div>

                <div>
                  <label class="text-[10px] font-mono uppercase text-white/40 font-bold block mb-1.5">Description & Modpack Notes</label>
                  <textarea v-model="hubModal.form.desc" placeholder="Comprehensive description, shaders support, and key highlights..." class="kip-input h-32 resize-none custom-scroll text-xs leading-relaxed"></textarea>
                </div>

                <div class="p-3 bg-black/40 rounded-xl border border-white/5">
                  <span class="text-[9px] font-mono text-white/40 uppercase font-bold block mb-1">Payload Synchronizer:</span>
                  <span class="text-xs font-mono text-blue-400 font-bold">{{ contentList.filter(m => !m.disabled).length }} active mods will be registered in this preset payload.</span>
                </div>

                <button @click="publishToHub" :disabled="hubModal.loading || !hubModal.form.title.trim()" class="kip-btn-primary py-3.5 bg-blue-500 hover:bg-blue-400 text-white font-black text-xs uppercase tracking-wider shadow-[0_0_20px_rgba(59,130,246,0.3)]">
                  <Loader v-if="hubModal.loading" class="w-4 h-4 animate-spin" />
                  <UploadCloud v-else class="w-4 h-4" />
                  <span>Publish Profile to Global Hub</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>

    <transition name="fade">
      <div
        v-if="p2pModal.isOpen"
        class="fixed inset-0 bg-black/85 backdrop-blur-xl z-[200] flex items-center justify-center p-8 cursor-default"
        @click.self="p2pModal.isOpen = false"
      >
        <div class="kip-card p-8 w-full max-w-xl border-amber-500/30 flex flex-col relative overflow-hidden shadow-[0_0_60px_rgba(245,158,11,0.2)]">
          <div class="flex items-center gap-4 mb-4 relative z-10">
            <div class="p-3 bg-amber-500/20 border border-amber-500/40 rounded-2xl text-amber-400">
              <Magnet class="w-6 h-6" />
            </div>
            <div>
              <h3 class="text-2xl font-black uppercase text-white tracking-wider">Swarm P2P Acceleration</h3>
              <p class="text-xs text-white/40 font-mono">BitTorrent Mesh Ingestion & Peer-to-Peer Instance Synchronization</p>
            </div>
          </div>

          <div class="flex flex-col gap-4 relative z-10 my-4">
            <label class="text-[10px] font-mono uppercase text-white/40 font-bold block">BitTorrent Magnet URI</label>
            <input
              v-model="p2pModal.magnet"
              type="text"
              placeholder="magnet:?xt=urn:btih:..."
              class="kip-input font-mono text-xs text-amber-400 focus:border-amber-500"
            >

            <div v-if="p2pModal.activeSeeds.length > 0" class="flex flex-col gap-2 p-3 bg-black/40 rounded-xl border border-white/5">
              <span class="text-[9px] font-mono uppercase text-amber-400 font-bold">Active P2P Seeding Torrents:</span>
              <div v-for="seed in p2pModal.activeSeeds" :key="seed.name" class="flex justify-between items-center text-xs font-mono">
                <span class="text-white/80 truncate max-w-xs">{{ seed.name }}</span>
                <span class="text-amber-400">{{ formatBytes(seed.total_upload) }} uploaded</span>
              </div>
            </div>
          </div>

          <div class="flex justify-end gap-3 relative z-10 pt-2 border-t border-white/5">
            <button @click="p2pModal.isOpen = false" class="kip-btn-ghost px-6 py-2.5 text-xs font-bold uppercase">{{ t('Cancel') }}</button>
            <button
              @click="startSwarmDownload"
              :disabled="!p2pModal.magnet.trim() || p2pModal.loading"
              class="kip-btn-primary px-8 py-2.5 bg-amber-500 hover:bg-amber-400 text-black border-amber-400 shadow-[0_0_20px_rgba(245,158,11,0.3)] font-black text-xs uppercase tracking-wider"
            >
              <Loader v-if="p2pModal.loading" class="w-4 h-4 animate-spin" />
              <Download v-else class="w-4 h-4 fill-current" />
              <span>Initiate P2P Download</span>
            </button>
          </div>
        </div>
      </div>
    </transition>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch, nextTick } from 'vue'
import type { Component } from 'vue'
import { Network } from 'vis-network'
import {
  FolderOpen,
  ArrowUpCircle,
  Package,
  LayoutGrid,
  GitMerge,
  RefreshCw,
  Search,
  SearchX,
  Check,
  CheckSquare,
  ToggleRight,
  ToggleLeft,
  Trash2,
  X,
  FileText,
  Layers,
  Clock,
  Box,
  Palette,
  Sun,
  Loader,
  Globe,
  Magnet,
  Download,
  UploadCloud,
  CloudOff,
  DownloadCloud,
  PackagePlus,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type LocalModRecord,
  type ModGraphDataDto,
  type GenericActionResult,
  type HubPreset,
  type SwarmStatusDto,
  type ImportDroppedModsResult,
} from '@/bridge'

type ContentTabType = 'mods' | 'resourcepacks' | 'shaderpacks'

const activeContentTab = ref<ContentTabType>('mods')
const contentList = ref<LocalModRecord[]>([])
const isContentLoading = ref<boolean>(true)
const isGraphView = ref<boolean>(false)
const isCheckingUpdates = ref<boolean>(false)
const isExporting = ref<boolean>(false)
const isImporting = ref<boolean>(false)
const isDraggingOver = ref<boolean>(false)
const localSearchQuery = ref<string>('')
const statusFilter = ref<'all' | 'active' | 'disabled'>('all')
const sortBy = ref<'name' | 'size' | 'date'>('name')
const selectedFilenames = ref<Set<string>>(new Set())

const graphContainer = ref<HTMLElement | null>(null)
let networkInstance: Network | null = null

const contentDetailsModal = ref<{
  isOpen: boolean
  item: LocalModRecord | null
}>({
  isOpen: false,
  item: null,
})

const hubModal = ref<{
  isOpen: boolean
  tab: 'browse' | 'publish'
  presets: HubPreset[]
  loading: boolean
  form: {
    title: string
    author: string
    desc: string
  }
}>({
  isOpen: false,
  tab: 'browse',
  presets: [],
  loading: false,
  form: {
    title: '',
    author: state.settings.ms_name || state.settings.offline_username || 'Operator',
    desc: '',
  },
})

const p2pModal = ref<{
  isOpen: boolean
  magnet: string
  loading: boolean
  activeSeeds: SwarmStatusDto[]
}>({
  isOpen: false,
  magnet: '',
  loading: false,
  activeSeeds: [],
})

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

const getContentTabIcon = (tab: ContentTabType): Component => {
  switch (tab) {
    case 'resourcepacks':
      return Palette
    case 'shaderpacks':
      return Sun
    default:
      return Box
  }
}

const getContentTabLabel = (tab: ContentTabType): string => {
  switch (tab) {
    case 'resourcepacks':
      return 'Resourcepacks'
    case 'shaderpacks':
      return 'Shaders'
    default:
      return 'Modules'
  }
}

const formatBytes = (bytes: number): string => {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

const getLoaderBadgeClass = (loader: string): string => {
  const l = loader.toLowerCase()
  if (l === 'fabric') return 'bg-amber-500/10 text-amber-400 border-amber-500/20'
  if (l === 'forge') return 'bg-rose-500/10 text-rose-400 border-rose-500/20'
  if (l === 'neoforge') return 'bg-orange-500/10 text-orange-400 border-orange-500/20'
  if (l === 'quilt') return 'bg-purple-500/10 text-purple-400 border-purple-500/20'
  return 'bg-white/10 text-white/70 border-white/20'
}

const isAllSelected = computed<boolean>(() => {
  return filteredContent.value.length > 0 && selectedFilenames.value.size === filteredContent.value.length
})

const filteredContent = computed<LocalModRecord[]>(() => {
  let list = [...contentList.value]
  const q = localSearchQuery.value.trim().toLowerCase()

  if (q) {
    list = list.filter(
      (m) =>
        m.name.toLowerCase().includes(q) ||
        m.filename.toLowerCase().includes(q) ||
        m.author.toLowerCase().includes(q) ||
        m.id.toLowerCase().includes(q)
    )
  }

  if (statusFilter.value === 'active') {
    list = list.filter((m) => !m.disabled)
  } else if (statusFilter.value === 'disabled') {
    list = list.filter((m) => m.disabled)
  }

  if (sortBy.value === 'size') {
    list.sort((a, b) => b.size_bytes - a.size_bytes)
  } else if (sortBy.value === 'date') {
    list.sort((a, b) => b.date_modified.localeCompare(a.date_modified))
  } else {
    list.sort((a, b) => a.name.localeCompare(b.name))
  }

  return list
})

const loadContent = async (): Promise<void> => {
  isContentLoading.value = true
  try {
    contentList.value = await bridge.getLocalMods(activeContentTab.value)
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isContentLoading.value = false
  }
}

const toggleContentState = async (item: LocalModRecord): Promise<void> => {
  try {
    const success = await bridge.toggleMod(item.filename, activeContentTab.value)
    if (success) {
      item.disabled = !item.disabled
      item.filename = item.disabled ? `${item.filename}.disabled` : item.filename.replace('.disabled', '')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const deleteContentItem = async (item: LocalModRecord): Promise<void> => {
  try {
    const res: GenericActionResult = await bridge.deleteMod(item.filename, activeContentTab.value)
    if (res.success) {
      showToast(t('Deleted'), res.msg, 'success')
      selectedFilenames.value.delete(item.filename)
      await loadContent()
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const toggleItemSelection = (filename: string): void => {
  if (selectedFilenames.value.has(filename)) {
    selectedFilenames.value.delete(filename)
  } else {
    selectedFilenames.value.add(filename)
  }
}

const toggleSelectAll = (): void => {
  if (isAllSelected.value) {
    selectedFilenames.value.clear()
  } else {
    selectedFilenames.value = new Set(filteredContent.value.map((m) => m.filename))
  }
}

const batchToggle = async (enable: boolean): Promise<void> => {
  if (selectedFilenames.value.size === 0) return
  try {
    const targets = Array.from(selectedFilenames.value)
    const count = await bridge.batchToggleMods(targets, enable, activeContentTab.value)
    showToast(t('Batch Operation'), `${enable ? 'Activated' : 'Suspended'} ${count} packages.`, 'success')
    selectedFilenames.value.clear()
    await loadContent()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const batchDelete = async (): Promise<void> => {
  if (selectedFilenames.value.size === 0) return
  try {
    const targets = Array.from(selectedFilenames.value)
    const count = await bridge.batchDeleteMods(targets, activeContentTab.value)
    showToast(t('Batch Operation'), `Purged ${count} packages from disk.`, 'success')
    selectedFilenames.value.clear()
    await loadContent()
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const openCurrentFolder = async (): Promise<void> => {
  try {
    await bridge.openContentFolder(activeContentTab.value)
  } catch {
    showToast(t('Error'), 'Could not invoke native file manager.', 'danger')
  }
}

const openContentDetails = (item: LocalModRecord): void => {
  contentDetailsModal.value.item = item
  contentDetailsModal.value.isOpen = true
}

const importContentDialog = async (): Promise<void> => {
  if (isImporting.value) return
  isImporting.value = true
  try {
    const selected = await bridge.pickFile()
    if (selected && selected.trim()) {
      showToast(t('Importing'), 'Integrating content payload...', 'info')
      const res = await bridge.importDroppedMods([selected.trim()])
      if (res.success) {
        showToast(t('Imported'), `Successfully imported package into active instance.`, 'success')
        await loadContent()
      } else {
        showToast(t('Error'), res.msg || 'Import failed.', 'danger')
      }
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isImporting.value = false
  }
}

const handleDirectDrop = async (e: DragEvent): Promise<void> => {
  isDraggingOver.value = false
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const filePaths = Array.from(e.dataTransfer.files)
      .map((f) => (f as unknown as { path?: string }).path)
      .filter((p): p is string => typeof p === 'string' && p.length > 0)

    if (filePaths.length > 0) {
      try {
        const res: ImportDroppedModsResult = await bridge.importDroppedMods(filePaths)
        if (res.success) {
          showToast(t('Imported'), `Integrated ${res.count} archives into instance.`, 'success')
          await loadContent()
        }
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : String(err)
        showToast(t('Error'), msg, 'danger')
      }
    }
  }
}

const checkModUpdates = async (): Promise<void> => {
  if (isCheckingUpdates.value) return
  isCheckingUpdates.value = true
  showToast(t('Scanning'), t('Checking for mod updates...'), 'info')

  try {
    const res = await bridge.checkModUpdates()
    if (res.success && res.updates && res.updates.length > 0) {
      showToast(t('Updating'), `Downloading ${res.updates.length} updates...`, 'info')
      const applyRes = await bridge.applyModUpdates(res.updates)
      if (applyRes.success) {
        showToast(t('Success'), applyRes.msg, 'success')
        await loadContent()
      } else {
        showToast(t('Error'), applyRes.msg, 'danger')
      }
    } else if (res.success) {
      showToast(t('Up to date'), t('All mods are on the latest version.'), 'success')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isCheckingUpdates.value = false
  }
}

const exportModpack = async (): Promise<void> => {
  if (isExporting.value) return
  isExporting.value = true
  showToast(t('Exporting'), t('Generating modpack archive...'), 'info')

  try {
    const res: GenericActionResult = await bridge.exportModpack()
    if (res.success) {
      showToast(t('Exported'), res.msg, 'success')
    } else {
      showToast(t('Error'), res.msg, 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    isExporting.value = false
  }
}

const openHub = (): void => {
  hubModal.value.isOpen = true
  hubModal.value.tab = 'browse'
  loadHubPresets()
}

const loadHubPresets = async (): Promise<void> => {
  hubModal.value.loading = true
  try {
    const presets = await bridge.fetchHub()
    hubModal.value.presets = presets || []
  } catch {
    hubModal.value.presets = []
  } finally {
    hubModal.value.loading = false
  }
}

const publishToHub = async (): Promise<void> => {
  if (!hubModal.value.form.title.trim()) return
  hubModal.value.loading = true

  try {
    const activeMods = contentList.value
      .filter((m) => !m.disabled)
      .map((m) => m.name || m.filename)

    if (activeMods.length === 0) {
      showToast(t('Error'), t('No active mods found to share.'), 'danger')
      hubModal.value.loading = false
      return
    }

    const success = await bridge.publishHub(
      hubModal.value.form.title,
      hubModal.value.form.author,
      hubModal.value.form.desc,
      activeMods
    )

    if (success) {
      showToast(t('Success'), t('Profile published to Hub!'), 'success')
      hubModal.value.form.title = ''
      hubModal.value.form.desc = ''
      hubModal.value.tab = 'browse'
      await loadHubPresets()
    } else {
      showToast(t('Error'), t('Failed to publish profile.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    hubModal.value.loading = false
  }
}

const applyHubPreset = async (preset: HubPreset): Promise<void> => {
  hubModal.value.isOpen = false
  showToast(t('Applying'), `Downloading preset: ${preset.title}`, 'info')

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
      showToast(t('Success'), `Downloaded ${res.downloaded || 0} mods.`, 'success')
      await loadContent()
    } else {
      showToast(t('Error'), t('Failed to download preset mods.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

const openSwarm = async (): Promise<void> => {
  p2pModal.value.isOpen = true
  p2pModal.value.magnet = ''
  try {
    p2pModal.value.activeSeeds = await bridge.swarmSeedStatus()
  } catch {
    p2pModal.value.activeSeeds = []
  }
}

const startSwarmDownload = async (): Promise<void> => {
  if (!p2pModal.value.magnet.trim()) return
  p2pModal.value.loading = true

  try {
    const res = await bridge.swarmDownload(p2pModal.value.magnet.trim(), 'MODS_DIR')
    if (res && res.success) {
      showToast(t('Swarm'), t('P2P download started...'), 'info')
      p2pModal.value.isOpen = false
    } else {
      showToast(t('Error'), res?.msg || t('Swarm download failed.'), 'danger')
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  } finally {
    p2pModal.value.loading = false
  }
}

const renderGraph = async (): Promise<void> => {
  if (!graphContainer.value) return
  try {
    const data = await invokeSafe<ModGraphDataDto>('get_mod_graph_data')
    if (networkInstance) {
      networkInstance.destroy()
      networkInstance = null
    }

    networkInstance = new Network(
      graphContainer.value,
      {
        nodes: data.nodes,
        edges: data.edges,
      },
      {
        nodes: { font: { color: '#ffffff' }, borderWidth: 2 },
        edges: {
          smooth: {
            enabled: true,
            type: 'continuous',
            roundness: 0.5,
          },
        },
        physics: {
          barnesHut: { gravitationalConstant: -2000, centralGravity: 0.3, springLength: 95 },
        },
      }
    )
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

watch(isGraphView, (val) => {
  if (val) {
    nextTick(() => {
      renderGraph()
    })
  } else if (networkInstance) {
    networkInstance.destroy()
    networkInstance = null
  }
})

onMounted(() => {
  loadContent()
})

onBeforeUnmount(() => {
  if (networkInstance) {
    networkInstance.destroy()
    networkInstance = null
  }
})
</script>