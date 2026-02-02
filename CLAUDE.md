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
- `src/clip.ts` - Clip/Frame management for time-series data (work in progress)

**3D Rendering**:
- Uses Three.js with OrbitControls for camera manipulation
- Scene includes ambient + directional lighting, fog, and grid helper
- Main scene group named "main_scene" for adding 3D objects
- Window resize handling is global (window.onresize)
- all other ui elements are floating on top of the scene

### Backend (Rust)

**Main Library**: src/lib.rs
- `MonitorTab` struct - Main API for logging robotics data
  - Creates HTTP server (axum) for serving static files
  - Creates WebSocket server (tungstenite) for streaming data
  - Implements graceful shutdown via oneshot channels
  - `log()` method for sending typed data with timestamps
- `LoggableData` trait - Defines serialization for all data types
- Data type implementations: Box3, Sphere, Cylinder, Capsule, Stl, MeshMaterial, Matrix4, Arrow3

**WebSocket Server**: examples/app.rs
- Example usage of MonitorTab
- Demonstrates logging various geometry types
- Default ports: HTTP (9876), WebSocket (9877)

**Server Architecture**:
- HTTP server uses axum with graceful shutdown support
- WebSocket server uses tungstenite with mpsc channel for data streaming
- Both servers run in separate threads spawned by MonitorTab
- Servers automatically shut down when MonitorTab is dropped

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
- 8: Matrix4 (128 bytes - 16 f64s column major)
- 9: Arrow3 (51 bytes - 2 Vector3s + 3 bytes color)

**Important**: All numeric data uses little-endian byte order. Timestamp is 128-bit (16 bytes) nanoseconds since UNIX_EPOCH.

## Usage Example

```rust
use kivi::MonitorTab;

// Create monitor (starts HTTP and WebSocket servers)
let monitor = MonitorTab::new(9876, 5173, 9877);
// Args: http_port, open_port, ws_port

// Log data with paths
monitor.log("robot/position", 42.0)?;
monitor.log("robot/box", Box3 { size: [1.0, 2.0, 3.0] })?;
monitor.log("robot/transform", nalgebra::Matrix4::identity())?;

// Monitor automatically cleans up when dropped
```

## Build Configuration

- Uses Vite with rolldown-vite (custom Vite fork)
- TypeScript with project references (tsconfig.app.json, tsconfig.node.json)
- Vue 3 with `<script setup>` SFCs
- Rust edition 2024

## Known Issues / Work in Progress

- Frame.apply() method in src/frame.ts is incomplete (lines 155-162)
- Duplicate Frame/DataType definitions across App.vue, frame.ts, and clip.ts
- TimeSeries functionality not yet integrated with renderer
- Path parsing differs between frame.ts (splits by '/') and App.vue (doesn't split)
