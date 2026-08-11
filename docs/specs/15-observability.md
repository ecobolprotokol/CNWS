# CNWS
## Observability Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Observability Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (OBSERVABILITY SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS DAS; seluruh spesifikasi subsystem |
| Hulu ke | Implementasi telemetry layer, monitoring, alerting, debugging tools |
| Otoritas | Spesifikasi tunggal untuk metrics, logging, dan tracing CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    Subsystem Specs       Observability Spec          Implementation
─────────────────────   ──────────────────    ────────────────────────    ─────────────
"MUST produce logs"   ──► Runtime metrics  ──► Metric definitions      ──► Telemetry Layer
"SHOULD produce       ──► Security events  ──► Event schemas            Metrics SDK
 metrics"              ──► Revision events  ──► Tracing spans           Logging SDK
"SHOULD produce       ──► Error events     ──► Correlation IDs          Tracing SDK
 traces"               ──► Corruption       ──► Diagnostic interfaces    Diagnostic CLI
                          events                                        Exporters
```

`[OBS-DOC-1]` Dokumen ini mendefinisikan **secara presisi bagaimana metrics, logging, dan tracing bekerja** di CNWS.

`[OBS-DOC-2]` Setiap komponen CNWS MUST mengimplementasikan observability sesuai spesifikasi ini.

`[OBS-DOC-3]` Jika terjadi konflik dengan spesifikasi lain untuk hal behavior, spesifikasi tersebut menang. Untuk hal observability format dan schema, dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-OBS-01 | Metrics menggunakan OpenMetrics format (Prometheus-compatible). |
| DF-OBS-02 | Logging menggunakan structured JSON. |
| DF-OBS-03 | Tracing menggunakan OpenTelemetry-compatible spans. |
| DF-OBS-04 | Correlation ID menggunakan UUIDv7. |
| DF-OBS-05 | Metric prefix: `cnws_`. |
| DF-OBS-06 | Label cardinality limit: 10,000 unique combinations per metric. |
| DF-OBS-07 | Log levels: ERROR, WARN, INFO, DEBUG, TRACE. |
| DF-OBS-08 | Sampling default: 100% untuk errors, 1% untuk traces. |
| DF-OBS-09 | Retention: metrics 90 hari, logs 30 hari, traces 7 hari. |
| DF-OBS-10 | Observability overhead budget: < 5% CPU, < 2% memory. |
| DF-OBS-11 | Diagnostic endpoints tersedia via CLI dan optional HTTP. |
| DF-OBS-12 | Security events MUST logged dengan severity minimum WARN. |
| DF-OBS-13 | Corruption events MUST logged dengan severity ERROR. |
| DF-OBS-14 | Revision events MUST logged dengan severity INFO. |

---

# 1. Executive Summary

## 1.1 Observability Philosophy

`[OBS-EXEC-1]` Observability CNWS mengikuti prinsip:

1. **Structured**: semua telemetry terstruktur, bukan free-form text.
2. **Correlated**: setiap event dapat dikorelasikan lintas komponen.
3. **Bounded**: cardinality dan volume dibatasi untuk mencegah resource exhaustion.
4. **Actionable**: setiap metric/event memberikan informasi yang dapat ditindaklanjuti.
5. **Low overhead**: observability MUST NOT signifikan mempengaruhi performa.

## 1.2 Three Pillars

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS OBSERVABILITY                        │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   METRICS   │  │   LOGGING   │  │   TRACING   │        │
│  │             │  │             │  │             │        │
│  │ OpenMetrics │  │ Structured  │  │ OpenTelemetry│       │
│  │ Counters    │  │ JSON events │  │ Spans       │        │
│  │ Gauges      │  │ Levels      │  │ Attributes  │        │
│  │ Histograms  │  │ Correlation │  │ Hierarchy   │        │
│  └─────────────┘  └─────────────┘  └─────────────┘        │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              DIAGNOSTICS                             │   │
│  │                                                     │   │
│  │  Runtime state │ Revision state │ Corruption state   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              CORRELATION                             │   │
│  │                                                     │   │
│  │  Request ID │ Operation ID │ Revision ID │ Tile ID  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 1.3 Observability Objectives

| Objective | Description |
|---|---|
| Monitoring | Real-time health dan performance monitoring |
| Alerting | Deteksi anomali dan failure |
| Debugging | Root cause analysis untuk failures |
| Auditing | Security dan compliance audit trail |
| Capacity Planning | Resource utilization trends |
| Performance Optimization | Identifikasi bottlenecks |

---

# 2. Observability Architecture

## 2.1 Telemetry Pipeline

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS COMPONENTS                           │
│                                                             │
│  Runtime  Storage  Conversion  Revision  Memory  Security   │
│     │        │         │          │         │        │      │
│     └────────┴─────────┴──────────┴─────────┴────────┘      │
│                          │                                   │
│                          ▼                                   │
│              ┌───────────────────────┐                       │
│              │  TELEMETRY COLLECTOR  │                       │
│              │                       │                       │
│              │  ┌─────────────────┐  │                       │
│              │  │ Metrics Buffer  │  │                       │
│              │  └────────┬────────┘  │                       │
│              │  ┌────────┴────────┐  │                       │
│              │  │  Event Buffer   │  │                       │
│              │  └────────┬────────┘  │                       │
│              │  ┌────────┴────────┐  │                       │
│              │  │  Span Buffer    │  │                       │
│              │  └────────┬────────┘  │                       │
│              └───────────┼───────────┘                       │
│                          │                                   │
└──────────────────────────┼───────────────────────────────────┘
                           │
              ┌────────────┼────────────┐
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │ Metrics  │ │  Logs    │ │  Traces  │
        │ Exporter │ │ Exporter │ │ Exporter │
        └──────────┘ └──────────┘ └──────────┘
              │            │            │
              ▼            ▼            ▼
        ┌──────────┐ ┌──────────┐ ┌──────────┐
        │Prometheus│ │ Loki /   │ │ Jaeger / │
        │ OTLP     │ │ Syslog   │ │ OTLP     │
        └──────────┘ └──────────┘ └──────────┘
```

## 2.2 Telemetry Collector

`[OBS-ARCH-1]` Telemetry Collector MUST:

1. Buffer events sebelum export.
2. Bounded buffer untuk mencegah memory exhaustion.
3. Async export untuk tidak blocking operasi utama.
4. Backpressure jika buffer penuh.
5. Graceful shutdown dengan flush buffer.

```rust
struct TelemetryCollector {
    metrics_buffer: BoundedBuffer<MetricEvent>,
    log_buffer: BoundedBuffer<LogEvent>,
    span_buffer: BoundedBuffer<SpanEvent>,
    
    // Configuration
    buffer_size: usize,          // default: 10,000 events
    flush_interval_ms: u64,      // default: 1,000 ms
    batch_size: usize,           // default: 1,000 events
}
```

