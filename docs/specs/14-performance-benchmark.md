# CNWS
## Performance Benchmark Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Performance Benchmark Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (PERFORMANCE SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS Runtime Spec; CNWS Testing Spec |
| Hulu ke | Implementasi benchmark suite, CI/CD performance gates, certification |
| Otoritas | Spesifikasi tunggal untuk pengukuran performa CNWS |
| Tujuan | Setiap klaim performa MUST terukur secara objektif dan reproducible |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract      Runtime Spec         Performance Benchmark Spec    Implementation
─────────────────────     ────────────────     ──────────────────────────    ─────────────
"O(1) Cell resolve"   ──► RuntimeResolver  ──► Benchmark methodology     ──► Benchmark suite
"<10% active ratio"       Budget enforcement     Workload definitions         CI perf gates
"Bounded memory"          Cache hierarchy        Hardware profiles            Certification
"Bytes moved/token"       Prefetch               Acceptance thresholds        Performance reports
```

`[PERF-DOC-1]` Dokumen ini mendefinisikan **bagaimana setiap klaim performa CNWS diukur secara objektif**.

`[PERF-DOC-2]` Setiap klaim performa dalam Engineering Contract dan spesifikasi lainnya MUST memiliki benchmark yang sesuai dalam dokumen ini.

`[PERF-DOC-3]` Jika terjadi konflik dengan spesifikasi lain untuk hal behavior, spesifikasi tersebut menang. Untuk hal measurement methodology dan acceptance thresholds, dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-PERF-01 | Benchmark suite bersifat normatif untuk certification. |
| DF-PERF-02 | Workloads: tiny, small, medium, large. |
| DF-PERF-03 | Hardware profiles: CPU-only, GPU-small, GPU-large, Multi-GPU. |
| DF-PERF-04 | Model fixtures deterministik dengan seed tetap. |
| DF-PERF-05 | Warm cache: 3 iteration warmup sebelum measurement. |
| DF-PERF-06 | Cold cache: cache clear + OS page cache drop. |
| DF-PERF-07 | Latency diukur P50, P95, P99. |
| DF-PERF-08 | Throughput diukur tokens/second dan Cells/second. |
| DF-PERF-09 | Memory diukur peak RSS dan peak VRAM. |
| DF-PERF-10 | Bytes moved diukur melalui I/O tracing. |
| DF-PERF-11 | Active parameter ratio dihitung dari Cell activation log. |
| DF-PERF-12 | Acceptance thresholds bersifat hard untuk certification. |
| DF-PERF-13 | Benchmark MUST reproducible dengan seed tetap. |
| DF-PERF-14 | Regression detection: >10% degradation memblokir merge. |

---

# 1. Executive Summary

## 1.1 Performance Claims to Verify

`[PERF-EXEC-1]` Dokumen ini memverifikasi klaim performa berikut:

| Claim | Source | Benchmark ID |
|---|---|---|
| O(1) Cell resolve | Engineering Contract, Runtime Spec | CNWS-BENCH-RESOLVE |
| O(1) Tile lookup | Engineering Contract, Runtime Spec | CNWS-BENCH-LOOKUP |
| < 10% active parameter ratio | Engineering Contract | CNWS-BENCH-ACTIVE |
| Bounded memory | Engineering Contract, Runtime Spec | CNWS-BENCH-MEMORY |
| Bytes moved/token tracked | Engineering Contract | CNWS-BENCH-BYTES |
| Adaptive compute | Runtime Spec | CNWS-BENCH-ADAPTIVE |
| MoE selective loading | Runtime Spec | CNWS-BENCH-MOE |
| Conversion bounded-memory | Conversion Spec | CNWS-BENCH-CONV-MEM |
| Conversion throughput | Conversion Spec | CNWS-BENCH-CONV-THRU |
| Retrieval O(log N) | Memory Spec | CNWS-BENCH-MEM-RETRIEVE |

## 1.2 Measurement Philosophy

`[PERF-EXEC-2]` Prinsip pengukuran:

1. **Objective**: semua metrik terukur, bukan estimasi.
2. **Reproducible**: hasil sama untuk run yang sama dengan seed tetap.
3. **Comparable**: hasil dapat dibandingkan antar implementasi.
4. **Statistically valid**: multiple runs, percentile reporting.
5. **Hardware-aware**: hasil dinormalisasi terhadap hardware profile.

---

# 2. Benchmark Workloads

## 2.1 Workload Categories

`[PERF-WL-1]` Workload dikategorikan berdasarkan ukuran model:

| Category | Model Size | Parameters | Use Case |
|---|---|---|---|
| Tiny | < 100 MB | < 10 M | Unit testing, CI |
| Small | 100 MB – 1 GB | 10 M – 500 M | Development, quick validation |
| Medium | 1 GB – 50 GB | 500 M – 20 B | Integration testing, staging |
| Large | 50 GB – 500 GB | 20 B – 200 B | Production benchmark |
| XLarge | > 500 GB | > 200 B | Scale testing |

## 2.2 Workload Definitions

### 2.2.1 Dense Model Workloads

```yaml
workload: dense_tiny
  model_type: dense
  num_layers: 4
  hidden_dim: 512
  num_heads: 8
  vocab_size: 1000
  dtype: bf16
  estimated_size: ~12 MB
  estimated_params: ~3 M

workload: dense_small
  model_type: dense
  num_layers: 12
  hidden_dim: 2048
  num_heads: 16
  vocab_size: 32000
  dtype: bf16
  estimated_size: ~700 MB
  estimated_params: ~175 M

