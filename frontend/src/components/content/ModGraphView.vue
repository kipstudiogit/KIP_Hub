<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { Network } from 'vis-network'
import { invoke } from '@tauri-apps/api/core'
import { GitMerge, X } from 'lucide-vue-next'
import type { ModGraphDataDto, GraphNode } from '../../types/content'
import { showToast } from '../../composables/useToasts'
import { t } from '../../composables/useI18n'

const graphContainer = ref<HTMLElement | null>(null)
let networkInstance: Network | null = null
const graphDataCache = ref<ModGraphDataDto | null>(null)
const graphSearchQuery = ref('')
const selectedGraphNode = ref<GraphNode | null>(null)

async function renderGraph(): Promise<void> {
  if (!graphContainer.value) return
  try {
    const data = await invoke<ModGraphDataDto>('get_mod_graph_data')
    graphDataCache.value = data

    if (networkInstance) {
      networkInstance.destroy()
      networkInstance = null
    }

    networkInstance = new Network(
      graphContainer.value,
      {
        nodes: data.nodes as any,
        edges: data.edges as any,
      },
      {
        nodes: {
          borderWidth: 2,
          shadow: {
            enabled: true,
            color: 'rgba(0,0,0,0.6)',
            size: 10,
            x: 0,
            y: 4,
          },
        },
        edges: {
          smooth: {
            enabled: true,
            type: 'cubicBezier',
            roundness: 0.5,
          },
          width: 2,
        },
        physics: {
          barnesHut: {
            gravitationalConstant: -3500,
            centralGravity: 0.25,
            springLength: 120,
            springConstant: 0.04,
            damping: 0.09,
            avoidOverlap: 0.3,
          },
          stabilization: {
            iterations: 150,
            updateInterval: 25,
          },
        },
        interaction: {
          hover: true,
          tooltipDelay: 100,
          hideEdgesOnDrag: false,
          multiselect: false,
        },
      }
    )

    networkInstance.on('click', (params: any) => {
      if (params.nodes.length > 0) {
        const nodeId = params.nodes[0]
        const found = graphDataCache.value?.nodes.find((n) => n.id === nodeId)
        if (found) {
          selectedGraphNode.value = found
        }
      } else {
        selectedGraphNode.value = null
      }
    })
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err)
    showToast(t('Error'), msg, 'danger')
  }
}

function fitGraphView(): void {
  if (networkInstance) {
    networkInstance.fit({ animation: { duration: 600, easingFunction: 'easeInOutQuad' } })
  }
}

function recalculateGraphPhysics(): void {
  if (networkInstance) {
    networkInstance.stabilize()
  }
}

function filterGraphFocus(): void {
  if (!networkInstance || !graphDataCache.value) return
  const query = graphSearchQuery.value.trim().toLowerCase()
  if (!query) {
    fitGraphView()
    return
  }

  const match = graphDataCache.value.nodes.find(
    (n) => n.id.toLowerCase().includes(query) || n.label.toLowerCase().includes(query)
  )

  if (match) {
    networkInstance.focus(match.id, {
      scale: 1.5,
      animation: { duration: 500, easingFunction: 'easeInOutQuad' },
    })
    networkInstance.selectNodes([match.id])
    selectedGraphNode.value = match
  }
}

onMounted(() => {
  renderGraph()
})

onBeforeUnmount(() => {
  if (networkInstance) {
    networkInstance.destroy()
    networkInstance = null
  }
})
</script>

<template>
  <div class="w-full h-full kip-card relative overflow-hidden shrink-0 min-h-[550px] border border-white/10 flex flex-col">
    <div class="p-4 border-b border-white/10 bg-black/60 flex justify-between items-center z-10">
      <div class="flex items-center gap-3">
        <GitMerge class="w-5 h-5 text-indigo-400" />
        <span class="font-black text-sm uppercase tracking-wider text-white">Neural Dependency Topology</span>
        <span class="text-xs font-mono text-white/40">({{ graphDataCache?.nodes?.length || 0 }} modules linked)</span>
      </div>

      <div class="flex items-center gap-2">
        <input
          v-model="graphSearchQuery"
          @input="filterGraphFocus"
          type="text"
          placeholder="Search & Focus node..."
          class="kip-input py-1 px-3 text-xs w-52 font-mono"
        >
        <button @click="fitGraphView" class="kip-btn-ghost px-3 py-1.5 text-xs font-mono font-bold">Fit View</button>
        <button @click="recalculateGraphPhysics" class="kip-btn-ghost px-3 py-1.5 text-xs font-mono font-bold text-indigo-400">Re-stabilize</button>
      </div>
    </div>

    <div class="flex-1 relative w-full h-full">
      <div ref="graphContainer" class="w-full h-full"></div>

      <div class="absolute bottom-4 left-4 kip-card p-4 text-xs z-10 flex flex-col gap-2 bg-black/80 border border-white/10 backdrop-blur-md">
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-[#10B981]"></span> <span class="font-bold font-mono text-white/80">Optimization Engine</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-[#A855F7]"></span> <span class="font-bold font-mono text-white/80">Core Library (API)</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded-full bg-[#6366F1]"></span> <span class="font-bold font-mono text-white/80">Active Gameplay Module</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-3 rounded bg-[#F59E0B]"></span> <span class="font-bold font-mono text-white/80">Missing Companion</span></div>
        <div class="flex items-center gap-2"><span class="w-3 h-1 bg-[#EF4444]"></span> <span class="font-bold font-mono text-white/80">Declared Collision</span></div>
      </div>

      <transition name="fade">
        <div v-if="selectedGraphNode" class="absolute top-4 right-4 w-80 kip-card p-5 bg-black/90 border border-indigo-500/40 z-20 shadow-2xl flex flex-col gap-3">
          <div class="flex justify-between items-start">
            <h4 class="font-black text-white text-base leading-tight">{{ selectedGraphNode.label }}</h4>
            <button @click="selectedGraphNode = null" class="text-white/40 hover:text-white p-1"><X class="w-4 h-4" /></button>
          </div>
          <div class="text-xs font-mono space-y-1 border-t border-white/5 pt-2">
            <div><span class="text-white/40">Identifier:</span> <span class="text-indigo-300 font-bold">{{ selectedGraphNode.id }}</span></div>
            <div><span class="text-white/40">Category:</span> <span class="text-white font-bold">{{ selectedGraphNode.modType }}</span></div>
            <div><span class="text-white/40">Version:</span> <span class="text-white font-bold">{{ selectedGraphNode.version }}</span></div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>