---

# 3. Metric Definitions

## 3.1 Metric Naming Convention

`[OBS-MET-1]` Semua metric MUST menggunakan prefix `cnws_`.

`[OBS-MET-2]` Metric name format:

```text
cnws_<subsystem>_<metric_name>_<unit>
```

Contoh:
```text
cnws_runtime_tile_load_duration_seconds
cnws_storage_segment_count_total
cnws_conversion_bytes_processed_bytes
cnws_cache_hit_ratio_percent
```

## 3.2 Metric Types

| Type | Description | Example |
|---|---|---|
| Counter | Monotonically increasing | Total Tiles loaded |
| Gauge | Point-in-time value | Current cache usage |
| Histogram | Distribution of values | Load latency distribution |
| Summary | Pre-computed quantiles | P50/P95/P99 latency |

## 3.3 Runtime Metrics

### 3.3.1 Cell Resolution

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_runtime_cell_resolve_total` | Counter | count | Total Cell resolutions |
| `cnws_runtime_cell_resolve_duration_seconds` | Histogram | seconds | Cell resolution latency |
| `cnws_runtime_cell_resolve_errors_total` | Counter | count | Cell resolution errors |

```yaml
cnws_runtime_cell_resolve_duration_seconds:
  type: histogram
  buckets: [0.000001, 0.000005, 0.00001, 0.00005, 0.0001, 0.0005, 0.001]
  labels:
    - cell_type
    - status  # success, error
```

### 3.3.2 Tile Operations

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_runtime_tile_load_total` | Counter | count | Total Tile loads |
| `cnws_runtime_tile_load_duration_seconds` | Histogram | seconds | Tile load latency |
| `cnws_runtime_tile_load_bytes` | Counter | bytes | Bytes loaded |
| `cnws_runtime_tile_load_errors_total` | Counter | count | Tile load errors |
| `cnws_runtime_tile_verify_total` | Counter | count | Tile verifications |
| `cnws_runtime_tile_verify_failures_total` | Counter | count | Verification failures |

### 3.3.3 Execution

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_runtime_execution_total` | Counter | count | Total executions |
| `cnws_runtime_execution_duration_seconds` | Histogram | seconds | Execution duration |
| `cnws_runtime_execution_steps` | Histogram | count | Steps per execution |
| `cnws_runtime_execution_cells_active` | Gauge | count | Active Cells |
| `cnws_runtime_execution_flops` | Counter | flops | FLOPs used |
| `cnws_runtime_execution_bytes_moved` | Counter | bytes | Bytes moved |
| `cnws_runtime_execution_budget_exceeded_total` | Counter | count | Budget exceeded events |
| `cnws_runtime_active_parameter_ratio` | Gauge | ratio | Active/total parameter ratio |

### 3.3.4 Adaptive Compute

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_runtime_difficulty_score` | Histogram | score | Difficulty estimation distribution |
| `cnws_runtime_compute_multiplier` | Histogram | ratio | Compute multiplier applied |
| `cnws_runtime_halt_total` | Counter | count | Halt events |
| `cnws_runtime_halt_reason_total` | Counter | count | Halt events by reason |

## 3.4 Storage Metrics

### 3.4.1 Store Operations

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_storage_store_open_total` | Counter | count | Store opens |
| `cnws_storage_store_size_bytes` | Gauge | bytes | Total store size |
| `cnws_storage_segment_count_total` | Gauge | count | Total segments |
| `cnws_storage_tile_count_total` | Gauge | count | Total Tiles |
| `cnws_storage_cell_count_total` | Gauge | count | Total Cells |

### 3.4.2 I/O Operations

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_storage_io_read_bytes` | Counter | bytes | Bytes read |
| `cnws_storage_io_write_bytes` | Counter | bytes | Bytes written |
| `cnws_storage_io_read_duration_seconds` | Histogram | seconds | Read latency |
| `cnws_storage_io_write_duration_seconds` | Histogram | seconds | Write latency |
| `cnws_storage_io_errors_total` | Counter | count | I/O errors |

## 3.5 Cache Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_cache_entries` | Gauge | count | Cache entries |
| `cnws_cache_size_bytes` | Gauge | bytes | Cache size |
| `cnws_cache_budget_bytes` | Gauge | bytes | Cache budget |
| `cnws_cache_utilization_ratio` | Gauge | ratio | Cache utilization |
| `cnws_cache_hit_total` | Counter | count | Cache hits |
| `cnws_cache_miss_total` | Counter | count | Cache misses |
| `cnws_cache_hit_ratio` | Gauge | ratio | Hit ratio |
| `cnws_cache_eviction_total` | Counter | count | Evictions |
| `cnws_cache_eviction_bytes` | Counter | bytes | Bytes evicted |
| `cnws_cache_admission_total` | Counter | count | Admissions |
| `cnws_cache_admission_rejected_total` | Counter | count | Rejected admissions |

```yaml
cnws_cache_hit_ratio:
  type: gauge
  labels:
    - level  # gpu, cpu, nvme
    - representation
```

## 3.6 Conversion Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_conversion_total` | Counter | count | Total conversions |
| `cnws_conversion_duration_seconds` | Histogram | seconds | Conversion duration |
| `cnws_conversion_bytes_processed` | Counter | bytes | Bytes processed |
| `cnws_conversion_bytes_stored` | Counter | bytes | Bytes stored |
| `cnws_conversion_tiles_created` | Counter | count | Tiles created |
| `cnws_conversion_tiles_deduplicated` | Counter | count | Tiles deduplicated |
| `cnws_conversion_cells_created` | Counter | count | Cells created |
| `cnws_conversion_errors_total` | Counter | count | Conversion errors |
| `cnws_conversion_peak_memory_bytes` | Gauge | bytes | Peak memory usage |

## 3.7 Revision Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_revision_total` | Counter | count | Total revisions |
| `cnws_revision_commit_duration_seconds` | Histogram | seconds | Commit duration |
| `cnws_revision_branch_total` | Counter | count | Branches created |
| `cnws_revision_merge_total` | Counter | count | Merges performed |
| `cnws_revision_merge_conflicts_total` | Counter | count | Merge conflicts |
| `cnws_revision_rollback_total` | Counter | count | Rollbacks |
| `cnws_revision_active` | Gauge | revision_number | Active revision |
| `cnws_revision_dag_depth` | Gauge | count | DAG depth |
| `cnws_revision_delta_tiles` | Histogram | count | Tiles changed per revision |
| `cnws_revision_delta_bytes` | Histogram | bytes | Bytes changed per revision |

