# Footprints Measurement Report (FOOTPRINTS.md)

**English** | [日本語版](../ja/FOOTPRINTS.md)

This report documents the actual binary sizes and memory usage of `MyNMON` in optimized release builds.

---

## 1. Environment

- **OS**: Windows 11 Pro (Build 26200)
- **CPU**: Intel(R) Core(TM) i7 / 12 threads
- **RAM**: 16 GB
- **Rust Version**: rustc 1.96.0 (or latest stable toolchain)
- **Cargo Release Profile Settings (Cargo.toml)**:
  ```toml
  [profile.release]
  opt-level = 'z'       # Optimize for size
  lto = true            # Link-time Optimization
  codegen-units = 1     # Integrate compilation units
  panic = 'abort'       # Disable panic unwinding
  strip = true          # Strip symbols
  ```

---

## 2. Metrics

### 2.1 Binary Size
Comparison of binary sizes compiled under the release profile:

| Configuration | Command | Binary Size |
| :--- | :--- | :--- |
| **Full Build (default)** | `cargo build --release` | **688,128 bytes (~672 KB)** |
| **Minimal Build (cpu only)** | `cargo build --release --no-default-features --features cpu` | **331,776 bytes (~324 KB)** |

> [!TIP]
   > Disabling default features and selecting only required modules (such as CPU) using Cargo Features can reduce binary size by approximately 52% (down to 324 KB).

### 2.2 Run-time Memory Footprint
We measured the Working Set memory usage of the process after running for 2 seconds:

- **Process Name**: `MyNMON.exe`
- **Physical Memory Usage (WorkingSet64)**: **~21.3 MB (22,409,216 bytes)**

> [!NOTE]
   > The application reserves around 20 MB of memory because the underlying `sysinfo` crate maintains internal buffers caching active process lists and hardware statistics. This footprint is exceptionally lightweight for a CUI system monitor and will not affect other applications.

---

## 3. Conclusion

`MyNMON` achieves an extremely compact binary size (~324 KB) and low memory/CPU utilization (~21.3 MB).
This enables the application to run smoothly in background scripts or as a foreground real-time utility in environments with severely constrained resources or on production servers.
We will strive to keep future feature additions within this resource limit (binary size < 350 KB, memory usage < 25 MB).
