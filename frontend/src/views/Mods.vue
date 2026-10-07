<script setup lang="ts">
import { ref, onMounted } from 'vue'
import type { Component } from 'vue'
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
  Trash2,
  Box,
  Palette,
  Sun,
  Loader,
  Download,
  PackagePlus,
  Archive,
  Globe,
  Magnet,
  DownloadCloud,
} from 'lucide-vue-next'
import { t, showToast } from '@/store'
import { bridge, type LocalModRecord } from '@/bridge'
import { useContentManager } from '../composables/useContentManager'
import type { ContentTabType, ModpackRecordDto } from '../types/content'

import ModGraphView from '@/components/content/ModGraphView.vue'
import CommunityHubModal from '@/components/content/CommunityHubModal.vue'
import SwarmModal from '@/components/content/SwarmModal.vue'
import ContentDetailsModal from '@/components/content/ContentDetailsModal.vue'
import ContentUpdatesModal from '@/components/content/ContentUpdatesModal.vue'

const isGraphView = ref(false)

const {
  activeContentTab,
  modpacksList,
  isContentLoading,
  isCheckingUpdates,
  isExporting,
  isImporting,
  isDraggingOver,
  localSearchQuery,
  statusFilter,
  sortBy,
  selectedFilenames,
  availableUpdatesList,
  isUpdatesModalOpen,
  isApplyingUpdates,
  updateProgressPercent,
  updateCurrentFileName,
  isHubModalOpen,
  hubPresets,
  isHubLoading,
  isSwarmModalOpen,
  swarmSeeds,
  isDetailsModalOpen,
  selectedInspectItem,
  filteredContent,
  loadContent,
  toggleContentState,
  deleteContentItem,
  batchToggle,
  batchDelete,
  checkModUpdates,
  applyUpdatesBatch,
  mountModpack,
  exportModpack,
  importDroppedFiles,
  openHub,
  openSwarm,
  inspectItem,
} = useContentManager()

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

function getContentTabIcon(tab: ContentTabType): Component {
  switch (tab) {
    case 'modpacks':
      return Archive
    case 'resourcepacks':
      return Palette
    case 'shaderpacks':
      return Sun
    default:
      return Box
  }
}

function getContentTabLabel(tab: ContentTabType): string {
  switch (tab) {
    case 'modpacks':
      return 'Modpacks'
    case 'resourcepacks':
      return 'Resourcepacks'
    case 'shaderpacks':
      return 'Shaders'
    default:
      return 'Modules'
  }
}

