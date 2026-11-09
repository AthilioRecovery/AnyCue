# ADR 0001: Rust / Tauri / local-first

**Date:** 2026-11-09  
**Status:** Accepted for alpha

## Context

The app needs desktop audio, control over latency, a clean UI and an open
source architecture that a contributor can understand without reverse
engineering existing applications.

## Decision

- Rust for the native decoder, audio I/O, DSP core and CLI.
- Tauri 2 + Svelte 5 + TypeScript for the desktop UI.
- Browser-preview implemented separately in vanilla JavaScript for immediate
  runnable evaluation without access to Windows or a compiler.
- External virtual cable for Windows v1, no bundled driver.
- No telemetry, no remote API by default. A daemon/API is deferred until
  the audio engine is stable, instead of advertising a nonfunctional API.

## Consequences

The alpha native UI process owns the engine. A future separate daemon will
require IPC, lifecycle management and a redesigned event stream. This tradeoff
allows a narrow initial implementation but is **not** the final target.
