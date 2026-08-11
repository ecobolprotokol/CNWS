# CNWS
## Memory & Retrieval Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Memory & Retrieval Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (MEMORY SYSTEM SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS Cell & Schema Spec; CNWS Runtime Spec |
| Hulu ke | Implementasi Memory System, Retrieval Engine, Consolidation Engine |
| Otoritas | Spesifikasi tunggal untuk seluruh LATTICE memory CNWS |
| Prinsip Dijaga | Memory **bukan sekadar storage Cell biasa**; ia adalah first-class persistent intelligence state |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    Cell & Schema Spec    Memory & Retrieval Spec    Implementation
─────────────────────   ──────────────────    ────────────────────────   ─────────────
Memory first-class    ──► Memory CellTypes ──► Memory architecture     ──► Memory System
WorkingState bounded    Memory metadata        Retrieval algorithms        Retrieval Engine
Context O(1)            Index vector           Consolidation               Consolidation
No KV-cache             Associations           Forgetting/retention        Retention Policy
```

`[MEM-DOC-1]` Dokumen ini mendefinisikan **LATTICE memory system** secara lengkap.

`[MEM-DOC-2]` Memory **bukan sekadar storage Cell biasa**. Memory memiliki:
- Struktur key-value dengan associations
- Content-addressed retrieval mechanism
- Consolidation (kompilasi memory yang sering diakses)
- Forgetting/retention policy
- Working memory yang bounded
- Interaksi langsung dengan WorkingState

`[MEM-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[MEM-DOC-4]` Jika terjadi konflik dengan Cell & Schema Spec untuk hal struktur Cell, Cell & Schema Spec menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-MEM-01 | Memory adalah first-class persistent state, bukan cache. |
| DF-MEM-02 | Memory types: Episodic, Semantic, Procedural, Working, Consolidated. |
| DF-MEM-03 | Indexing menggunakan HNSW dengan dimensions = 512. |
| DF-MEM-04 | Similarity metric default = Cosine. |
| DF-MEM-05 | Retrieval top-k default k = 16, threshold = 0.3. |
| DF-MEM-06 | Working memory bound default = 256 MiB. |
| DF-MEM-07 | Consolidation triggers: access_count, time, importance. |
| DF-MEM-08 | Forgetting menggunakan decay + importance scoring. |
| DF-MEM-09 | Consolidated memory MUST NOT di-forget. |
| DF-MEM-10 | Context MUST ditangani melalui memory, bukan KV-cache. |
| DF-MEM-11 | Retrieval complexity MUST O(log N) untuk ANN search. |
| DF-MEM-12 | Memory lifecycle: CREATED → ACTIVE → CONSOLIDATED → ARCHIVED/FORGOTTEN. |

---

# 1. Executive Summary

## 1.1 Memory Bukan Storage Cell Biasa

`[MEM-EXEC-1]` Memory CNWS **bukan** sekadar storage Cell biasa. Perbedaan fundamental:

| Aspek | Storage Cell Biasa | Memory |
|---|---|---|
| Purpose | Store data | Store learned information |
| Access pattern | Explicit request | Content-addressed retrieval |
| Associations | Dependencies (static) | Learned associations (dynamic) |
| Consolidation | Tidak ada | Compile frequent patterns |
| Forgetting | GC reachability | Decay + importance scoring |
| Working interface | Load on demand | WorkingState integration |
| Lifecycle | Immutable | Consolidation, forgetting |

`[MEM-EXEC-2]` Memory adalah **first-class persistent intelligence state**: ia adalah bagian dari model yang belajar, bukan sekadar tempat menyimpan data.

## 1.2 Tujuan Memory System

`[MEM-EXEC-3]` Memory System MUST mendukung:

1. **Persistent context**: context tidak hilang antar sesi
2. **Content-addressed retrieval**: retrieve by content, bukan position
3. **Bounded working memory**: active context terbatas
4. **Consolidation**: kompilasi memory yang sering diakses
5. **Forgetting**: melepas memory yang tidak relevan
6. **Association traversal**: navigate learned relationships

## 1.3 Memory Types Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    LATTICE MEMORY                            │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Working Memory (bounded, ephemeral)                  │   │
│  │   - Active computation context                       │   │
│  │   - NOT persisted                                    │   │
│  └────────────────────────┬────────────────────────────┘   │
│                           │ read/write                      │
│                           ▼                                 │
│  ┌─────────────────────────────────────────────────────┐   │
│  │ Persistent Memory                                    │   │
│  │                                                      │   │
│  │   ┌──────────────┐  ┌──────────────┐               │   │
│  │   │ Episodic     │  │ Semantic     │               │   │
│  │   │ (experiences)│  │ (facts)      │               │   │
│  │   └──────┬───────┘  └──────┬───────┘               │   │
│  │          │                  │                        │   │
│  │          │   consolidation  │                        │   │
│  │          └──────────────────┘                        │   │
│  │                     │                                │   │
│  │                     ▼                                │   │
│  │          ┌──────────────────┐                        │   │
│  │          │ Consolidated     │                        │   │
│  │          │ (compiled)       │                        │   │
│  │          └──────────────────┘                        │   │
│  │                                                      │   │
│  │   ┌──────────────┐                                  │   │
│  │   │ Procedural   │                                  │   │
│  │   │ (patterns)   │                                  │   │
│  │   └──────────────┘                                  │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

# 2. Memory Architecture Overview

## 2.1 Memory Hierarchy

`[MEM-ARCH-1]` Memory hierarchy:

| Level | Name | Persistence | Bound | Latency |
|---|---|---|---|---|
| L0 | Working Memory | Ephemeral | 256 MiB | < 1 μs |
| L1 | Hot Memory | Persistent | 4 GiB | < 10 μs |
| L2 | Warm Memory | Persistent | 1 TiB | < 1 ms |
| L3 | Cold Memory | Persistent | Unbounded | < 100 ms |

## 2.2 Memory Cell Structure

`[MEM-ARCH-2]` Memory Cell menggunakan CellType range `0x20–0x2F` (dari Cell & Schema Spec):

```rust
struct MemoryCell {
    // Common Cell fields
    id: Blake3Hash,              // BLAKE3-256 of key+value
    cell_type: CellType,         // 0x20-0x2F
    version: CellVersion,
    
    // Memory-specific
    memory_type: MemoryType,
    key_vector: Vec<f32>,        // retrieval key
    value_payload: Vec<u8>,      // stored content
    consolidation_level: u8,     // 0=raw, 1=consolidated, 2=compiled
    
    // Associations
    associations: Vec<MemoryAssociation>,
    
    // Access statistics live in the mutable MemoryIndex, not in the immutable entry.
    created_at_ns: u64,
    importance_score: f32,
    
    // Metadata
    metadata: CellMetadata,
}

enum MemoryType {
    Episodic,       // 0x20
    Semantic,       // 0x21
    Procedural,     // 0x22
    Working,        // 0x23
    Consolidated,   // 0x24
    Association,    // 0x25
}

struct MemoryAssociation {
    target: Blake3Hash,          // target memory ID
    strength: f32,               // 0.0-1.0
    association_type: AssociationType,
}

enum AssociationType {
    Temporal,       // happened together
    Semantic,       // semantically related
    Causal,         // cause-effect
    Procedural,     // part of same procedure
}
```

## 2.3 Memory Identity

`[MEM-ARCH-3]` Memory identity:

```text
memory_id = BLAKE3-256(key_vector_bytes || value_payload_bytes)
```

`[MEM-ARCH-4]` Memory identity MUST independen dari:
- Storage location
- Access statistics
- Associations
- Consolidation level

## 2.4 Memory Architecture Invariants

| ID | Invariant |
|---|---|
| MEM-ARCH-INV-1 | Memory MUST first-class persistent state |
| MEM-ARCH-INV-2 | Memory MUST content-addressed |
| MEM-ARCH-INV-3 | Memory MUST memiliki associations |
| MEM-ARCH-INV-4 | Memory MUST memiliki access statistics |
| MEM-ARCH-INV-5 | Memory MUST memiliki importance score |

---

# 3. Episodic Memory

## 3.1 Definition

`[MEM-EPI-1]` Episodic Memory menyimpan **specific experiences and sequences**.

`[MEM-EPI-2]` Episodic Memory digunakan untuk:
- Context dari percakapan/interaksi
- Sequence of events
- Specific instances
- Temporal relationships

## 3.2 Episodic Memory Structure

```rust
struct EpisodicMemory {
    // MemoryCell fields
    id: Blake3Hash,
    memory_type: MemoryType::Episodic,
    
    // Episodic-specific
    episode_id: u64,
    sequence_position: u64,
    temporal_context: TemporalContext,
    
    // Content
    key_vector: Vec<f32>,
    value_payload: Vec<u8>,
    
    // Associations
    associations: Vec<MemoryAssociation>,
}

struct TemporalContext {
    started_at_ns: u64,
    ended_at_ns: u64,
    duration_ns: u64,
    session_id: Option<u64>,
}
```

## 3.3 Episodic Memory Operations

### 3.3.1 Write

```pseudo
function episodic_write(content: Vec<u8>, context: TemporalContext) -> MemoryId:
    // 1. Encode content to key vector
    key_vector = encode_to_vector(content)
    
    // 2. Create memory entry
    entry = MemoryCell {
        id: compute_memory_id(key_vector, content),
        memory_type: Episodic,
        key_vector: key_vector,
        value_payload: content,
        temporal_context: context,
        created_at_ns: now(),
        access_count: 0,
        importance_score: default_importance,
    }
    
    // 3. Store
    store_memory(entry)
    
    // 4. Index
    index_memory(entry)
    
    return entry.id
```

### 3.3.2 Retrieve

```pseudo
function episodic_retrieve(query: Vec<f32>, k: usize, time_range: Option<TimeRange>) -> Vec<MemoryEntry>:
    // 1. ANN search
    candidates = ann_search(query, k * 2)
    
    // 2. Filter by time range
    if time_range is not None:
        candidates = filter_by_time(candidates, time_range)
    
    // 3. Score by relevance and recency
    scored = []
    for candidate in candidates:
        score = similarity(query, candidate.key_vector)
              × recency_boost(candidate)
              × importance(candidate)
        scored.append((candidate, score))
    
    // 4. Sort and take top-k
    scored.sort_by_score_desc()
    return scored[:k]
```

## 3.4 Episodic Memory Characteristics

| Characteristic | Value |
|---|---|
| Persistence | Persistent |
| Default retention | 30 days (configurable) |
| Consolidation target | Semantic Memory |
| Access pattern | Recent-biased |
| Forgetting | Decay + consolidation |

## 3.5 Episodic Memory Invariants

| ID | Invariant |
|---|---|
| MEM-EPI-INV-1 | Episodic Memory MUST memiliki temporal context |
| MEM-EPI-INV-2 | Episodic Memory MUST indexed untuk retrieval |
| MEM-EPI-INV-3 | Episodic Memory SHOULD di-consolidate ke Semantic |
| MEM-EPI-INV-4 | Episodic Memory MAY di-forget setelah retention period |

---

# 4. Semantic Memory

## 4.1 Definition

`[MEM-SEM-1]` Semantic Memory menyimpan **factual knowledge and associations**.

`[MEM-SEM-2]` Semantic Memory digunakan untuk:
- Facts and concepts
- Learned associations
- Generalized knowledge
- Long-term knowledge

## 4.2 Semantic Memory Structure

```rust
struct SemanticMemory {
    // MemoryCell fields
    id: Blake3Hash,
    memory_type: MemoryType::Semantic,
    
    // Semantic-specific
    concept_id: Option<String>,
    domain_tags: Vec<String>,
    
    // Content
    key_vector: Vec<f32>,
    value_payload: Vec<u8>,
    
    // Associations (rich graph)
    associations: Vec<MemoryAssociation>,
    
    // Provenance
    derived_from: Vec<Blake3Hash>,  // source episodic memories
}
```

## 4.3 Semantic Memory Operations

### 4.3.1 Write

```pseudo
function semantic_write(fact: Vec<u8>, domain: String, associations: Vec<MemoryAssociation>) -> MemoryId:
    // 1. Encode to key vector
    key_vector = encode_to_vector(fact)
    
    // 2. Create memory entry
    entry = MemoryCell {
        id: compute_memory_id(key_vector, fact),
        memory_type: Semantic,
        key_vector: key_vector,
        value_payload: fact,
        domain_tags: [domain],
        associations: associations,
        importance_score: high_importance,
    }
    
    // 3. Store and index
    store_memory(entry)
    index_memory(entry)
    
    return entry.id
```

### 4.3.2 Retrieve

```pseudo
function semantic_retrieve(query: Vec<f32>, k: usize, domain: Option<String>) -> Vec<MemoryEntry>:
    // 1. ANN search
    candidates = ann_search(query, k * 2)
    
    // 2. Filter by domain
    if domain is not None:
        candidates = filter_by_domain(candidates, domain)
    
    // 3. Score by relevance and importance
    scored = []
    for candidate in candidates:
        score = similarity(query, candidate.key_vector)
              × importance(candidate)
              × confidence(candidate)
        scored.append((candidate, score))
    
    // 4. Sort and take top-k
    scored.sort_by_score_desc()
    return scored[:k]
```

### 4.3.3 Association Traversal

```pseudo
function traverse_associations(start: MemoryId, depth: usize, min_strength: f32) -> Vec<MemoryEntry>:
    visited = set()
    result = []
    queue = [(start, 0)]
    
    while queue is not empty:
        (current, current_depth) = queue.pop()
        
        if current in visited:
            continue
        visited.add(current)
        
        if current_depth > depth:
            continue
        
        memory = load_memory(current)
        result.append(memory)
        
        // Traverse associations
        for assoc in memory.associations:
            if assoc.strength >= min_strength:
                queue.append((assoc.target, current_depth + 1))
    
    return result
```

## 4.4 Semantic Memory Characteristics

| Characteristic | Value |
|---|---|
| Persistence | Persistent (long-term) |
| Default retention | Indefinite |
| Consolidation source | Episodic Memory |
| Access pattern | Content-based |
| Forgetting | Rarely (only low importance) |

## 4.5 Semantic Memory Invariants

| ID | Invariant |
|---|---|
| MEM-SEM-INV-1 | Semantic Memory MUST memiliki associations |
| MEM-SEM-INV-2 | Semantic Memory MUST indexed untuk retrieval |
| MEM-SEM-INV-3 | Semantic Memory SHOULD NOT di-forget kecuali low importance |
| MEM-SEM-INV-4 | Semantic Memory MUST track provenance |

---

# 5. Procedural Memory

## 5.1 Definition

`[MEM-PROC-1]` Procedural Memory menyimpan **learned composition patterns and execution strategies**.

`[MEM-PROC-2]` Procedural Memory digunakan untuk:
- Learned composition patterns
- Execution strategies
- Routing patterns
- "How to do things"

## 5.2 Procedural Memory Structure

```rust
struct ProceduralMemory {
    // MemoryCell fields
    id: Blake3Hash,
    memory_type: MemoryType::Procedural,
    
    // Procedural-specific
    pattern_type: PatternType,
    cell_sequence: Vec<Blake3Hash>,  // Cells in the pattern
    execution_mode: ExecutionMode,
    
    // Content
    key_vector: Vec<f32>,
    value_payload: Vec<u8>,
    
    // Statistics
    execution_count: u64,
    avg_execution_us: u64,
    success_rate: f32,
}

enum PatternType {
    CompositionPattern,
    ExecutionStrategy,
    RoutingPattern,
    RetrievalStrategy,
}
```

## 5.3 Procedural Memory Operations

### 5.3.1 Write

```pseudo
function procedural_write(pattern: CompositionPattern, stats: ExecutionStats) -> MemoryId:
    // 1. Encode pattern to key vector
    key_vector = encode_pattern_to_vector(pattern)
    
    // 2. Create memory entry
    entry = MemoryCell {
        id: compute_memory_id(key_vector, serialize(pattern)),
        memory_type: Procedural,
        pattern_type: PatternType::CompositionPattern,
        cell_sequence: pattern.cell_ids,
        execution_mode: pattern.mode,
        key_vector: key_vector,
        value_payload: serialize(pattern),
        execution_count: stats.count,
        avg_execution_us: stats.avg_us,
        success_rate: stats.success_rate,
    }
    
    // 3. Store and index
    store_memory(entry)
    index_memory(entry)
    
    return entry.id
```

### 5.3.2 Retrieve

```pseudo
function procedural_retrieve(query: Vec<f32>, k: usize) -> Vec<MemoryEntry>:
    // 1. ANN search
    candidates = ann_search(query, k)
    
    // 2. Score by relevance and success rate
    scored = []
    for candidate in candidates:
        score = similarity(query, candidate.key_vector)
              × candidate.success_rate
              × frequency_boost(candidate)
        scored.append((candidate, score))
    
    // 3. Sort and take top-k
    scored.sort_by_score_desc()
    return scored[:k]
```

## 5.4 Procedural Memory Characteristics

| Characteristic | Value |
|---|---|
| Persistence | Persistent |
| Default retention | Indefinite |
| Consolidation source | Frequent compositions |
| Access pattern | Execution-triggered |
| Forgetting | Only if success rate < threshold |

## 5.5 Procedural Memory Invariants

| ID | Invariant |
|---|---|
| MEM-PROC-INV-1 | Procedural Memory MUST memiliki cell_sequence |
| MEM-PROC-INV-2 | Procedural Memory MUST track execution statistics |
| MEM-PROC-INV-3 | Procedural Memory MUST indexed untuk retrieval |
| MEM-PROC-INV-4 | Procedural Memory dengan low success rate MAY di-forget |

---

# 6. Working Memory

## 6.1 Definition

`[MEM-WORK-1]` Working Memory adalah **active computation context** yang bounded.

`[MEM-WORK-2]` Working Memory **bukan** persistent; ia adalah interface antara WorkingState dan persistent memory.

## 6.2 Working Memory Structure

```rust
struct WorkingMemory {
    // Bounded entries
    entries: Vec<WorkingMemoryEntry>,
    
    // Budget
    max_entries: usize,
    max_bytes: u64,
    used_bytes: u64,
}

struct WorkingMemoryEntry {
    memory_ref: Blake3Hash,      // reference to persistent memory
    key_vector: Vec<f32>,        // cached key
    value_summary: Vec<u8>,      // cached value (compressed)
    loaded_at_ns: u64,
    access_count_in_session: u64,
}
```

## 6.3 Working Memory Operations

### 6.3.1 Load from Persistent

```pseudo
function working_load(memory_id: Blake3Hash) -> Result<WorkingMemoryEntry>:
    // 1. Check if already in working memory
    if let Some(entry) = working_memory.get(memory_id):
        entry.access_count_in_session += 1
        return Ok(entry)
    
    // 2. Check budget
    memory = load_memory(memory_id)
    if not working_memory.can_admit(memory.size):
        // Evict LRU entry
        working_memory.evict_lru()
    
    // 3. Load into working memory
    entry = WorkingMemoryEntry {
        memory_ref: memory_id,
        key_vector: memory.key_vector,
        value_summary: compress(memory.value_payload),
        loaded_at_ns: now(),
        access_count_in_session: 1,
    }
    
    working_memory.add(entry)
    
    return Ok(entry)
```

### 6.3.2 Write to Persistent

```pseudo
function working_write(key: Vec<f32>, value: Vec<u8>, mem_type: MemoryType) -> MemoryId:
    // 1. Create persistent memory entry
    memory_id = persistent_write(key, value, mem_type)
    
    // 2. Add to working memory
    working_load(memory_id)
    
    return memory_id
```

## 6.4 Working Memory Characteristics

| Characteristic | Value |
|---|---|
| Persistence | Ephemeral (NOT persisted) |
| Default bound | 256 MiB |
| Max entries | 256 |
| Eviction | LRU |
| Interface | WorkingState |

## 6.5 Working Memory Invariants

| ID | Invariant |
|---|---|
| MEM-WORK-INV-1 | Working Memory MUST bounded |
| MEM-WORK-INV-2 | Working Memory MUST NOT persisted |
| MEM-WORK-INV-3 | Working Memory MUST evict LRU saat penuh |
| MEM-WORK-INV-4 | Working Memory MUST interface dengan WorkingState |
| MEM-WORK-INV-5 | Working Memory entries MUST reference persistent memory |

---

# 7. Memory Indexing

## 7.1 Index Structure

`[MEM-IDX-1]` Memory indexing MUST menggunakan ANN (Approximate Nearest Neighbor) structure.

`[MEM-IDX-2]` Default index structure: **HNSW** (Hierarchical Navigable Small World).

## 7.2 HNSW Parameters

```rust
struct HnswIndex {
    dimensions: u32,             // default 512
    m: u32,                      // max connections per layer, default 32
    ef_construction: u32,        // default 200
    ef_search: u32,              // default max(k*2, 64)
    metric: SimilarityMetric,    // default Cosine
}
```

`[MEM-IDX-3]` Default HNSW parameters:

| Parameter | Default | Range |
|---|---|---|
| `dimensions` | 512 | 128–2048 |
| `m` | 32 | 16–64 |
| `ef_construction` | 200 | 100–500 |
| `ef_search` | max(k*2, 64) | 32–512 |
| `metric` | Cosine | Cosine, DotProduct, Euclidean |

## 7.3 Index Operations

### 7.3.1 Insert

```pseudo
function index_insert(memory: MemoryCell):
    // 1. Normalize key vector (for cosine)
    key = normalize(memory.key_vector)
    
    // 2. Insert into HNSW
    hnsw_index.insert(memory.id, key)
    
    // 3. Update type-specific index
    match memory.memory_type:
        case Episodic:
            episodic_index.insert(memory.id, key)
        case Semantic:
            semantic_index.insert(memory.id, key)
        case Procedural:
            procedural_index.insert(memory.id, key)
```

### 7.3.2 Search

```pseudo
function index_search(query: Vec<f32>, k: usize, mem_type: Option<MemoryType>) -> Vec<(MemoryId, f32)>:
    // 1. Normalize query
    query = normalize(query)
    
    // 2. Select index
    index = match mem_type:
        case Some(Episodic): episodic_index
        case Some(Semantic): semantic_index
        case Some(Procedural): procedural_index
        case None: global_index
    
    // 3. ANN search
    results = index.search(query, k, ef_search)
    
    return results
```

## 7.4 Index Maintenance

`[MEM-IDX-4]` Index MUST mendukung incremental insert.

`[MEM-IDX-5]` Index MAY di-rebuild jika fragmentasi tinggi.

`[MEM-IDX-6]` Index rebuild MUST NOT mengganggu active queries.

## 7.5 Index Persistence

`[MEM-IDX-7]` Index MUST di-persist ke `.cd/memory/index.cd`.

`[MEM-IDX-8]` Index MUST dapat di-rebuild dari memory entries.

## 7.6 Index Invariants

| ID | Invariant |
|---|---|
| MEM-IDX-INV-1 | Index MUST mendukung ANN search |
| MEM-IDX-INV-2 | Index MUST incremental |
| MEM-IDX-INV-3 | Index MUST O(log N) untuk search |
| MEM-IDX-INV-4 | Index MUST persisted |
| MEM-IDX-INV-5 | Index MUST dapat di-rebuild |

---

# 8. Memory Retrieval

## 8.1 Retrieval Algorithm

`[MEM-RET-1]` Memory retrieval MUST content-addressed.

`[MEM-RET-2]` Retrieval complexity MUST O(log N) untuk ANN search.

## 8.2 Retrieval Protocol

```pseudo
function retrieve(query: Vec<f32>, config: RetrievalConfig) -> Vec<MemoryEntry>:
    // 1. Normalize query
    query = normalize(query)
    
    // 2. ANN search
    candidates = index_search(query, config.k * 2, config.mem_type)
    
    // 3. Apply threshold
    filtered = [(id, score) for (id, score) in candidates if score >= config.threshold]
    
    // 4. Score by multiple factors
    scored = []
    for (id, similarity) in filtered:
        memory = load_memory(id)
        
        score = similarity
              × importance(memory)
              × recency_boost(memory)
              × access_boost(memory)
              × type_boost(memory, config.mem_type)
        
        scored.append((memory, score))
    
    // 5. Sort by score
    scored.sort_by_score_desc()
    
    // 6. Take top-k
    results = scored[:config.k]
    
    // 7. Update mutable index statistics without rewriting the entry
    for (memory, _) in results:
        index.increment_access(memory.id, now())
    
    return results
```

## 8.3 Retrieval Config

```rust
struct RetrievalConfig {
    k: usize,                    // default 16
    threshold: f32,              // default 0.3
    mem_type: Option<MemoryType>,
    include_associations: bool,  // default true
    association_depth: usize,    // default 1
    time_range: Option<TimeRange>,
    domain_filter: Option<String>,
}
```

## 8.4 Scoring Factors

`[MEM-RET-3]` Scoring factors:

```text
score = similarity(query, memory.key_vector)
      × importance(memory)
      × recency_boost(memory)
      × access_boost(memory)
      × type_boost(memory, requested_type)
```

Where:

```text
similarity       = cosine_similarity(query, key_vector)
importance       = memory.importance_score  (0.0-1.0)
recency_boost    = 1.0 + 0.2 × exp(-age_hours / 24)
access_boost     = 1.0 + 0.1 × log(1 + access_count)
type_boost       = 1.2 if type matches, 1.0 otherwise
```

## 8.5 Association Traversal

`[MEM-RET-4]` Retrieval MAY include association traversal.

```pseudo
function retrieve_with_associations(query: Vec<f32>, config: RetrievalConfig) -> Vec<MemoryEntry>:
    // 1. Direct retrieval
    direct = retrieve(query, config)
    
    // 2. Association traversal
    if config.include_associations:
        associated = []
        for memory in direct:
            neighbors = traverse_associations(
                memory.id,
                depth = config.association_depth,
                min_strength = 0.5,
            )
            associated.extend(neighbors)
        
        // 3. Deduplicate
        results = deduplicate(direct + associated)
    else:
        results = direct
    
    return results
```

## 8.6 Retrieval Invariants

| ID | Invariant |
|---|---|
| MEM-RET-INV-1 | Retrieval MUST content-addressed |
| MEM-RET-INV-2 | Retrieval MUST O(log N) |
| MEM-RET-INV-3 | Retrieval MUST update access statistics |
| MEM-RET-INV-4 | Retrieval MUST deterministic untuk query sama |
| MEM-RET-INV-5 | Retrieval MUST menghormati threshold |

---

# 9. Consolidation

## 9.1 Definition

`[MEM-CON-1]` Consolidation adalah proses **kompilasi memory yang sering diakses menjadi bentuk yang lebih efisien**.

`[MEM-CON-2]` Consolidation mengubah:
- Frequent episodic → semantic
- Frequent compositions → procedural
- Multiple related entries → single consolidated entry

## 9.2 Consolidation Triggers

`[MEM-CON-3]` Consolidation triggers:

| Trigger | Condition | Action |
|---|---|---|
| Access count | access_count > 100 | Consolidate |
| Time | age > 7 days AND access_count > 10 | Consolidate |
| Importance | importance_score > 0.8 | Consolidate |
| Similarity | Multiple similar entries | Merge |

## 9.3 Consolidation Protocol

```pseudo
function consolidate(memories: Vec<MemoryId>) -> MemoryId:
    // 1. Load memories
    entries = [load_memory(id) for id in memories]
    
    // 2. Verify consolidation eligibility
    for entry in entries:
        if entry.consolidation_level >= 2:
            return Err(AlreadyConsolidated)
    
    // 3. Merge content
    merged_key = weighted_average([e.key_vector for e in entries], weights=importance)
    merged_value = merge_values([e.value_payload for e in entries])
    
    // 4. Create consolidated memory
    consolidated = MemoryCell {
        id: compute_memory_id(merged_key, merged_value),
        memory_type: MemoryType::Consolidated,
        key_vector: merged_key,
        value_payload: merged_value,
        consolidation_level: 1,
        importance_score: max([e.importance_score for e in entries]),
        derived_from: memories,
    }
    
    // 5. Store consolidated memory
    store_memory(consolidated)
    index_memory(consolidated)
    
    // 6. Mark source memories as consolidated
    for entry in entries:
        entry.consolidation_level = 1
        entry.metadata.consolidated_into = consolidated.id
    
    // 7. Create revision
    create_revision_for_consolidation(memories, consolidated.id)
    
    return consolidated.id
```

## 9.4 Episodic → Semantic Consolidation

```pseudo
function consolidate_episodic_to_semantic(episodic_ids: Vec<MemoryId>) -> MemoryId:
    // 1. Load episodic memories
    episodes = [load_memory(id) for id in episodic_ids]
    
    // 2. Extract common patterns
    pattern = extract_common_pattern(episodes)
    
    // 3. Create semantic memory
    semantic = MemoryCell {
        id: compute_memory_id(pattern.key, pattern.value),
        memory_type: MemoryType::Semantic,
        key_vector: pattern.key,
        value_payload: pattern.value,
        consolidation_level: 1,
        importance_score: compute_importance(episodes),
        derived_from: episodic_ids,
    }
    
    // 4. Store
    store_memory(semantic)
    index_memory(semantic)
    
    return semantic.id
```

## 9.5 Consolidation Invariants

| ID | Invariant |
|---|---|
| MEM-CON-INV-1 | Consolidation MUST menghasilkan memory baru |
| MEM-CON-INV-2 | Consolidation MUST NOT menghapus source memories |
| MEM-CON-INV-3 | Consolidation MUST track provenance |
| MEM-CON-INV-4 | Consolidation MUST menghasilkan revision baru |
| MEM-CON-INV-5 | Consolidated memory MUST NOT di-forget |

---

# 10. Forgetting / Retention Policy

## 10.1 Definition

`[MEM-FOR-1]` Forgetting adalah proses **melepas memory yang tidak lagi relevan**.

`[MEM-FOR-2]` Forgetting berbeda dari GC: GC berbasis reachability, forgetting berbasis importance dan decay.

## 10.2 Forgetting Policy

```rust
struct ForgettingPolicy {
    // Decay
    decay_rate: f32,             // default 0.1 per day
    decay_floor: f32,            // default 0.1
    
    // Importance threshold
    min_importance: f32,         // default 0.2
    
    // Access threshold
    min_access_count: u64,       // default 1
    
    // Retention period
    retention_days: u64,         // default 30 for episodic
    
    // Protection
    protect_consolidated: bool,  // default true
    protect_semantic: bool,      // default true
}
```

## 10.3 Forgetting Algorithm

```pseudo
function apply_forgetting(policy: ForgettingPolicy) -> Vec<MemoryId>:
    forgotten = []
    
    for memory in all_memories():
        // Skip protected memories
        if policy.protect_consolidated and memory.consolidation_level >= 1:
            continue
        if policy.protect_semantic and memory.memory_type == Semantic:
            continue
        
        // Compute retention score
        retention_score = compute_retention_score(memory, policy)
        
        // Check if should forget
        if retention_score < policy.min_importance:
            forgotten.append(memory.id)
    
    // Execute forgetting
    for memory_id in forgotten:
        forget_memory(memory_id)
    
    return forgotten
```

## 10.4 Retention Score Computation

```text
retention_score = importance
                × recency_factor
                × access_factor
                × consolidation_factor

Where:
  importance          = memory.importance_score
  recency_factor      = exp(-age_days × decay_rate)
  access_factor       = log(1 + access_count) / log(1 + max_access)
  consolidation_factor = 2.0 if consolidated, 1.0 otherwise
```

## 10.5 Forgetting Rules

`[MEM-FOR-3]` Forgetting rules:

| Memory Type | Default Retention | Forgetting Condition |
|---|---|---|
| Episodic | 30 days | age > retention AND low importance |
| Semantic | Indefinite | Only if importance < 0.1 |
| Procedural | Indefinite | Only if success_rate < 0.3 |
| Consolidated | Indefinite | NEVER |
| Working | Session | End of session |

## 10.6 Forgetting Invariants

| ID | Invariant |
|---|---|
| MEM-FOR-INV-1 | Forgetting MUST berdasarkan importance dan decay |
| MEM-FOR-INV-2 | Consolidated memory MUST NOT di-forget |
| MEM-FOR-INV-3 | Forgetting MUST track provenance |
| MEM-FOR-INV-4 | Forgetting MUST menghasilkan revision baru |
| MEM-FOR-INV-5 | Forgetting MUST reversible (melalui revision) |

---

# 11. Memory Cell Lifecycle

## 11.1 Lifecycle State Machine

```text
┌──────────┐   create   ┌──────────┐   access   ┌──────────┐
│ CREATED  │──────────►│  ACTIVE  │──────────►│   HOT    │
└──────────┘           └────┬─────┘           └────┬─────┘
                            │                      │
                            │ consolidation        │ consolidation
                            ▼                      ▼
                       ┌──────────┐           ┌──────────┐
                       │CONSOLIDATED│◄────────│CONSOLIDATED│
                       └────┬─────┘           └──────────┘
                            │
                            │ forgetting (rare)
                            ▼
                       ┌──────────┐
                       │ ARCHIVED │
                       └────┬─────┘
                            │
                            │ GC
                            ▼
                       ┌──────────┐
                       │FORGOTTEN │
                       └──────────┘
```

## 11.2 Lifecycle States

| State | Deskripsi |
|---|---|
| CREATED | Memory baru dibuat, belum diakses |
| ACTIVE | Memory aktif, dapat diakses |
| HOT | Memory sering diakses |
| CONSOLIDATED | Memory sudah di-consolidate |
| ARCHIVED | Memory diarsipkan, jarang diakses |
| FORGOTTEN | Memory di-forget, menunggu GC |

## 11.3 Lifecycle Transitions

`[MEM-LIFE-1]` Lifecycle transitions:

| From | To | Trigger |
|---|---|---|
| CREATED | ACTIVE | First access |
| ACTIVE | HOT | access_count > threshold |
| ACTIVE | CONSOLIDATED | Consolidation |
| HOT | CONSOLIDATED | Consolidation |
| ACTIVE | ARCHIVED | Long inactivity |
| CONSOLIDATED | ARCHIVED | Explicit archive |
| ARCHIVED | FORGOTTEN | Forgetting policy |
| FORGOTTEN | (deleted) | GC |

## 11.4 Lifecycle Invariants

| ID | Invariant |
|---|---|
| MEM-LIFE-INV-1 | Memory lifecycle MUST dipatuhi |
| MEM-LIFE-INV-2 | CONSOLIDATED memory MUST NOT di-forget |
| MEM-LIFE-INV-3 | FORGOTTEN memory MUST menunggu GC |
| MEM-LIFE-INV-4 | Lifecycle transitions MUST tracked |

---

# 12. WorkingState ↔ Persistent Memory

## 12.1 Interaction Model

`[MEM-WS-1]` WorkingState berinteraksi dengan persistent memory melalui Working Memory.

```text
┌─────────────────────────────────────────────────────────────┐
│                    WorkingState                              │
│                                                             │
│   active_vector                                             │
│   context_entries ──────┐                                   │
│   current_cells         │                                   │
│   composition_stack     │                                   │
│                         │                                   │
└─────────────────────────┼───────────────────────────────────┘
                          │
                          │ read/write
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Working Memory                            │
│                    (bounded, ephemeral)                      │
│                                                             │
│   entries: Vec<WorkingMemoryEntry>                          │
│   max_bytes: 256 MiB                                        │
│                                                             │
└─────────────────────────┬───────────────────────────────────┘
                          │
                          │ load/store
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Persistent Memory                         │
│                                                             │
│   Episodic │ Semantic │ Procedural │ Consolidated           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 12.2 Context as Memory

`[MEM-WS-2]` Context MUST ditangani melalui memory, bukan KV-cache.

`[MEM-WS-3]` Context MUST NOT tumbuh linear terhadap sequence length.

```pseudo
function update_context(state: WorkingState, new_content: Vec<u8>):
    // 1. Compress content to memory entry
    key = encode_to_vector(new_content)
    value = compress(new_content)
    
    // 2. Write to persistent memory
    memory_id = persistent_write(key, value, MemoryType::Episodic)
    
    // 3. Add to working memory
    working_load(memory_id)
    
    // 4. Update WorkingState context
    state.context_entries.append(memory_id)
    
    // 5. Evict old context if over limit
    if state.context_entries.len() > max_context_entries:
        evict_oldest_context(state)
```

## 12.3 Context Retrieval

```pseudo
function retrieve_context(state: WorkingState, query: Vec<f32>) -> Vec<MemoryEntry>:
    // 1. Search working memory first
    working_results = working_memory.search(query, k=4)
    
    // 2. Search persistent memory
    persistent_results = persistent_retrieve(query, k=8)
    
    // 3. Merge and deduplicate
    results = merge(working_results, persistent_results)
    
    return results
```

## 12.4 WorkingState Integration

`[MEM-WS-4]` WorkingState MUST menggunakan memory untuk context.

`[MEM-WS-5]` WorkingState MUST NOT menyimpan context sebagai KV-cache.

`[MEM-WS-6]` WorkingState context entries MUST bounded.

## 12.5 WorkingState ↔ Memory Invariants

| ID | Invariant |
|---|---|
| MEM-WS-INV-1 | Context MUST melalui memory |
| MEM-WS-INV-2 | Context MUST NOT KV-cache |
| MEM-WS-INV-3 | Context MUST NOT tumbuh linear |
| MEM-WS-INV-4 | WorkingState context MUST bounded |
| MEM-WS-INV-5 | Working Memory MUST interface ke persistent |

---

# 13. Memory Budget

## 13.1 Budget Structure

`[MEM-BUD-1]` Memory budget MUST hard-enforced.

```rust
struct MemoryBudget {
    // Working memory
    working_memory_bytes: u64,    // default 256 MiB
    working_memory_entries: usize, // default 256
    
    // Hot memory
    hot_memory_bytes: u64,        // default 4 GiB
    
    // Total persistent
    total_persistent_bytes: u64,  // default 1 TiB
    
    // Context
    max_context_entries: usize,   // default 256
    max_context_bytes: u64,       // default 256 MiB
}
```

## 13.2 Budget Enforcement

```pseudo
function enforce_budget(request: MemoryRequest, budget: MemoryBudget) -> BudgetDecision:
    match request:
        case WorkingMemoryLoad(size):
            if working_memory.used + size > budget.working_memory_bytes:
                // Try eviction
                if can_evict(size):
                    evict(size)
                    return Admit
                else:
                    return Reject
            return Admit
        
        case PersistentMemoryWrite(size):
            if persistent_memory.used + size > budget.total_persistent_bytes:
                // Try forgetting
                forgotten = apply_forgetting(policy)
                if persistent_memory.used + size > budget.total_persistent_bytes:
                    return Reject
            return Admit
        
        case ContextAdd(size):
            if context.entries.len() >= budget.max_context_entries:
                evict_oldest_context()
            if context.bytes + size > budget.max_context_bytes:
                evict_context_until_fits(size)
            return Admit
```

## 13.3 Budget Defaults

`[MEM-BUD-2]` Default memory budget:

| Parameter | Default | Configurable |
|---|---|---|
| `working_memory_bytes` | 256 MiB | YES |
| `working_memory_entries` | 256 | YES |
| `hot_memory_bytes` | 4 GiB | YES |
| `total_persistent_bytes` | 1 TiB | YES |
| `max_context_entries` | 256 | YES |
| `max_context_bytes` | 256 MiB | YES |

## 13.4 Budget Invariants

| ID | Invariant |
|---|---|
| MEM-BUD-INV-1 | Memory budget MUST hard-enforced |
| MEM-BUD-INV-2 | Working memory MUST bounded |
| MEM-BUD-INV-3 | Context MUST bounded |
| MEM-BUD-INV-4 | Budget violation MUST explicit |
| MEM-BUD-INV-5 | Budget MUST configurable |

---

# 14. Retrieval Complexity

## 14.1 Complexity Analysis

`[MEM-CPLX-1]` Retrieval complexity:

| Operation | Complexity | Notes |
|---|---|---|
| ANN search | O(log N) | HNSW |
| Top-k retrieval | O(k × log N) | HNSW + scoring |
| Direct lookup by ID | O(1) | HashMap |
| Association traversal | O(d × b) | d=depth, b=branching |
| Index insert | O(log N) | HNSW |
| Index delete | O(log N) | HNSW |

Where:
- N = total memory entries
- k = number of results
- d = traversal depth
- b = average branching factor

## 14.2 Complexity Guarantees

`[MEM-CPLX-2]` ANN search MUST O(log N).

`[MEM-CPLX-3]` Retrieval MUST NOT O(N) scan.

`[MEM-CPLX-4]` Direct lookup MUST O(1).

## 14.3 Scalability

`[MEM-CPLX-5]` Memory system MUST scalable:

| Memory Entries | Expected Search Latency |
|---|---|
| 1K | < 100 μs |
| 100K | < 1 ms |
| 1M | < 10 ms |
| 10M | < 100 ms |
| 100M | < 1 s |

## 14.4 Complexity Invariants

| ID | Invariant |
|---|---|
| MEM-CPLX-INV-1 | ANN search MUST O(log N) |
| MEM-CPLX-INV-2 | Retrieval MUST NOT O(N) scan |
| MEM-CPLX-INV-3 | Direct lookup MUST O(1) |
| MEM-CPLX-INV-4 | Index insert MUST O(log N) |
| MEM-CPLX-INV-5 | Scalability MUST terjamin |

---

# 15. Memory Serialization

## 15.1 Memory Segment Format

`[MEM-SER-1]` Memory entries disimpan dalam segment khusus dengan ekstensi `.mcd` (dari .cd Format Spec).

`[MEM-SER-2]` Memory segment menggunakan magic `CNWSMEM1`.

## 15.2 Memory Entry Serialization

```text
MemoryEntry (binary):
  memory_id:          32 bytes (BLAKE3-256)
  memory_type:        1 byte
  consolidation_level: 1 byte
  key_dim:            8 bytes (u64 LE)
  value_size:         8 bytes (u64 LE)
  association_count:  8 bytes (u64 LE)
  created_at_ns:      8 bytes (u64 LE)
  importance_score:   4 bytes (f32 LE)
  padding:            3 bytes
  key_vector:         key_dim × 4 bytes (f32 LE)
  value_payload:      value_size bytes
    associations:       association_count × 48 bytes
```

Access statistics (`access_count`, `last_access_ns`) MUST be stored in the mutable MemoryIndex defined by the `.cd` Format Specification, not in the immutable MemoryEntry serialization.

## 15.3 Memory Index Serialization

`[MEM-SER-3]` Memory index disimpan di `.cd/memory/index.cd`.

`[MEM-SER-4]` Index format mengikuti .cd Format Spec §10.4.

## 15.4 Memory in MANIFEST.cd

```json
{
  "memory": {
    "episodic_entries": 1048576,
    "semantic_entries": 4194304,
    "procedural_entries": 262144,
    "consolidated_entries": 524288,
    "working_memory_bound_bytes": 268435456,
    "total_memory_bytes": 1099511627776,
    "index": {
      "structure": "HNSW",
      "dimensions": 512,
      "m": 32,
      "ef_construction": 200
    }
  }
}
```

---

# 16. Error Handling

## 16.1 Memory Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-MEM-NOTFOUND` | Memory entry not found |
| `CNWS-E-MEM-BUDGET` | Memory budget exceeded |
| `CNWS-E-MEM-INDEX` | Index error |
| `CNWS-E-MEM-RETRIEVAL` | Retrieval failed |
| `CNWS-E-MEM-CONSOLIDATION` | Consolidation failed |
| `CNWS-E-MEM-FORGETTING` | Forgetting failed |
| `CNWS-E-MEM-WORKING` | Working memory error |
| `CNWS-E-MEM-ASSOCIATION` | Association error |

## 16.2 Error Severity

| Severity | Examples | Action |
|---|---|---|
| Fatal | MEM-INDEX corrupt | Rebuild index |
| Recoverable | MEM-BUDGET, MEM-NOTFOUND | Evict or fallback |
| Warning | MEM-CONSOLIDATION partial | Log and continue |

---

# 17. Final Memory Contract

## 17.1 Ringkasan Keputusan Memory

| ID | Keputusan |
|---|---|
| MEM-F01 | Memory adalah first-class persistent state, bukan cache. |
| MEM-F02 | Memory types: Episodic, Semantic, Procedural, Working, Consolidated. |
| MEM-F03 | Episodic Memory menyimpan specific experiences. |
| MEM-F04 | Semantic Memory menyimpan factual knowledge. |
| MEM-F05 | Procedural Memory menyimpan learned patterns. |
| MEM-F06 | Working Memory bounded 256 MiB, ephemeral. |
| MEM-F07 | Indexing menggunakan HNSW, dimensions 512. |
| MEM-F08 | Retrieval top-k default k=16, threshold=0.3. |
| MEM-F09 | Retrieval complexity O(log N). |
| MEM-F10 | Consolidation triggers: access_count, time, importance. |
| MEM-F11 | Forgetting menggunakan decay + importance scoring. |
| MEM-F12 | Consolidated memory MUST NOT di-forget. |
| MEM-F13 | Context MUST melalui memory, bukan KV-cache. |
| MEM-F14 | Context MUST NOT tumbuh linear. |
| MEM-F15 | Memory budget hard-enforced. |
| MEM-F16 | Memory lifecycle: CREATED → ACTIVE → CONSOLIDATED → ARCHIVED/FORGOTTEN. |
| MEM-F17 | Memory identity = BLAKE3-256(key + value). |
| MEM-F18 | Memory associations MUST tracked. |
| MEM-F19 | Memory access statistics MUST tracked. |
| MEM-F20 | Memory MUST indexed untuk retrieval. |

## 17.2 Memory Invariants

| ID | Invariant |
|---|---|
| MEM-INV-1 | Memory MUST first-class persistent state. |
| MEM-INV-2 | Memory MUST content-addressed. |
| MEM-INV-3 | Memory MUST memiliki associations. |
| MEM-INV-4 | Memory MUST memiliki access statistics. |
| MEM-INV-5 | Memory MUST memiliki importance score. |
| MEM-INV-6 | Working Memory MUST bounded. |
| MEM-INV-7 | Working Memory MUST NOT persisted. |
| MEM-INV-8 | Context MUST melalui memory. |
| MEM-INV-9 | Context MUST NOT KV-cache. |
| MEM-INV-10 | Context MUST NOT tumbuh linear. |
| MEM-INV-11 | Retrieval MUST content-addressed. |
| MEM-INV-12 | Retrieval MUST O(log N). |
| MEM-INV-13 | Consolidation MUST menghasilkan memory baru. |
| MEM-INV-14 | Consolidated memory MUST NOT di-forget. |
| MEM-INV-15 | Forgetting MUST berdasarkan importance dan decay. |
| MEM-INV-16 | Memory budget MUST hard-enforced. |
| MEM-INV-17 | Memory lifecycle MUST dipatuhi. |
| MEM-INV-18 | Memory MUST indexed. |
| MEM-INV-19 | Memory MUST persisted ke .cd. |
| MEM-INV-20 | Memory MUST deterministic untuk query sama. |

## 17.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Memory & Retrieval final dan mengikat** untuk LATTICE memory CNWS. Ia mendefinisikan bagaimana memory berfungsi sebagai first-class persistent intelligence state, dari episodic hingga procedural, dari indexing hingga consolidation, dari retrieval hingga forgetting.

Memory CNWS **bukan sekadar storage Cell biasa**. Ia adalah bagian dari model yang belajar, mengingat, mengkonsolidasi, dan melupakan — dengan mekanisme yang terdefinisi dan terukur.

Seluruh implementasi Memory System, Retrieval Engine, Consolidation Engine, dan Retention Policy CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan Memory yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN MEMORY & RETRIEVAL SPECIFICATION**
