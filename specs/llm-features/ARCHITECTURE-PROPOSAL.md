# Architecture Proposal: Eliminate C++ Build Dependencies

## Problem

Current build compiles **4 heavy C/C++ libraries from source** every clean build:

| Crate | What it compiles | Time |
|-------|-----------------|------|
| `whisper-rs-sys` | whisper.cpp + ggml + CUDA kernels | ~2 min |
| `ort-sys` | ONNX Runtime (C++) | ~45s |
| `knf-rs-sys` | Kaldi native functions (C++) | ~15s |
| `aws-lc-sys` | AWS libcrypto (C/asm) | ~30s |

**Total: ~3.5 min** of C++ compilation. The actual Rust code compiles in ~15s.

Additionally: CUDA 12.9 + GCC 15 (Fedora 43) is broken out of the box — requires GCC 14 as CUDA host compiler + a header patch for glibc 2.41 incompatibility.

## Proposed Architecture: External Servers + Pure Rust

**Replace in-process C++ with HTTP calls to pre-built servers:**

```
BEFORE (monolith, compiles C++):
┌─────────────────────────────┐
│  plaude-cli binary          │
│  ├─ whisper-rs (C++)        │  ← 2 min compile
│  ├─ pyannote-rs/ort (C++)   │  ← 1 min compile
│  ├─ genai (pure Rust)       │  ← fast
│  └─ device transport        │  ← fast
└─────────────────────────────┘

AFTER (thin client, pure Rust):
┌──────────────────┐     ┌───────────────────────┐
│  plaude-cli      │────▶│ whisper.cpp server     │
│  (pure Rust,     │     │ (pre-built binary,     │
│   ~15s build)    │     │  GPU-accelerated)      │
│                  │────▶│                        │
│                  │     └───────────────────────┘
│                  │     ┌───────────────────────┐
│                  │────▶│ Ollama                 │
│                  │     │ (LLM summarization,    │
│                  │     │  already installed)     │
└──────────────────┘     └───────────────────────┘
```

### Change 1: Transcription → whisper.cpp server

**Instead of:** `whisper-rs` (compiles whisper.cpp from source)
**Use:** `whisper-server` running locally, accessed via OpenAI-compatible HTTP API

```bash
# User installs whisper.cpp server once (pre-built, GPU-enabled):
# Fedora: dnf install whisper-cpp  (or build from source once)
# Or: docker run ghcr.io/ggml-org/whisper.cpp:main-cuda

# Server runs in background:
whisper-server --model ~/.cache/plaude/models/ggml-large-v3-turbo.bin \
  --host 127.0.0.1 --port 8178 --convert

# plaude-cli calls it via HTTP:
plaude transcribe recording.wav
# → POST http://127.0.0.1:8178/v1/audio/transcriptions
```

**Rust side:** Replace `whisper-rs` with a simple HTTP POST using `reqwest` (already in deps). The API is OpenAI-compatible (`/v1/audio/transcriptions`).

**Benefits:**
- Eliminates whisper-rs-sys (2 min compile, CUDA cmake, GCC 14 hack)
- Users choose their own GPU backend at whisper-server install time
- Works with any OpenAI-compatible STT API (cloud fallback)
- Server stays running between commands — no model reload per invocation

### Change 2: Diarization → native-pyannote-rs (pure Rust)

**Instead of:** `pyannote-rs` (wraps ONNX Runtime C++)
**Use:** `native-pyannote-rs` (pure Rust via Burn framework)

- Same models, same accuracy
- No ort-sys (45s compile), no knf-rs-sys (15s compile)
- Burn ndarray backend (CPU) default, WGPU/CUDA optional
- v0.1.4 on crates.io, actively maintained

### Change 3: TLS → rustls-only (drop OpenSSL/aws-lc)

`aws-lc-sys` compiles because `ureq` (HTTP client for model downloads) pulls in native TLS. Switch to `rustls` everywhere:
- `ureq` already supports `rustls` feature
- `reqwest` already uses `rustls` in our genai dep chain
- Drop `openssl` and `aws-lc-sys` entirely

## Impact on Build Time

| Component | Before | After |
|-----------|--------|-------|
| whisper-rs-sys | ~2 min | **0** (HTTP client) |
| ort-sys + knf-rs-sys | ~1 min | **0** (native-pyannote-rs, pure Rust) |
| aws-lc-sys | ~30s | **0** (rustls-only) |
| Rust code | ~15s | ~18s (slightly more HTTP code) |
| **Total clean build** | **~4 min** | **~18s** |

## Impact on User Experience

| Aspect | Before | After |
|--------|--------|-------|
| Build | 4 min, needs CUDA toolkit + GCC 14 | 18s, no C toolchain |
| First transcription | Model loads per invocation (~5s) | Server pre-loaded, instant |
| GPU setup | Cargo feature flags + cmake | whisper-server handles it |
| Offline | Yes | Yes (local server) |
| Cloud fallback | No | Yes (OpenAI API compatible) |

## Migration Path

1. **Phase A:** Add `whisper-server` HTTP client alongside `whisper-rs`, behind feature flag
2. **Phase B:** Switch `pyannote-rs` → `native-pyannote-rs`
3. **Phase C:** Switch `ureq` TLS to rustls, drop `openssl`/`aws-lc-sys`
4. **Phase D:** Make HTTP-based transcription the default, deprecate in-process
5. **Phase E:** Remove `whisper-rs`, `ort-sys`, `knf-rs-sys` from deps

Each phase ships independently with value.

## Open Questions

1. **Should plaude manage the whisper-server lifecycle?** Options: (a) user manages it, (b) `plaude transcribe --serve` starts it, (c) auto-start on first use
2. **Diarization via server?** whisper.cpp server doesn't support diarization yet. Keep `native-pyannote-rs` in-process for now, or run a separate diarization server?
3. **Model management:** Who downloads models — plaude-cli or the server? Recommendation: server manages its own models.