workload: dense_medium
  model_type: dense
  num_layers: 32
  hidden_dim: 4096
  num_heads: 32
  vocab_size: 128256
  dtype: bf16
  estimated_size: ~14 GB
  estimated_params: ~3.5 B

workload: dense_large
  model_type: dense
  num_layers: 80
  hidden_dim: 8192
  num_heads: 64
  vocab_size: 128256
  dtype: bf16
  estimated_size: ~140 GB
  estimated_params: ~35 B
```

### 2.2.2 MoE Model Workloads

```yaml
workload: moe_tiny
  model_type: moe
  num_layers: 4
  hidden_dim: 512
  num_heads: 8
  num_experts: 8
  experts_per_token: 2
  dtype: bf16
  estimated_size: ~50 MB
  total_params: ~12 M
  active_params: ~3 M

workload: moe_small
  model_type: moe
  num_layers: 12
  hidden_dim: 2048
  num_experts: 16
  experts_per_token: 2
  dtype: bf16
  estimated_size: ~3 GB
  total_params: ~750 M
  active_params: ~100 M

workload: moe_medium
  model_type: moe
  num_layers: 32
  hidden_dim: 4096
  num_experts: 64
  experts_per_token: 8
  dtype: bf16
  estimated_size: ~90 GB
  total_params: ~22 B
  active_params: ~3 B

workload: moe_large
  model_type: moe
  num_layers: 56
  hidden_dim: 6144
  num_experts: 128
  experts_per_token: 8
  dtype: bf16
  estimated_size: ~400 GB
  total_params: ~100 B
  active_params: ~12 B
```

### 2.2.3 Input Workloads

```yaml
input_short:
  sequence_length: 32
  batch_size: 1

input_medium:
  sequence_length: 512
  batch_size: 4

input_long:
  sequence_length: 4096
  batch_size: 1

input_batch:
  sequence_length: 128
  batch_size: 32
```

## 2.3 Workload Invariants

| ID | Invariant |
|---|---|
| PERF-WL-INV-1 | Workload MUST deterministik |
| PERF-WL-INV-2 | Workload MUST memiliki size estimate |
| PERF-WL-INV-3 | Workload MUST memiliki parameter count |
| PERF-WL-INV-4 | Workload fixtures MUST versioned |

---

# 3. Hardware Profiles

## 3.1 Standard Hardware Profiles

`[PERF-HW-1]` Benchmark MUST dijalankan pada hardware profile yang terstandarisasi.

| Profile ID | Name | CPU | RAM | GPU | VRAM | Storage |
|---|---|---|---|---|---|---|
| HW-CPU | CPU Only | 8 cores | 32 GB | None | — | NVMe 1 TB |
| HW-GPU-S | GPU Small | 8 cores | 32 GB | RTX 3060 | 12 GB | NVMe 1 TB |
| HW-GPU-M | GPU Medium | 16 cores | 64 GB | RTX 4090 | 24 GB | NVMe 2 TB |
| HW-GPU-L | GPU Large | 32 cores | 128 GB | A100 | 80 GB | NVMe 4 TB |
| HW-GPU-XL | GPU XLarge | 64 cores | 256 GB | H100 | 80 GB | NVMe 8 TB |
| HW-MULTI | Multi-GPU | 64 cores | 512 GB | 4× H100 | 320 GB | NVMe 16 TB |

## 3.2 Hardware Profile Requirements

`[PERF-HW-2]` Setiap benchmark report MUST mencatat:

```yaml
hardware_report:
  profile_id: HW-GPU-M
  cpu:
    model: "AMD Ryzen 9 7950X"
    cores: 16
    threads: 32
    base_clock_ghz: 4.5
  ram:
    total_gb: 64
    type: DDR5
    speed_mts: 5200
  gpu:
    model: "NVIDIA RTX 4090"
    vram_gb: 24
    compute_capability: "8.9"
    fp8_supported: true
  storage:
    type: NVMe
    capacity_tb: 2
    sequential_read_mbps: 7000
    sequential_write_mbps: 6000
    random_read_iops: 1000000
  os:
    name: "Ubuntu 24.04"
    kernel: "6.8.0"
  cnws_version: "1.0.0"
  benchmark_date: "2026-08-11T00:00:00Z"
```

## 3.3 Hardware Normalization

`[PERF-HW-3]` Untuk perbandingan antar hardware, hasil SHOULD dinormalisasi:

```text
normalized_latency = measured_latency × (reference_hw_score / actual_hw_score)
```

`[PERF-HW-4]` Normalization reference: HW-GPU-M.

## 3.4 Hardware Invariants

| ID | Invariant |
|---|---|
| PERF-HW-INV-1 | Hardware profile MUST dicatat dalam report |
| PERF-HW-INV-2 | Benchmark MUST tidak dijalankan pada hardware yang tidak memenuhi minimum |
| PERF-HW-INV-3 | Hardware monitoring MUST aktif selama benchmark |
| PERF-HW-INV-4 | Thermal throttling MUST dideteksi dan dilaporkan |

---

# 4. Model & Dataset Fixtures

## 4.1 Model Fixtures

`[PERF-FIX-1]` Model fixtures MUST deterministik.

`[PERF-FIX-2]` Model fixtures dihasilkan dengan seed tetap:

```rust
const BENCHMARK_SEED: u64 = 0x434E5753; // "CNWS"