## 3.8 Memory Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_memory_entries_total` | Gauge | count | Total memory entries |
| `cnws_memory_entries_by_type` | Gauge | count | Entries by type |
| `cnws_memory_size_bytes` | Gauge | bytes | Total memory size |
| `cnws_memory_write_total` | Counter | count | Memory writes |
| `cnws_memory_retrieve_total` | Counter | count | Retrievals |
| `cnws_memory_retrieve_duration_seconds` | Histogram | seconds | Retrieval latency |
| `cnws_memory_retrieve_k` | Histogram | count | K values used |
| `cnws_memory_consolidation_total` | Counter | count | Consolidations |
| `cnws_memory_forgetting_total` | Counter | count | Forgetting events |
| `cnws_memory_working_size_bytes` | Gauge | bytes | Working memory size |
| `cnws_memory_working_entries` | Gauge | count | Working memory entries |

## 3.9 GC Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_gc_runs_total` | Counter | count | GC runs |
| `cnws_gc_duration_seconds` | Histogram | seconds | GC duration |
| `cnws_gc_tiles_reclaimed` | Counter | count | Tiles reclaimed |
| `cnws_gc_bytes_reclaimed` | Counter | bytes | Bytes reclaimed |
| `cnws_gc_tiles_reachable` | Gauge | count | Reachable Tiles |
| `cnws_gc_tiles_unreachable` | Gauge | count | Unreachable Tiles |

## 3.10 Error Metrics

| Metric Name | Type | Unit | Description |
|---|---|---|---|
| `cnws_errors_total` | Counter | count | Total errors |
| `cnws_errors_by_code` | Counter | count | Errors by code |
| `cnws_errors_by_severity` | Counter | count | Errors by severity |
| `cnws_errors_recoverable_total` | Counter | count | Recoverable errors |
| `cnws_errors_fatal_total` | Counter | count | Fatal errors |

## 3.11 Metric Definition Invariants

| ID | Invariant |
|---|---|
| OBS-MET-INV-1 | Semua metric MUST menggunakan prefix `cnws_` |
| OBS-MET-INV-2 | Metric name MUST snake_case |
| OBS-MET-INV-3 | Unit MUST dalam suffix (seconds, bytes, total) |
| OBS-MET-INV-4 | Counter MUST monotonically increasing |
| OBS-MET-INV-5 | Histogram MUST memiliki buckets yang sesuai |
| OBS-MET-INV-6 | Setiap metric MUST memiliki description |

---

# 4. Labels

## 4.1 Standard Labels

`[OBS-LBL-1]` Labels standar yang MAY digunakan:

| Label | Description | Example |
|---|---|---|
| `cell_type` | Cell type | `ATTENTION_Q_PROJ`, `EXPERT_WEIGHT` |
| `tile_id` | Tile identity | `b3:7f3a8e...` |
| `segment_id` | Segment identity | `1`, `42` |
| `level` | Cache level | `gpu`, `cpu`, `nvme` |
| `representation` | Representation ID | `bf16`, `fp8`, `int8` |
| `memory_type` | Memory type | `episodic`, `semantic`, `procedural` |
| `status` | Operation status | `success`, `error` |
| `error_code` | Error code | `CNWS-E-CORRUPT` |
| `operation` | Operation name | `load`, `store`, `retrieve` |
| `format` | Source format | `safetensors`, `gguf`, `pytorch` |
| `revision` | Revision ID | `b3:rev01...` |
| `model_id` | Model identity | `example-org/model-70b` |

## 4.2 Label Naming Convention

`[OBS-LBL-2]` Label names MUST:

1. snake_case
2. Lowercase
3. Tidak menggunakan reserved names (`__name__`, `job`, `instance`)
4. Deskriptif dan tidak ambigu

## 4.3 Label Cardinality Rules

`[OBS-LBL-3]` Label cardinality MUST dibatasi:

| Rule | Limit |
|---|---|
| Unique label values per label | ≤ 1,000 |
| Unique label combinations per metric | ≤ 10,000 |
| Total unique time series | ≤ 100,000 |

`[OBS-LBL-4]` High-cardinality values MUST NOT digunakan sebagai labels:

| High Cardinality (FORBIDDEN) | Alternative |
|---|---|
| `tile_id` (millions of values) | Use in logs/traces |
| `request_id` (unique per request) | Use in logs/traces |
| `timestamp` | Use in logs/traces |
| `file_path` | Use in logs/traces |
| `user_id` | Use in logs/traces |

`[OBS-LBL-5]` Jika cardinality limit exceeded:

1. Metric MUST di-aggregate.
2. Excess labels MUST di-drop.
3. Warning event MUST logged.

## 4.4 Label Invariants

| ID | Invariant |
|---|---|
| OBS-LBL-INV-1 | Labels MUST snake_case |
| OBS-LBL-INV-2 | Label cardinality MUST bounded |
| OBS-LBL-INV-3 | High-cardinality values MUST NOT jadi labels |
| OBS-LBL-INV-4 | Label values MUST strings |
| OBS-LBL-INV-5 | Label set per metric MUST konsisten |

---

# 5. Units

## 5.1 Standard Units

`[OBS-UNIT-1]` Units yang digunakan:

| Unit | Suffix | Description |
|---|---|---|
| Count | `_total` | Monotonically increasing count |
| Bytes | `_bytes` | Data size |
| Seconds | `_seconds` | Duration |
| Milliseconds | `_milliseconds` | Duration (alternative) |
| Microseconds | `_microseconds` | Duration (alternative) |
| Ratio | `_ratio` | 0.0 - 1.0 |
| Percent | `_percent` | 0 - 100 |
| FLOPS | `_flops` | Floating point operations |
| IOPS | `_iops` | I/O operations per second |

## 5.2 Unit Conversion

`[OBS-UNIT-2]` Untuk consistency:

1. Duration SHOULD menggunakan seconds (base unit).
2. Size MUST menggunakan bytes (base unit).
3. Ratio MUST menggunakan 0.0-1.0, bukan percent.

## 5.3 Unit Invariants

| ID | Invariant |
|---|---|
| OBS-UNIT-INV-1 | Unit MUST dalam metric name suffix |
| OBS-UNIT-INV-2 | Base units MUST digunakan (seconds, bytes) |
| OBS-UNIT-INV-3 | Ratio MUST 0.0-1.0 |
| OBS-UNIT-INV-4 | Unit MUST konsisten antar metric sejenis |

---

# 6. Cardinality Rules

## 6.1 Cardinality Budget

`[OBS-CARD-1]` Cardinality budget per subsystem:

| Subsystem | Max Time Series |
|---|---|
| Runtime | 10,000 |
| Storage | 5,000 |
| Cache | 5,000 |
| Conversion | 2,000 |
| Revision | 2,000 |
| Memory | 3,000 |
| GC | 500 |
| Errors | 2,000 |
| **Total** | **≤ 30,000** |