function formatBytes(bytes: number): string {
  if (!bytes || bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(1))} ${sizes[i]}`
}

function toggleItemSelection(filename: string): void {
  if (selectedFilenames.value.has(filename)) {
    selectedFilenames.value.delete(filename)
  } else {
    selectedFilenames.value.add(filename)
  }
}

async function openCurrentFolder(): Promise<void> {
  try {
    await bridge.openContentFolder(activeContentTab.value)
  } catch {
    showToast(t('Error'), 'Could not launch explorer', 'danger')
  }
}

async function handleDirectDrop(e: DragEvent): Promise<void> {
  isDraggingOver.value = false
  if (e.dataTransfer?.files && e.dataTransfer.files.length > 0) {
    const filePaths = Array.from(e.dataTransfer.files)
      .map((f) => (f as unknown as { path?: string }).path)
      .filter((p): p is string => typeof p === 'string' && p.length > 0)

    if (filePaths.length > 0) {
      await importDroppedFiles(filePaths)
    }
  }
}

async function importContentDialog(): Promise<void> {
  if (isImporting.value) return
  isImporting.value = true
  try {
    const selected = await bridge.pickFile()
    if (selected && selected.trim()) {
      await importDroppedFiles([selected.trim()])
    }
  } finally {
    isImporting.value = false
  }
}

onMounted(() => {
  loadContent()
})
</script>

<template>
  <div
    class="h-full flex flex-col min-h-0 relative select-none"
    @dragover.prevent="isDraggingOver = true"
    @dragleave.prevent="isDraggingOver = false"
    @drop.prevent="handleDirectDrop"
  >
    <!-- Drag Hover Overlay -->
    <transition name="fade">
      <div
        v-if="isDraggingOver"
        class="absolute inset-0 bg-indigo-950/90 backdrop-blur-md z-50 flex flex-col items-center justify-center border-4 border-dashed border-indigo-400 m-4 rounded-3xl pointer-events-none shadow-[0_0_80px_rgba(99,102,241,0.5)]"
      >
        <PackagePlus class="w-20 h-20 text-indigo-400 animate-bounce mb-4" />
        <h3 class="text-3xl font-black text-white uppercase tracking-wider">Drop Content Here</h3>
        <p class="text-sm text-indigo-200 mt-2 font-mono">Mod JARs, Modpack .mrpack, Resource Packs or Shaders will be categorized automatically</p>
      </div>
    </transition>

    <!-- Top Action Bar -->
    <header class="flex justify-between items-start mb-6 shrink-0 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Content Manager') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Manifest Topology, Live Hash-Verifier & Neural Dependency Graph</p>
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

        <button @click="importContentDialog" :disabled="isImporting" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-cyan-400 border-cyan-500/20 hover:bg-cyan-500/20">
          <Loader v-if="isImporting" class="w-4 h-4 animate-spin" />
          <Download v-else class="w-4 h-4" />
          <span>{{ t('Import') }}</span>
        </button>

        <button @click="checkModUpdates" :disabled="isCheckingUpdates" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-emerald-400 border-emerald-500/20 hover:bg-emerald-500/20 relative">
          <Loader v-if="isCheckingUpdates" class="w-4 h-4 animate-spin" />
          <ArrowUpCircle v-else class="w-4 h-4" />
          <span>{{ t('Update') }}</span>
          <span v-if="availableUpdatesList.length > 0" class="absolute -top-1.5 -right-1.5 px-1.5 py-0.2 rounded-full text-[9px] font-mono font-black bg-emerald-500 text-black shadow-[0_0_8px_rgba(16,185,129,0.8)]">
            {{ availableUpdatesList.length }}
          </span>
        </button>

        <button @click="exportModpack" :disabled="isExporting" class="kip-btn-ghost px-3.5 py-2 text-xs font-bold uppercase tracking-wider text-purple-400 border-purple-500/20 hover:bg-purple-500/20">
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
    </header>

    <!-- Categories Strip & Filters -->
    <div class="flex justify-between items-center gap-4 mb-5 shrink-0 z-10">
      <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
        <button
          v-for="tab in (['mods', 'modpacks', 'resourcepacks', 'shaderpacks'] as const)"
          :key="tab"
          @click="activeContentTab = tab; selectedFilenames.clear(); loadContent()"
          class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2"
          :class="activeContentTab === tab ? 'bg-indigo-500/20 text-indigo-400 border border-indigo-500/30 shadow-[0_0_20px_rgba(99,102,241,0.2)]' : 'text-white/40 hover:text-white'"
        >
          <component :is="getContentTabIcon(tab)" class="w-3.5 h-3.5" />
          <span>{{ getContentTabLabel(tab) }}</span>
        </button>
      </div>

      <div class="flex items-center gap-2">
        <div class="relative flex-1">
          <Search class="absolute left-3 top-1/2 -translate-y-1/2 text-white/30 w-3.5 h-3.5" />
          <input
            v-model="localSearchQuery"
            type="text"
            placeholder="Filter active packages..."
            class="kip-input py-1.5 pl-8 text-xs font-mono w-48"
          >
        </div>

        <div v-if="activeContentTab !== 'modpacks'" class="flex p-1 bg-black/40 rounded-xl border border-white/10">
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

    <!-- View Mode: Dependency Graph or Matrix Grid -->
    <div v-if="isGraphView" class="flex-1 min-h-0 z-10">
      <ModGraphView />
    </div>

    <!-- View Mode: Modpacks -->
    <div v-else-if="activeContentTab === 'modpacks'" class="grid grid-cols-2 gap-4 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10">
      <div v-if="modpacksList.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
        <Archive class="w-16 h-16 text-white/20" />
        <span class="text-white/40 font-mono text-xs">No stored modpacks found in current instance. Drop .mrpack or .zip archives here.</span>
      </div>

      <div
        v-for="pack in modpacksList"
        :key="pack.filename"
        class="kip-card p-5 flex flex-col justify-between relative overflow-hidden group border border-white/5 hover:border-cyan-500/40 transition-all duration-300 bg-black/40"
      >
        <div>
          <div class="flex items-start justify-between gap-3 mb-2">
            <div class="flex items-center gap-3">
              <div class="p-3 rounded-2xl bg-cyan-500/10 border border-cyan-500/30 text-cyan-400 shrink-0">
                <Archive class="w-6 h-6" />
              </div>
              <div>
                <h3 class="font-black text-lg text-white leading-tight">{{ pack.name }}</h3>
                <span class="text-xs font-mono text-white/40">by <span class="text-white font-bold">{{ pack.author }}</span> • v{{ pack.version }}</span>
              </div>
            </div>
            <span class="text-[9px] font-mono text-white/30 shrink-0">{{ formatBytes(pack.sizeBytes) }}</span>
          </div>

          <p class="text-xs text-white/60 line-clamp-2 leading-relaxed font-sans mb-4">{{ pack.description }}</p>

          <div class="flex gap-2 mb-4 flex-wrap">
            <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-bold bg-cyan-500/10 text-cyan-300 border border-cyan-500/20">MC {{ pack.mcVersion }}</span>
            <span class="px-2.5 py-0.5 rounded text-[9px] font-black uppercase tracking-wider bg-white/5 text-white/70 border border-white/10">{{ pack.loader }}</span>
            <span class="px-2.5 py-0.5 rounded text-[9px] font-mono font-bold bg-purple-500/10 text-purple-300 border border-purple-500/20">{{ pack.modCount }} mods</span>
          </div>
        </div>

        <div class="pt-3 border-t border-white/5 flex items-center justify-between gap-3">
          <button @click="mountModpack(pack.filename)" class="kip-btn-primary flex-1 py-2 text-xs font-black uppercase tracking-wider bg-cyan-500 hover:bg-cyan-400 text-black border-cyan-400 shadow-[0_0_15px_rgba(6,182,212,0.3)]">
            <DownloadCloud class="w-3.5 h-3.5 fill-current" /> Mount Into Instance
          </button>
        </div>
      </div>
    </div>

    <!-- View Mode: Modules / Resourcepacks / Shaders -->
    <div v-else class="grid grid-cols-2 gap-4 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10">
      <div v-if="filteredContent.length === 0 && !isContentLoading" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
        <SearchX class="w-16 h-16 text-white/20" />
        <span class="text-white/40 font-mono text-xs">{{ t('No packages found matching current filter constraints.') }}</span>
      </div>

      <article
        v-for="item in filteredContent"
        :key="item.filename"
        @click="inspectItem(item)"
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
              v-for="ldr in item.loaders"
              :key="ldr"
              class="px-2 py-0.5 rounded text-[8px] font-black uppercase tracking-wider border bg-white/10 text-white/70 border-white/20"
            >
              {{ ldr }}
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
      </article>
    </div>

    <!-- Floating Batch Action Deck -->
    <div
      v-if="selectedFilenames.size > 0 && !isGraphView"
      class="fixed bottom-6 left-1/2 -translate-x-1/2 bg-black/80 backdrop-blur-2xl border border-indigo-500/40 px-6 py-3 rounded-2xl flex items-center gap-4 z-40 shadow-2xl"
    >
      <span class="text-xs font-mono font-bold text-white">{{ selectedFilenames.size }} selected</span>
      <div class="w-px h-4 bg-white/20"></div>
      <button @click="batchToggle(true)" class="kip-btn-ghost px-3 py-1.5 text-xs text-emerald-400 border-emerald-500/30">Enable</button>
      <button @click="batchToggle(false)" class="kip-btn-ghost px-3 py-1.5 text-xs text-amber-400 border-amber-500/30">Disable</button>
      <button @click="batchDelete" class="kip-btn-danger px-3 py-1.5 text-xs">Delete</button>
      <button @click="selectedFilenames.clear()" class="text-white/40 hover:text-white p-1">Clear</button>
    </div>

    <!-- Modals -->
    <CommunityHubModal
      v-model="isHubModalOpen"
      :presets="hubPresets"
      :loading="isHubLoading"
    />

    <SwarmModal
      v-model="isSwarmModalOpen"
      :active-seeds="swarmSeeds"
    />

    <ContentDetailsModal
      v-model="isDetailsModalOpen"
      :item="selectedInspectItem"
      @toggle="toggleContentState"
      @delete="deleteContentItem"
    />

    <ContentUpdatesModal
      v-model="isUpdatesModalOpen"
      :updates="availableUpdatesList"
      :is-applying="isApplyingUpdates"
      :progress="updateProgressPercent"
      :current-file="updateCurrentFileName"
      @apply="applyUpdatesBatch"
    />
  </div>
</template>