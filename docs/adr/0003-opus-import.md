# ADR 0003: Do not advertise Opus import

## Status

Accepted.

## Context

AnyCue currently uses Symphonia 0.5 for decoding. Its configured codecs do not
provide an Opus decoder, so accepting `.opus` files produces an import that
cannot reliably play. Adding a libopus adapter would introduce a native build
and Windows MSVC validation obligation that has not been completed here.

## Decision

Remove `.opus` from the accepted extensions and validate every accepted file by
probing and decoding it during import. The library records the decoded duration.

## Consequences

Users get an immediate, actionable invalid-or-unsupported-file error instead of
a sound that fails later. Opus support can be reconsidered once a compatible
dependency has had its license and Windows MSVC build validated.
