# Roadmap

Only checked boxes are implemented in source; native code still needs a full
Windows build and hardware verification.

- [x] M0: MIT license, own branding, clean-room docs, monorepo and CI config
- [x] M1 (source): output enumeration, offline decoding, direct playback, meter
- [x] M2 (source): microphone capture, virtual output selection, basic limiter
- [x] M3 (source): local library, categories, sound grid, favorites, volume
- [ ] M4: OS-global hotkeys, key conflict detection, Auto Keys PTT
- [ ] M5: adaptive resampler, accurate EBU R128 LUFS, gate, block voice, delay
- [ ] M6: native recorder, non-destructive editor, Windows TTS
- [ ] M7: separately running daemon, named pipes, JSON-RPC 2.0,
      localhost opt-in WebSocket and Python/TS SDKs
- [ ] M8: profiles, settings schema, i18n, tray, auto-start, accessibility audit
- [ ] M9: reproducible Windows release, signed installer, SBOM, 3 real screenshots,
      benchmarks and successful CI runs

First engineering priority: compile native Windows code, fix any dependency API
changes, test routing on physical devices, then implement drift compensation
and ensure the realtime callback is allocation-free.