## 6.2 Cardinality Enforcement

```pseudo
function enforce_cardinality(metric_name, labels):
    // Compute label combination hash
    combo_hash = hash(labels)
    
    // Check if already seen
    if combo_hash in seen_combinations[metric_name]:
        return ALLOW
    
    // Check cardinality limit
    if seen_combinations[metric_name].len() >= CARDINALITY_LIMIT:
        // Drop new combination
        log_warning("Cardinality limit exceeded for {}", metric_name)
        increment_metric("cnws_observability_cardinality_dropped_total")
        return DROP
    
    // Allow
    seen_combinations[metric_name].add(combo_hash)
    return ALLOW
```

## 6.3 Cardinality Invariants

| ID | Invariant |
|---|---|
| OBS-CARD-INV-1 | Cardinality MUST bounded per metric |
| OBS-CARD-INV-2 | Cardinality limit MUST enforced at collection time |
| OBS-CARD-INV-3 | Cardinality exceeded MUST logged |
| OBS-CARD-INV-4 | Total time series MUST ≤ 100,000 |

---

# 7. Event Schema

## 7.1 Log Levels

`[OBS-EVT-1]` Log levels:

| Level | Numeric | Description | Use Case |
|---|---|---|---|
| ERROR | 0 | Failure requiring attention | Corruption, fatal errors |
| WARN | 1 | Potential issue | Degraded mode, budget exceeded |
| INFO | 2 | Normal operations | Revision commits, conversions |
| DEBUG | 3 | Detailed operations | Tile loads, cache operations |
| TRACE | 4 | Very detailed | Individual operations |

## 7.2 Structured Event Format

`[OBS-EVT-2]` Semua log events MUST structured JSON:

```json
{
  "timestamp": "2026-08-11T12:34:56.789Z",
  "level": "INFO",
  "event_type": "tile_loaded",
  "correlation_id": "01926e8a-1234-7000-8000-123456789012",
  "component": "runtime.cache",
  "message": "Tile loaded successfully",
  
  "context": {
    "tile_id": "b3:7f3a8e...",
    "cell_id": "b3:1c33...",
    "representation": "bf16",
    "level": "gpu",
    "size_bytes": 134217728,
    "duration_ms": 12.5,
    "cache_hit": false
  },
  
  "metadata": {
    "cnws_version": "1.0.0",
    "model_id": "example-org/model-70b",
    "revision": "b3:rev01...",
    "instance_id": "node-01",
    "process_id": 12345,
    "thread_id": 67890
  }
}
```

## 7.3 Event Type Taxonomy

`[OBS-EVT-3]` Event types MUST menggunakan taxonomy berikut:

### 7.3.1 Lifecycle Events

| Event Type | Level | Description |
|---|---|---|
| `store_opened` | INFO | Store opened |
| `store_closed` | INFO | Store closed |
| `conversion_started` | INFO | Conversion started |
| `conversion_completed` | INFO | Conversion completed |
| `conversion_failed` | ERROR | Conversion failed |
| `runtime_started` | INFO | Runtime started |
| `runtime_stopped` | INFO | Runtime stopped |

### 7.3.2 Tile Events

| Event Type | Level | Description |
|---|---|---|
| `tile_created` | DEBUG | Tile created |
| `tile_loaded` | DEBUG | Tile loaded |
| `tile_evicted` | DEBUG | Tile evicted |
| `tile_verified` | TRACE | Tile verified |
| `tile_corrupted` | ERROR | Tile corruption detected |
| `tile_quarantined` | WARN | Tile quarantined |

### 7.3.3 Revision Events

| Event Type | Level | Description |
|---|---|---|
| `revision_committed` | INFO | Revision committed |
| `revision_branch_created` | INFO | Branch created |
| `revision_merged` | INFO | Merge completed |
| `revision_conflict` | WARN | Merge conflict |
| `revision_rollback` | INFO | Rollback performed |
| `revision_resolved` | DEBUG | Revision resolved |

### 7.3.4 Memory Events

| Event Type | Level | Description |
|---|---|---|
| `memory_written` | DEBUG | Memory entry written |
| `memory_retrieved` | DEBUG | Memory retrieved |
| `memory_consolidated` | INFO | Memory consolidated |
| `memory_forgotten` | INFO | Memory forgotten |
| `memory_budget_exceeded` | WARN | Memory budget exceeded |

### 7.3.5 Security Events

| Event Type | Level | Description |
|---|---|---|
| `security_checkpoint_rejected` | ERROR | Malicious checkpoint rejected |
| `security_path_traversal_blocked` | WARN | Path traversal blocked |
| `security_resource_exhaustion_blocked` | WARN | Resource exhaustion blocked |
| `security_manifest_tampering_detected` | ERROR | Manifest tampering detected |
| `security_version_downgrade_blocked` | WARN | Version downgrade blocked |
| `security_quarantine` | WARN | Item quarantined |

### 7.3.6 Error Events

| Event Type | Level | Description |
|---|---|---|
| `error_recoverable` | WARN | Recoverable error |
| `error_fatal` | ERROR | Fatal error |
| `error_budget_exceeded` | WARN | Budget exceeded |
| `error_timeout` | WARN | Operation timeout |

## 7.4 Event Schema Validation

`[OBS-EVT-4]` Event schema MUST divalidasi:

```pseudo
function validate_event(event):
    // Required fields
    assert event.timestamp is valid ISO 8601
    assert event.level in [ERROR, WARN, INFO, DEBUG, TRACE]
    assert event.event_type is not empty
    assert event.correlation_id is valid UUID
    assert event.component is not empty
    assert event.message is not empty
    
    // Context validation
    if event.event_type starts with "tile_":
        assert event.context.tile_id is valid Blake3Hash
    
    if event.event_type starts with "revision_":
        assert event.context.revision_id is valid RevisionId
    
    // Size limit
    assert len(serialize(event)) <= 65536  // 64 KiB max
```

## 7.5 Event Invariants

| ID | Invariant |
|---|---|
| OBS-EVT-INV-1 | Events MUST structured JSON |
| OBS-EVT-INV-2 | Events MUST memiliki correlation_id |
| OBS-EVT-INV-3 | Events MUST memiliki timestamp ISO 8601 |
| OBS-EVT-INV-4 | Security events MUST severity minimum WARN |
| OBS-EVT-INV-5 | Corruption events MUST severity ERROR |
| OBS-EVT-INV-6 | Event size MUST ≤ 64 KiB |

---

# 8. Tracing Spans

## 8.1 Span Model

`[OBS-TRACE-1]` Tracing menggunakan OpenTelemetry-compatible spans.