fn generate_benchmark_model(config: WorkloadConfig) -> Model {
    let mut rng = DeterministicRng::new(BENCHMARK_SEED);
    // Generate model with deterministic weights
    // ...
}
```

## 4.2 Fixture Generation

```pseudo
function generate_fixture(workload):
    // Set seed
    rng = DeterministicRng.new(BENCHMARK_SEED)
    
    // Generate weights
    for layer in 0..workload.num_layers:
        for cell_type in [Q_PROJ, K_PROJ, V_PROJ, OUT, MLP_GATE, MLP_UP, MLP_DOWN]:
            shape = compute_shape(workload, cell_type)
            weights = rng.generate_bf16(shape)
            cell = create_cell(cell_type, weights)
    
    // Convert to .cd
    convert_to_cd(model, "fixtures/" + workload.name + ".cd")
    
    // Verify determinism
    hash1 = hash_store("fixtures/" + workload.name + ".cd")
    regenerate_fixture(workload)
    hash2 = hash_store("fixtures/" + workload.name + ".cd")
    assert hash1 == hash2
```

## 4.3 Dataset Fixtures

`[PERF-FIX-3]` Input dataset untuk benchmark:

```yaml
dataset_benchmark_v1:
  seed: 0x434E5753
  num_prompts: 1000
  prompt_length_distribution:
    min: 1
    max: 4096
    mean: 256
  generation_lengths:
    min: 1
    max: 512
    mean: 64
```

## 4.4 Fixture Invariants

| ID | Invariant |
|---|---|
| PERF-FIX-INV-1 | Fixtures MUST deterministik |
| PERF-FIX-INV-2 | Fixtures MUST versioned |
| PERF-FIX-INV-3 | Fixtures MUST memiliki hash verification |
| PERF-FIX-INV-4 | Fixtures MUST tersimpan di repository terpisah |

---

# 5. Measurement Methodology

## 5.1 General Measurement Protocol

`[PERF-MEAS-1]` Setiap benchmark MUST mengikuti protokol:

```text
1. Setup
   ├── Load model fixture
   ├── Configure hardware profile
   └── Clear caches

2. Warmup (if warm cache test)
   ├── Run 3 iterations
   └── Discard measurements

3. Measurement
   ├── Run N iterations (default: 10)
   ├── Record metrics per iteration
   └── Compute statistics

4. Teardown
   ├── Clear caches
   └── Record final state

5. Report
   ├── Raw data
   ├── Statistics (P50, P95, P99, mean, stddev)
   └── Hardware profile
```

## 5.2 Statistical Requirements

`[PERF-MEAS-2]` Setiap benchmark MUST:

| Requirement | Value |
|---|---|
| Minimum iterations | 10 |
| Warmup iterations | 3 |
| Percentiles reported | P50, P95, P99 |
| Mean reported | YES |
| Std deviation reported | YES |
| Outlier handling | Exclude > 3σ |

## 5.3 Timing Methodology

`[PERF-MEAS-3]` Timing MUST menggunakan monotonic clock.

```rust
use std::time::Instant;

fn benchmark_operation<F, T>(operation: F, iterations: usize) -> BenchmarkResult
where
    F: Fn() -> T,
{
    let mut latencies = Vec::with_capacity(iterations);
    
    // Warmup
    for _ in 0..WARMUP_ITERATIONS {
        operation();
    }
    
    // Measurement
    for _ in 0..iterations {
        let start = Instant::now();
        operation();
        let elapsed = start.elapsed();
        latencies.push(elapsed);
    }
    
    BenchmarkResult::from_latencies(latencies)
}
```

## 5.4 Memory Measurement

`[PERF-MEAS-4]` Memory MUST diukur melalui:

| Metric | Method |
|---|---|
| Peak RSS | `/proc/self/status` VmHWM atau `getrusage` |
| Peak VRAM | GPU monitoring API (NVML, ROCm) |
| Working set | RSS - shared memory |
| Allocation tracking | Custom allocator (optional) |

## 5.5 I/O Measurement

`[PERF-MEAS-5]` I/O MUST diukur melalui:

| Metric | Method |
|---|---|
| Bytes read | `iostat` atau `/proc/self/io` |
| Bytes written | `iostat` atau `/proc/self/io` |
| IOPS | I/O operation counter |
| Bytes moved (logical) | CNWS internal counter |

## 5.6 Measurement Invariants

| ID | Invariant |
|---|---|
| PERF-MEAS-INV-1 | Timing MUST monotonic clock |
| PERF-MEAS-INV-2 | Memory MUST peak measurement |
| PERF-MEAS-INV-3 | I/O MUST bytes-level tracking |
| PERF-MEAS-INV-4 | Statistics MUST P50/P95/P99 |
| PERF-MEAS-INV-5 | Warmup MUST dilakukan untuk warm cache |
| PERF-MEAS-INV-6 | Measurement MUST reproducible |

---

# 6. Warm/Cold Cache Testing

## 6.1 Cache State Definitions

`[PERF-CACHE-1]` Dua cache state yang diuji:

| State | Definition |
|---|---|
| Cold cache | Tidak ada data di cache CNWS, OS page cache di-drop |
| Warm cache | Data sudah di cache CNWS setelah 3+ akses |

## 6.2 Cold Cache Protocol

```pseudo
function cold_cache_benchmark(operation):
    // Step 1: Clear CNWS caches
    cnws_cache.clear()
    
    // Step 2: Drop OS page cache (requires root)
    sync()
    echo 3 > /proc/sys/vm/drop_caches
    
    // Step 3: Verify cold state
    assert cnws_cache.is_empty()
    assert os_page_cache_dropped()
    
    // Step 4: Run measurement
    result = measure(operation)
    
    return result
