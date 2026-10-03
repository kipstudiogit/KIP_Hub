<template>
  <div class="h-full flex flex-col min-h-0 relative select-none">
    <div class="flex justify-between items-start shrink-0 mb-6 z-10">
      <div>
        <h2 class="text-3xl font-black uppercase tracking-tight text-white flex items-center gap-3">
          {{ t('Store') }}
        </h2>
        <p class="text-white/40 text-xs font-mono tracking-wider mt-0.5">Automated Content Registry, Satellite Modpacks & Dependency Resolver</p>
      </div>

      <div class="flex gap-2">
        <div class="flex p-1 bg-black/40 rounded-2xl border border-white/10 backdrop-blur-xl">
          <button
            @click="setProvider('modrinth')"
            class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2"
            :class="storeData.provider === 'modrinth' ? 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30 shadow-[0_0_20px_rgba(16,185,129,0.2)]' : 'text-white/40 hover:text-white'"
          >
            <Box class="w-3.5 h-3.5" /> Modrinth
          </button>
          <button
            @click="setProvider('curseforge')"
            class="px-5 py-2 rounded-xl text-xs font-black uppercase tracking-wider transition flex items-center gap-2"
            :class="storeData.provider === 'curseforge' ? 'bg-orange-500/20 text-orange-400 border border-orange-500/30 shadow-[0_0_20px_rgba(249,115,22,0.2)]' : 'text-white/40 hover:text-white'"
          >
            <Flame class="w-3.5 h-3.5" /> CurseForge
          </button>
        </div>
      </div>
    </div>

    <div class="flex gap-2 overflow-x-auto custom-scroll pb-2 mb-4 shrink-0 z-10">
      <button
        v-for="cat in categoryBadges"
        :key="cat.id"
        @click="setStoreCategory(cat.id)"
        class="px-4 py-1.5 rounded-full text-xs font-bold transition flex items-center gap-1.5 border"
        :class="storeData.category === cat.id ? 'bg-indigo-500 text-white border-indigo-400 shadow-[0_0_15px_rgba(99,102,241,0.5)]' : 'bg-white/5 border-white/5 text-white/60 hover:border-white/20 hover:text-white'"
      >
        <component :is="cat.icon" class="w-3 h-3" />
        <span>{{ t(cat.name) }}</span>
      </button>
      <button
        v-if="storeData.category"
        @click="setStoreCategory('')"
        class="px-3 py-1.5 rounded-full text-xs font-bold bg-red-500/20 text-red-400 border border-red-500/30 hover:bg-red-500/30 flex items-center"
      >
        <X class="w-3 h-3" />
      </button>
    </div>

    <div class="flex gap-3 mb-6 shrink-0 relative z-30">
      <div class="relative flex-1 max-w-[170px] custom-dropdown">
        <div
          @click="activeDropdown = activeDropdown === 'type' ? null : 'type'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer transition hover:border-white/30"
          :class="activeDropdown === 'type' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ typeOptions.find(o => o.value === storeData.type)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2 shrink-0" :class="{'rotate-180': activeDropdown === 'type'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'type'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
            <div
              v-for="opt in typeOptions"
              :key="opt.value"
              @click="storeData.type = opt.value; activeDropdown = null; performStoreSearch()"
              class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase transition"
              :class="storeData.type === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
            >
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1 max-w-[170px] custom-dropdown">
        <div
          @click="activeDropdown = activeDropdown === 'version' ? null : 'version'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer transition hover:border-white/30"
          :class="activeDropdown === 'version' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10'"
        >
          <span class="font-mono text-xs font-bold text-white truncate">{{ storeData.game_version || t('All Versions') }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2 shrink-0" :class="{'rotate-180': activeDropdown === 'version'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'version'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50 max-h-56 overflow-y-auto custom-scroll">
            <div
              @click="storeData.game_version = ''; activeDropdown = null; performStoreSearch()"
              class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold transition border-b border-white/5"
              :class="!storeData.game_version ? 'text-indigo-400' : 'text-white/70'"
            >
              {{ t('All Versions') }}
            </div>
            <div
              v-for="v in state.mcVersions"
              :key="v"
              @click="storeData.game_version = v; activeDropdown = null; performStoreSearch()"
              class="px-4 py-2 hover:bg-white/5 cursor-pointer text-xs font-mono font-bold transition"
              :class="storeData.game_version === v ? 'text-indigo-400' : 'text-white/70'"
            >
              {{ v }}
            </div>
          </div>
        </transition>
      </div>

      <div v-if="storeData.type === 'mod' || storeData.type === 'modpack'" class="relative flex-1 max-w-[150px] custom-dropdown">
        <div
          @click="activeDropdown = activeDropdown === 'loader' ? null : 'loader'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer transition hover:border-white/30"
          :class="activeDropdown === 'loader' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ loaderOptions.find(o => o.value === storeData.loader)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2 shrink-0" :class="{'rotate-180': activeDropdown === 'loader'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'loader'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
            <div
              v-for="opt in loaderOptions"
              :key="opt.value"
              @click="storeData.loader = opt.value; activeDropdown = null; performStoreSearch()"
              class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase transition"
              :class="storeData.loader === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
            >
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1 max-w-[160px] custom-dropdown">
        <div
          @click="activeDropdown = activeDropdown === 'sort' ? null : 'sort'"
          class="bg-black/60 border rounded-xl px-4 py-2.5 flex justify-between items-center cursor-pointer transition hover:border-white/30"
          :class="activeDropdown === 'sort' ? 'border-indigo-500 shadow-[0_0_15px_rgba(99,102,241,0.2)]' : 'border-white/10'"
        >
          <span class="font-bold text-xs uppercase tracking-wider text-white truncate">{{ sortOptions.find(o => o.value === storeData.sort_index)?.label }}</span>
          <ChevronDown class="w-3.5 h-3.5 text-white/50 transition-transform ml-2 shrink-0" :class="{'rotate-180': activeDropdown === 'sort'}" />
        </div>
        <transition name="fade">
          <div v-if="activeDropdown === 'sort'" class="absolute top-full left-0 w-full mt-2 bg-[#121214]/95 backdrop-blur-xl border border-white/10 rounded-xl shadow-2xl py-1 z-50">
            <div
              v-for="opt in sortOptions"
              :key="opt.value"
              @click="storeData.sort_index = opt.value; activeDropdown = null; performStoreSearch()"
              class="px-4 py-2.5 hover:bg-white/5 cursor-pointer text-xs font-bold uppercase transition"
              :class="storeData.sort_index === opt.value ? 'text-indigo-400 bg-indigo-500/10' : 'text-white/70'"
            >
              {{ opt.label }}
            </div>
          </div>
        </transition>
      </div>

      <div class="relative flex-1">
        <Search class="absolute left-4 top-1/2 -translate-y-1/2 text-white/30 w-4 h-4" />
        <input
          v-model="storeData.query"
          @input="debounceStoreSearch"
          type="text"
          :placeholder="t('Search modules, packages and libraries...')"
          class="kip-input pl-11 py-2 text-xs font-mono"
        >
      </div>
    </div>

    <div class="grid grid-cols-2 gap-5 flex-1 overflow-y-auto custom-scroll pr-2 pb-10 min-h-0 z-10" @scroll="handleScroll">
      <template v-if="isStoreLoading && storeData.offset === 0">
        <div v-for="i in 8" :key="i" class="kip-card p-5 flex flex-col justify-between min-h-[220px]">
          <div class="flex items-start gap-4">
            <div class="w-16 h-16 rounded-2xl skeleton-box shrink-0"></div>
            <div class="flex-1 space-y-3 mt-1">
              <div class="h-4 w-3/4 rounded skeleton-box"></div>
              <div class="h-3 w-1/2 rounded skeleton-box"></div>
            </div>
          </div>
          <div class="h-10 w-full rounded-xl skeleton-box mt-4"></div>
        </div>
      </template>

      <template v-else>
        <div v-if="storeResults.length === 0" class="col-span-2 text-center py-24 flex flex-col items-center gap-4">
          <SearchX class="w-16 h-16 text-white/20" />
          <span class="text-white/40 font-mono text-xs">{{ t('No results found in central repositories.') }}</span>
        </div>

        <div
          v-for="item in storeResults"
          :key="item.project_id"
          @click="openStoreItemDetails(item)"
          class="kip-card kip-card-hover p-5 flex flex-col justify-between cursor-pointer relative overflow-hidden group min-h-[220px] transition-all duration-300 border border-white/5 hover:border-indigo-500/30 hover:bg-black/50"
        >
          <div
            class="absolute -right-12 -top-12 w-36 h-36 rounded-full blur-3xl pointer-events-none opacity-20 transition-all duration-500"
            :class="item.provider === 'curseforge' ? 'bg-orange-500' : 'bg-emerald-500'"
          ></div>

          <div>
            <div class="flex gap-4 items-start mb-3 relative z-10">
              <div class="relative shrink-0">
                <img
                  :src="item.icon_url || fallbackModIcon"
                  class="w-16 h-16 rounded-2xl object-cover bg-black/60 p-1 border border-white/10 shadow-xl group-hover:scale-105 transition-transform duration-300"
                >
                <span
                  v-if="item.is_installed"
                  class="absolute -bottom-1 -right-1 p-1 bg-emerald-500 text-black rounded-lg shadow-md border border-emerald-300"
                  :title="t('Installed in instance')"
                >
                  <Check class="w-3 h-3 stroke-[3]" />
                </span>
              </div>

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2 mb-1">
                  <h3 class="font-black text-white text-base truncate leading-tight">{{ item.title }}</h3>
                  <span
                    class="px-2 py-0.5 rounded text-[8px] font-mono font-bold uppercase tracking-wider border shrink-0"
                    :class="item.provider === 'curseforge' ? 'bg-orange-500/10 text-orange-400 border-orange-500/20' : 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20'"
                  >
                    {{ item.provider }}
                  </span>
                </div>

                <div class="flex items-center gap-3 text-xs text-white/40 mb-2">
                  <span class="flex items-center gap-1 truncate font-medium"><User class="w-3 h-3 text-white/30" /> {{ item.author }}</span>
                  <span class="flex items-center gap-1 text-emerald-400 font-mono font-bold shrink-0"><Download class="w-3 h-3" /> {{ formatNumber(item.downloads) }}</span>
                </div>

                <div class="flex gap-1.5 overflow-hidden flex-wrap max-h-[22px]">
                  <span
                    v-for="cat in (item.categories || []).slice(0, 3)"
                    :key="cat"
                    class="px-2 py-0.5 rounded text-[9px] font-mono font-bold bg-white/5 text-white/50 border border-white/10 uppercase"
                  >
                    {{ cat }}
                  </span>
                </div>
              </div>
            </div>

            <p class="text-xs text-white/60 line-clamp-2 leading-relaxed font-medium mb-4 relative z-10">{{ item.description }}</p>
          </div>

          <div class="relative z-10 flex items-center gap-2 pt-2">
            <template v-if="item.is_installed">
              <button
                @click.stop="openStoreItemDetails(item)"
                class="flex-1 py-2.5 rounded-xl font-bold text-xs uppercase tracking-wider bg-emerald-500/10 text-emerald-400 border border-emerald-500/30 flex items-center justify-center gap-2"
              >
                <CheckCircle2 class="w-4 h-4" />
                <span>{{ t('Installed') }}</span>
              </button>
              <button
                @click.stop="uninstallStoreItem(item)"
                class="p-2.5 rounded-xl bg-red-500/10 text-red-400 hover:bg-red-500 hover:text-white border border-red-500/20 transition"
                :title="t('Uninstall from instance')"
              >
                <Trash2 class="w-4 h-4" />
              </button>
            </template>

            <template v-else>
              <button
                @click.stop="downloadStoreItem(item)"
                :disabled="item.downloading"
                class="w-full py-2.5 rounded-xl font-black text-xs uppercase tracking-widest transition-all duration-300 relative overflow-hidden flex items-center justify-center gap-2 border shadow-lg"
                :class="item.downloading ? 'bg-indigo-950 text-indigo-300 border-indigo-500/30' : 'bg-indigo-500 hover:bg-indigo-400 text-white border-indigo-400 shadow-[0_0_20px_rgba(99,102,241,0.3)]'"
              >
                <div
                  v-if="item.downloading"
                  class="absolute top-0 left-0 h-full bg-indigo-600/40 transition-all duration-150 pointer-events-none"
                  :style="{ width: (item.progress || 0) + '%' }"
                ></div>
                <Loader v-if="item.downloading" class="w-4 h-4 animate-spin relative z-10" />
                <DownloadCloud v-else class="w-4 h-4 relative z-10" />
                <span class="relative z-10">{{ item.downloading ? `${item.status_text || 'Installing'} (${Math.round(item.progress || 0)}%)` : t('Install 1-Click') }}</span>
              </button>
            </template>
          </div>
        </div>

        <div v-if="isStoreLoadingMore" class="col-span-2 flex justify-center py-6">
          <Loader class="w-8 h-8 animate-spin text-indigo-400" />
        </div>
      </template>
    </div>

    <transition name="fade">
      <div
        v-if="storeDetailsModal.isOpen"
        class="fixed inset-0 bg-black/90 backdrop-blur-2xl z-[200] flex items-center justify-center p-8 cursor-default"
        @click.self="storeDetailsModal.isOpen = false"
      >
        <div class="relative w-full h-full kip-card p-0 flex flex-col max-w-5xl shadow-[0_0_60px_rgba(0,0,0,0.8)] overflow-hidden border border-white/10">
          <div class="p-6 border-b border-white/5 flex justify-between items-center bg-black/60 shrink-0 relative overflow-hidden">
            <div
              class="absolute -top-32 -right-32 w-80 h-80 blur-[100px] rounded-full pointer-events-none opacity-20"
              :class="storeDetailsModal.item?.provider === 'curseforge' ? 'bg-orange-500' : 'bg-emerald-500'"
            ></div>

            <div class="flex items-center gap-5 relative z-10">
              <img
                :src="storeDetailsModal.item?.icon_url || fallbackModIcon"
                class="w-16 h-16 rounded-2xl bg-black/60 p-1 object-cover border border-white/10 shadow-2xl"
              >
              <div>
                <h3 class="text-2xl font-black text-white leading-tight mb-1">{{ storeDetailsModal.item?.title }}</h3>
                <p class="text-xs text-white/50 flex items-center gap-3">
                  <span>{{ t('Created by') }} <span class="text-white font-bold">{{ storeDetailsModal.item?.author }}</span></span>
                  <span class="text-white/20">•</span>
                  <span class="text-emerald-400 font-mono font-bold">{{ formatNumber(storeDetailsModal.item?.downloads || 0) }} downloads</span>
                </p>
              </div>
            </div>

            <div class="flex items-center gap-3 relative z-10">
              <button
                v-if="storeDetailsModal.item?.is_installed"
                @click="uninstallStoreItem(storeDetailsModal.item)"
                class="kip-btn-danger px-4 py-2 text-xs uppercase tracking-wider font-bold"
              >
                <Trash2 class="w-3.5 h-3.5" />
                <span>{{ t('Uninstall') }}</span>
              </button>
              <button
                v-else-if="storeDetailsModal.item"
                @click="downloadStoreItem(storeDetailsModal.item)"
                :disabled="storeDetailsModal.item.downloading"
                class="kip-btn-primary px-6 py-2.5 text-xs uppercase tracking-wider font-black shadow-[0_0_20px_rgba(99,102,241,0.4)]"
              >
                <Loader v-if="storeDetailsModal.item.downloading" class="w-4 h-4 animate-spin" />
                <DownloadCloud v-else class="w-4 h-4" />
                <span>{{ t('Install') }}</span>
              </button>

              <button @click="storeDetailsModal.isOpen = false" class="p-2 rounded-xl text-white/40 hover:text-white hover:bg-white/5 transition">
                <X class="w-5 h-5" />
              </button>
            </div>
          </div>

          <div class="flex border-b border-white/5 bg-black/30 shrink-0">
            <button
              @click="storeDetailsModal.tab = 'description'"
              class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
              :class="storeDetailsModal.tab === 'description' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'"
            >
              <FileText class="w-3.5 h-3.5" /> Overview & Gallery
            </button>
            <button
              @click="storeDetailsModal.tab = 'versions'"
              class="flex-1 py-3.5 font-bold transition flex items-center justify-center gap-2 text-xs uppercase tracking-wider border-b-2"
              :class="storeDetailsModal.tab === 'versions' ? 'text-indigo-400 border-indigo-400 bg-indigo-500/5' : 'text-white/40 border-transparent hover:text-white'"
            >
              <Layers class="w-3.5 h-3.5" /> Release Catalog ({{ storeDetailsModal.versions.length }})
            </button>
          </div>

          <div class="flex-1 overflow-y-auto custom-scroll bg-[#030305]/95 p-8 relative min-h-0">
            <div v-show="storeDetailsModal.tab === 'description'" class="flex flex-col gap-6">
              <div
                v-if="storeDetailsModal.details?.gallery && storeDetailsModal.details.gallery.length > 0"
                class="flex gap-4 overflow-x-auto custom-scroll pb-4 snap-x shrink-0"
              >
                <img
                  v-for="img in storeDetailsModal.details.gallery"
                  :key="img.url"
                  :src="img.url"
                  class="h-56 rounded-2xl object-cover border border-white/10 shadow-2xl cursor-pointer hover:border-indigo-500 transition-all duration-300 snap-center"
                  @click="openExternalUrl(img.url)"
                >
              </div>

              <div class="p-6 bg-black/40 border border-white/5 rounded-3xl markdown-body text-white/80 leading-relaxed font-sans" v-html="storeDetailsModal.html"></div>
            </div>

            <div v-show="storeDetailsModal.tab === 'versions'" class="flex flex-col gap-3">
              <div v-if="storeDetailsModal.versions.length === 0" class="py-20 text-center text-white/40 font-mono text-xs">
                {{ t('No releases available matching current platform filters.') }}
              </div>

              <div
                v-for="ver in storeDetailsModal.versions"
                :key="ver.id"
                class="p-5 bg-black/40 border border-white/5 rounded-2xl flex flex-col gap-3 transition-colors hover:border-white/15"
              >
                <div class="flex justify-between items-start">
                  <div>
                    <div class="flex items-center gap-3 mb-1">
                      <h4 class="font-black text-white text-base leading-tight">{{ ver.name || ver.version_number }}</h4>
                      <span class="px-2 py-0.5 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 font-mono text-[9px] font-bold">
                        {{ ver.version_number }}
                      </span>
                    </div>
                    <span class="text-[10px] font-mono text-white/40 flex items-center gap-1">
                      <Clock class="w-3 h-3" /> {{ new Date(ver.date).toLocaleDateString() }}
                    </span>
                  </div>

                  <button
                    @click="downloadSpecificVersion(ver)"
                    class="kip-btn-primary px-5 py-2 text-xs font-black uppercase tracking-wider"
                  >
                    <DownloadCloud class="w-3.5 h-3.5" />
                    <span>Download</span>
                  </button>
                </div>

                <div v-if="ver.dependencies && ver.dependencies.length > 0" class="pt-3 border-t border-white/5 flex flex-wrap gap-2 items-center">
                  <span class="text-[9px] font-mono text-white/40 uppercase font-bold">Required Dependencies:</span>
                  <span
                    v-for="dep in ver.dependencies"
                    :key="dep.project_id"
                    class="px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 text-[9px] font-mono font-bold"
                  >
                    {{ dep.project_id }}
                  </span>
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
import { ref, onMounted, onBeforeUnmount } from 'vue'
import type { Component } from 'vue'
import { marked } from 'marked'
import {
  Search,
  SearchX,
  Box,
  Flame,
  X,
  ChevronDown,
  User,
  Download,
  DownloadCloud,
  Check,
  CheckCircle2,
  Trash2,
  Loader,
  FileText,
  Layers,
  Clock,
  Cpu,
  Sparkles,
  Wrench,
  Globe2,
  Compass,
} from 'lucide-vue-next'
import { state, t, showToast } from '@/store'
import {
  bridge,
  invokeSafe,
  type StoreItemRecordDto,
  type StoreSearchResultDto,
  type StoreDetailsResponseDto,
  type StoreItemDetailsDto,
  type StoreItemVersionDto,
  type StoreInstallResultDto,
} from '@/bridge'

interface CategoryBadge {
  id: string
  name: string
  icon: Component
}

interface SelectOption {
  value: string
  label: string
}

const storeData = ref({
  provider: 'modrinth' as 'modrinth' | 'curseforge',
  type: 'mod',
  loader: '',
  game_version: '',
  query: '',
  offset: 0,
  sort_index: 'relevance',
  category: '',
})

const isStoreLoading = ref<boolean>(false)
const isStoreLoadingMore = ref<boolean>(false)
const hasMoreStoreItems = ref<boolean>(true)
const storeResults = ref<StoreItemRecordDto[]>([])
let searchDebounceTimeout: ReturnType<typeof setTimeout> | null = null

const activeDropdown = ref<'type' | 'version' | 'loader' | 'sort' | null>(null)

const categoryBadges: CategoryBadge[] = [
  { id: 'optimization', name: 'Optimization', icon: Cpu },
  { id: 'technology', name: 'Technology', icon: Wrench },
  { id: 'magic', name: 'Magic', icon: Sparkles },
  { id: 'utility', name: 'Utility', icon: Compass },
  { id: 'worldgen', name: 'World Gen', icon: Globe2 },
]

const typeOptions: SelectOption[] = [
  { value: 'mod', label: 'Mods' },
  { value: 'modpack', label: 'Modpacks' },
  { value: 'resourcepack', label: 'Resourcepacks' },
  { value: 'shader', label: 'Shaders' },
]

const loaderOptions: SelectOption[] = [
  { value: '', label: 'All Loaders' },
  { value: 'fabric', label: 'Fabric' },
  { value: 'forge', label: 'Forge' },
  { value: 'neoforge', label: 'NeoForge' },
  { value: 'quilt', label: 'Quilt' },
]

const sortOptions: SelectOption[] = [
  { value: 'relevance', label: 'Relevance' },
  { value: 'downloads', label: 'Downloads' },
  { value: 'newest', label: 'Newest' },
  { value: 'updated', label: 'Updated' },
]

const fallbackModIcon =
  'data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="64" height="64" viewBox="0 0 24 24" fill="none" stroke="%236366f1" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"><path d="m7.5 4.27 9 5.15"/><path d="M21 8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16Z"/><path d="m3.3 7 8.7 5 8.7-5"/><path d="M12 22V12"/></svg>'

const storeDetailsModal = ref<{
  isOpen: boolean
  item: StoreItemRecordDto | null
  details: StoreItemDetailsDto | null
  html: string
  tab: 'description' | 'versions'
  versions: StoreItemVersionDto[]
}>({
  isOpen: false,
  item: null,
  details: null,
  html: '',
  tab: 'description',
  versions: [],
})

const formatNumber = (num: number): string => {
  if (!num) return '0'
  if (num >= 1000000) return (num / 1000000).toFixed(1) + 'M'
  if (num >= 1000) return (num / 1000).toFixed(1) + 'K'
  return num.toString()
}

const openExternalUrl = (url: string): void => {
  window.open(url, '_blank')
}

const setProvider = (providerName: 'modrinth' | 'curseforge'): void => {
  storeData.value.provider = providerName
  performStoreSearch()
}

const setStoreCategory = (cat: string): void => {
  storeData.value.category = storeData.value.category === cat ? '' : cat
  performStoreSearch()
}

const debounceStoreSearch = (): void => {
  if (searchDebounceTimeout) clearTimeout(searchDebounceTimeout)
  searchDebounceTimeout = setTimeout(() => {
    performStoreSearch()
  }, 350)
}

const performStoreSearch = async (): Promise<void> => {
  isStoreLoading.value = true
  storeData.value.offset = 0
  hasMoreStoreItems.value = true

  try {
    const res: StoreSearchResultDto = await invokeSafe<StoreSearchResultDto>('search_store', {
      provider: storeData.value.provider,
      query: storeData.value.query.trim() || null,
      projectType: storeData.value.type || null,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
      category: storeData.value.category || null,
      sortIndex: storeData.value.sort_index || null,
      offset: storeData.value.offset,
    })

    if (res && res.success) {
      storeResults.value = (res.hits || []).map((h) => ({
        ...h,
        downloading: false,
        progress: 0,
        status_text: '',
      }))
      if (res.hits.length < 16) {
        hasMoreStoreItems.value = false
      }
    } else {
      storeResults.value = []
      hasMoreStoreItems.value = false
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Store Error'), msg, 'danger')
  } finally {
    isStoreLoading.value = false
  }
}

const loadMoreStoreItems = async (): Promise<void> => {
  if (isStoreLoadingMore.value || isStoreLoading.value || !hasMoreStoreItems.value) return
  isStoreLoadingMore.value = true
  storeData.value.offset += 16

  try {
    const res: StoreSearchResultDto = await invokeSafe<StoreSearchResultDto>('search_store', {
      provider: storeData.value.provider,
      query: storeData.value.query.trim() || null,
      projectType: storeData.value.type || null,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
      category: storeData.value.category || null,
      sortIndex: storeData.value.sort_index || null,
      offset: storeData.value.offset,
    })

    if (res && res.success && res.hits.length > 0) {
      const mapped = res.hits.map((h) => ({
        ...h,
        downloading: false,
        progress: 0,
        status_text: '',
      }))
      storeResults.value.push(...mapped)
      if (res.hits.length < 16) {
        hasMoreStoreItems.value = false
      }
    } else {
      hasMoreStoreItems.value = false
    }
  } catch {
    hasMoreStoreItems.value = false
  } finally {
    isStoreLoadingMore.value = false
  }
}

const handleScroll = (e: Event): void => {
  const el = e.target as HTMLElement
  if (el.scrollHeight - el.scrollTop <= el.clientHeight + 200) {
    loadMoreStoreItems()
  }
}

const openStoreItemDetails = async (item: StoreItemRecordDto): Promise<void> => {
  storeDetailsModal.value.item = item
  storeDetailsModal.value.details = null
  storeDetailsModal.value.versions = []
  storeDetailsModal.value.tab = 'description'
  storeDetailsModal.value.html = '<div class="py-16 text-center text-indigo-400 font-mono text-xs animate-pulse">Loading technical manifest...</div>'
  storeDetailsModal.value.isOpen = true

  try {
    const res: StoreDetailsResponseDto = await invokeSafe<StoreDetailsResponseDto>('get_store_full_details', {
      provider: item.provider,
      projectId: item.project_id,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (res && res.success) {
      storeDetailsModal.value.details = res.details
      storeDetailsModal.value.versions = res.versions || []
      storeDetailsModal.value.html = (await marked.parse(res.details.body)) as string
    } else {
      storeDetailsModal.value.html = '<div class="text-center text-red-400 py-10 font-mono text-xs">Failed to load payload manifest.</div>'
    }
  } catch {
    storeDetailsModal.value.html = '<div class="text-center text-red-400 py-10 font-mono text-xs">Remote network timeout.</div>'
  }
}

const downloadStoreItem = async (item: StoreItemRecordDto): Promise<void> => {
  if (item.downloading) return
  item.downloading = true
  item.progress = 5
  item.status_text = 'Resolving'

  try {
    const detailsRes: StoreDetailsResponseDto = await invokeSafe<StoreDetailsResponseDto>('get_store_full_details', {
      provider: item.provider,
      projectId: item.project_id,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (!detailsRes || !detailsRes.success || detailsRes.versions.length === 0) {
      showToast(t('Error'), 'No compatible versions found.', 'danger')
      item.downloading = false
      return
    }

    const targetVer = detailsRes.versions[0]
    if (!targetVer) return
    const targetFile = targetVer.files.find((f) => f.primary) || targetVer.files[0]
    if (!targetFile) return

    const res: StoreInstallResultDto = await invokeSafe<StoreInstallResultDto>('download_store_item', {
      provider: item.provider,
      projectId: item.project_id,
      versionId: targetVer.id,
      url: targetFile.url,
      filename: targetFile.filename,
      projectType: item.project_type,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (res && res.success) {
      item.is_installed = true
      item.installed_filename = res.filename
      showToast(t('Installed'), res.message, 'success')
      if (res.installed_dependencies.length > 0) {
        showToast(
          t('Dependencies Synced'),
          `Integrated ${res.installed_dependencies.length} required runtime libraries.`,
          'info'
        )
      }
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Install Error'), msg, 'danger')
  } finally {
    item.downloading = false
    item.progress = 0
  }
}

const downloadSpecificVersion = async (ver: StoreItemVersionDto): Promise<void> => {
  const item = storeDetailsModal.value.item
  if (!item) return
  const targetFile = ver.files.find((f) => f.primary) || ver.files[0]
  if (!targetFile) return

  try {
    const res: StoreInstallResultDto = await invokeSafe<StoreInstallResultDto>('download_specific_file', {
      provider: item.provider,
      projectId: item.project_id,
      versionId: ver.id,
      url: targetFile.url,
      filename: targetFile.filename,
      projectType: item.project_type,
      loader: storeData.value.loader || null,
      gameVersion: storeData.value.game_version || null,
    })

    if (res && res.success) {
      item.is_installed = true
      item.installed_filename = res.filename
      showToast(t('Installed'), res.message, 'success')
      storeDetailsModal.value.isOpen = false
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Install Error'), msg, 'danger')
  }
}

const uninstallStoreItem = async (item: StoreItemRecordDto): Promise<void> => {
  if (!item.installed_filename) return
  try {
    const ok = await invokeSafe<boolean>('uninstall_store_item', {
      projectType: item.project_type,
      filename: item.installed_filename,
    })
    if (ok) {
      item.is_installed = false
      item.installed_filename = null
      showToast(t('Removed'), `Purged ${item.title} from instance.`, 'success')
      if (storeDetailsModal.value.isOpen && storeDetailsModal.value.item?.project_id === item.project_id) {
        storeDetailsModal.value.item.is_installed = false
      }
    }
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Uninstall Error'), msg, 'danger')
  }
}

const closeDropdowns = (e: MouseEvent): void => {
  const target = e.target as HTMLElement | null
  if (!target || !target.closest('.custom-dropdown')) {
    activeDropdown.value = null
  }
}

onMounted(async () => {
  window.addEventListener('click', closeDropdowns)

  if (window.__TAURI__?.event?.listen) {
    window.__TAURI__.event.listen<{ project_id: string; filename: string; progress: number; status: string }>(
      'storeDownloadProgress',
      (e) => {
        const item = storeResults.value.find((i) => i.project_id === e.payload.project_id)
        if (item) {
          item.downloading = true
          item.progress = e.payload.progress
          item.status_text = e.payload.status
        }
      }
    )
  }

  if (state.mcVersions.length === 0) {
    try {
      state.mcVersions = await bridge.getMcVersions()
    } catch {}
  }

  await performStoreSearch()
})

onBeforeUnmount(() => {
  window.removeEventListener('click', closeDropdowns)
  if (searchDebounceTimeout) clearTimeout(searchDebounceTimeout)
})
</script>

<style scoped>
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3) {
  color: #fff;
  font-weight: 800;
  margin-top: 1rem;
  margin-bottom: 0.5rem;
}
.markdown-body :deep(p) {
  margin-bottom: 0.75rem;
  line-height: 1.6;
}
.markdown-body :deep(ul) {
  list-style-type: disc;
  padding-left: 1.5rem;
  margin-bottom: 0.75rem;
}
.markdown-body :deep(code) {
  background: rgba(255, 255, 255, 0.1);
  padding: 0.2rem 0.4rem;
  border-radius: 0.375rem;
  font-family: monospace;
}
</style>