```text
Trace: inference_request
│
├── Span: execute
│   ├── Span: derive_query
│   ├── Span: select_cells
│   │   ├── Span: ann_search
│   │   └── Span: score_candidates
│   ├── Span: load_tiles
│   │   ├── Span: cache_lookup
│   │   ├── Span: tile_read [tile_id=b3:...]
│   │   ├── Span: tile_verify [tile_id=b3:...]
│   │   └── Span: tile_transfer [tile_id=b3:...]
│   ├── Span: execute_cells
│   │   ├── Span: execute_cell [cell_id=b3:...]
│   │   └── Span: execute_cell [cell_id=b3:...]
│   └── Span: compose
│
└── Span: decode
```

## 8.2 Span Structure

```rust
struct Span {
    // Identity
    trace_id: TraceId,           // 128-bit
    span_id: SpanId,             // 64-bit
    parent_span_id: Option<SpanId>,
    
    // Naming
    name: String,
    kind: SpanKind,              // Internal, Client, Server
    
    // Timing
    start_time: Timestamp,
    end_time: Timestamp,
    
    // Status
    status: SpanStatus,          // Ok, Error
    
    // Attributes
    attributes: HashMap<String, AttributeValue>,
    
    // Events
    events: Vec<SpanEvent>,
    
    // Links
    links: Vec<SpanLink>,
}

enum SpanKind {
    Internal,    // Internal operation
    Client,      // Client-side of remote call
    Server,      // Server-side of remote call
    Producer,    // Async message producer
    Consumer,    // Async message consumer
}
```

## 8.3 Standard Span Attributes

`[OBS-TRACE-2]` Standard span attributes:

| Attribute | Type | Description |
|---|---|---|
| `cnws.operation` | String | Operation name |
| `cnws.component` | String | Component name |
| `cnws.model_id` | String | Model identity |
| `cnws.revision` | String | Revision ID |
| `cnws.cell_id` | String | Cell ID (if applicable) |
| `cnws.tile_id` | String | Tile ID (if applicable) |
| `cnws.tile_size_bytes` | Int | Tile size |
| `cnws.representation` | String | Representation ID |
| `cnws.cache_level` | String | Cache level |
| `cnws.cache_hit` | Bool | Cache hit |
| `cnws.budget_used_flops` | Int | FLOPs used |
| `cnws.budget_used_bytes` | Int | Bytes moved |
| `cnws.error_code` | String | Error code (if error) |

## 8.4 Span Hierarchy

`[OBS-TRACE-3]` Span hierarchy untuk operasi utama:

### 8.4.1 Inference Trace

```text
inference_request
├── encode
├── execution_loop (iterative)
│   ├── budget_check
│   ├── derive_query
│   ├── select_cells
│   │   ├── ann_search
│   │   └── score_and_filter
│   ├── resolve_tiles
│   ├── load_tiles
│   │   ├── cache_lookup
│   │   ├── segment_read
│   │   ├── decompress
│   │   ├── verify_hash
│   │   └── transfer_to_device
│   ├── execute_cells
│   │   └── execute_cell (per Cell)
│   ├── compose
│   └── halt_check
└── decode
```

### 8.4.2 Conversion Trace

```text
conversion
├── detect_format
├── validate_source
├── read_metadata
├── process_tensors (per tensor)
│   ├── read_tensor
│   ├── normalize
│   ├── plan_tiles
│   ├── hash_tiles
│   ├── dedup_check
│   ├── compress
│   └── write_tiles
├── build_manifest
└── commit
```

### 8.4.3 Revision Trace

```text
revision_commit
├── validate_changes
├── stage_manifest
├── append_wal
├── rename_manifest
├── update_superblock
└── complete_commit
```

## 8.5 Span Events

`[OBS-TRACE-4]` Span events untuk mencatat kejadian dalam span:

```json
{
  "name": "tile_cache_miss",
  "timestamp": "2026-08-11T12:34:56.789Z",
  "attributes": {
    "tile_id": "b3:7f3a8e...",
    "level": "gpu",
    "fallback_level": "cpu"
  }
}
```

## 8.6 Sampling Strategy

`[OBS-TRACE-5]` Sampling strategy:

| Condition | Sampling Rate |
|---|---|
| Error traces | 100% |
| Slow traces (> P99 threshold) | 100% |
| Normal traces | 1% (configurable) |
| Debug mode | 100% |

## 8.7 Tracing Invariants

| ID | Invariant |
|---|---|
| OBS-TRACE-INV-1 | Spans MUST memiliki trace_id dan span_id |
| OBS-TRACE-INV-2 | Spans MUST memiliki timing |
| OBS-TRACE-INV-3 | Error spans MUST memiliki status Error |
| OBS-TRACE-INV-4 | Error traces MUST 100% sampled |
| OBS-TRACE-INV-5 | Span attributes MUST menggunakan standard names |

---

# 9. Correlation IDs

## 9.1 Correlation ID Format

`[OBS-CORR-1]` Correlation ID menggunakan **UUIDv7** (time-ordered).

```text
Format: 01926e8a-1234-7000-8000-123456789012
        ├──────┤ ├────┤
        timestamp random
```

`[OBS-CORR-2]` UUIDv7 dipilih karena:
- Time-ordered: dapat di-sort berdasarkan waktu.
- Globally unique: tidak perlu koordinasi.
- 128-bit: cukup untuk seluruh operasi.

## 9.2 Correlation ID Types

| Type | Scope | Description |
|---|---|---|
| `request_id` | Per request | Korelasi seluruh operasi dalam satu request |
| `operation_id` | Per operation | Korelasi satu operasi spesifik |
| `conversion_id` | Per conversion | Korelasi seluruh conversion |
| `revision_id` | Per revision | Korelasi revision operations |
| `recovery_id` | Per recovery | Korelasi recovery operations |

## 9.3 Correlation Propagation

`[OBS-CORR-3]` Correlation ID MUST di-propagate:

```text
Request arrives
    │
    ├── Generate request_id
    │
    ├── Log with request_id
    │
    ├── Create span with request_id
    │
    ├── Call subsystem
    │   ├── Pass request_id
    │   ├── Log with request_id
    │   └── Create child span
    │
    └── Response with request_id
```

## 9.4 Cross-Component Correlation

`[OBS-CORR-4]` Untuk korelasi lintas komponen:

```json
{
  "correlation": {
    "request_id": "01926e8a-1234-7000-8000-123456789012",
    "operation_id": "01926e8a-5678-7000-8000-123456789012",
    "parent_operation_id": "01926e8a-9abc-7000-8000-123456789012",
    "component": "runtime.cache",
    "tile_id": "b3:7f3a8e...",
    "revision": "b3:rev01..."
  }
}
```

## 9.5 Correlation Invariants