```

## 6.3 Warm Cache Protocol

```pseudo
function warm_cache_benchmark(operation):
    // Step 1: Warmup (3 iterations)
    for i in 0..3:
        operation()
    
    // Step 2: Verify warm state
    assert cnws_cache.hit_rate() > 0.90
    
    // Step 3: Run measurement
    result = measure(operation)
    
    return result
```

## 6.4 Cache Test Matrix

| Test | Cache State | Operation |
|---|---|---|
| CNWS-BENCH-COLD-LOAD | Cold | Load Tile from NVMe |
| CNWS-BENCH-WARM-LOAD | Warm | Load Tile from cache |
| CNWS-BENCH-COLD-RESOLVE | Cold | Resolve Cell + load Tiles |
| CNWS-BENCH-WARM-RESOLVE | Warm | Resolve Cell + load Tiles |
| CNWS-BENCH-COLD-INFER | Cold | Full inference |
| CNWS-BENCH-WARM-INFER | Warm | Full inference |

---

# 7. Latency Benchmarks

## 7.1 CNWS-BENCH-RESOLVE: Cell Resolution Latency

**Claim**: O(1) Cell resolve

```yaml
benchmark: CNWS-BENCH-RESOLVE
  description: Measure Cell resolution latency
  claim: O(1) with respect to total Cell count
  
  methodology:
    1. Load model with N Cells
    2. Resolve random Cell by name
    3. Measure latency
    4. Repeat for N = [100, 1K, 10K, 100K]
    5. Verify latency independent of N
  
  workloads: [dense_tiny, dense_small, dense_medium, dense_large]
  
  measurement:
    iterations: 1000
    warmup: 100
    percentiles: [P50, P95, P99]
  
  acceptance:
    p50_latency_us: "<= 1"
    p99_latency_us: "<= 10"
    scaling: "O(1) verified - latency not increasing with N"
```

## 7.2 CNWS-BENCH-LOOKUP: Tile Lookup Latency

**Claim**: O(1) Tile lookup

```yaml
benchmark: CNWS-BENCH-LOOKUP
  description: Measure Tile lookup latency
  claim: O(1) with respect to total Tile count
  
  methodology:
    1. Load model with M Tiles
    2. Lookup random Tile by ID
    3. Measure latency
    4. Repeat for M = [100, 1K, 10K, 100K, 1M]
    5. Verify latency independent of M
  
  measurement:
    iterations: 1000
    warmup: 100
    percentiles: [P50, P95, P99]
  
  acceptance:
    p50_latency_us: "<= 10"
    p99_latency_us: "<= 100"
    scaling: "O(1) verified"
```

## 7.3 CNWS-BENCH-TILE-LOAD: Tile Load Latency

```yaml
benchmark: CNWS-BENCH-TILE-LOAD
  description: Measure Tile load latency from storage
  
  variants:
    cold_nvme:
      cache_state: cold
      source: NVMe
      tile_size: 128 MiB
      acceptance_p50_ms: "<= 50"
      acceptance_p99_ms: "<= 200"
    
    warm_cpu:
      cache_state: warm
      source: CPU RAM
      tile_size: 128 MiB
      acceptance_p50_ms: "<= 1"
      acceptance_p99_ms: "<= 5"
    
    warm_gpu:
      cache_state: warm
      source: GPU VRAM
      tile_size: 128 MiB
      acceptance_p50_us: "<= 100"
      acceptance_p99_us: "<= 500"
```

## 7.4 CNWS-BENCH-INFER: Inference Latency

```yaml
benchmark: CNWS-BENCH-INFER
  description: Measure end-to-end inference latency
  
  variants:
    single_token:
      input: 1 token
      output: 1 token
      batch_size: 1
    
    short_sequence:
      input: 32 tokens
      output: 32 tokens
      batch_size: 1
    
    long_sequence:
      input: 4096 tokens
      output: 128 tokens
      batch_size: 1
    
    batch:
      input: 128 tokens
      output: 64 tokens
      batch_size: 32
  
  measurement:
    metrics:
      - time_to_first_token_ms
      - time_per_output_token_ms
      - total_latency_ms
    iterations: 10
    warmup: 3
```

---

# 8. Throughput Benchmarks

## 8.1 CNWS-BENCH-THRU-INFER: Inference Throughput

```yaml
benchmark: CNWS-BENCH-THRU-INFER
  description: Measure inference throughput
  
  methodology:
    1. Run continuous inference for 60 seconds
    2. Count total tokens generated
    3. Compute tokens/second
  
  workloads: [dense_small, dense_medium, moe_small, moe_medium]
  
  measurement:
    duration_seconds: 60
    batch_sizes: [1, 4, 8, 16, 32]
  
  metrics:
    - tokens_per_second
    - Cells_resolved_per_second
    - Tiles_loaded_per_second
  
  acceptance:
    # Hardware-specific thresholds
    HW-GPU-M:
      dense_small_tokens_per_sec: ">= 100"
      moe_small_tokens_per_sec: ">= 150"
```

## 8.2 CNWS-BENCH-THRU-CONV: Conversion Throughput

```yaml
benchmark: CNWS-BENCH-THRU-CONV
  description: Measure conversion throughput
  
  methodology:
    1. Convert checkpoint to .cd
    2. Measure total time
    3. Compute MB/s
  
  workloads: [dense_small, dense_medium, dense_large]
  
  measurement:
    metrics:
      - total_time_seconds
      - throughput_mbps
      - tiles_per_second
  
  acceptance:
    throughput_mbps: ">= 500"
    bounded_memory: true
