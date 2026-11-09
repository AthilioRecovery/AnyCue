<div align="center">

<img src="assets/banner.svg" alt="AnyCue" width="720">

# AnyCue

**Your sounds, whenever you want.**

Open-source soundboard for Windows 10/11. Built with Rust, Svelte 5 and Tauri 2.

[![Windows CI](https://github.com/AthilioRecovery/AnyCue/actions/workflows/windows-build.yml/badge.svg)](https://github.com/AthilioRecovery/AnyCue/actions/workflows/windows-build.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![Windows](https://img.shields.io/badge/platform-Windows%2010%2F11-lightgrey)

[English](README.md) · [Português](README.pt-BR.md) · [Windows setup](WINDOWS.md) · [Roadmap](docs/ROADMAP.md)

</div>

> **Alpha v0.1.1:** A genuine Windows desktop application, not a browser website. Compiled Windows binaries are produced in GitHub Actions. Advanced features listed in the roadmap are **not shipped yet**.

## Download for Windows

1. Open [Build AnyCue for Windows](https://github.com/AthilioRecovery/AnyCue/actions/workflows/windows-build.yml).
2. Open the latest **successful** run and download **AnyCue-Windows-x64** under Artifacts.
3. Unzip and run `AnyCue.exe` (portable) or the `AnyCue_0.1.1_x64-setup.exe` installer.

A virtual audio cable such as VB-CABLE (installed separately) is necessary to route audio into Discord or games. Windows 10 may require the Microsoft WebView2 Runtime. See [WINDOWS.md](WINDOWS.md).

**Tauri bundles the interface inside the executable.** Development mode runs Vite on localhost; the production executable needs no HTTP server or IP.

## What works today

| Feature | Status |
|---|---|
| Windows desktop interface using the compact dark theme | Implemented |
| Import MP3, WAV, FLAC, OGG, Opus, AAC, M4A | Implemented through Symphonia (codec support may vary) |
| Local sound library, categories, search, favorites and edits | Implemented |
| Select real microphone, speaker output and virtual playback output | Implemented |
| Play sounds to speakers, virtual output or both | Implemented |
| Experimental mic capture into the virtual output mix | Implemented, requires device testing |
| Per-sound/master volume, stop, approximate output meter | Implemented |
| Keyboard keys 1–9 and Escape | Works only when the window is focused |
| Global hotkeys, automated PTT, LUFS, recording, TTS, waveform editor | Planned; not implemented |
| Named-pipe API, WebSocket, SDKs and service daemon | Planned; not implemented |

The initial UI was adapted from the user-supplied compact dark frontend. The concept artwork below is still concept art, **not a screenshot** of the released interface.

<div align="center"><img src="assets/preview-hotbar.svg" alt="Earlier AnyCue concept art" width="760"></div>

## Architecture

- `apps/desktop/`: Svelte 5/TypeScript UI and Tauri 2 Windows window
- `crates/core/`: testable DSP primitives
- `crates/audio-io/`: CPAL/WASAPI audio streams and Symphonia decoder
- `crates/library/`: versioned JSON audio library
- `crates/cli/`: device diagnosis and basic playback
- `.github/workflows/`: Windows tests and executable/installer build

CPAL streams remain on a dedicated worker thread; Tauri commands communicate by channel. Audio and library data stay local. No telemetry is implemented.

## Build from source

Install **Windows 10/11**, Node.js 22, Rust stable MSVC, Visual Studio C++ Build Tools and WebView2, then:

```powershell
git clone https://github.com/AthilioRecovery/AnyCue.git
cd AnyCue
.\ABRIR-DESKTOP.bat
# Or: .\COMPILAR-WINDOWS.bat
```

The installer and standalone exe are created in `target/release/` after a successful Windows build. See [WINDOWS.md](WINDOWS.md).

## Roadmap and contributing

See [ROADMAP](docs/ROADMAP.md), [CONTRIBUTING](CONTRIBUTING.md), [SECURITY](SECURITY.md) and [CODE_OF_CONDUCT](CODE_OF_CONDUCT.md). For issues, use [GitHub Issues](https://github.com/AthilioRecovery/AnyCue/issues).

## License

MIT. See [LICENSE](LICENSE) and [THIRD_PARTY.md](THIRD_PARTY.md).