| ID | Invariant |
|---|---|
| OBS-CORR-INV-1 | Setiap request MUST memiliki request_id |
| OBS-CORR-INV-2 | request_id MUST UUIDv7 |
| OBS-CORR-INV-3 | request_id MUST di-propagate ke semua komponen |
| OBS-CORR-INV-4 | Semua logs dalam request MUST memiliki request_id |
| OBS-CORR-INV-5 | Semua spans dalam request MUST memiliki trace_id = request_id |

---

# 10. Runtime Diagnostics

## 10.1 Diagnostic Endpoints

`[OBS-DIAG-1]` Runtime diagnostics tersedia via:

1. **CLI commands** (MUST)
2. **HTTP endpoints** (MAY, untuk monitoring integration)

## 10.2 CLI Diagnostic Commands

```bash
# Health check
cnws diag health

# Store status
cnws diag store-status

# Cache status
cnws diag cache-status --level gpu,cpu,nvme

# Active Cells
cnws diag active-cells --limit 100

# Memory status
cnws diag memory-status

# Budget status
cnws diag budget-status

# Tile lookup
cnws diag tile-info <tile-id>

# Cell lookup
cnws diag cell-info <cell-id>
```

## 10.3 Health Check Response

```json
{
  "status": "healthy",
  "timestamp": "2026-08-11T12:34:56Z",
  "cnws_version": "1.0.0",
  "uptime_seconds": 86400,
  
  "components": {
    "store": {
      "status": "healthy",
      "path": "/data/model.cd",
      "size_bytes": 1099511627776
    },
    "runtime": {
      "status": "healthy",
      "active_cells": 42,
      "active_tiles": 128
    },
    "cache": {
      "status": "healthy",
      "gpu": {"used_bytes": 8589934592, "budget_bytes": 17179869184, "utilization": 0.5},
      "cpu": {"used_bytes": 17179869184, "budget_bytes": 34359738368, "utilization": 0.5},
      "hit_rate": 0.94
    },
    "memory": {
      "status": "healthy",
      "working_entries": 128,
      "working_bytes": 134217728
    }
  },
  
  "issues": []
}
```

## 10.4 Store Status Response

```json
{
  "model_id": "example-org/model-70b",
  "format_version": "1.0.0",
  "created_at": "2026-08-01T00:00:00Z",
  "last_modified": "2026-08-11T12:00:00Z",
  
  "counts": {
    "cells": 16384,
    "tiles": 65536,
    "segments": 256,
    "revisions": 42,
    "memory_entries": 1048576
  },
  
  "sizes": {
    "logical_bytes": 1536000000000,
    "stored_bytes": 1099511627776,
    "compression_ratio": 0.715
  },
  
  "active_revision": {
    "id": "b3:rev42...",
    "number": 42,
    "branch": "main"
  },
  
  "gc_status": {
    "last_gc": "2026-08-11T00:00:00Z",
    "unreachable_tiles": 128,
    "reclaimable_bytes": 134217728
  }
}
```

## 10.5 Cache Status Response

```json
{
  "levels": {
    "gpu": {
      "entries": 128,
      "used_bytes": 8589934592,
      "budget_bytes": 17179869184,
      "utilization": 0.5,
      "hit_rate": 0.96,
      "evictions_last_hour": 42
    },
    "cpu": {
      "entries": 512,
      "used_bytes": 17179869184,
      "budget_bytes": 34359738368,
      "utilization": 0.5,
      "hit_rate": 0.92,
      "evictions_last_hour": 128
    },
    "nvme": {
      "entries": 65536,
      "used_bytes": 1099511627776,
      "hit_rate": 0.88
    }
  },
  
  "total_hit_rate": 0.94,
  "total_evictions_last_hour": 170,
  
  "pinned_tiles": 8,
  "prefetch_queue_depth": 4
}
```

## 10.6 Runtime Diagnostic Invariants

| ID | Invariant |
|---|---|
| OBS-DIAG-INV-1 | Health check MUST tersedia |
| OBS-DIAG-INV-2 | Diagnostic commands MUST read-only |
| OBS-DIAG-INV-3 | Diagnostic response MUST JSON |
| OBS-DIAG-INV-4 | Diagnostic MUST NOT mempengaruhi performa signifikan |

---

# 11. Revision Diagnostics

## 11.1 Revision Diagnostic Commands

```bash
# List revisions
cnws diag revisions --limit 50

# Show revision details
cnws diag revision-info <revision-id>

# Show revision DAG
cnws diag revision-dag --format ascii,json,graphviz

# Show revision delta
cnws diag revision-delta <revision-id>

# Compare revisions
cnws diag revision-compare <rev-a> <rev-b>

# Show effective graph
cnws diag effective-graph <revision-id>
```

## 11.2 Revision DAG Response

```json
{
  "root": "b3:rev00...",
  "head": "b3:rev42...",
  "depth": 42,
  "branches": ["main", "coding", "reasoning"],
  
  "nodes": [
    {
      "id": "b3:rev00...",
      "number": 0,
      "parents": [],
      "children": ["b3:rev01..."],
      "branch": "main",
      "created_at": "2026-08-01T00:00:00Z",
      "message": "base import"
    },
    {
      "id": "b3:rev01...",
      "number": 1,
      "parents": ["b3:rev00..."],
      "children": ["b3:rev02...", "b3:rev10..."],
      "branch": "main",
      "created_at": "2026-08-02T00:00:00Z",
      "message": "fine-tune step 1"
    }
  ]
}
```

## 11.3 Revision Delta Response

```json
{
  "revision_id": "b3:rev42...",
  "parent_id": "b3:rev41...",
  
  "changes": {
    "cells_added": 2,
    "cells_refined": 5,
    "cells_removed": 0,
    "tiles_added": 8,
    "tiles_replaced": 5,
    "memory_added": 10,
    "routing_updated": true,
    "compositions_added": 1
  },
  
  "delta_size": {
    "logical_bytes": 5368709120,
    "stored_bytes": 3758096384
  },
  
  "affected_cells": [
    "b3:cell1...",
    "b3:cell2...",
    "b3:cell3..."
  ],
  
  "shared_tiles": {
    "from_parent": 65528,
    "new": 8,
    "deduplication_ratio": 0.99988
  }
}
```

## 11.4 Revision Diagnostic Invariants

| ID | Invariant |
|---|---|
| OBS-REVD-INV-1 | Revision DAG MUST dapat divisualisasi |
| OBS-REVD-INV-2 | Revision delta MUST dapat diinspeksi |
| OBS-REVD-INV-3 | Effective graph MUST dapat di-resolve |
| OBS-REVD-INV-4 | Revision diagnostics MUST read-only |

---

# 12. Corruption Diagnostics

