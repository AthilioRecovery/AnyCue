# Security Policy

## Supported versions

AnyCue is in early development. Only the latest release (and `main`) receives security fixes.

## Reporting a vulnerability

Please **do not open a public issue** for security problems.

Send an email to **[eu@athiliodev.com.br](mailto:eu@athiliodev.com.br)** with:

- a description of the problem and its impact,
- steps to reproduce (or a proof of concept),
- the AnyCue version and your Windows version.

You can expect an acknowledgement within a few days. Once the issue is fixed, you will be credited in the release notes unless you prefer to stay anonymous.

## Scope

Of particular interest: the local automation API (named pipe and WebSocket), global hotkey and Auto Keys handling, installer and update integrity, and anything that could expose data outside your machine.

AnyCue sends no telemetry and makes no network calls by default.