```

## 8.3 CNWS-BENCH-THRU-LOAD: Tile Loading Throughput

```yaml
benchmark: CNWS-BENCH-THRU-LOAD
  description: Measure Tile loading throughput
  
  methodology:
    1. Load N Tiles sequentially
    2. Measure total bytes loaded
    3. Compute MB/s
  
  variants:
    sequential_read:
      access_pattern: sequential
    random_read:
      access_pattern: random
    parallel_read:
      access_pattern: parallel
      parallelism: 8
  
  acceptance:
    sequential_mbps: ">= 3000"  # NVMe dependent
    random_mbps: ">= 1000"
```

---

# 9. Memory Benchmarks

## 9.1 CNWS-BENCH-MEMORY: Bounded Memory

**Claim**: Bounded memory conversion dan runtime

```yaml
benchmark: CNWS-BENCH-MEMORY
  description: Verify bounded memory behavior
  
  variants:
    conversion_memory:
      description: Peak RAM during conversion
      workloads: [dense_small, dense_medium, dense_large]
      methodology:
        1. Start conversion
        2. Monitor peak RSS continuously
        3. Record peak
      acceptance:
        dense_small_peak_gb: "<= 4"
        dense_medium_peak_gb: "<= 4"
        dense_large_peak_gb: "<= 4"
        # Peak RAM MUST NOT scale with model size
    
    runtime_memory:
      description: Peak RAM during inference
      workloads: [dense_small, dense_medium]
      methodology:
        1. Load model
        2. Run inference
        3. Monitor peak RSS
      acceptance:
        runtime_overhead_gb: "<= 2"
        # Runtime overhead independent of model size
    
    vram_memory:
      description: Peak VRAM during inference
      workloads: [dense_small, dense_medium]
      methodology:
        1. Load model to GPU
        2. Run inference
        3. Monitor peak VRAM
      acceptance:
        vram_budget_respected: true
        no_oom: true
```

## 9.2 CNWS-BENCH-MEM-CACHE: Cache Memory

```yaml
benchmark: CNWS-BENCH-MEM-CACHE
  description: Verify cache memory bounds
  
  methodology:
    1. Set cache budget (e.g., 4 GB CPU, 8 GB GPU)
    2. Load Tiles until cache full
    3. Continue loading (trigger eviction)
    4. Verify memory never exceeds budget
  
  acceptance:
    cpu_cache_never_exceeds_budget: true
    gpu_cache_never_exceeds_budget: true
    eviction_works: true
```

## 9.3 CNWS-BENCH-MEM-WORKING: Working Memory

```yaml
benchmark: CNWS-BENCH-MEM-WORKING
  description: Verify working memory bound
  
  methodology:
    1. Set working memory budget (256 MiB)
    2. Process long context (100K tokens)
    3. Monitor working memory
  
  acceptance:
    working_memory_never_exceeds: true
    context_not_linear: true
```

---

# 10. Bytes Moved Benchmarks

## 10.1 CNWS-BENCH-BYTES: Bytes Moved per Token

**Claim**: Bytes moved/token tracked dan minimized

```yaml
benchmark: CNWS-BENCH-BYTES
  description: Measure bytes moved per token
  
  methodology:
    1. Run inference with I/O tracing
    2. Count bytes read from storage
    3. Count bytes transferred CPU↔GPU
    4. Compute bytes/token
  
  variants:
    cold_cache:
      cache_state: cold
      expected_higher: true
    warm_cache:
      cache_state: warm
      expected_lower: true
    moe_selective:
      model_type: moe
      expected_much_lower: true
  
  metrics:
    - storage_bytes_read
    - cpu_to_gpu_bytes
    - gpu_to_cpu_bytes
    - total_bytes_moved
    - bytes_per_token
  
  acceptance:
    warm_cache_bytes_per_token: "< model_active_params × dtype_size × 1.2"
    moe_bytes_per_token: "< active_experts × expert_size × 1.2"
```

## 10.2 Bytes Moved Comparison

```yaml
benchmark: CNWS-BENCH-BYTES-COMPARE
  description: Compare bytes moved vs dense baseline
  
  methodology:
    1. Run dense model (all params active)
    2. Run MoE model (selective experts)
    3. Compare bytes moved
  
  expected:
    moe_bytes_reduction: ">= 80% vs dense equivalent"
    selective_loading_bytes_reduction: ">= 90% vs full model load"
```

---

# 11. Active Parameter Ratio Benchmarks

## 11.1 CNWS-BENCH-ACTIVE: Active Parameter Ratio

**Claim**: < 10% active parameter ratio

```yaml
benchmark: CNWS-BENCH-ACTIVE
  description: Measure active parameter ratio
  
  methodology:
    1. Run inference
    2. Log all Cells accessed
    3. Sum parameters in accessed Cells
    4. Divide by total model parameters
  
  workloads: [dense_small, dense_medium, moe_small, moe_medium]
  
  metrics:
    - total_parameters
    - active_parameters
    - active_ratio
    - cells_accessed
    - tiles_loaded
  
  acceptance:
    dense_active_ratio: "<= 1.0"  # Dense uses all params
    moe_active_ratio: "<= 0.10"   # MoE uses < 10%
    selective_active_ratio: "<= 0.10"  # Selective loading < 10%