## 12.1 Corruption Diagnostic Commands

```bash
# Check integrity
cnws diag integrity --scope all,segments,manifest,tiles

# Show corruption status
cnws diag corruption-status

# Show quarantine list
cnws diag quarantine-list

# Verify specific Tile
cnws diag verify-tile <tile-id>

# Verify specific segment
cnws diag verify-segment <segment-id>

# Recovery status
cnws diag recovery-status
```

## 12.2 Integrity Check Response

```json
{
  "timestamp": "2026-08-11T12:34:56Z",
  "scope": "all",
  "duration_seconds": 45.2,
  
  "results": {
    "manifest": {
      "valid": true,
      "hash_match": true
    },
    "superblock": {
      "valid": true,
      "magic_valid": true,
      "version_supported": true
    },
    "segments": {
      "total": 256,
      "valid": 255,
      "corrupted": 1,
      "corrupted_ids": [42]
    },
    "tiles": {
      "total": 65536,
      "verified": 65536,
      "valid": 65530,
      "corrupted": 6,
      "corrupted_ids": [
        "b3:tile1...",
        "b3:tile2...",
        "b3:tile3...",
        "b3:tile4...",
        "b3:tile5...",
        "b3:tile6..."
      ]
    },
    "indexes": {
      "cells_idx": true,
      "tiles_idx": true,
      "memory_idx": true
    }
  },
  
  "quarantine": {
    "tiles": 6,
    "segments": 1,
    "total_bytes": 805306368
  }
}
```

## 12.3 Corruption Event Schema

```json
{
  "timestamp": "2026-08-11T12:34:56.789Z",
  "level": "ERROR",
  "event_type": "tile_corrupted",
  "correlation_id": "01926e8a-1234-7000-8000-123456789012",
  "component": "storage.integrity",
  "message": "Tile corruption detected",
  
  "context": {
    "tile_id": "b3:7f3a8e...",
    "expected_hash": "b3:7f3a8e...",
    "actual_hash": "b3:9c2b1f...",
    "segment_id": 42,
    "offset": 1048576,
    "size_bytes": 134217728,
    "detection_method": "blake3_verification",
    "cell_id": "b3:1c33...",
    "representation": "bf16"
  },
  
  "actions": {
    "quarantined": true,
    "quarantine_path": "corrupt/b3_7f3a8e.quarantine",
    "recovery_attempted": true,
    "recovery_source": "replica",
    "recovery_success": true
  }
}
```

## 12.4 Quarantine List Response

```json
{
  "total_items": 7,
  "total_bytes": 939524096,
  
  "items": [
    {
      "quarantine_id": "q-000001",
      "item_type": "tile",
      "item_id": "b3:7f3a8e...",
      "quarantined_at": "2026-08-11T10:00:00Z",
      "reason": "blake3_mismatch",
      "size_bytes": 134217728,
      "status": "quarantined",
      "recovery_attempts": [
        {
          "timestamp": "2026-08-11T10:01:00Z",
          "source": "replica",
          "success": true
        }
      ]
    }
  ]
}
```

## 12.5 Corruption Diagnostic Invariants

| ID | Invariant |
|---|---|
| OBS-CORR-INV-1 | Integrity check MUST tersedia |
| OBS-CORR-INV-2 | Corruption events MUST severity ERROR |
| OBS-CORR-INV-3 | Quarantine list MUST dapat diinspeksi |
| OBS-CORR-INV-4 | Recovery status MUST dapat diinspeksi |
| OBS-CORR-INV-5 | Corruption diagnostics MUST read-only |

---

# 13. Export & Integration

## 13.1 Metrics Export

`[OBS-EXP-1]` Metrics MUST diekspor dalam format:

| Format | Status | Use Case |
|---|---|---|
| OpenMetrics (Prometheus) | MUST | Primary metrics format |
| OTLP (OpenTelemetry) | SHOULD | OpenTelemetry integration |
| JSON | MAY | Debugging, custom integration |

## 13.2 Logs Export

`[OBS-EXP-2]` Logs MUST diekspor ke:

| Destination | Status | Use Case |
|---|---|---|
| Stdout/stderr | MUST | Development, container logging |
| File | MUST | Persistent logging |
| Syslog | MAY | Centralized logging |
| OTLP | SHOULD | OpenTelemetry integration |

## 13.3 Traces Export

`[OBS-EXP-3]` Traces SHOULD diekspor via:

| Protocol | Status |
|---|---|
| OTLP (gRPC) | SHOULD |
| OTLP (HTTP) | SHOULD |
| Jaeger | MAY |
| Zipkin | MAY |

## 13.4 Export Configuration

```rust
struct ExportConfig {
    // Metrics
    metrics_enabled: bool,           // default: true
    metrics_format: MetricsFormat,   // default: OpenMetrics
    metrics_port: u16,               // default: 9090
    metrics_interval_ms: u64,        // default: 15000
    
    // Logs
    logs_enabled: bool,              // default: true
    logs_level: LogLevel,            // default: INFO
    logs_output: LogOutput,          // default: Stdout + File
    logs_file_path: PathBuf,         // default: /var/log/cnws/cnws.log
    logs_max_size_mb: u64,           // default: 100
    logs_max_files: u32,             // default: 10
    
    // Traces
    traces_enabled: bool,            // default: true
    traces_exporter: TraceExporter,  // default: OTLP
    traces_endpoint: String,         // default: localhost:4317
    traces_sampling_rate: f64,       // default: 0.01
}
```

## 13.5 Export Invariants

| ID | Invariant |
|---|---|
| OBS-EXP-INV-1 | Metrics MUST diekspor dalam OpenMetrics |
| OBS-EXP-INV-2 | Logs MUST ke stdout dan file |
| OBS-EXP-INV-3 | Export MUST async dan non-blocking |
| OBS-EXP-INV-4 | Export failure MUST NOT crash aplikasi |
| OBS-EXP-INV-5 | Export MUST memiliki backpressure |

---

# 14. Retention & Sampling

## 14.1 Retention Periods

`[OBS-RET-1]` Retention periods default:

| Data Type | Retention | Configurable |
|---|---|---|
| Metrics (raw) | 15 hari | YES |
| Metrics (aggregated) | 90 hari | YES |
| Logs | 30 hari | YES |
| Traces | 7 hari | YES |
| Security events | 365 hari | YES |
| Corruption events | 365 hari | YES |
| Audit trail | 365 hari | YES |

## 14.2 Sampling Strategy

`[OBS-SAMP-1]` Sampling strategy:

