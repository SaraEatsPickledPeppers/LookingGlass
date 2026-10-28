# Linux Performance Tuning Guide

## Profiling Commands

### Standardized Benchmark

Run a headless benchmark with JSON output:

```bash
# Basic benchmark (300 frames ≈ 5 seconds at 60 FPS)
LOOKING_GLASS_FRAME_PROFILE=./perf_results cargo run --release -- --profile --frames 300 --map skool1

# With data path
LOOKING_GLASS_FRAME_PROFILE=./perf_results cargo run --release -- --profile --frames 300 --data /path/to/alice --map skool1
```

Output: `./perf_results/skool1.json` with detailed metrics:
- Frame timing (avg/min/max/median/p95/p99)
- CPU span breakdown by subsystem
- Memory usage over time
- GPU timing (placeholder for future Vulkan queries)

### Runtime Overlay (F11)

Press **F11** during gameplay to toggle the performance overlay showing:
- Current FPS (with min/max)
- Frame time in ms (with min/max)
- Memory usage (MB)
- Total frames rendered

## Linux-Specific Profiling Tools

### perf (CPU Profiling)

```bash
# Record performance data
perf record -g -- cargo run --release -- --map skool1 --frames 600

# Generate report
perf report

# Flamegraph (requires flamegraph.pl)
perf script | ./flamegraph.pl > perf.svg
```

Focus areas to investigate:
- `frame_cpu` span breakdown
- `scene.draw` / render passes
- NPC/physics update loops
- Asset loading stalls

### perf stat (Hardware Counters)

```bash
# Cache misses, branch prediction, instructions per cycle
perf stat -e cycles,instructions,cache-references,cache-misses,branch-misses \
  cargo run --release -- --map skool1 --frames 300
```

### Valgrind/Callgrind (Detailed CPU)

```bash
valgrind --tool=callgrind --dump-instr=yes \
  cargo run --release -- --map skool1 --frames 100

# Analyze with kcachegrind or gprof2dot
```

### Intel VTune / AMD uProf (Advanced)

GUI-based profilers with GPU integration. Useful for:
- Microarchitecture analysis
- Memory bandwidth saturation
- SIMD utilization

### Vulkan Validation Layers (GPU)

Enable validation layers for driver overhead analysis:

```bash
VK_LAYER_PATH=/usr/share/vulkan/explicit_layer.d \
VK_INSTANCE_LAYERS=VK_LAYER_KHRONOS_validation \
cargo run --release -- --map skool1 --frames 100
```

For GPU timestamps (requires `VK_EXT_calibrated_timestamps`):
```bash
# With NVIDIA/AMD proprietary drivers
VK_EXT_calibrated_timestamps=1 cargo run --release
```

### NVIDIA Nsight Systems / AMD Radeon GPU Profiler

For GPU timeline analysis:
- Frame duration on GPU
- Draw call breakdown
- Shader occupancy
- Memory transfers

## Common Bottlenecks & Fixes

### CPU-Bound Scenarios

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| High `interactions.advance_school` time | Complex mover logic | Profile individual movers, consider spatial partitioning |
| High `npcs.update` time | Too many active NPCs | Implement distance culling, LOD |
| High `scene.draw` submit time | Too many draw calls | Batch instanced meshes, reduce state changes |
| High `collision::World` time | Dense brush geometry | Simplify collision meshes, use broad-phase |

### Memory Issues

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| Growing memory over time | Resource leaks | Check `Assets` cache eviction, `Drop` implementations |
| High baseline RSS | Large asset loading | Lazy-load textures, stream audio |
| Frequent GC pauses | N/A (Rust) | N/A |

### GPU-Bound Scenarios

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| High frame time, low CPU usage | Fill rate / overdraw | Reduce transparency, optimize shader complexity |
| Spikes on level transition | Shader compilation | Pre-warm pipeline cache |
| High VRAM usage | Texture resolution | Compress textures (BCn), mipmapping |

## Recommended Launch Options

```bash
# Maximum performance (disable vsync for benchmarking)
cargo run --release -- --map skool1 --frames 300

# With perf integration
perf record -g -o perf.data -- cargo run --release -- --map skool1 --frames 600

# Memory profiling
heaptrack cargo run --release -- --map skool1 --frames 300
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `LOOKING_GLASS_FRAME_PROFILE` | Enable built-in profiler, set output directory |
| `RUST_LOG=trace` | Enable detailed logging (with `--verbose`) |
| `MESA_GL_VERSION_OVERRIDE=4.5` | Force OpenGL version (Mesa) |
| `RADV_PERFTEST=aco` | AMD RADV shader compiler (ACO) |
| `__GL_SHADER_DISK_CACHE=1` | NVIDIA shader disk cache |

## CPU Governor

For consistent benchmarking:

```bash
# Set performance governor (requires root)
cpupower frequency-set -g performance

# Or per-core
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
```

## Kernel Parameters

```bash
# Reduce latency spikes
echo 1 > /proc/sys/kernel/sched_autogroup_enabled
echo 0 > /proc/sys/kernel/watchdog
```

## Analysis Workflow

1. **Baseline**: Run `--profile --frames 300` → examine JSON
2. **CPU**: `perf record -g` → identify hot functions
3. **Memory**: `heaptrack` or built-in memory tracking
4. **GPU**: Nsight/RGP → analyze draw calls, shaders
5. **Iterate**: Apply fixes, re-benchmark

## Reporting

When filing performance issues, include:
- JSON output from `--profile`
- `perf report` top 20 functions
- GPU profiler screenshot (if GPU-bound)
- Hardware specs: CPU, GPU, driver version, kernel version
- Launch command used