```

## 11.2 Active Ratio Verification

```pseudo
function measure_active_ratio(model, input):
    // Reset counters
    reset_activation_log()
    
    // Run inference
    output = execute(model, input)
    
    // Collect statistics
    total_params = model.total_parameters()
    active_cells = get_activated_cells()
    active_params = sum(cell.parameters for cell in active_cells)
    
    active_ratio = active_params / total_params
    
    // Log detailed breakdown
    for cell in active_cells:
        log(f"Cell {cell.id}: {cell.parameters} params")
    
    return ActiveRatioReport {
        total_params,
        active_params,
        active_ratio,
        cells_accessed: active_cells.len(),
    }
```

---

# 12. Adaptive Compute Benchmarks

## 12.1 CNWS-BENCH-ADAPTIVE: Adaptive Compute

**Claim**: Compute proportional to difficulty

```yaml
benchmark: CNWS-BENCH-ADAPTIVE
  description: Verify adaptive compute behavior
  
  methodology:
    1. Create inputs with varying difficulty
    2. Run inference on each
    3. Measure compute used (steps, Cells, FLOPs)
    4. Verify correlation: harder input → more compute
  
  difficulty_levels:
    easy: "Simple completion, high confidence"
    medium: "Standard reasoning"
    hard: "Complex multi-step reasoning"
  
  metrics:
    - steps_taken
    - cells_executed
    - flops_used
    - bytes_moved
    - wall_time_us
  
  acceptance:
    easy_compute: "< 0.3 × medium_compute"
    hard_compute: "> 2.0 × medium_compute"
    budget_respected: true
    min_depth_respected: true
    max_depth_respected: true
```

## 12.2 Adaptive Compute Verification

```pseudo
function verify_adaptive_compute():
    results = {}
    
    for difficulty in [easy, medium, hard]:
        input = create_input(difficulty)
        
        // Run multiple times for statistics
        computes = []
        for i in 0..10:
            result = execute(input, budget)
            computes.append(result.compute_used)
        
        results[difficulty] = {
            avg_flops: mean(computes.flops),
            avg_steps: mean(computes.steps),
            avg_cells: mean(computes.cells),
        }
    
    // Verify adaptive behavior
    assert results[easy].avg_flops < 0.3 × results[medium].avg_flops
    assert results[hard].avg_flops > 2.0 × results[medium].avg_flops
    
    // Verify budget respected
    for difficulty in [easy, medium, hard]:
        assert results[difficulty].avg_flops <= budget.max_flops
```

---

# 13. MoE Selective Loading Benchmarks

## 13.1 CNWS-BENCH-MOE: MoE Selective Loading

**Claim**: Only selected experts loaded

```yaml
benchmark: CNWS-BENCH-MOE
  description: Verify MoE selective loading
  
  methodology:
    1. Load MoE model (64 experts, top-K=2)
    2. Run inference
    3. Count experts loaded
    4. Verify only top-K experts loaded
  
  workloads: [moe_small, moe_medium]
  
  metrics:
    - total_experts
    - experts_loaded
    - experts_loaded_ratio
    - tiles_loaded
    - bytes_loaded
    - expert_dedup_count (for batch)
  
  acceptance:
    experts_loaded: "<= top_k"
    experts_loaded_ratio: "<= 0.05"  # 2/64 = 3.1%
    all_experts_not_loaded: true
    batch_dedup_works: true
```

## 13.2 MoE Batch Deduplication

```yaml
benchmark: CNWS-BENCH-MOE-DEDUP
  description: Verify expert deduplication in batch
  
  methodology:
    1. Create batch where multiple tokens select same expert
    2. Run inference
    3. Count unique expert loads
  
  test_case:
    batch_size: 8
    token_experts:
      token_0: expert_7
      token_1: expert_7
      token_2: expert_42
      token_3: expert_7
      token_4: expert_42
      token_5: expert_7
      token_6: expert_7
      token_7: expert_42
  
  expected:
    unique_experts_loaded: 2  # expert_7 and expert_42
    total_expert_requests: 8
    dedup_ratio: 0.25  # 2/8
```

---

# 14. Conversion Benchmarks

## 14.1 CNWS-BENCH-CONV-MEM: Conversion Memory

```yaml
benchmark: CNWS-BENCH-CONV-MEM
  description: Verify conversion bounded memory
  
  methodology:
    1. Convert checkpoints of varying sizes
    2. Monitor peak RSS
    3. Verify peak RSS independent of model size
  
  workloads: [dense_small, dense_medium, dense_large]
  
  metrics:
    - peak_rss_gb
    - model_size_gb
    - rss_to_model_ratio
  
  acceptance:
    peak_rss_gb: "<= 4"
    rss_to_model_ratio: "<= 0.1"  # RSS < 10% of model size
    # Peak RSS MUST NOT scale with model size
```

## 14.2 CNWS-BENCH-CONV-THRU: Conversion Throughput

```yaml
benchmark: CNWS-BENCH-CONV-THRU
  description: Measure conversion throughput
  
  methodology:
    1. Convert checkpoint
    2. Measure time
    3. Compute throughput
  
  workloads: [dense_small, dense_medium]
  
  metrics:
    - total_time_seconds
    - throughput_mbps
    - tiles_written_per_second
    - tiles_deduplicated
  
  acceptance:
    throughput_mbps: ">= 500"
