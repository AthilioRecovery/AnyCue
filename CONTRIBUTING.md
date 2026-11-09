# Contributing to AnyCue

Thanks for wanting to help! This project is early, so small, focused contributions are the easiest to review.

## Ground rules

- **Clean-room.** AnyCue is an independent project. Do not copy or translate code, assets, text, protocols, file formats or layouts from other soundboards, and do not submit anything derived from decompiled software.
- **Be kind.** See the [Code of Conduct](CODE_OF_CONDUCT.md).
- **No telemetry.** Contributions must not add tracking or default network calls.

## Getting started

```bash
git clone https://github.com/AthilioRecovery/AnyCue.git
cd AnyCue
cargo build --workspace
cargo test --workspace
```

Requires Windows 10/11, Rust (stable) and Node.js 20+.

## Workflow

1. Open an issue first for anything larger than a small fix.
2. Fork, create a branch (`feat/short-name` or `fix/short-name`), and keep the change small.
3. Add or update tests.
4. Make sure formatting and lints pass: `cargo fmt`, `cargo clippy --workspace -- -D warnings`.
5. Open a pull request describing what changed and why.

## Commit messages

Use [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `chore:`, `refactor:`, `test:`, `ci:`. Write in English, imperative mood, under about 70 characters.

```
feat(audio): add microphone device selection
```

## Real-time audio code

The audio thread must not allocate, block on locks, or do I/O. Any `unsafe` block needs a `// SAFETY:` comment explaining why it is sound.

## Reporting bugs and security issues

Bugs go to [GitHub Issues](https://github.com/AthilioRecovery/AnyCue/issues). Security problems go to [eu@athiliodev.com.br](mailto:eu@athiliodev.com.br), see [SECURITY.md](SECURITY.md).
