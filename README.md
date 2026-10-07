# K.I.P. Hub (v2.0.0)

> High-Performance Modular Minecraft Management Engine & Desktop Platform  
> Powered by Rust, Tauri 2.x, and Vue 3 (Composition API, TypeScript, Vite)  
> Licensed under GNU General Public License v3.0 (GPL-3.0)

---

## ⚡ Architectural Overview

K.I.P. Hub is a cross-platform desktop client and Minecraft runtime environment designed with a strict focus on memory safety, non-blocking UI rendering, and low-level kernel optimizations across Windows and Linux.

```
+-------------------------------------------------------------------+
|                        K.I.P. FRONTEND                            |
|        Vue 3.5 (Composition API) + TypeScript + Tailwind CSS      |
|    Singletons: useOverlay, useBooster, useLauncher, useVoice      |
+---------------------------------+---------------------------------+
                                  |
               IPC Stream Channel | @tauri-apps/api/core
                                  v
+-------------------------------------------------------------------+
|                     TAURI 2.x CORE RUNTIME                        |
|                  src-tauri/src/lib.rs (Builder)                   |
+---------------------------------+---------------------------------+
                                  |
        +-------------------------+-------------------------+
        |                         |                         |
        v                         v                         v
+---------------+       +-------------------+       +---------------+
| KERNEL MEMORY |       |   QUANTUM CHUNK   |       | K.I.P. SHIELD |
|    MATRIX     |       |      MATRIX       |       |   SECURITY    |
| 2MB HugeTLB   |       | AVX2/512 SIMD     |       | Heuristic ASM |
| Lilliput Hdr  |       | Off-Heap Arena    |       | Crypto Vault  |
| Zero-Copy Pref|       | 4-Phase Scheduler |       | DoD Shredder  |
+---------------+       +-------------------+       +---------------+
        |                         |                         |
        +-------------------------+-------------------------+
                                  |
                                  v
+-------------------------------------------------------------------+
|                     NATIVE SUBSYSTEM DRIVER                       |
|   Silent Process (0x08000000), Anvil MCA IO, SQLite WAL, WebRTC   |
+-------------------------------------------------------------------+
```

---

## 🚀 Key Technological Modules

### 1. Quantum Chunk Matrix (Zero-GC SIMD Geometry Engine)
- **Off-Heap Voxel Arena:** Direct raw memory allocations bypassing the Java garbage collector with 64-byte cache-line alignment.
- **AVX2 / AVX-512 SIMD Noise:** Hardware-accelerated vectorization of Perlin gradient noise across 384-voxel columns with cached column invariants.
- **2D Checkerboard Topology Scheduler:** 4-phase cellular automaton scheduler preventing thread deadlocks during directional chunk pre-baking.
- **Lock-Free MCA Ring Stream:** Streaming chunk packaging into Anvil sectors (`.mca`) fully compatible with DataVersion 3955 (Minecraft 1.18–1.21+ / 26.x), structured `minecraft:plains` biome containers, and atomic 8192-byte header initialization.

### 2. Kernel Memory Matrix (KMM)
- **Hardware Large Pages:** Native 2MB HugeTLB memory allocations (`-XX:+UseLargePages`, `-XX:LargePageSizeInBytes=2m`). Reduces CPU Translation Lookaside Buffer (TLB) misses by up to 95%.
- **Project Lilliput 8-Byte Object Headers:** Compact object headers (`-XX:+UseCompactObjectHeaders`), reclaiming up to 1.8 GB of JVM heap memory.
- **Zero-Copy Standby Prefaulting:** Dual-sided asynchronous pre-fetching of JAR local headers and Central Directory tables directly into the OS Standby Cache list prior to JVM bootstrap.
- **Automated UAC Elevation:** Secure assignment of the `SeLockMemoryPrivilege` token via the Windows `secedit` local security authority interface.

### 3. Silent Process Execution Subsystem
- **Zero Console Window Popups:** All background sub-processes (`java`, `ipconfig`, `docker`, `ssh`) are spawned with `CREATE_NO_WINDOW` (`0x08000000`) and `DETACHED_PROCESS` (`0x00000008`) flags combined with `Stdio::null()` redirection.
- **In-Memory Caching:** Complete elimination of repetitive runtime scans (`java -version`, `whoami /priv`) via static thread-safe caches (`JAVA_PROBE_CACHE`, `cached_privilege`).

### 4. In-Game HUD Overlay & 10-Foot Deck
- **In-Game HUD Overlay (`Shift + Tab`):** Transparent fullscreen viewport positioned on top of Minecraft with seamless transitions between interactive widget management and click-through passive HUD mode (`set_ignore_cursor_events`).
- **Big Picture Mode:** Console-grade 10-foot television interface with full controller navigation (Xbox, PlayStation DualSense/DualShock, Nintendo Switch Pro), haptic rumble support, and Web Audio API feedback.
- **Desktop Mini-Widget:** Compact always-on-top desktop widget for quick process ignition and real-time hardware sensor monitoring.