```

## 14.3 CNWS-BENCH-CONV-DETERMINISM: Conversion Determinism

```yaml
benchmark: CNWS-BENCH-CONV-DETERMINISM
  description: Verify conversion determinism
  
  methodology:
    1. Convert same checkpoint twice
    2. Compare .cd stores byte-by-byte
  
  acceptance:
    stores_identical: true
    tile_ids_identical: true
    manifest_hash_identical: true
```

---

# 15. Memory Retrieval Benchmarks

## 15.1 CNWS-BENCH-MEM-RETRIEVE: Memory Retrieval

**Claim**: O(log N) retrieval

```yaml
benchmark: CNWS-BENCH-MEM-RETRIEVE
  description: Verify memory retrieval O(log N)
  
  methodology:
    1. Create N memory entries
    2. Retrieve with random query
    3. Measure latency
    4. Repeat for N = [1K, 10K, 100K, 1M]
    5. Verify O(log N) scaling
  
  metrics:
    - retrieval_latency_us
    - n_entries
    - scaling_factor
  
  acceptance:
    latency_1k_us: "<= 100"
    latency_1m_us: "<= 10000"
    scaling: "O(log N) verified"
    # Latency should grow logarithmically, not linearly
```

---

# 16. Acceptance Thresholds

## 16.1 Summary of Acceptance Thresholds

`[PERF-ACC-1]` Acceptance thresholds untuk certification:

| Benchmark | Metric | Threshold | Priority |
|---|---|---|---|
| CNWS-BENCH-RESOLVE | P50 latency | ≤ 1 μs | MUST |
| CNWS-BENCH-RESOLVE | P99 latency | ≤ 10 μs | MUST |
| CNWS-BENCH-LOOKUP | P50 latency | ≤ 10 μs | MUST |
| CNWS-BENCH-LOOKUP | P99 latency | ≤ 100 μs | MUST |
| CNWS-BENCH-ACTIVE | MoE active ratio | ≤ 10% | MUST |
| CNWS-BENCH-MEMORY | Conversion peak RSS | ≤ 4 GiB | MUST |
| CNWS-BENCH-MEMORY | RSS/model ratio | ≤ 10% | MUST |
| CNWS-BENCH-BYTES | Warm bytes/token | ≤ active_params × dtype × 1.2 | MUST |
| CNWS-BENCH-ADAPTIVE | Budget respected | 100% | MUST |
| CNWS-BENCH-MOE | Experts loaded | ≤ top_k | MUST |
| CNWS-BENCH-CONV-THRU | Throughput | ≥ 500 MB/s | SHOULD |
| CNWS-BENCH-MEM-RETRIEVE | Scaling | O(log N) | MUST |

## 16.2 Threshold Enforcement

`[PERF-ACC-2]` Threshold enforcement:

| Level | Enforcement |
|---|---|
| MUST | Failure blocks certification |
| SHOULD | Failure logged, warning issued |
| MAY | Failure informational only |

## 16.3 Regression Detection

`[PERF-ACC-3]` Regression detection:

```pseudo
function check_regression(current_result, baseline_result):
    for metric in current_result.metrics:
        degradation = (current - baseline) / baseline
        
        if degradation > 0.10:  # > 10% degradation
            return Regression {
                metric: metric,
                degradation: degradation,
                severity: BLOCKING,
            }
        elif degradation > 0.05:  # > 5% degradation
            return Regression {
                metric: metric,
                degradation: degradation,
                severity: WARNING,
            }
    
    return NoRegression
```

`[PERF-ACC-4]` Regression > 10% MUST memblokir merge.

`[PERF-ACC-5]` Regression > 5% SHOULD dilaporkan sebagai warning.

---

# 17. Benchmark Reporting

## 17.1 Report Format

`[PERF-REP-1]` Benchmark report MUST menggunakan format JSON:

```json
{
  "benchmark_report": {
    "version": "1.0.0",
    "cnws_version": "1.0.0",
    "date": "2026-08-11T00:00:00Z",
    "hardware": {
      "profile_id": "HW-GPU-M",
      "cpu": "...",
      "gpu": "...",
      "ram_gb": 64,
      "vram_gb": 24
    },
    "results": [
      {
        "benchmark_id": "CNWS-BENCH-RESOLVE",
        "workload": "dense_small",
        "cache_state": "warm",
        "metrics": {
          "p50_us": 0.8,
          "p95_us": 1.2,
          "p99_us": 2.1,
          "mean_us": 0.9,
          "stddev_us": 0.3
        },
        "iterations": 1000,
        "passed": true
      }
    ],
    "summary": {
      "total_benchmarks": 25,
      "passed": 24,
      "failed": 1,
      "skipped": 0
    }
  }
}
```

## 17.2 Report Requirements

`[PERF-REP-2]` Report MUST mencakup:

| Field | Required |
|---|---|
| CNWS version | MUST |
| Hardware profile | MUST |
| Date | MUST |
| All benchmark results | MUST |
| Raw measurement data | SHOULD |
| Regression comparison | SHOULD |
| Pass/fail summary | MUST |

## 17.3 Report Storage

`[PERF-REP-3]` Reports MUST disimpan minimum 1 tahun.

`[PERF-REP-4]` Reports MUST dapat dibandingkan antar versi.

---

# 18. Benchmark Automation

## 18.1 CI Integration

`[PERF-CI-1]` Performance benchmarks MUST dijalankan di CI:

| Trigger | Benchmarks |
|---|---|
| Every commit | Quick benchmarks (tiny workload) |
| Every PR | Standard benchmarks (small workload) |
| Nightly | Full benchmarks (medium workload) |
| Release | Full benchmarks (all workloads) |

## 18.2 CI Performance Gates

```yaml
ci_performance_gates:
  quick:
    workloads: [dense_tiny, moe_tiny]
    benchmarks: [RESOLVE, LOOKUP, MEMORY]
    timeout_minutes: 5
  
  standard:
    workloads: [dense_small, moe_small]
    benchmarks: [ALL]
    timeout_minutes: 30
  
  full:
    workloads: [dense_medium, moe_medium]
    benchmarks: [ALL]
    timeout_minutes: 120
