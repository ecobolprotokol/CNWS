# CNWS
## Runtime & Execution Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Runtime & Execution Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (RUNTIME BEHAVIOR SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS DAS; CNWS Cell & Schema Spec |
| Hulu ke | Implementasi CNWS Lattice Runtime |
| Otoritas | Spesifikasi perilaku runtime tunggal untuk eksekusi CNWS |
| Traceability | Menerjemahkan FAC-13 s/d FAC-31 menjadi implementable behavior |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract        DAS                    Runtime & Execution Spec
─────────────────────       ──────────────         ────────────────────────
FAC-13..FAC-31          ──► Component layout   ──► Executable behavior
"MUST be content-based"     Module boundaries        Query derivation
"MUST be adaptive"          Interfaces               Selection algorithm
"MUST enforce budget"       Threading                Budget enforcement
                                                     Scheduling policy
                                                     Halt conditions
```

`[RT-DOC-1]` Dokumen ini menerjemahkan invariant arsitektural menjadi **perilaku yang dapat diimplementasikan dan diuji**.

`[RT-DOC-2]` Setiap bagian MUST memiliki traceability ke FAC yang bersangkutan.

`[RT-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Traceability Matrix

| Dokumen Ini | FAC |
|---|---|
| §4 Query Derivation | FAC-13 |
| §5 Cell Selection | FAC-13, FAC-27 |
| §6 Routing | FAC-13, FAC-27 |
| §7 Composition | FAC-14 |
| §8 Execution Planning | FAC-14, FAC-26 |
| §9 Adaptive Depth | FAC-15, FAC-28 |
| §10 Adaptive Compute | FAC-28, FAC-31 |
| §11 Halt Conditions | FAC-15, FAC-29 |
| §12 Compute Budget | FAC-29 |
| §13 Memory Budget | FAC-19, FAC-29 |
| §14 Prefetch | FAC-26, FAC-27 |
| §15 Eviction | FAC-19 |
| §16 Representation Selection | FAC-26, FAC-30 |
| §17 GPU/CPU/NVMe Scheduling | FAC-26, FAC-27 |
| §18 Deterministic Execution | FAC-34 |
| §5, §13 (context) | FAC-16, FAC-17, FAC-18 |
| §10 (active ratio) | FAC-30 |

## 0.4 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-RT-01 | Query derivation menggunakan projection dari WorkingState. |
| DF-RT-02 | Cell selection menggunakan ANN search dengan threshold. |
| DF-RT-03 | Default similarity metric = cosine. |
| DF-RT-04 | Default selection k = 16, threshold = 0.3. |
| DF-RT-05 | Adaptive depth menggunakan learned halt predictor. |
| DF-RT-06 | Default max depth = 25, min depth = 3. |
| DF-RT-07 | Difficulty estimator menggunakan lightweight classifier. |
| DF-RT-08 | Budget enforcement bersifat hard, bukan advisory. |
| DF-RT-09 | Prefetch menggunakan dependency-aware policy. |
| DF-RT-10 | Eviction menggunakan LRU-by-priority dengan byte capacity. |
| DF-RT-11 | Representation selection berdasarkan hardware profile + accuracy policy. |
| DF-RT-12 | Deterministic execution menggunakan seeded RNG dan deterministic kernels. |

---

# 1. Executive Summary

## 1.1 Tujuan Runtime

CNWS Lattice Runtime adalah weight orchestration layer yang memutuskan:

1. **Cell apa** yang dimuat untuk input tertentu
2. **Representation apa** yang digunakan
3. **Kapan** dimuat
4. **Kapan** dibuang
5. **Di level cache mana** ditempatkan
6. **Berapa banyak compute** dialokasikan
7. **Kapan berhenti**

`[RT-EXEC-1]` Runtime MUST NOT memuat seluruh model.

`[RT-EXEC-2]` Runtime MUST mengaktifkan hanya Cell yang relevan.

`[RT-EXEC-3]` Runtime MUST enforce budget secara hard.

`[RT-EXEC-4]` Runtime MUST deterministic untuk input dan state yang sama.

## 1.2 Execution Loop Overview

```text
Input
  │
  ▼
┌─────────────────────────────────────────────────┐
│ ENCODE                                          │
│   input → WorkingState                          │
└──────────────────┬──────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────┐
│ EXECUTION LOOP (iterative)                      │
│                                                 │
│   ┌─────────────────────────────────────────┐   │
│   │ 1. Budget Check                         │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 2. Query Derivation                     │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 3. Cell Selection (Routing)             │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 4. Tile Resolution & Loading            │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 5. Cell Execution                       │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 6. Composition                          │   │
│   └──────────────────┬──────────────────────┘   │
│                      │                          │
│   ┌──────────────────▼──────────────────────┐   │
│   │ 7. Halt Check                           │   │
│   │    if halt → EXIT LOOP                  │   │
│   │    else → back to step 1                │   │
│   └─────────────────────────────────────────┘   │
│                                                 │
└──────────────────┬──────────────────────────────┘
                   │
                   ▼
┌─────────────────────────────────────────────────┐
│ DECODE                                          │
│   WorkingState → output                         │
└─────────────────────────────────────────────────┘
```

`[RT-EXEC-5]` Execution loop MUST iteratif, bukan fixed-depth.

`[RT-EXEC-6]` Setiap iterasi MUST memeriksa budget.

`[RT-EXEC-7]` Setiap iterasi MUST memeriksa halt condition.

---

# 2. Runtime State

## 2.1 WorkingState

`[RT-STATE-1]` WorkingState adalah satu-satunya state aktif selama eksekusi.

```rust
struct WorkingState {
    // Primary representation
    active_vector: Vec<f32>,
    
    // Context (content-addressed, NOT KV-cache)
    context_entries: Vec<MemoryRef>,
    
    // Active Cells
    current_cells: Vec<CellRef>,
    
    // Composition state
    composition_stack: Vec<ComposeOp>,
    
    // Budget tracking
    compute_used: ComputeUsage,
    
    // Step tracking
    steps_taken: u32,
    
    // Control
    halt_signal: bool,
    confidence: f32,
    
    // Determinism
    rng_state: RngState,
}
```

`[RT-STATE-2]` WorkingState MUST bounded.

`[RT-STATE-3]` Context entries MUST content-addressed memory references, bukan KV-cache entries.

`[RT-STATE-4]` Context MUST NOT tumbuh linear terhadap sequence length.

## 2.2 ComputeUsage

```rust
struct ComputeUsage {
    flops: u64,
    bytes_moved: u64,
    cells_executed: u64,
    steps: u32,
    wall_time_us: u64,
}
```

`[RT-STATE-5]` ComputeUsage MUST di-update setiap operasi.

`[RT-STATE-6]` ComputeUsage MUST tersedia untuk observability.

---

# 3. Runtime Configuration

## 3.1 RuntimeConfig

```rust
struct RuntimeConfig {
    // Selection
    selection_k: u32,
    selection_threshold: f32,
    selection_metric: SimilarityMetric,
    
    // Depth
    min_depth: u32,
    max_depth: u32,
    
    // Budget
    max_flops: u64,
    max_bytes_moved: u64,
    max_wall_time_us: u64,
    
    // Memory
    gpu_budget: u64,
    cpu_budget: u64,
    working_memory_limit: u64,
    
    // Prefetch
    prefetch_policy: PrefetchPolicy,
    prefetch_depth: u32,
    
    // Eviction
    eviction_policy: EvictionPolicy,
    
    // Representation
    accuracy_policy: AccuracyPolicy,
    
    // Determinism
    seed: Option<u64>,
    deterministic_mode: bool,
}
```

## 3.2 Default Configuration

`[RT-CFG-1]` Default configuration:

| Parameter | Default | Range |
|---|---|---|
| `selection_k` | 16 | 1–256 |
| `selection_threshold` | 0.3 | 0.0–1.0 |
| `selection_metric` | Cosine | — |
| `min_depth` | 3 | 1–10 |
| `max_depth` | 25 | 10–100 |
| `max_flops` | 10 GFLOP | — |
| `max_bytes_moved` | 1 GB | — |
| `max_wall_time_us` | 1000000 | — |
| `prefetch_policy` | DependencyAware | — |
| `prefetch_depth` | 2 | 0–10 |
| `eviction_policy` | LRUBypriority | — |
| `accuracy_policy` | Balanced | — |
| `deterministic_mode` | true | — |

`[RT-CFG-2]` Konfigurasi MUST dapat di-override per request.

`[RT-CFG-3]` Konfigurasi MUST divalidasi sebelum digunakan.

---

# 4. Query Derivation

## 4.1 Purpose

Query derivation mengubah WorkingState menjadi Query untuk Cell selection.

`[RT-QD-1]` Query derivation MUST content-based, bukan position-based. **(FAC-13)**

## 4.2 Query Derivation Algorithm

```pseudo
function derive_query(state: WorkingState, config: RuntimeConfig) -> Query:
    // 1. Project active vector to query space
    query_vector = project(state.active_vector)
    
    // 2. Incorporate context if available
    if state.context_entries is not empty:
        context_vector = aggregate_context(state.context_entries)
        query_vector = blend(query_vector, context_vector, alpha=0.7)
    
    // 3. Normalize for cosine similarity
    if config.selection_metric == Cosine:
        query_vector = normalize(query_vector)
    
    // 4. Build Query object
    return Query {
        vector: query_vector,
        max_cells: config.selection_k,
        threshold: config.selection_threshold,
        budget: remaining_budget(state, config),
    }
```

## 4.3 Projection

`[RT-QD-2]` Projection dari active_vector ke query space:

```pseudo
function project(active_vector: Vec<f32>) -> Vec<f32>:
    // Linear projection to query dimensions
    // query_dim == index_vector_dim (default 512)
    return W_proj @ active_vector + b_proj
```

`[RT-QD-3]` Projection parameters MUST loaded dari Routing Cell.

`[RT-QD-4]` Projection dimensions MUST sama dengan index vector dimensions.

## 4.4 Context Aggregation

`[RT-QD-5]` Context aggregation MUST content-addressed.

```pseudo
function aggregate_context(entries: Vec<MemoryRef>) -> Vec<f32>:
    vectors = []
    for entry in entries:
        mem = retrieve_memory(entry)
        vectors.append(mem.key_vector)
    
    // Weighted average by relevance
    weights = compute_relevance_weights(vectors, current_query)
    return weighted_average(vectors, weights)
```

`[RT-QD-6]` Context aggregation MUST NOT menggunakan sequential positional encoding.

## 4.5 Blending

`[RT-QD-7]` Blending query dengan context:

```text
blended = alpha * query_vector + (1 - alpha) * context_vector
```

`[RT-QD-8]` Default alpha = 0.7 (query-dominant).

## 4.6 Query Invariants

| ID | Invariant |
|---|---|
| RT-QD-INV-1 | Query MUST derived dari WorkingState |
| RT-QD-INV-2 | Query MUST NOT menggunakan positional index |
| RT-QD-INV-3 | Query dimensions MUST match index dimensions |
| RT-QD-INV-4 | Query derivation MUST deterministic |
| RT-QD-INV-5 | Context aggregation MUST content-addressed |

---

# 5. Cell Selection

## 5.1 Purpose

Cell selection memilih Cell yang relevan untuk eksekusi berdasarkan Query.

`[RT-SEL-1]` Cell selection MUST content-based. **(FAC-13)**

`[RT-SEL-2]` Cell selection MUST hanya memilih Cell relevan. **(FAC-27)**

## 5.2 Selection Algorithm

```pseudo
function select_cells(query: Query, config: RuntimeConfig) -> Vec<CellRef>:
    // 1. Coarse filter by domain (optional, for large Cell sets)
    candidates = coarse_filter(query, domain_clusters)
    
    // 2. ANN search
    results = ann_search(
        index = cell_index,
        query = query.vector,
        k = query.max_cells * 2,  // over-fetch for filtering
        metric = config.selection_metric,
    )
    
    // 3. Score each candidate
    scored = []
    for (cell_id, distance) in results:
        score = compute_score(cell_id, distance, query)
        scored.append((cell_id, score))
    
    // 4. Apply threshold
    filtered = [(id, s) for (id, s) in scored if s >= query.threshold]
    
    // 5. Sort by score descending
    filtered.sort_by_score_desc()
    
    // 6. Take top-k
    selected = filtered[:query.max_cells]
    
    // 7. Verify compatibility with current state
    verified = verify_compatibility(selected, current_state)
    
    return verified
```

## 5.3 Score Computation

`[RT-SEL-3]` Score computation:

```text
score(cell, query) = similarity(cell.index_vector, query.vector)
                   × confidence(cell)
                   × recency_boost(cell)
                   × budget_factor(cell, query.budget)
```

Where:

```text
similarity     = cosine_similarity(cell.index_vector, query.vector)
confidence     = cell.routing_statistics.success_rate
recency_boost  = 1.0 + 0.1 × exp(-age_in_steps / 10)
budget_factor  = 1.0 if cell.cost <= remaining_budget
                 0.5 if cell.cost <= 2 × remaining_budget
                 0.0 otherwise
```

`[RT-SEL-4]` Similarity metric MUST sesuai konfigurasi.

`[RT-SEL-5]` Confidence MUST berasal dari routing statistics.

## 5.4 ANN Search

`[RT-SEL-6]` ANN search MUST O(log N) terhadap total Cell count.

`[RT-SEL-7]` ANN index structure SHOULD HNSW dengan:
- `m = 32`
- `ef_construction = 200`
- `ef_search = max(k * 2, 64)`

`[RT-SEL-8]` ANN recall MUST ≥ 0.95 untuk k ≤ 64.

## 5.5 Empty Selection Handling

`[RT-SEL-9]` Jika selection kosong:

```pseudo
function handle_empty_selection(state, config):
    if config.allow_growth:
        // Create new Cell
        new_cell = create_cell(state)
        return [new_cell]
    else:
        // Relax threshold and retry
        relaxed_threshold = config.selection_threshold * 0.5
        results = select_cells(query, relaxed_threshold)
        if results is empty:
            return halt_with_partial_result(state)
        return results
```

`[RT-SEL-10]` Cell creation MUST mengikuti Learning Engine specification.

## 5.6 Selection Invariants

| ID | Invariant |
|---|---|
| RT-SEL-INV-1 | Selection MUST content-based |
| RT-SEL-INV-2 | Selection MUST O(log N) |
| RT-SEL-INV-3 | Selection MUST menghormati budget |
| RT-SEL-INV-4 | Selection MUST deterministic untuk input sama |
| RT-SEL-INV-5 | Selection MUST NOT memilih Cell yang tidak kompatibel |

---

# 6. Routing

## 6.1 Purpose

Routing adalah mekanisme seleksi Cell yang dapat dipelajari.

`[RT-ROUTE-1]` Routing MUST content-based. **(FAC-13)**

## 6.2 Routing Policy

```rust
struct RoutingPolicy {
    // Selection parameters
    default_k: u32,
    default_threshold: f32,
    metric: SimilarityMetric,
    
    // Domain filters
    domain_clusters: Vec<DomainCluster>,
    
    // Score weights
    similarity_weight: f32,
    confidence_weight: f32,
    recency_weight: f32,
    
    // Budget awareness
    budget_aware: bool,
    
    // Version
    policy_version: u64,
}
```

## 6.3 Routing Statistics

```rust
struct RoutingStatistics {
    per_cell: HashMap<CellId, CellRoutingStats>,
    per_edge: HashMap<(CellId, CellId), EdgeRoutingStats>,
    global: GlobalRoutingStats,
}

struct CellRoutingStats {
    selection_count: u64,
    success_count: u64,
    success_rate: f32,
    avg_contribution: f32,
    last_selected_step: u64,
}

struct EdgeRoutingStats {
    traversal_count: u64,
    success_count: u64,
    success_rate: f32,
}
```

## 6.4 Routing Update

`[RT-ROUTE-2]` Routing statistics MUST di-update setelah eksekusi.

```pseudo
function update_routing(selected_cells, state, output_quality):
    for cell in selected_cells:
        stats = routing_stats.per_cell[cell.id]
        stats.selection_count += 1
        
        if output_quality > success_threshold:
            stats.success_count += 1
        
        stats.success_rate = stats.success_count / stats.selection_count
        stats.avg_contribution = update_moving_average(
            stats.avg_contribution,
            cell.contribution,
        )
        stats.last_selected_step = state.steps_taken
    
    // Update edge statistics
    for (from, to) in consecutive_pairs(selected_cells):
        edge_stats = routing_stats.per_edge[(from, to)]
        edge_stats.traversal_count += 1
        if output_quality > success_threshold:
            edge_stats.success_count += 1
```

`[RT-ROUTE-3]` Routing update MUST incremental.

`[RT-ROUTE-4]` Routing update MUST NOT memerlukan full retraining.

## 6.5 Routing Persistence

`[RT-ROUTE-5]` Routing statistics MAY di-persist ke `.cd` tanpa revision baru.

`[RT-ROUTE-6]` Routing policy change MUST membuat revision baru.

## 6.6 Routing Invariants

| ID | Invariant |
|---|---|
| RT-ROUTE-INV-1 | Routing MUST content-based |
| RT-ROUTE-INV-2 | Routing statistics MUST di-update setelah eksekusi |
| RT-ROUTE-INV-3 | Routing update MUST incremental |
| RT-ROUTE-INV-4 | Routing policy version MUST tracked |
| RT-ROUTE-INV-5 | Routing MUST deterministic untuk state sama |

---

# 7. Composition

## 7.1 Purpose

Composition menggabungkan output Cell menjadi WorkingState baru.

`[RT-COMP-1]` Computation MUST dynamically composed per input. **(FAC-14)**

## 7.2 Composition Algorithm

```pseudo
function compose(state: WorkingState, outputs: Vec<CellOutput>) -> WorkingState:
    // 1. Determine composition mode
    mode = determine_composition_mode(outputs)
    
    // 2. Apply composition
    match mode:
        case Sequential:
            new_vector = compose_sequential(state.active_vector, outputs)
        case Parallel:
            new_vector = compose_parallel(state.active_vector, outputs)
        case Conditional:
            new_vector = compose_conditional(state.active_vector, outputs)
        case Iterative:
            new_vector = compose_iterative(state.active_vector, outputs)
    
    // 3. Update context if needed
    new_context = update_context(state.context_entries, outputs)
    
    // 4. Build new state
    return WorkingState {
        active_vector: new_vector,
        context_entries: new_context,
        current_cells: outputs.cell_ids,
        composition_stack: state.composition_stack,
        compute_used: state.compute_used + compute_cost(outputs),
        steps_taken: state.steps_taken + 1,
        halt_signal: false,
        confidence: compute_confidence(outputs),
        rng_state: state.rng_state,
    }
```

## 7.3 Composition Modes

### 7.3.1 Sequential Composition

`[RT-COMP-2]` Sequential: output Cell i menjadi input Cell i+1.

```pseudo
function compose_sequential(vector, outputs):
    result = vector
    for output in outputs:
        result = output.apply(result)
    return result
```

### 7.3.2 Parallel Composition

`[RT-COMP-3]` Parallel: semua Cell dieksekusi pada input yang sama, hasil digabung.

```pseudo
function compose_parallel(vector, outputs):
    results = []
    for output in outputs:
        results.append(output.apply(vector))
    return merge(results)
```

### 7.3.3 Merge Operation

`[RT-COMP-4]` Merge operation:

```text
merge(results) = weighted_sum(results, weights)
               | concat(results)
               | gating(results)
```

`[RT-COMP-5]` Merge operation MUST ditentukan oleh composition pattern.

## 7.4 Context Update

`[RT-COMP-6]` Context update MUST content-addressed. **(FAC-16)**

```pseudo
function update_context(entries, outputs):
    // Compose outputs into memory entry
    if should_memorize(outputs):
        new_entry = compress_to_memory(outputs)
        entries.append(new_entry)
    
    // Evict old entries if over limit
    if entries.len() > max_context_entries:
        entries = evict_context_entries(entries)
    
    return entries
```

`[RT-COMP-7]` Context entries MUST bounded.

`[RT-COMP-8]` Context MUST NOT tumbuh linear terhadap sequence length. **(FAC-17)**

## 7.5 Composition Invariants

| ID | Invariant |
|---|---|
| RT-COMP-INV-1 | Composition MUST dynamic per input |
| RT-COMP-INV-2 | Composition MUST NOT fixed-depth |
| RT-COMP-INV-3 | Context update MUST content-addressed |
| RT-COMP-INV-4 | Context MUST bounded |
| RT-COMP-INV-5 | Composition MUST deterministic |

---

# 8. Execution Planning

## 8.1 Purpose

Execution planning menentukan urutan dan paralelisme eksekusi Cell.

`[RT-PLAN-1]` Execution plan MUST dynamic. **(FAC-14)**

`[RT-PLAN-2]` Execution MUST NOT memerlukan full-model loading. **(FAC-26)**

## 8.2 Execution Plan Structure

```rust
struct ExecutionPlan {
    steps: Vec<PlanStep>,
    parallel_groups: Vec<Vec<CellId>>,
    dependencies: HashMap<CellId, Vec<CellId>>,
    estimated_cost: ComputeUsage,
}

struct PlanStep {
    cells: Vec<CellId>,
    parallel: bool,
    estimated_flops: u64,
    estimated_bytes: u64,
}
```

## 8.3 Planning Algorithm

```pseudo
function build_plan(selected_cells, state, config) -> ExecutionPlan:
    // 1. Build dependency graph
    deps = build_dependency_graph(selected_cells)
    
    // 2. Topological sort
    ordered = topological_sort(deps)
    
    // 3. Identify parallel groups
    parallel_groups = identify_parallel_groups(ordered, deps)
    
    // 4. Estimate cost
    cost = estimate_cost(parallel_groups)
    
    // 5. Check budget
    if cost > remaining_budget(state, config):
        // Reduce plan
        parallel_groups = reduce_to_budget(parallel_groups, budget)
    
    // 6. Build plan
    return ExecutionPlan {
        steps: build_steps(parallel_groups),
        parallel_groups: parallel_groups,
        dependencies: deps,
        estimated_cost: cost,
    }
```

## 8.4 Parallel Group Identification

`[RT-PLAN-3]` Cell tanpa dependency antar mereka MUST dapat dieksekusi parallel.

```pseudo
function identify_parallel_groups(ordered, deps):
    groups = []
    current_group = []
    
    for cell in ordered:
        if has_dependency_in_group(cell, current_group, deps):
            groups.append(current_group)
            current_group = [cell]
        else:
            current_group.append(cell)
    
    if current_group is not empty:
        groups.append(current_group)
    
    return groups
```

## 8.5 Plan Adaptation

`[RT-PLAN-4]` Execution plan MAY di-adaptasi saat runtime berdasarkan:
- Cell availability
- Cache state
- Budget remaining
- Priority changes

## 8.6 Planning Invariants

| ID | Invariant |
|---|---|
| RT-PLAN-INV-1 | Plan MUST dynamic |
| RT-PLAN-INV-2 | Plan MUST menghormati dependencies |
| RT-PLAN-INV-3 | Plan MUST menghormati budget |
| RT-PLAN-INV-4 | Plan MUST memungkinkan parallel execution |
| RT-PLAN-INV-5 | Plan MUST NOT memerlukan full-model loading |

---

# 9. Adaptive Depth

## 9.1 Purpose

Adaptive depth menentukan berapa banyak iterasi komposisi dilakukan.

`[RT-DEPTH-1]` Depth MUST adaptif terhadap input. **(FAC-15)**

`[RT-DEPTH-2]` Depth MUST NOT fixed. **(FAC-15)**

## 9.2 Depth Control

```pseudo
function should_continue(state, config) -> bool:
    // Check minimum depth
    if state.steps_taken < config.min_depth:
        return true
    
    // Check maximum depth
    if state.steps_taken >= config.max_depth:
        return false
    
    // Check halt conditions
    if check_halt(state):
        return false
    
    // Check budget
    if budget_exhausted(state, config):
        return false
    
    return true
```

## 9.3 Depth Range

`[RT-DEPTH-3]` Depth range:

| Parameter | Default | Range |
|---|---|---|
| `min_depth` | 3 | 1–10 |
| `max_depth` | 25 | 10–100 |

`[RT-DEPTH-4]` `min_depth` MUST dipenuhi sebelum halt diizinkan.

`[RT-DEPTH-5]` `max_depth` MUST hard-enforced.

## 9.4 Difficulty-Based Depth

`[RT-DEPTH-6]` Depth target SHOULD berdasarkan difficulty:

```text
Easy input:    target_depth = min_depth + 2
Medium input:  target_depth = (min_depth + max_depth) / 2
Hard input:    target_depth = max_depth - 5
```

`[RT-DEPTH-7]` Difficulty estimation MUST lightweight (< 1% total budget).

## 9.5 Depth Invariants

| ID | Invariant |
|---|---|
| RT-DEPTH-INV-1 | Depth MUST adaptif |
| RT-DEPTH-INV-2 | Depth MUST NOT fixed |
| RT-DEPTH-INV-3 | min_depth MUST dipenuhi |
| RT-DEPTH-INV-4 | max_depth MUST hard-enforced |
| RT-DEPTH-INV-5 | Depth estimation MUST lightweight |

---

# 10. Adaptive Compute

## 10.1 Purpose

Adaptive compute mengalokasikan compute berdasarkan difficulty.

`[RT-AC-1]` Compute MUST adaptif terhadap difficulty. **(FAC-28)**

`[RT-AC-2]` Total knowledge dapat tumbuh tanpa menaikkan compute per token. **(FAC-31)**

## 10.2 Difficulty Estimation

```rust
struct DifficultyEstimator {
    // Lightweight classifier
    model: SmallModel,
    
    // Features
    features: Vec<FeatureExtractor>,
}

enum Difficulty {
    Easy,      // 0.0 - 0.3
    Medium,    // 0.3 - 0.7
    Hard,      // 0.7 - 1.0
}
```

`[RT-AC-3]` Difficulty estimator MUST lightweight.

`[RT-AC-4]` Difficulty estimation MUST menggunakan < 1% total budget.

## 10.3 Budget Allocation

```pseudo
function allocate_budget(input, config) -> ComputeBudget:
    difficulty = estimate_difficulty(input)
    
    match difficulty:
        case Easy:
            multiplier = 0.2
        case Medium:
            multiplier = 1.0
        case Hard:
            multiplier = 5.0
    
    return ComputeBudget {
        max_flops: config.max_flops * multiplier,
        max_bytes_moved: config.max_bytes_moved * multiplier,
        max_steps: config.max_depth,
        max_wall_time_us: config.max_wall_time_us * multiplier,
    }
```

`[RT-AC-5]` Budget multiplier:

| Difficulty | Multiplier |
|---|---|
| Easy | 0.2 |
| Medium | 1.0 |
| Hard | 5.0 |

## 10.4 Active Parameter Ratio

`[RT-AC-6]` Active parameter ratio MUST < 10%. **(FAC-30)**

```pseudo
function check_active_ratio(active_params, total_params):
    ratio = active_params / total_params
    assert ratio < 0.10, "Active parameter ratio exceeds 10%"
```

`[RT-AC-7]` Active parameter ratio MUST tracked sebagai metric.

## 10.5 Compute Scaling

`[RT-AC-8]` Compute per token MUST O(1) terhadap total knowledge.

`[RT-AC-9]` Menambah Cell baru MUST NOT menaikkan compute per token.

## 10.6 Adaptive Compute Invariants

| ID | Invariant |
|---|---|
| RT-AC-INV-1 | Compute MUST adaptif |
| RT-AC-INV-2 | Difficulty estimation MUST lightweight |
| RT-AC-INV-3 | Active ratio MUST < 10% |
| RT-AC-INV-4 | Compute MUST O(1) terhadap total knowledge |
| RT-AC-INV-5 | Budget multiplier MUST terdefinisi |

---

# 11. Halt Conditions

## 11.1 Purpose

Halt conditions menentukan kapan eksekusi berhenti.

`[RT-HALT-1]` Halt condition MUST dievaluasi setiap iterasi. **(FAC-15)**

## 11.2 Halt Condition Types

```rust
enum HaltCondition {
    // Confidence-based
    ConfidenceThreshold(f32),
    
    // Budget-based
    BudgetExhausted,
    
    // Convergence-based
    NoNewCellsSelected,
    StateConvergence(f32),
    
    // Depth-based
    MaxDepthReached,
    
    // Time-based
    Timeout(u64),
    
    // Explicit
    ExplicitHalt,
}
```

## 11.3 Halt Evaluation

```pseudo
function check_halt(state, config) -> bool:
    // Confidence threshold
    if state.confidence >= config.confidence_threshold:
        return true
    
    // Budget exhausted
    if budget_exhausted(state, config):
        return true
    
    // No new Cells selected
    if state.current_cells is empty:
        return true
    
    // Max depth
    if state.steps_taken >= config.max_depth:
        return true
    
    // Timeout
    if state.compute_used.wall_time_us >= config.max_wall_time_us:
        return true
    
    // Explicit halt signal
    if state.halt_signal:
        return true
    
    return false
```

## 11.4 Confidence Computation

`[RT-HALT-2]` Confidence computation:

```text
confidence = weighted_average(
    cell_confidences,
    composition_quality,
    convergence_score,
)
```

`[RT-HALT-3]` Default confidence threshold = 0.90.

## 11.5 Budget Exhaustion

`[RT-HALT-4]` Budget exhaustion check:

```pseudo
function budget_exhausted(state, config):
    if state.compute_used.flops >= budget.max_flops:
        return true
    if state.compute_used.bytes_moved >= budget.max_bytes_moved:
        return true
    return false
```

## 11.6 Halt Invariants

| ID | Invariant |
|---|---|
| RT-HALT-INV-1 | Halt MUST dievaluasi setiap iterasi |
| RT-HALT-INV-2 | Halt MUST deterministic |
| RT-HALT-INV-3 | min_depth MUST dipenuhi sebelum confidence halt |
| RT-HALT-INV-4 | max_depth MUST hard-enforced |
| RT-HALT-INV-5 | Budget exhaustion MUST menghentikan eksekusi |

---

# 12. Compute Budget

## 12.1 Purpose

Compute budget membatasi total compute per eksekusi.

`[RT-BUD-1]` Budget MUST hard-enforced. **(FAC-29)**

## 12.2 Budget Structure

```rust
struct ComputeBudget {
    max_flops: u64,
    max_bytes_moved: u64,
    max_steps: u32,
    max_wall_time_us: u64,
    max_cells_per_step: u32,
}
```

## 12.3 Budget Tracking

```rust
struct BudgetTracker {
    budget: ComputeBudget,
    used: ComputeUsage,
}

impl BudgetTracker {
    fn check(&self) -> BudgetStatus {
        if self.used.flops >= self.budget.max_flops {
            return BudgetStatus::Exhausted("flops");
        }
        if self.used.bytes_moved >= self.budget.max_bytes_moved {
            return BudgetStatus::Exhausted("bytes_moved");
        }
        if self.used.steps >= self.budget.max_steps {
            return BudgetStatus::Exhausted("steps");
        }
        if self.used.wall_time_us >= self.budget.max_wall_time_us {
            return BudgetStatus::Exhausted("wall_time");
        }
        BudgetStatus::Available(self.remaining())
    }
    
    fn record(&mut self, usage: ComputeUsage) {
        self.used.flops += usage.flops;
        self.used.bytes_moved += usage.bytes_moved;
        self.used.steps += 1;
        self.used.wall_time_us += usage.wall_time_us;
    }
}
```

## 12.4 Budget Enforcement

`[RT-BUD-2]` Budget check MUST dilakukan sebelum setiap operasi mahal.

```pseudo
function enforce_budget(tracker, operation_cost):
    status = tracker.check()
    
    match status:
        case Available(remaining):
            if operation_cost > remaining:
                return BudgetDecision::Reject
            else:
                return BudgetDecision::Allow
        
        case Exhausted(reason):
            return BudgetDecision::Halt(reason)
```

`[RT-BUD-3]` Budget enforcement MUST hard, bukan advisory.

`[RT-BUD-4]` Budget violation MUST menghasilkan error atau halt, bukan silent continue.

## 12.5 Budget Reporting

`[RT-BUD-5]` Budget usage MUST dilaporkan setelah eksekusi:

```json
{
  "budget": {
    "max_flops": 10000000000,
    "max_bytes_moved": 1073741824,
    "max_steps": 25
  },
  "used": {
    "flops": 2100000000,
    "bytes_moved": 251658240,
    "steps": 7
  },
  "utilization": {
    "flops": 0.21,
    "bytes_moved": 0.23,
    "steps": 0.28
  }
}
```

## 12.6 Budget Invariants

| ID | Invariant |
|---|---|
| RT-BUD-INV-1 | Budget MUST hard-enforced |
| RT-BUD-INV-2 | Budget check MUST sebelum operasi mahal |
| RT-BUD-INV-3 | Budget violation MUST explicit |
| RT-BUD-INV-4 | Budget usage MUST tracked |
| RT-BUD-INV-5 | Budget MUST dilaporkan |

---

# 13. Memory Budget

## 13.1 Purpose

Memory budget membatasi penggunaan memori.

`[RT-MEMB-1]` Working memory MUST bounded. **(FAC-19)**

`[RT-MEMB-2]` Memory budget MUST hard-enforced. **(FAC-29)**

## 13.2 Memory Budget Structure

```rust
struct MemoryBudget {
    gpu_bytes: u64,
    cpu_bytes: u64,
    working_memory_bytes: u64,
    reserved_gpu: u64,
    reserved_cpu: u64,
}
```

## 13.3 Memory Tracking

```rust
struct MemoryTracker {
    budget: MemoryBudget,
    gpu_used: u64,
    cpu_used: u64,
    working_used: u64,
}

impl MemoryTracker {
    fn can_admit(&self, size: u64, level: CacheLevel) -> bool {
        match level {
            CacheLevel::GPU => {
                self.gpu_used + size <= self.budget.gpu_bytes
            }
            CacheLevel::CPU => {
                self.cpu_used + size <= self.budget.cpu_bytes
            }
            CacheLevel::Working => {
                self.working_used + size <= self.budget.working_memory_bytes
            }
        }
    }
}
```

## 13.4 Admission Control

`[RT-MEMB-3]` Admission control MUST dilakukan sebelum load Tile baru.

```pseudo
function admit_tile(tile, level, tracker):
    if tracker.can_admit(tile.size, level):
        tracker.record(tile.size, level)
        return Admit
    else:
        // Try eviction
        evicted = evict_to_make_room(tile.size, level, tracker)
        if evicted:
            tracker.record(tile.size, level)
            return Admit
        else:
            return Reject
```

## 13.5 Context Memory

`[RT-MEMB-4]` Context memory MUST bounded. **(FAC-17)**

```rust
struct ContextBudget {
    max_entries: u64,
    max_bytes: u64,
}
```

`[RT-MEMB-5]` Default context budget:
- `max_entries` = 256
- `max_bytes` = 256 MiB

## 13.6 Memory Budget Invariants

| ID | Invariant |
|---|---|
| RT-MEMB-INV-1 | Working memory MUST bounded |
| RT-MEMB-INV-2 | Memory budget MUST hard-enforced |
| RT-MEMB-INV-3 | Admission control MUST sebelum load |
| RT-MEMB-INV-4 | Context memory MUST bounded |
| RT-MEMB-INV-5 | Context MUST NOT tumbuh linear |

---

# 14. Prefetch

## 14.1 Purpose

Prefetch memuat Tile sebelum dibutuhkan untuk overlap I/O dengan compute.

`[RT-PF-1]` Prefetch MUST berdasarkan execution dependency. **(FAC-26, FAC-27)**

## 14.2 Prefetch Policies

```rust
enum PrefetchPolicy {
    NextLayer,        // Prefetch layer berikutnya
    DependencyAware,  // Prefetch berdasarkan dependency graph
    MoETopK,          // Prefetch top-K experts
    Sequential,       // Prefetch sequential
    Adaptive,         // Adaptive berdasarkan runtime stats
}
```

`[RT-PF-2]` Default policy = `DependencyAware`.

## 14.3 Prefetch Algorithm

```pseudo
function prefetch(current_cells, state, config):
    // 1. Determine prefetch targets
    targets = match config.prefetch_policy:
        case NextLayer:
            get_next_layer_cells(current_cells)
        case DependencyAware:
            get_dependency_targets(current_cells)
        case MoETopK:
            get_moe_topk_targets(current_cells, state)
        case Sequential:
            get_sequential_targets(current_cells)
        case Adaptive:
            get_adaptive_targets(current_cells, state)
    
    // 2. Filter already cached
    targets = filter_cached(targets)
    
    // 3. Check budget
    targets = filter_by_budget(targets, remaining_budget)
    
    // 4. Limit by prefetch depth
    targets = targets[:config.prefetch_depth * avg_cells_per_layer]
    
    // 5. Submit async loads
    for target in targets:
        async_load(target)
```

## 14.4 Dependency-Aware Prefetch

`[RT-PF-3]` Dependency-aware prefetch menggunakan dependency graph:

```pseudo
function get_dependency_targets(current_cells):
    targets = []
    for cell in current_cells:
        // Get Cells that depend on current Cells
        dependents = get_dependents(cell)
        targets.extend(dependents)
        
        // Get Cells in next composition step
        next_step = get_next_composition_step(cell)
        targets.extend(next_step)
    
    return deduplicate(targets)
```

## 14.5 MoE Prefetch

`[RT-PF-4]` MoE prefetch berdasarkan router output:

```pseudo
function get_moe_topk_targets(current_cells, state):
    // Router output contains top-K expert IDs
    expert_ids = get_topk_experts(state)
    
    // Deduplicate
    expert_ids = deduplicate(expert_ids)
    
    // Resolve to Cells
    targets = []
    for expert_id in expert_ids:
        cells = resolve_expert_cells(expert_id)
        targets.extend(cells)
    
    return targets
```

## 14.6 Prefetch Depth Adaptation

`[RT-PF-5]` Prefetch depth SHOULD adaptif berdasarkan:

| Factor | Adaptation |
|---|---|
| Cache hit rate tinggi | Kurangi depth |
| Cache hit rate rendah | Naikkan depth |
| I/O latency tinggi | Naikkan depth |
| GPU utilization tinggi | Kurangi depth |
| Memory pressure tinggi | Kurangi depth |

```pseudo
function adapt_prefetch_depth(current_depth, stats):
    if stats.cache_hit_rate > 0.95:
        return max(0, current_depth - 1)
    elif stats.cache_hit_rate < 0.70:
        return min(config.max_prefetch_depth, current_depth + 1)
    elif stats.memory_pressure > 0.90:
        return max(0, current_depth - 1)
    else:
        return current_depth
```

## 14.7 Prefetch Invariants

| ID | Invariant |
|---|---|
| RT-PF-INV-1 | Prefetch MUST berdasarkan dependency |
| RT-PF-INV-2 | Prefetch MUST NOT melanggar budget |
| RT-PF-INV-3 | Prefetch MUST async |
| RT-PF-INV-4 | Prefetch MUST dapat dibatalkan |
| RT-PF-INV-5 | Prefetch depth SHOULD adaptif |

---

# 15. Eviction

## 15.1 Purpose

Eviction membebaskan cache untuk Tile baru.

`[RT-EV-1]` Eviction MUST berbasis byte capacity. **(FAC-19)**

## 15.2 Eviction Policies

```rust
enum EvictionPolicy {
    FIFO,
    LRU,
    LFU,
    LRUBypriority,   // Default
    ARC,             // Adaptive Replacement Cache
}
```

`[RT-EV-2]` Default policy = `LRUBypriority`.

## 15.3 Priority Classes

`[RT-EV-3]` Priority classes:

```text
Pinned > High > Normal > Low
```

| Priority | Eviction Order | Use Case |
|---|---|---|
| Pinned | Never evict while pinned | Active execution |
| High | Last | Hot Tiles, frequent access |
| Normal | Middle | Regular Tiles |
| Low | First | Cold Tiles, one-time access |

## 15.4 Eviction Algorithm

```pseudo
function evict(bytes_needed, level, cache):
    evicted_bytes = 0
    evicted_tiles = []
    
    // Iterate priority classes from low to high
    for priority in [Low, Normal, High]:
        if evicted_bytes >= bytes_needed:
            break
        
        // Get entries in this priority class, sorted by LRU
        entries = cache.get_entries_by_priority(priority)
        entries.sort_by_last_access_ascending()
        
        for entry in entries:
            if evicted_bytes >= bytes_needed:
                break
            
            // Skip pinned
            if entry.priority == Pinned:
                continue
            
            // Evict
            cache.remove(entry.tile_id)
            evicted_bytes += entry.size_bytes
            evicted_tiles.append(entry.tile_id)
    
    return evicted_tiles
```

## 15.5 Eviction Rules

`[RT-EV-4]` Tile `Pinned` MUST NOT dievict selama pin aktif.

`[RT-EV-5]` Eviction MUST async untuk menghindari blocking.

`[RT-EV-6]` Eviction MUST mencatat metrics.

## 15.6 Write-Back

`[RT-EV-7]` Tile immutable → tidak ada write-back.

`[RT-EV-8]` Eviction hanya menghapus dari cache, tidak dari storage.

## 15.7 Eviction Invariants

| ID | Invariant |
|---|---|
| RT-EV-INV-1 | Eviction MUST berbasis byte capacity |
| RT-EV-INV-2 | Eviction MUST menghormati priority |
| RT-EV-INV-3 | Pinned Tiles MUST NOT dievict |
| RT-EV-INV-4 | Eviction MUST async |
| RT-EV-INV-5 | Eviction MUST dicatat |

---

# 16. Representation Selection

## 16.1 Purpose

Representation selection memilih bentuk Tile yang sesuai untuk hardware dan workload.

`[RT-REP-1]` Representation selection MUST berdasarkan hardware dan workload. **(FAC-26, FAC-30)**

## 16.2 Selection Algorithm

```pseudo
function select_representation(tile, hardware, workload, config) -> RepresentationId:
    // Get available representations
    available = tile.representations
    
    // Filter by hardware support
    supported = filter_by_hardware(available, hardware)
    
    // Filter by accuracy policy
    accurate = filter_by_accuracy(supported, config.accuracy_policy)
    
    // Score each representation
    scored = []
    for repr in accurate:
        score = score_representation(repr, hardware, workload)
        scored.append((repr, score))
    
    // Select best
    scored.sort_by_score_desc()
    return scored[0].id
```

## 16.3 Scoring

`[RT-REP-2]` Representation scoring:

```text
score(repr) = accuracy_score(repr)
            × memory_efficiency(repr)
            × compute_efficiency(repr)
            × hardware_compatibility(repr)
```

## 16.4 Default Selection Rules

`[RT-REP-3]` Default selection rules:

```text
if hardware.gpu_fp8_supported
   and tile.has_representation("fp8_e4m3")
   and workload.accuracy_policy != STRICT:
    choose fp8_e4m3

elif hardware.gpu
   and tile.has_representation("fp16")
   and vram_budget_ok:
    choose fp16

elif hardware.cpu
   and tile.has_representation("int8")
   and workload.accuracy_policy != STRICT:
    choose int8

else:
    choose canonical representation
```

## 16.5 Accuracy Policies

```rust
enum AccuracyPolicy {
    Strict,      // Always highest accuracy
    Balanced,    // Balance accuracy and efficiency
    Fast,        // Prefer speed/efficiency
    Custom(f32), // Custom accuracy threshold
}
```

`[RT-REP-4]` Accuracy policy mapping:

| Policy | Allowed Representations |
|---|---|
| Strict | canonical, bf16, fp32 |
| Balanced | bf16, fp16, fp8, int8 |
| Fast | fp8, int8, int4 |

## 16.6 Representation Invariants

| ID | Invariant |
|---|---|
| RT-REP-INV-1 | Selection MUST berdasarkan hardware |
| RT-REP-INV-2 | Selection MUST berdasarkan workload |
| RT-REP-INV-3 | Canonical representation MUST selalu tersedia |
| RT-REP-INV-4 | Selection MUST deterministic |
| RT-REP-INV-5 | Selection MUST menghormati accuracy policy |

---

# 17. GPU/CPU/NVMe Scheduling

## 17.1 Purpose

Scheduling mengatur eksekusi dan data movement antar device.

`[RT-SCHED-1]` Scheduling MUST memungkinkan overlap I/O dengan compute. **(FAC-26)**

## 17.2 Device Roles

| Device | Role |
|---|---|
| GPU | Cell execution, active Tiles |
| CPU | Hot cache, routing, composition |
| NVMe | Canonical storage, staging |
| Network | Remote Tiles (optional) |

## 17.3 Scheduling Queues

```text
┌─────────────────────────────────────────────────┐
│              SCHEDULING QUEUES                   │
├─────────────────────────────────────────────────┤
│                                                 │
│  Compute Queue (GPU)                            │
│    └── Cell execution                           │
│                                                 │
│  I/O Queue (NVMe → CPU)                         │
│    └── Tile loading                             │
│                                                 │
│  Transfer Queue (CPU → GPU)                     │
│    └── H2D transfer                             │
│                                                 │
│  Decompression Queue (CPU)                      │
│    └── Tile decompression                       │
│                                                 │
│  Prefetch Queue                                 │
│    └── Async prefetch                           │
│                                                 │
└─────────────────────────────────────────────────┘
```

## 17.4 Overlap Strategy

`[RT-SCHED-2]` Overlap strategy:

```text
Time ──────────────────────────────────────────►

GPU Compute: [Cells N]──────────[Cells N+1]──────
H2D Transfer: ────[Tiles N+1]────────[Tiles N+2]
Decompression: ────────[Tiles N+1]──────────[Tiles N+2]
I/O Load:    ────────────[Tiles N+1]──────────────
Prefetch:    ────[Tiles N+2]────────[Tiles N+3]
```

`[RT-SCHED-3]` Semua queues MUST dapat berjalan concurrent.

## 17.5 Device Placement

`[RT-SCHED-4]` Device placement decision:

```pseudo
function place_tile(tile, hardware, budget) -> Device:
    // Check if needed for immediate execution
    if tile.needed_immediately:
        if budget.gpu_available(tile.size):
            return Device::GPU
        else:
            return Device::CPU
    
    // Hot Tile
    if tile.access_count > hot_threshold:
        if budget.cpu_available(tile.size):
            return Device::CPU
    
    // Default: NVMe (canonical storage)
    return Device::NVMe
```

## 17.6 Scheduling Invariants

| ID | Invariant |
|---|---|
| RT-SCHED-INV-1 | Scheduling MUST memungkinkan overlap |
| RT-SCHED-INV-2 | Queues MUST bounded |
| RT-SCHED-INV-3 | Backpressure MUST diterapkan |
| RT-SCHED-INV-4 | Device placement MUST menghormati budget |
| RT-SCHED-INV-5 | Scheduling MUST deterministic dalam mode deterministic |

---

# 18. Deterministic Execution

## 18.1 Purpose

Deterministic execution menjamin hasil yang sama untuk input dan state yang sama.

`[RT-DET-1]` Eksekusi MUST deterministic untuk input dan state sama. **(FAC-34)**

## 18.2 Sources of Non-Determinism

| Source | Mitigation |
|---|---|
| Floating-point order | Deterministic kernels |
| Parallel execution order | Deterministic scheduling |
| Random number generation | Seeded RNG |
| Memory allocation order | Deterministic allocator |
| Hash map iteration | Sorted iteration |
| Cache state | Cache state part of execution state |

## 18.3 Deterministic Rules

`[RT-DET-2]` Aturan deterministic:

1. **Cell selection**: jika ada tie dalam score, pilih berdasarkan CellId ascending.
2. **Parallel execution**: hasil dikumpulkan dalam urutan CellId ascending.
3. **Composition**: operasi komposisi dilakukan dalam urutan deterministik.
4. **RNG**: semua RNG menggunakan seed eksplisit.
5. **Floating-point**: gunakan deterministic reduction order.

## 18.4 Seeded RNG

```rust
struct RngState {
    seed: u64,
    counter: u64,
}

impl RngState {
    fn next(&mut self) -> u64 {
        self.counter += 1;
        deterministic_hash(self.seed, self.counter)
    }
}
```

`[RT-DET-3]` RNG MUST menggunakan deterministic algorithm.

`[RT-DET-4]` Seed MUST dapat dikonfigurasi.

## 18.5 Deterministic Mode

`[RT-DET-5]` Deterministic mode MUST default aktif.

`[RT-DET-6]` Non-deterministic mode MAY diaktifkan untuk performa, tetapi MUST ditandai eksplisit.

## 18.6 Reproducibility

`[RT-DET-7]` Eksekusi dapat direproduksi jika:

1. Input sama
2. Model state (revision) sama
3. Configuration sama
4. Seed sama
5. Hardware floating-point behavior sama

## 18.7 Deterministic Invariants

| ID | Invariant |
|---|---|
| RT-DET-INV-1 | Eksekusi MUST deterministic |
| RT-DET-INV-2 | Tie-breaking MUST deterministic |
| RT-DET-INV-3 | RNG MUST seeded |
| RT-DET-INV-4 | Deterministic mode MUST default |
| RT-DET-INV-5 | Non-determinism MUST eksplisit ditandai |

---

# 19. Execution State Machine

## 19.1 Global State Machine

```text
┌──────────┐   start    ┌──────────┐
│  IDLE    │───────────►│ ENCODING │
└──────────┘            └────┬─────┘
                             │ encoded
                             ▼
┌──────────┐   halt     ┌──────────┐
│ DECODING │◄───────────│ EXECUTING│
└────┬─────┘            └────┬─────┘
     │                       │
     │ decoded               │ loop
     ▼                       ▼
┌──────────┐            ┌──────────┐
│  DONE    │            │ ROUTING  │
└──────────┘            └────┬─────┘
                             │ selected
                             ▼
                        ┌──────────┐
                        │ LOADING  │
                        └────┬─────┘
                             │ loaded
                             ▼
                        ┌──────────┐
                        │COMPOSING │
                        └────┬─────┘
                             │ composed
                             │
                             └──► back to EXECUTING or DECODING
```

## 19.2 State Transitions

| From | To | Trigger |
|---|---|---|
| IDLE | ENCODING | execute() called |
| ENCODING | EXECUTING | Encoding complete |
| EXECUTING | ROUTING | Start iteration |
| ROUTING | LOADING | Cells selected |
| LOADING | COMPOSING | Tiles loaded |
| COMPOSING | EXECUTING | Not halt |
| COMPOSING | DECODING | Halt |
| DECODING | DONE | Output ready |

## 19.3 Error States

```text
Any state + error → ERROR
ERROR + recoverable → retry
ERROR + fatal → FAILED
```

---

# 20. Error Handling

## 20.1 Runtime Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-RT-BUDGET` | Budget exceeded |
| `CNWS-E-RT-NOTFOUND` | Cell/Tile not found |
| `CNWS-E-RT-CORRUPT` | Tile corruption detected |
| `CNWS-E-RT-INCOMPATIBLE` | Cell incompatible |
| `CNWS-E-RT-TIMEOUT` | Execution timeout |
| `CNWS-E-RT-HALT` | Halt condition error |
| `CNWS-E-RT-MEMORY` | Memory budget exceeded |
| `CNWS-E-RT-LOAD` | Tile loading failed |
| `CNWS-E-RT-REPRESENTATION` | No suitable representation |

## 20.2 Error Severity

| Severity | Examples | Action |
|---|---|---|
| Recoverable | RT-TIMEOUT, RT-BUDGET | Return partial result |
| Retryable | RT-LOAD, RT-NOTFOUND | Retry with backoff |
| Fatal | RT-CORRUPT, RT-INCOMPATIBLE | Abort execution |

---

# 21. Traceability to FAC

## 21.1 Complete Traceability

| FAC | Requirement | Runtime Implementation |
|---|---|---|
| FAC-13 | Cell selection content-based | §4 Query Derivation, §5 Cell Selection, §6 Routing |
| FAC-14 | Computation dynamically composed | §7 Composition, §8 Execution Planning |
| FAC-15 | No fixed-depth layer stack | §9 Adaptive Depth, §11 Halt Conditions |
| FAC-16 | Context content-addressed memory | §7.4 Context Update, §13.5 Context Memory |
| FAC-17 | Context not linear growth | §7.4, §13.5 |
| FAC-18 | Memory first-class persistent | §13 Memory Budget |
| FAC-19 | Working memory bounded | §13 Memory Budget |
| FAC-26 | No full-model loading | §8 Execution Planning, §14 Prefetch, §17 Scheduling |
| FAC-27 | Only relevant Cells activated | §5 Cell Selection, §6 Routing |
| FAC-28 | Compute adaptive to difficulty | §10 Adaptive Compute |
| FAC-29 | Budget hard-enforced | §12 Compute Budget, §13 Memory Budget |
| FAC-30 | Active parameter ratio < 10% | §10.4 Active Parameter Ratio |
| FAC-31 | Knowledge growth without compute growth | §10.5 Compute Scaling |
| FAC-34 | Deterministic execution | §18 Deterministic Execution |

---

# 22. Final Runtime Contract

## 22.1 Ringkasan Keputusan Runtime

| ID | Keputusan |
|---|---|
| RT-F01 | Query derivation menggunakan projection dari WorkingState. |
| RT-F02 | Cell selection menggunakan ANN search dengan threshold. |
| RT-F03 | Default selection k = 16, threshold = 0.3. |
| RT-F04 | Routing statistics di-update setelah eksekusi. |
| RT-F05 | Composition menggunakan mode sequential/parallel/conditional/iterative. |
| RT-F06 | Context update content-addressed, bukan KV-cache. |
| RT-F07 | Execution plan dynamic berdasarkan dependency. |
| RT-F08 | Adaptive depth dengan min_depth = 3, max_depth = 25. |
| RT-F09 | Difficulty estimator lightweight. |
| RT-F10 | Budget multiplier: Easy=0.2, Medium=1.0, Hard=5.0. |
| RT-F11 | Active parameter ratio < 10%. |
| RT-F12 | Budget enforcement hard. |
| RT-F13 | Context budget: max_entries=256, max_bytes=256 MiB. |
| RT-F14 | Prefetch policy default DependencyAware, depth=2. |
| RT-F15 | Eviction policy default LRUBypriority. |
| RT-F16 | Priority classes: Pinned > High > Normal > Low. |
| RT-F17 | Representation selection berdasarkan hardware + accuracy. |
| RT-F18 | Scheduling memungkinkan overlap I/O dan compute. |
| RT-F19 | Deterministic mode default aktif. |
| RT-F20 | Execution state machine: IDLE → ENCODING → EXECUTING → DECODING → DONE. |

## 22.2 Runtime Invariants

| ID | Invariant |
|---|---|
| RT-INV-1 | Execution MUST iteratif, bukan fixed-depth. |
| RT-INV-2 | Cell selection MUST content-based. |
| RT-INV-3 | Composition MUST dynamic. |
| RT-INV-4 | Context MUST NOT tumbuh linear. |
| RT-INV-5 | Budget MUST hard-enforced. |
| RT-INV-6 | Working memory MUST bounded. |
| RT-INV-7 | Prefetch MUST berdasarkan dependency. |
| RT-INV-8 | Eviction MUST berbasis byte capacity. |
| RT-INV-9 | Representation selection MUST deterministic. |
| RT-INV-10 | Execution MUST deterministic dalam mode deterministic. |
| RT-INV-11 | Active parameter ratio MUST < 10%. |
| RT-INV-12 | Compute per token MUST O(1) terhadap total knowledge. |
| RT-INV-13 | Halt condition MUST dievaluasi setiap iterasi. |
| RT-INV-14 | Routing statistics MUST di-update. |
| RT-INV-15 | Error handling MUST explicit. |

## 22.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi perilaku runtime final dan mengikat** untuk CNWS Lattice Runtime. Ia menerjemahkan FAC-13 sampai FAC-31 menjadi algoritma, parameter, dan invariant yang dapat diimplementasikan dan diuji.

Seluruh implementasi CNWS Lattice Runtime MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan runtime yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN RUNTIME & EXECUTION SPECIFICATION**