| Data Type | Sampling | Notes |
|---|---|---|
| Error logs | 100% | Selalu log semua errors |
| Warn logs | 100% | Selalu log semua warnings |
| Info logs | 100% | Log semua info events |
| Debug logs | Configurable | Default: disabled di production |
| Trace logs | Configurable | Default: disabled di production |
| Error traces | 100% | Selalu trace errors |
| Slow traces | 100% | Trace yang > P99 threshold |
| Normal traces | 1% | Configurable |

## 14.3 Log Rotation

`[OBS-RET-2]` Log rotation:

```text
Max file size: 100 MB (configurable)
Max files: 10 (configurable)
Rotation: size-based
Compression: gzip untuk rotated files
```

## 14.4 Retention Invariants

| ID | Invariant |
|---|---|
| OBS-RET-INV-1 | Security events MUST retained 365 hari |
| OBS-RET-INV-2 | Corruption events MUST retained 365 hari |
| OBS-RET-INV-3 | Log rotation MUST otomatis |
| OBS-RET-INV-4 | Retention MUST configurable |

---

# 15. Performance Overhead

## 15.1 Overhead Budget

`[OBS-PERF-1]` Observability overhead budget:

| Resource | Budget |
|---|---|
| CPU | < 5% dari total CPU usage |
| Memory | < 2% dari total memory usage |
| I/O | < 1% dari total I/O bandwidth |
| Latency | < 1% tambahan latency per operasi |

## 15.2 Overhead Measurement

`[OBS-PERF-2]` Overhead MUST diukur:

```pseudo
function measure_overhead():
    // Run workload without observability
    baseline = run_workload(observability=false)
    
    // Run workload with observability
    with_obs = run_workload(observability=true)
    
    // Compute overhead
    cpu_overhead = (with_obs.cpu - baseline.cpu) / baseline.cpu
    memory_overhead = (with_obs.memory - baseline.memory) / baseline.memory
    latency_overhead = (with_obs.latency - baseline.latency) / baseline.latency
    
    assert cpu_overhead < 0.05
    assert memory_overhead < 0.02
    assert latency_overhead < 0.01
```

## 15.3 Overhead Invariants

| ID | Invariant |
|---|---|
| OBS-PERF-INV-1 | Observability CPU overhead MUST < 5% |
| OBS-PERF-INV-2 | Observability memory overhead MUST < 2% |
| OBS-PERF-INV-3 | Observability MUST NOT blocking operasi utama |
| OBS-PERF-INV-4 | Observability buffer MUST bounded |

---

# 16. Final Observability Contract

## 16.1 Ringkasan Keputusan Observability

| ID | Keputusan |
|---|---|
| OBS-F01 | Metrics menggunakan OpenMetrics format. |
| OBS-F02 | Logging menggunakan structured JSON. |
| OBS-F03 | Tracing menggunakan OpenTelemetry-compatible spans. |
| OBS-F04 | Correlation ID menggunakan UUIDv7. |
| OBS-F05 | Metric prefix: `cnws_`. |
| OBS-F06 | Label cardinality limit: 10,000 per metric. |
| OBS-F07 | Log levels: ERROR, WARN, INFO, DEBUG, TRACE. |
| OBS-F08 | Sampling: 100% errors, 1% traces. |
| OBS-F09 | Retention: metrics 90d, logs 30d, traces 7d, security 365d. |
| OBS-F10 | Overhead budget: <5% CPU, <2% memory. |
| OBS-F11 | Diagnostic via CLI (MUST) dan HTTP (MAY). |
| OBS-F12 | Security events severity minimum WARN. |
| OBS-F13 | Corruption events severity ERROR. |
| OBS-F14 | Revision events severity INFO. |
| OBS-F15 | Metric naming: cnws_<subsystem>_<name>_<unit>. |
| OBS-F16 | Labels snake_case, bounded cardinality. |
| OBS-F17 | Units: seconds, bytes, total, ratio. |
| OBS-F18 | Events MUST structured JSON. |
| OBS-F19 | Spans MUST memiliki trace_id dan span_id. |
| OBS-F20 | Diagnostic commands MUST read-only. |

## 16.2 Observability Invariants

| ID | Invariant |
|---|---|
| OBS-INV-1 | Semua komponen MUST menghasilkan metrics. |
| OBS-INV-2 | Semua komponen MUST menghasilkan logs. |
| OBS-INV-3 | Operasi kompleks SHOULD menghasilkan traces. |
| OBS-INV-4 | Metrics MUST menggunakan prefix cnws_. |
| OBS-INV-5 | Logs MUST structured JSON. |
| OBS-INV-6 | Events MUST memiliki correlation_id. |
| OBS-INV-7 | Label cardinality MUST bounded. |
| OBS-INV-8 | High-cardinality values MUST NOT jadi labels. |
| OBS-INV-9 | Security events MUST severity minimum WARN. |
| OBS-INV-10 | Corruption events MUST severity ERROR. |
| OBS-INV-11 | Error traces MUST 100% sampled. |
| OBS-INV-12 | Observability MUST NOT blocking. |
| OBS-INV-13 | Observability overhead MUST < 5% CPU. |
| OBS-INV-14 | Diagnostic MUST read-only. |
| OBS-INV-15 | Health check MUST tersedia. |
| OBS-INV-16 | Integrity check MUST tersedia. |
| OBS-INV-17 | Revision diagnostics MUST tersedia. |
| OBS-INV-18 | Corruption diagnostics MUST tersedia. |
| OBS-INV-19 | Security events MUST retained 365 hari. |
| OBS-INV-20 | Export MUST async dan non-blocking. |

## 16.3 Observability Coverage Matrix

| Subsystem | Metrics | Logs | Traces | Diagnostics |
|---|---|---|---|---|
| Runtime | MUST | MUST | SHOULD | MUST |
| Storage | MUST | MUST | SHOULD | MUST |
| Cache | MUST | MUST | SHOULD | MUST |
| Conversion | MUST | MUST | SHOULD | MUST |
| Revision | MUST | MUST | SHOULD | MUST |
| Memory | MUST | MUST | SHOULD | MUST |
| GC | MUST | MUST | MAY | MUST |
| Security | MUST | MUST | SHOULD | MUST |
| Integrity | MUST | MUST | SHOULD | MUST |

## 16.4 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Observability final dan mengikat** untuk CNWS. Ia mendefinisikan secara presisi bagaimana metrics, logging, dan tracing bekerja, dari metric definitions hingga event schemas, dari tracing spans hingga correlation IDs, dari runtime diagnostics hingga corruption diagnostics.

Observability bukan afterthought — ia adalah bagian integral dari arsitektur CNWS yang memungkinkan monitoring, debugging, auditing, dan optimization seluruh sistem.

Seluruh implementasi komponen CNWS MUST mengimplementasikan observability sesuai spesifikasi ini.

Tidak ada keputusan observability yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN OBSERVABILITY SPECIFICATION**