```

## 18.3 CI Invariants

| ID | Invariant |
|---|---|
| PERF-CI-INV-1 | Quick benchmarks MUST pada setiap commit |
| PERF-CI-INV-2 | Regression > 10% MUST memblokir merge |
| PERF-CI-INV-3 | Benchmark failure MUST dilaporkan |
| PERF-CI-INV-4 | CI benchmarks MUST deterministik |

---

# 19. Final Performance Contract

## 19.1 Ringkasan Keputusan Performance

| ID | Keputusan |
|---|---|
| PERF-F01 | Benchmark suite normatif untuk certification. |
| PERF-F02 | Workloads: tiny, small, medium, large. |
| PERF-F03 | Hardware profiles: CPU-only, GPU-S/M/L/XL, Multi-GPU. |
| PERF-F04 | Model fixtures deterministik dengan seed 0x434E5753. |
| PERF-F05 | Warm cache: 3 iteration warmup. |
| PERF-F06 | Cold cache: cache clear + page cache drop. |
| PERF-F07 | Latency: P50, P95, P99. |
| PERF-F08 | Throughput: tokens/sec, Cells/sec. |
| PERF-F09 | Memory: peak RSS, peak VRAM. |
| PERF-F10 | Bytes moved: I/O tracing. |
| PERF-F11 | Active ratio: Cell activation log. |
| PERF-F12 | Acceptance thresholds hard untuk certification. |
| PERF-F13 | Benchmark MUST reproducible. |
| PERF-F14 | Regression > 10% memblokir merge. |
| PERF-F15 | Cell resolve P50 ≤ 1 μs. |
| PERF-F16 | Tile lookup P50 ≤ 10 μs. |
| PERF-F17 | MoE active ratio ≤ 10%. |
| PERF-F18 | Conversion peak RSS ≤ 4 GiB. |
| PERF-F19 | Conversion throughput ≥ 500 MB/s. |
| PERF-F20 | Memory retrieval O(log N). |

## 19.2 Performance Invariants

| ID | Invariant |
|---|---|
| PERF-INV-1 | Setiap klaim performa MUST memiliki benchmark. |
| PERF-INV-2 | Benchmark MUST deterministik. |
| PERF-INV-3 | Benchmark MUST menggunakan fixtures versioned. |
| PERF-INV-4 | Measurement MUST P50/P95/P99. |
| PERF-INV-5 | Warmup MUST 3 iterations untuk warm cache. |
| PERF-INV-6 | Cold cache MUST clear semua cache. |
| PERF-INV-7 | Hardware profile MUST dicatat. |
| PERF-INV-8 | Acceptance thresholds MUST hard untuk MUST. |
| PERF-INV-9 | Regression > 10% MUST memblokir merge. |
| PERF-INV-10 | Report MUST JSON format. |
| PERF-INV-11 | Reports MUST disimpan 1 tahun. |
| PERF-INV-12 | CI MUST menjalankan benchmarks. |
| PERF-INV-13 | Quick benchmarks MUST setiap commit. |
| PERF-INV-14 | O(1) claims MUST verified dengan scaling test. |
| PERF-INV-15 | Bounded memory MUST verified dengan model size variation. |

## 19.3 Performance Claims Verification Matrix

| Claim | Benchmark | Acceptance |
|---|---|---|
| O(1) Cell resolve | CNWS-BENCH-RESOLVE | P50 ≤ 1 μs, scaling verified |
| O(1) Tile lookup | CNWS-BENCH-LOOKUP | P50 ≤ 10 μs, scaling verified |
| < 10% active ratio | CNWS-BENCH-ACTIVE | MoE ratio ≤ 10% |
| Bounded memory | CNWS-BENCH-MEMORY | Peak RSS ≤ 4 GiB, tidak scale |
| Bytes moved/token | CNWS-BENCH-BYTES | Tracked, ≤ expected |
| Adaptive compute | CNWS-BENCH-ADAPTIVE | Budget respected, correlation verified |
| MoE selective | CNWS-BENCH-MOE | Experts loaded ≤ top_k |
| Conversion bounded | CNWS-BENCH-CONV-MEM | Peak ≤ 4 GiB |
| Conversion throughput | CNWS-BENCH-CONV-THRU | ≥ 500 MB/s |
| Retrieval O(log N) | CNWS-BENCH-MEM-RETRIEVE | Scaling verified |

## 19.4 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Performance Benchmark final dan mengikat** untuk CNWS. Ia mendefinisikan bagaimana setiap klaim performa diukur secara objektif, dari workload definitions hingga acceptance thresholds, dari measurement methodology hingga regression detection.

Setiap klaim performa dalam Engineering Contract dan spesifikasi lainnya MUST terverifikasi melalui benchmark dalam dokumen ini. Tidak ada klaim performa yang boleh dibuat tanpa benchmark yang sesuai.

Seluruh implementasi CNWS MUST lulus performance benchmarks untuk certification.

Tidak ada keputusan performance benchmark yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN PERFORMANCE BENCHMARK SPECIFICATION**
