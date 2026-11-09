# Realtime audio status and known gaps

The native callbacks receive WASAPI/CPAL audio blocks. They do not perform
disk reads, network calls or codec decoding. The output callback polls a
non-blocking command channel and plays up to 24 voices from pre-decoded `Arc`
buffers. The microphone capture callback writes mono samples to a bounded
`ArrayQueue` without blocking.

**Important:** This is not yet strictly real-time safe. Voice removal or `Arc`
drops may deallocate inside the callback. The standard MPSC channel is not a
certified wait-free message transport. Work to do:

- Switch to a bounded SPSC control/event queue whose destructor work is deferred.
- Pre-allocate voice slots and release decoded buffers off callback thread.
- Use per-device resampling / drift compensation for independent audio clocks.
- Design a lookahead limiter and measured LUFS/R128 normalization.
- Benchmark callback p99, xrun frequency and round-trip latency with WASAPI.
- Avoid side effects in callbacks (except atomic meter updates).
- Exercise hotplug, 44.1/48/96 kHz devices, and long-running capture.

`soft_limit` currently uses simple algebraic saturation. It prevents clipping
in the output but is **not** a substitute for a true peak limiter, dither,
LUFS normalization, noise gate or dynamic ducking.
