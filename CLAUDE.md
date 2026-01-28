# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Kivi is a real-time 3D visualization tool for robotics data. It consists of:
- A Vue 3 + TypeScript frontend using Three.js for 3D rendering
- A Rust backend (examples/app.rs) that streams data via WebSocket
- Custom binary protocol for efficient data transmission

## Development Commands

```bash
# Frontend development
bun run dev          # Start Vite dev server
bun run build        # Build for production (runs vue-tsc + vite build)

# Backend (Rust)
cargo run --example app    # Run the WebSocket server example
cargo build                # Build Rust components
```

## Architecture

### Frontend (Vue 3 + Three.js)

**Entry point**: src/main.ts mounts the Vue app

**Core components**:
- `src/App.vue` - Main component handling WebSocket connection and Frame parsing
- `src/renderer.ts` - Three.js scene setup (RobotRenderer class, camera, lights, grid)
- `src/frame.ts` - Frame class and data parsing logic (currently incomplete)
- `src/timeseries.ts` - Clip/Frame management for time-series data (work in progress)

**3D Rendering**:
- Uses Three.js with OrbitControls for camera manipulation
- Scene includes ambient + directional lighting, fog, and grid helper
- Main scene group named "main_scene" for adding 3D objects
- Window resize handling is global (window.onresize)

### Backend (Rust)

**WebSocket Server**: examples/app.rs
- Listens on port 9876 by default
- Uses tungstenite for WebSocket communication
- Monitor struct manages connection lifecycle

### Binary Protocol

**Frame format**: `[16 bytes timestamp][2 bytes path length][path string][2 bytes data type][data]`

**Data types** (defined in both Rust and TypeScript):
- 0: ScalarF64 (8 bytes)
- 1: Box3 (24 bytes - 3 f64s for size)
- 2: BoxLine3 (24 bytes - 3 f64s for size)
- 3: Sphere (8 bytes - radius)
- 4: Cylinder (16 bytes - radius + height)
- 5: Capsule (16 bytes - radius + height)
- 6: Stl (variable length binary data)
- 7: MeshMaterial (19 bytes - 3 bytes color + 8 bytes roughness + 8 bytes metalness)
- 8: Matrix4 (128 bytes - 16 f64s)
- 9: Arrow3 (51 bytes - 2 Vector3s + 3 bytes color)

**Important**: All numeric data uses little-endian byte order. Timestamp is 128-bit (16 bytes) nanoseconds since UNIX_EPOCH.

## Build Configuration

- Uses Vite with rolldown-vite (custom Vite fork)
- TypeScript with project references (tsconfig.app.json, tsconfig.node.json)
- Vue 3 with `<script setup>` SFCs
- Rust edition 2024

## Known Issues / Work in Progress

- Frame.apply() method in src/frame.ts is incomplete (lines 155-162)
- Duplicate Frame/DataType definitions across App.vue, frame.ts, and timeseries.ts
- TimeSeries functionality not yet integrated with renderer
- Path parsing differs between frame.ts (splits by '/') and App.vue (doesn't split)
