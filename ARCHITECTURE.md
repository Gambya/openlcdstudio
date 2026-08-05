# OpenLCD Studio Architecture

## Philosophy

OpenLCD Studio is designed around a modular architecture.

Every hardware implementation must be isolated from the rendering engine.

Every widget must be independent.

The renderer must never know which display is connected.

The GUI must never know how packets are transmitted.

---

## Layers

GUI

↓

Renderer

↓

Frame

↓

Protocol

↓

Driver

↓

USB / Serial

↓

LCD

---

## Crates

### openlcd-core

Defines:

- Device
- Frame
- Widget
- Theme
- Driver Trait

Contains no platform specific code.

---

### openlcd-render

Responsible for rendering:

- Images
- Videos
- GIFs
- SVG
- Text
- Widgets

Outputs RGBA FrameBuffers.

---

### openlcd-codec

Responsible for decoding media formats.

Supports:

- PNG
- JPEG
- WEBP
- GIF
- MP4

---

### openlcd-monitor

Collects real-time system information.

Examples:

- CPU
- GPU
- RAM
- Network
- Weather
- Clock

---

### openlcd-theme

Loads and saves themes.

Themes are JSON files.

---

### openlcd-plugin

Plugin API.

Allows external widgets and drivers.

---

### openlcd-driver

Abstraction layer between the renderer and hardware.

Contains the Driver trait.

---

### openlcd-protocol

Contains packet encoders and decoders.

No USB implementation.

---

### openlcd-fake-driver

Simulates a LCD.

Useful for testing.

Supports:

- Frame recording
- FPS monitor
- Brightness simulation
- Packet inspection

---

## Drivers

Every hardware is implemented as an independent crate.

Examples:

drivers/kmex-vmax

drivers/khalkan

drivers/corsair

drivers/nzxt

---

## Widgets

Every widget is an independent crate.

Widgets implement:

update()

draw()

settings()

serialize()

---

## Communication

Renderer

↓

FrameBuffer

↓

Driver

↓

Protocol

↓

Serial

---

## Goals

- 100% Rust

- Linux First

- Cross Platform

- Plugin Architecture

- No proprietary dependencies

- Hardware independent rendering

- Extensible theme system

- Fully documented protocol implementations