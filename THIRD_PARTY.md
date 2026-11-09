# Third-party software

AnyCue source is licensed under [MIT](LICENSE). Dependencies are resolved through Cargo and npm; their exact versions depend on the generated lockfiles. The list below covers primary direct dependencies only, not all transitives.

| Dependency | Purpose | Source |
|---|---|---|
| Tauri 2, tauri-plugin-dialog, tauri-build | Windows app shell, native dialogs | https://tauri.app |
| Svelte 5 | Desktop UI framework | https://svelte.dev |
| Vite, TypeScript, Svelte tooling | Frontend toolchain | https://vite.dev |
| CPAL 0.15 | Windows audio I/O through WASAPI | https://github.com/RustAudio/cpal |
| Symphonia 0.5 | Audio decoding | https://github.com/pdeljanov/Symphonia |
| crossbeam-queue 0.3 | Lock-free audio sample queue | https://github.com/crossbeam-rs/crossbeam |
| serde, serde_json | Serialization and library persistence | https://serde.rs |
| uuid 1 | Sound identifiers | https://github.com/uuid-rs/uuid |

**License verification is still required before release.** This alpha's CI tests and builds dependencies, but does **not** yet run `cargo-deny` or a full npm license scanner. Do not assume this table proves all transitive licenses are compatible.

AnyCue does not include or redistribute VB-CABLE, VoiceMeeter or any virtual audio driver. Install those separately under their own terms.
