//! Optional local CPU/GPU profiling; inactive during ordinary play.
//! Enhanced with detailed metrics for performance analysis.

use std::{
    cell::RefCell,
    collections::BTreeMap,
    sync::{
        atomic::{AtomicU64, Ordering},
        OnceLock,
    },
    time::Instant,
};

thread_local! {
    static SAMPLES: RefCell<BTreeMap<&'static str, Vec<f64>>> = RefCell::default();
    static GPU_SAMPLES: RefCell<BTreeMap<&'static str, Vec<f64>>> = RefCell::default();
    static MEMORY_SAMPLES: RefCell<Vec<u64>> = RefCell::default();
}

/// Global profiling state
static PROFILING_ENABLED: OnceLock<bool> = OnceLock::new();
static FRAME_COUNT: AtomicU64 = AtomicU64::new(0);
static TOTAL_FRAME_TIME: AtomicU64 = AtomicU64::new(0); // nanoseconds
static MIN_FRAME_TIME: AtomicU64 = AtomicU64::new(u64::MAX);
static MAX_FRAME_TIME: AtomicU64 = AtomicU64::new(0);
static START_TIME: OnceLock<Instant> = OnceLock::new();

/// Get current memory usage in bytes (Linux-specific)
fn current_memory_usage() -> u64 {
    #[cfg(target_os = "linux")]
    {
        std::fs::read_to_string("/proc/self/status")
            .ok()
            .and_then(|content| {
                content.lines().find_map(|line| {
                    if line.starts_with("VmRSS:") {
                        line.split_whitespace().nth(1).and_then(|v| v.parse::<u64>().ok())
                    } else {
                        None
                    }
                })
            })
            .map(|kb| kb * 1024)
            .unwrap_or(0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        0
    }
}

fn enabled() -> bool {
    *PROFILING_ENABLED.get_or_init(|| std::env::var_os("LOOKING_GLASS_FRAME_PROFILE").is_some())
}

/// Returns true if profiling is enabled via CLI flag or env var
pub fn is_enabled() -> bool {
    enabled()
}

/// Initialize profiling from CLI flag
pub fn init_from_cli(profile: bool) {
    if profile {
        PROFILING_ENABLED.set(true).ok();
        START_TIME.set(Instant::now()).ok();
    }
}

/// CPU profiling span
pub struct Span {
    label: &'static str,
    start: Option<Instant>,
}

pub fn span(label: &'static str) -> Span {
    Span {
        label,
        start: enabled().then(Instant::now),
    }
}

impl Span {
    pub fn cancel(&mut self) {
        self.start = None;
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(start) = self.start {
            let ms = start.elapsed().as_secs_f64() * 1000.;
            SAMPLES.with(|s| {
                let mut s = s.borrow_mut();
                let v = s.entry(self.label).or_default();
                if v.len() < 10000 {
                    v.push(ms);
                }
            });
        }
    }
}

/// GPU timing span (placeholder for future Vulkan timestamp queries)
pub struct GpuSpan {
    label: &'static str,
    start: Option<Instant>,
}

impl GpuSpan {
    pub fn new(label: &'static str) -> Self {
        Self {
            label,
            start: enabled().then(Instant::now),
        }
    }

    pub fn end(self) {
        if let Some(start) = self.start {
            let ms = start.elapsed().as_secs_f64() * 1000.;
            GPU_SAMPLES.with(|s| {
                let mut s = s.borrow_mut();
                let v = s.entry(self.label).or_default();
                if v.len() < 10000 {
                    v.push(ms);
                }
            });
        }
    }
}

/// Record a frame time for FPS calculation
pub fn record_frame_time(dt_ns: u64) {
    if !enabled() {
        return;
    }
    FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    TOTAL_FRAME_TIME.fetch_add(dt_ns, Ordering::Relaxed);

    // Update min/max
    loop {
        let current_min = MIN_FRAME_TIME.load(Ordering::Relaxed);
        if dt_ns >= current_min || MIN_FRAME_TIME.compare_exchange_weak(current_min, dt_ns, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            break;
        }
    }
    loop {
        let current_max = MAX_FRAME_TIME.load(Ordering::Relaxed);
        if dt_ns <= current_max || MAX_FRAME_TIME.compare_exchange_weak(current_max, dt_ns, Ordering::Relaxed, Ordering::Relaxed).is_ok() {
            break;
        }
    }
}

/// Record current memory usage
pub fn record_memory() {
    if !enabled() {
        return;
    }
    let mem = current_memory_usage();
    if mem > 0 {
        MEMORY_SAMPLES.with(|s| {
            let mut s = s.borrow_mut();
            if s.len() < 10000 {
                s.push(mem);
            }
        });
    }
}

/// Get profiling statistics as JSON
pub fn get_stats(map: &str) -> serde_json::Value {
    if !enabled() {
        return serde_json::json!({});
    }

    let cpu_stats: BTreeMap<_, _> = SAMPLES.with(|s| {
        s.borrow()
            .iter()
            .map(|(name, v)| {
                let mut samples = v.iter().skip(30).copied().collect::<Vec<_>>();
                samples.sort_by(f64::total_cmp);
                let n = samples.len();
                if n == 0 {
                    return (*name, serde_json::json!({}));
                }
                let sum: f64 = samples.iter().sum();
                let avg = sum / n as f64;
                let median = samples[n / 2];
                let p95 = samples[(n * 95 / 100).min(n - 1)];
                let p99 = samples[(n * 99 / 100).min(n - 1)];

                (*name, serde_json::json!({
                    "samples": n,
                    "avg_ms": avg,
                    "median_ms": median,
                    "p95_ms": p95,
                    "p99_ms": p99,
                    "min_ms": samples[0],
                    "max_ms": samples[n - 1],
                }))
            })
            .collect()
    });

    let gpu_stats: BTreeMap<_, _> = GPU_SAMPLES.with(|s| {
        s.borrow()
            .iter()
            .map(|(name, v)| {
                let mut samples = v.iter().skip(30).copied().collect::<Vec<_>>();
                samples.sort_by(f64::total_cmp);
                let n = samples.len();
                if n == 0 {
                    return (*name, serde_json::json!({}));
                }
                let sum: f64 = samples.iter().sum();
                let avg = sum / n as f64;
                let median = samples[n / 2];
                let p95 = samples[(n * 95 / 100).min(n - 1)];

                (*name, serde_json::json!({
                    "samples": n,
                    "avg_ms": avg,
                    "median_ms": median,
                    "p95_ms": p95,
                    "min_ms": samples[0],
                    "max_ms": samples[n - 1],
                }))
            })
            .collect()
    });

    let memory_stats = MEMORY_SAMPLES.with(|s| {
        let samples = s.borrow();
        if samples.is_empty() {
            serde_json::json!({})
        } else {
            let mut sorted = samples.clone();
            sorted.sort();
            let n = sorted.len();
            let sum: u64 = sorted.iter().sum();
            let avg = sum / n as u64;
            serde_json::json!({
                "samples": n,
                "avg_bytes": avg,
                "avg_mb": avg as f64 / 1_048_576.0,
                "min_bytes": sorted[0],
                "max_bytes": sorted[n - 1],
                "min_mb": sorted[0] as f64 / 1_048_576.0,
                "max_mb": sorted[n - 1] as f64 / 1_048_576.0,
            })
        }
    });

    let frame_count = FRAME_COUNT.load(Ordering::Relaxed);
    let total_time_ns = TOTAL_FRAME_TIME.load(Ordering::Relaxed);
    let min_frame_ns = MIN_FRAME_TIME.load(Ordering::Relaxed);
    let max_frame_ns = MAX_FRAME_TIME.load(Ordering::Relaxed);

    let (avg_fps, avg_frame_ms, min_fps, max_fps) = if frame_count > 0 && total_time_ns > 0 {
        let avg_frame_ms = (total_time_ns as f64 / frame_count as f64) / 1_000_000.0;
        let avg_fps = 1000.0 / avg_frame_ms;
        let min_frame_ms = (min_frame_ns as f64) / 1_000_000.0;
        let max_frame_ms = (max_frame_ns as f64) / 1_000_000.0;
        let min_fps = if max_frame_ms > 0.0 { 1000.0 / max_frame_ms } else { 0.0 };
        let max_fps = if min_frame_ms > 0.0 { 1000.0 / min_frame_ms } else { 0.0 };
        (avg_fps, avg_frame_ms, min_fps, max_fps)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    let elapsed_secs = START_TIME.get().map(|t| t.elapsed().as_secs_f64()).unwrap_or(0.0);

    serde_json::json!({
        "map": map,
        "duration_secs": elapsed_secs,
        "frames": frame_count,
        "fps": {
            "avg": avg_fps,
            "min": min_fps,
            "max": max_fps,
        },
        "frame_time_ms": {
            "avg": avg_frame_ms,
            "min": min_frame_ns as f64 / 1_000_000.0,
            "max": max_frame_ns as f64 / 1_000_000.0,
        },
        "cpu": cpu_stats,
        "gpu": gpu_stats,
        "memory": memory_stats,
    })
}

/// Write profiling report to JSON file
pub fn report(map: &str) -> anyhow::Result<()> {
    if !enabled() {
        return Ok(());
    }

    let stats = get_stats(map);
    let dir = std::path::PathBuf::from(std::env::var_os("LOOKING_GLASS_FRAME_PROFILE").unwrap());
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(format!("{map}.json")), serde_json::to_vec_pretty(&stats)?)?;
    Ok(())
}

/// Get current real-time performance metrics for overlay display
pub fn get_realtime_metrics() -> Option<RealtimeMetrics> {
    if !enabled() {
        return None;
    }

    let frame_count = FRAME_COUNT.load(Ordering::Relaxed);
    let total_time_ns = TOTAL_FRAME_TIME.load(Ordering::Relaxed);
    let min_frame_ns = MIN_FRAME_TIME.load(Ordering::Relaxed);
    let max_frame_ns = MAX_FRAME_TIME.load(Ordering::Relaxed);

    if frame_count == 0 {
        return None;
    }

    let avg_frame_ms = (total_time_ns as f64 / frame_count as f64) / 1_000_000.0;
    let avg_fps = if avg_frame_ms > 0.0 { 1000.0 / avg_frame_ms } else { 0.0 };
    let min_frame_ms = min_frame_ns as f64 / 1_000_000.0;
    let max_frame_ms = max_frame_ns as f64 / 1_000_000.0;
    let min_fps = if max_frame_ms > 0.0 { 1000.0 / max_frame_ms } else { 0.0 };
    let max_fps = if min_frame_ms > 0.0 { 1000.0 / min_frame_ms } else { 0.0 };

    let memory_mb = current_memory_usage() as f64 / 1_048_576.0;

    Some(RealtimeMetrics {
        fps: avg_fps,
        frame_time_ms: avg_frame_ms,
        min_fps,
        max_fps,
        min_frame_ms,
        max_frame_ms,
        memory_mb,
        frame_count,
    })
}

#[derive(Debug, Clone)]
pub struct RealtimeMetrics {
    pub fps: f64,
    pub frame_time_ms: f64,
    pub min_fps: f64,
    pub max_fps: f64,
    pub min_frame_ms: f64,
    pub max_frame_ms: f64,
    pub memory_mb: f64,
    pub frame_count: u64,
}

/// Reset all profiling counters (useful for benchmarking)
pub fn reset() {
    FRAME_COUNT.store(0, Ordering::Relaxed);
    TOTAL_FRAME_TIME.store(0, Ordering::Relaxed);
    MIN_FRAME_TIME.store(u64::MAX, Ordering::Relaxed);
    MAX_FRAME_TIME.store(0, Ordering::Relaxed);
    SAMPLES.with(|s| s.borrow_mut().clear());
    GPU_SAMPLES.with(|s| s.borrow_mut().clear());
    MEMORY_SAMPLES.with(|s| s.borrow_mut().clear());
    START_TIME.set(Instant::now()).ok();
}