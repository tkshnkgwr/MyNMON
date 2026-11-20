# Footprints Measurement Report (FOOTPRINTS.md)

**English** | [日本語版](../ja/FOOTPRINTS.md)

This report documents the actual binary sizes and memory usage of `MyNMON` in optimized release builds.

---

## 1. Environment

- **OS**: Windows 11 Pro (Build 26200)
- **CPU**: Intel(R) Core(TM) i7 / 12 threads
- **RAM**: 16 GB
- **Rust Version**: rustc 1.100.0 (measured on 2026-11-20)
- **Key Dependencies**: `sysinfo 0.40.4`, `crossterm 0.29.0`, `common_lib 0.4.0`
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

| Configuration                | Command                                                      | Binary Size                 |
| :--------------------------- | :----------------------------------------------------------- | :-------------------------- |
| **Full Build (default)**     | `cargo build --release`                                      | **302,592 bytes (~296 KB)** |
| **Minimal Build (cpu only)** | `cargo build --release --no-default-features --features cpu` | **256,000 bytes (~250 KB)** |

> [!TIP]
> Upgrading to `sysinfo 0.40.4` (which eliminated legacy `windows v0.52` and `rayon` dependencies) reduced the full-feature release binary from ~672 KB down to **~296 KB (~56% reduction)**. Selecting only the `cpu` feature further reduces it to **~250 KB**.

### 2.2 Run-time Memory Footprint

We measured the Working Set memory usage of the process after running for 2 seconds:

- **Process Name**: `MyNMON.exe`
- **Physical Memory Usage (WorkingSet64)**: **~21.3 MB (22,409,216 bytes)**

> [!NOTE]
> The application reserves around 20 MB of memory because the underlying `sysinfo` crate maintains internal buffers caching active process lists and hardware statistics. This footprint is exceptionally lightweight for a CUI system monitor and will not affect other applications.

---

## 3. Conclusion

`MyNMON` achieves an extremely compact binary size (~296 KB full build / ~250 KB minimal build) and low memory/CPU utilization (~21.3 MB).
This enables the application to run smoothly in resource-constrained environments or on production servers for real-time monitoring.
We will strive to keep future feature additions within this resource limit (binary size < 350 KB, memory usage < 25 MB).