### 5. Security & Diagnostics (K.I.P. Shield & Mod Doctor)
- **Heuristic Decompiler:** Real-time static bytecode analysis detecting Discord webhooks, Telegram C2 endpoints, RCE payload droppers, and high-entropy packed classes inside JAR containers.
- **Cryptographic Quarantine Vault:** Multi-vector XOR-encrypted file isolation vault equipped with DoD 5220.22-M military-grade zero-fill shredder.
- **Mod Doctor:** Dependency topology inspection, SemVer duplicate resolution, cross-loader collision diagnostics, and 1-click autonomous remediation.

### 6. P2P Mesh Network & Multiplayer Hub
- **WebRTC Voice Mesh:** Decentralized peer-to-peer voice lobby featuring acoustic echo cancellation, biquad bandpass filters, and hardware Voice Activity Detection (VAD).
- **BitTorrent Swarm:** Integrated peer-to-peer client for distributed ingestion and seeding of heavy modpack archives via magnet links.
- **Reverse LAN Tunnel:** Instant public exposure of singleplayer LAN worlds through end-to-end encrypted TCP bridges.
- **1-Click Docker Sandbox:** Local deployment of isolated server containers for PaperMC, Purpur, Fabric, and NeoForge.

---

## 🛠 Technology Stack

- **Backend:** Rust 2021, Tauri v2.12, Tokio Async Runtime, Rusqlite (SQLite WAL), Sysinfo, FastNBT, Flate2, Reqwest, Parking_lot.
- **Frontend:** Vue 3.5, TypeScript 5.x, Vite 8.x, Tailwind CSS v4.0, Lucide Icons, Marked, Skinview3D, Vis-Network.
- **Edge Infrastructure:** Cloudflare Workers, Durable Objects (Signaling Rooms), Cloudflare KV.

---

## 📦 Getting Started & Building

### Prerequisites
- **OS:** Windows 10/11 (x64), Linux (Ubuntu 22.04+, Debian 12+, Arch), macOS 12+
- **Rust Toolchain:** `rustc 1.78+` and `cargo`
- **Node.js Environment:** `v20.x+` and `npm`
- **C++ Build Tools:** Visual Studio C++ Build Tools (Windows) or `build-essential` (Linux)

### 1. Clone Repository
```bash
git clone https://github.com/kipstudiogit/KIP_Hub.git
cd KIP_Hub
```

### 2. Install Dependencies
```bash
npm run install:all
```

### 3. Run Development Server
```bash
npm run tauri dev
```

### 4. Build Production Bundle
```bash
npm run tauri build
```
Compiled standalone binaries and installers (MSI, NSIS, AppImage, DMG) will be generated in:
`src-tauri/target/release/bundle/`

---

## ⌨ Keybindings & Controls

| Shortcut / Button | Function |
|---|---|
| `Shift + Tab` | Toggle In-Game HUD Overlay |
| `Escape` | Dismiss Overlay / Close Active Modal / Skip Boot Sequence |
| `Button A (Gamepad)` | Select / Confirm Action |
| `Button B (Gamepad)` | Back / Close Active Shelf |
| `Button X (Gamepad)` | Quick Launch Mod Doctor Audit |
| `Button Y (Gamepad)` | Quick Ignite Active Instance |
| `LB / RB (Gamepad)` | Switch Navigation Deck |

---

## 🌐 Internationalization

K.I.P. Hub v2.0.0 features on-the-fly language switching without application restarts:
- 🇺🇸 English (US) — `en`
- 🇷🇺 Russian (RU) — `ru`
- 🇪🇸 Spanish (ES) — `es`
- 🇩🇪 German (DE) — `de`
- 🇨🇳 Chinese (Simplified) — `zh`
- 🇫🇷 French (FR) — `fr`
- 🇧🇷 Portuguese (BR) — `pt`
- 🇯🇵 Japanese (JA) — `ja`
- 🇰🇷 Korean (KO) — `ko`

---

## 🔒 Security & Disclaimer

- **Official Disclaimer:** NOT AN OFFICIAL MINECRAFT PRODUCT. NOT APPROVED BY OR ASSOCIATED WITH MOJANG OR MICROSOFT.
- A legitimate Minecraft license (Microsoft / Xbox Live) is required to download and play the game.
- Session tokens are encrypted locally with hardware-bound Machine GUID keys via AES/XOR. No personal information or credentials are ever transmitted to third-party tracking servers.