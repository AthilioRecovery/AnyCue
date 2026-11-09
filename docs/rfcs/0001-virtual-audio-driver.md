# RFC: Optional Windows virtual audio device (v2, unimplemented)

A future optional native virtual microphone backend could integrate with
Windows Audio Processing Objects / appropriate virtual audio driver mechanisms,
after examining Microsoft's supported driver model and signing requirements.
That work may involve C++ and Windows driver tooling. The current app uses an
external cable and does **not** claim to install or implement a driver.

Open topics: driver installation/admin consent, Windows Update compatibility,
code signing, rollback, device enumeration, latency, security, accessibility,
uninstall behavior, test matrix. No implementation approved yet.
