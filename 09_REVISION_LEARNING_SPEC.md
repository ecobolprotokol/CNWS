# CNWS
## Revision & Learning Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Revision & Learning Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (REVISION & LEARNING SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS Cell & Schema Spec; CNWS .cd Format Spec |
| Hulu ke | Implementasi Revision Manager, Learning Engine, GC, Specialization |
| Otoritas | Spesifikasi tunggal untuk Revision DAG dan Learning CNWS |
| Prinsip Dijaga | Revision DAG **bukan Git-like versioning**; ia adalah learning-integrated versioning |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    Cell & Schema Spec    Revision & Learning Spec    Implementation
─────────────────────   ──────────────────    ────────────────────────    ─────────────
Revision DAG          ──► Cell structure   ──► Revision creation       ──► Revision Manager
Learning primitives     CellType              Learning updates             Learning Engine
Incremental delta       Metadata              Merge/branch/rollback        Merge Engine
GC reachability         Lifecycle             Learning isolation           GC
                                              Specialization               Specialization
```

`[REV-DOC-1]` Dokumen ini mendefinisikan **bagaimana Revision DAG dan Learning terintegrasi** dalam CNWS.

`[REV-DOC-2]` Revision DAG CNWS **bukan Git-like versioning**. Ia adalah **learning-integrated versioning** di mana setiap learning update menghasilkan revision baru.

`[REV-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[REV-DOC-4]` Jika terjadi konflik dengan Cell & Schema Spec untuk hal struktur Cell, Cell & Schema Spec menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-REV-01 | Revision adalah learning-integrated versioning, bukan Git-like. |
| DF-REV-02 | Setiap learning update menghasilkan revision baru. |
| DF-REV-03 | Revision delta pada level Cell/Tile, bukan full model. |
| DF-REV-04 | Revision immutable setelah committed. |
| DF-REV-05 | Branching tidak menyalin Cells. |
| DF-REV-06 | Merge menggunakan three-way merge pada level Cell. |
| DF-REV-07 | Conflict resolution pada level Cell/Tile. |
| DF-REV-08 | Rollback tidak menghapus revision. |
| DF-REV-09 | GC berbasis reachability dari revision roots. |
| DF-REV-10 | Learning isolation: hanya Cell yang berubah yang dimodifikasi. |
| DF-REV-11 | Specialization tanpa duplikasi base Cells. |
| DF-REV-12 | Catastrophic forgetting dicegah melalui learning isolation. |
| DF-REV-13 | Affected-cell accounting: O(affected_cells), bukan O(total_cells). |
| DF-REV-14 | Learning commit bersifat atomic. |

---

# 1. Executive Summary

## 1.1 Revision DAG Bukan Git-like Versioning

`[REV-EXEC-1]` Revision DAG CNWS **bukan** Git-like versioning. Perbedaan fundamental:

| Aspek | Git | CNWS Revision DAG |
|---|---|---|
| Unit perubahan | File (text) | Cell/Tile (content-addressed) |
| Identitas | Path + content hash | BLAKE3-256 content address |
| Merge | Text-based three-way | Cell-level three-way |
| Learning | Tidak ada | Setiap learning = revision |
| Branch | Copy of history | Reference sharing |
| Diff | Line-based text diff | Cell/Tile delta |
| Storage | Packfile | Content-addressed Tiles |
| Purpose | Source code versioning | Intelligence state versioning |

`[REV-EXEC-2]` Revision DAG CNWS adalah **learning-integrated versioning**: setiap learning update (CellCreate, CellRefine, MemoryWrite, RoutingUpdate, CompositionCache) menghasilkan revision baru.

## 1.2 Tujuan Revision & Learning

`[REV-EXEC-3]` Revision & Learning MUST mendukung:

1. **Incremental learning**: belajar tanpa full retraining
2. **Specialization**: domain-specific tanpa duplikasi
3. **Branching**: multiple variants tanpa copy
4. **Merging**: kombinasi variants
5. **Rollback**: kembali ke state sebelumnya
6. **Catastrophic-forgetting prevention**: belajar tanpa lupa
7. **Affected-cell accounting**: cost proportional terhadap perubahan

## 1.3 Prinsip Utama

`[REV-EXEC-4]` Prinsip utama Revision & Learning:

1. **Revision menyimpan perubahan, bukan model.**
2. **Model efektif adalah hasil resolusi revision + ancestors.**
3. **Learning adalah revision.**
4. **Cell immutable; perubahan menghasilkan Cell baru.**
5. **Cost proportional terhadap affected cells, bukan total cells.**

---

# 2. Revision Model

## 2.1 Revision Object

`[REV-MOD-1]` Revision object MUST memiliki struktur berikut:

```rust
struct Revision {
    // Identity
    id: RevisionId,              // BLAKE3-256
    model_id: ModelId,
    revision_number: u64,
    
    // Lineage
    parents: Vec<RevisionId>,    // multiple parents for merge
    
    // Content
    root_manifest: ManifestId,
    changed_cells: Vec<CellId>,
    changed_tiles: TileChanges,
    changed_memory: Vec<CellId>,
    changed_routing: Vec<CellId>,
    changed_compositions: Vec<CellId>,
    
    // Metadata
    metadata: RevisionMetadata,
    created_at: Timestamp,
    author: Option<String>,
    message: Option<String>,
    
    // Learning context
    learning_context: Option<LearningContext>,
}
```

## 2.2 Revision Immutability

`[REV-MOD-2]` Revision MUST immutable setelah committed.

`[REV-MOD-3]` Perubahan selalu menghasilkan revision baru, bukan modifikasi revision existing.

```text
Revision 7 (committed)
    │
    ▼ (new change)
Revision 8 (new revision)
    │
    └── parent: Revision 7
```

## 2.3 Revision Lineage

`[REV-MOD-4]` Revision lineage membentuk DAG, bukan linear history.

```text
Rev 0 (base)
    │
    ├──► Rev 1 (fine-tune A)
    │       │
    │       ├──► Rev 2
    │       │
    │       └──► Rev 3
    │
    └──► Rev 4 (branch from Rev 0)
    
    Rev 2 ──┐
             ├──► Rev 5 (merge)
    Rev 4 ──┘
```

`[REV-MOD-5]` Revision MAY memiliki multiple parents (untuk merge).

## 2.4 Effective Model State

`[REV-MOD-6]` Model efektif adalah hasil resolusi revision + ancestors.

```pseudo
function resolve(rev: RevisionId) -> EffectiveGraph:
    // Walk revision DAG
    revisions = walk_to_root(rev)
    
    // Collect Cell mappings, latest wins
    cell_map = HashMap<CellId, TileMapping>()
    for revision in revisions (newest to oldest):
        for cell_id in revision.changed_cells:
            if cell_id not in cell_map:
                cell_map[cell_id] = revision.get_mapping(cell_id)
    
    // Build effective graph
    return EffectiveGraph {
        cells: cell_map,
        revision: rev,
    }
```

`[REV-MOD-7]` Resolusi MUST "latest wins": perubahan di revision lebih baru mengambil precedence.

## 2.5 Revision Resolution Cache

`[REV-MOD-8]` Resolusi DAG MUST di-cache untuk menghindari resolusi berulang saat inference.

```text
Startup:
  model.cd
    │
    ▼
  resolve(active_revision)
    │
    ▼
  build effective graph
    │
    ▼
  cache effective graph
    │
    ▼
  O(1) runtime lookup
```

`[REV-MOD-9]` DAG adalah version-control structure, bukan hot-path execution structure.

---

# 3. Revision Creation

## 3.1 Revision Creation Protocol

`[REV-CRE-1]` Revision creation MUST mengikuti protokol berikut:

```pseudo
function create_revision(base: RevisionId, changes: ChangeSet) -> RevisionId:
    // 1. Validate changes
    validate(changes)
    
    // 2. Apply changes to staging
    staged = apply_to_staging(base, changes)
    
    // 3. Compute new manifest
    manifest = build_manifest(staged)
    
    // 4. Compute revision identity
    rev_id = compute_revision_id(base, changes, manifest)
    
    // 5. Build revision object
    revision = Revision {
        id: rev_id,
        model_id: base.model_id,
        revision_number: base.revision_number + 1,
        parents: [base.id],
        root_manifest: manifest.id,
        changed_cells: changes.cell_ids(),
        changed_tiles: changes.tile_changes(),
        changed_memory: changes.memory_ids(),
        changed_routing: changes.routing_ids(),
        changed_compositions: changes.composition_ids(),
        metadata: build_metadata(changes),
        created_at: now(),
        author: current_author(),
        message: changes.message,
        learning_context: changes.learning_context,
    }
    
    // 6. Atomic commit
    atomic_commit(revision)
    
    return revision.id
```

## 3.2 Atomic Commit

`[REV-CRE-2]` Revision commit MUST atomic.

`[REV-CRE-3]` Atomic commit mengikuti protokol yang didefinisikan dalam .cd Format Specification:

```text
1. Write staging manifest
2. fsync staging
3. Append journal
4. fsync journal
5. Rename staging → MANIFEST.cd
6. fsync directory
7. Update SUPERBLOCK
8. fsync SUPERBLOCK
9. Append commit-complete
10. fsync journal
```

## 3.3 Revision Identity

`[REV-CRE-4]` Revision identity dihitung dari:

```text
revision_id = BLAKE3-256(
    parent_ids ||
    changed_cells ||
    changed_tiles ||
    manifest_hash ||
    created_at
)
```

## 3.4 Revision Creation Invariants

| ID | Invariant |
|---|---|
| REV-CRE-INV-1 | Revision MUST immutable setelah committed |
| REV-CRE-INV-2 | Revision commit MUST atomic |
| REV-CRE-INV-3 | Revision identity MUST deterministik |
| REV-CRE-INV-4 | Revision MUST menyimpan delta, bukan full state |
| REV-CRE-INV-5 | Revision MUST memiliki minimal satu parent (kecuali root) |

---

# 4. Learning Update Types

## 4.1 Overview

`[REV-LRN-1]` Learning di CNWS adalah **structural dan incremental**, bukan global parameter update.

`[REV-LRN-2]` Setiap learning update MUST menghasilkan revision baru.

`[REV-LRN-3]` Learning update types:

| Type | Deskripsi | Cost |
|---|---|---|
| CellCreate | Menambahkan Cell baru | O(cell_size) |
| CellRefine | Memodifikasi Cell existing | O(cell_size) |
| MemoryWrite | Menambahkan memory entry | O(entry_size) |
| RoutingUpdate | Mengubah routing | O(affected_edges) |
| CompositionCache | Menyimpan composition pattern | O(pattern_size) |

## 4.2 CellCreate

### 4.2.1 Definition

`[REV-CC-1]` CellCreate menambahkan Cell baru ke Lattice.

`[REV-CC-2]` CellCreate digunakan ketika:
- Knowledge baru yang belum ada
- Existing Cells tidak dapat menangani input
- Specialization memerlukan Cell domain-specific

### 4.2.2 CellCreate Protocol

```pseudo
function cell_create(spec: CellSpec, context: LearningContext) -> RevisionId:
    // 1. Validate spec
    validate(spec)
    
    // 2. Generate tiles
    tiles = plan_tiles(spec)
    
    // 3. Compute cell identity
    cell_id = compute_cell_id(spec, tiles)
    
    // 4. Check if already exists (dedup)
    if cell_exists(cell_id):
        return existing_revision_for(cell_id)
    
    // 5. Create Cell object
    cell = Cell {
        id: cell_id,
        cell_type: spec.cell_type,
        version: spec.version,
        input_schema: spec.input_schema,
        output_schema: spec.output_schema,
        tiles: tiles,
        index_vector: spec.index_vector,
        dependencies: spec.dependencies,
        metadata: spec.metadata,
        representations: spec.representations,
    }
    
    // 6. Write tiles to storage
    for tile in tiles:
        write_tile(tile)
    
    // 7. Create revision
    changes = ChangeSet {
        cells_added: [cell],
        message: "CellCreate: " + spec.description,
        learning_context: context,
    }
    
    revision = create_revision(active_revision(), changes)
    
    // 8. Update routing to include new Cell
    routing_update_for_new_cell(cell_id)
    
    return revision
```

### 4.2.3 CellCreate Invariants

| ID | Invariant |
|---|---|
| REV-CC-INV-1 | CellCreate MUST menghasilkan Cell baru |
| REV-CC-INV-2 | CellCreate MUST deduplicate jika Cell sudah ada |
| REV-CC-INV-3 | CellCreate MUST menghasilkan revision baru |
| REV-CC-INV-4 | CellCreate MUST update routing |
| REV-CC-INV-5 | CellCreate cost MUST O(cell_size) |

## 4.3 CellRefine

### 4.3.1 Definition

`[REV-CR-1]` CellRefine memodifikasi Cell existing dengan membuat version baru.

`[REV-CR-2]` CellRefine TIDAK memodifikasi Cell lama; ia membuat Cell baru yang menggantikan Cell lama.

### 4.3.2 CellRefine Protocol

```pseudo
function cell_refine(old_cell_id: CellId, updates: CellUpdates, context: LearningContext) -> RevisionId:
    // 1. Load old Cell
    old_cell = load_cell(old_cell_id)
    
    // 2. Apply updates
    new_spec = apply_updates(old_cell, updates)
    
    // 3. Create new Cell (new identity)
    new_cell = create_cell(new_spec)
    
    // 4. Link lineage
    new_cell.metadata.lineage.append(old_cell_id)
    
    // 5. Write new tiles
    for tile in new_cell.tiles:
        write_tile(tile)
    
    // 6. Create revision with replacement
    changes = ChangeSet {
        cells_replaced: [(old_cell_id, new_cell)],
        message: "CellRefine: " + old_cell_id,
        learning_context: context,
    }
    
    revision = create_revision(active_revision(), changes)
    
    // 7. Mark old Cell as refined (not deleted)
    mark_cell_refined(old_cell_id, new_cell.id)
    
    return revision
```

### 4.3.3 CellRefine Invariants

| ID | Invariant |
|---|---|
| REV-CR-INV-1 | CellRefine MUST menghasilkan Cell baru |
| REV-CR-INV-2 | CellRefine MUST NOT memodifikasi Cell lama |
| REV-CR-INV-3 | CellRefine MUST menghasilkan revision baru |
| REV-CR-INV-4 | CellRefine MUST link lineage |
| REV-CR-INV-5 | CellRefine cost MUST O(cell_size) |

## 4.4 MemoryWrite

### 4.4.1 Definition

`[REV-MW-1]` MemoryWrite menambahkan memory entry ke persistent memory.

`[REV-MW-2]` MemoryWrite digunakan untuk:
- Episodic memory (experiences)
- Semantic memory (facts)
- Procedural memory (patterns)

### 4.4.2 MemoryWrite Protocol

```pseudo
function memory_write(entry: MemoryEntry, context: LearningContext) -> RevisionId:
    // 1. Validate entry
    validate(entry)
    
    // 2. Compute memory identity
    memory_id = compute_memory_id(entry)
    
    // 3. Check if already exists (dedup)
    if memory_exists(memory_id):
        return existing_revision_for(memory_id)
    
    // 4. Write memory entry
    write_memory(entry)
    
    // 5. Create revision
    changes = ChangeSet {
        memory_added: [entry],
        message: "MemoryWrite: " + entry.description,
        learning_context: context,
    }
    
    revision = create_revision(active_revision(), changes)
    
    return revision
```

### 4.4.3 MemoryWrite Invariants

| ID | Invariant |
|---|---|
| REV-MW-INV-1 | MemoryWrite MUST menghasilkan memory entry baru |
| REV-MW-INV-2 | MemoryWrite MUST deduplicate jika entry sudah ada |
| REV-MW-INV-3 | MemoryWrite MUST menghasilkan revision baru |
| REV-MW-INV-4 | MemoryWrite cost MUST O(entry_size) |

## 4.5 RoutingUpdate

### 4.5.1 Definition

`[REV-RU-1]` RoutingUpdate mengubah routing policy atau statistics.

`[REV-RU-2]` Routing update types:
- **Policy change**: MUST menghasilkan revision baru
- **Statistics update**: MAY tidak menghasilkan revision baru

### 4.5.2 RoutingUpdate Protocol

```pseudo
function routing_update(updates: Vec<RoutingUpdate>, context: LearningContext) -> Option<RevisionId>:
    // Separate policy changes from statistics updates
    policy_changes = filter(updates, is_policy_change)
    stats_updates = filter(updates, is_stats_update)
    
    // Apply statistics updates (no revision needed)
    for update in stats_updates:
        apply_stats_update(update)
    
    // Policy changes require revision
    if policy_changes is not empty:
        changes = ChangeSet {
            routing_changed: policy_changes,
            message: "RoutingUpdate: policy change",
            learning_context: context,
        }
        revision = create_revision(active_revision(), changes)
        return Some(revision)
    
    return None
```

### 4.5.3 RoutingUpdate Invariants

| ID | Invariant |
|---|---|
| REV-RU-INV-1 | Routing policy change MUST menghasilkan revision baru |
| REV-RU-INV-2 | Routing statistics update MAY tidak menghasilkan revision |
| REV-RU-INV-3 | RoutingUpdate MUST incremental |
| REV-RU-INV-4 | RoutingUpdate cost MUST O(affected_edges) |

## 4.6 CompositionCache

### 4.6.1 Definition

`[REV-CPC-1]` CompositionCache menyimpan composition pattern yang sering digunakan.

`[REV-CPC-2]` CompositionCache digunakan untuk:
- Mengurangi overhead composition
- Menyimpan macro-Cells
- Caching hot paths

### 4.6.2 CompositionCache Protocol

```pseudo
function composition_cache(pattern: CompositionPattern, context: LearningContext) -> RevisionId:
    // 1. Validate pattern
    validate(pattern)
    
    // 2. Compute pattern identity
    pattern_id = compute_pattern_id(pattern)
    
    // 3. Check if already exists (dedup)
    if pattern_exists(pattern_id):
        return existing_revision_for(pattern_id)
    
    // 4. Write composition pattern
    write_composition(pattern)
    
    // 5. Create revision
    changes = ChangeSet {
        compositions_added: [pattern],
        message: "CompositionCache: " + pattern.description,
        learning_context: context,
    }
    
    revision = create_revision(active_revision(), changes)
    
    return revision
```

### 4.6.3 CompositionCache Invariants

| ID | Invariant |
|---|---|
| REV-CPC-INV-1 | CompositionCache MUST menghasilkan pattern baru |
| REV-CPC-INV-2 | CompositionCache MUST deduplicate jika pattern sudah ada |
| REV-CPC-INV-3 | CompositionCache MUST menghasilkan revision baru |
| REV-CPC-INV-4 | CompositionCache cost MUST O(pattern_size) |

## 4.7 Learning Update Summary

| Update Type | Menghasilkan Revision | Cost | Scope |
|---|---|---|---|
| CellCreate | MUST | O(cell_size) | Local |
| CellRefine | MUST | O(cell_size) | Local |
| MemoryWrite | MUST | O(entry_size) | Local |
| RoutingUpdate (policy) | MUST | O(affected_edges) | Local |
| RoutingUpdate (stats) | MAY NOT | O(affected_edges) | Local |
| CompositionCache | MUST | O(pattern_size) | Local |

---

# 5. Branching

## 5.1 Branching Definition

`[REV-BR-1]` Branching membuat variant baru tanpa menyalin Cells.

`[REV-BR-2]` Branch adalah revision baru yang merefer ancestor.

## 5.2 Branching Protocol

```pseudo
function branch(base: RevisionId, name: String, metadata: BranchMetadata) -> RevisionId:
    // 1. Load base revision
    base_rev = load_revision(base)
    
    // 2. Create branch revision (no Cell changes)
    branch_rev = Revision {
        id: compute_branch_id(base, name),
        model_id: base_rev.model_id,
        revision_number: base_rev.revision_number + 1,
        parents: [base],
        root_manifest: base_rev.root_manifest,  // Same manifest
        changed_cells: [],                       // No changes
        changed_tiles: TileChanges::empty(),
        changed_memory: [],
        changed_routing: [],
        changed_compositions: [],
        metadata: RevisionMetadata {
            branch_name: name,
            branch_metadata: metadata,
        },
        created_at: now(),
        author: current_author(),
        message: "Branch: " + name,
        learning_context: None,
    }
    
    // 3. Commit branch revision
    atomic_commit(branch_rev)
    
    return branch_rev.id
```

## 5.3 Branching Invariants

| ID | Invariant |
|---|---|
| REV-BR-INV-1 | Branch MUST NOT menyalin Cells |
| REV-BR-INV-2 | Branch MUST merefer ancestor |
| REV-BR-INV-3 | Branch MUST menghasilkan revision baru |
| REV-BR-INV-4 | Branch cost MUST O(1) (hanya metadata) |

## 5.4 Branching Example

```text
Base (Rev 0)
    │
    ├── branch("coding") ──► Rev 1 (coding)
    │                            │
    │                            └── CellRefine ──► Rev 2
    │
    └── branch("reasoning") ──► Rev 3 (reasoning)
                                     │
                                     └── CellCreate ──► Rev 4

Shared Tiles:
  Base Tiles: A, B, C, D
  
  Coding (Rev 2):
    A → shared
    B → shared
    C → shared
    D → shared
    E → new (coding-specific)
  
  Reasoning (Rev 4):
    A → shared
    B → shared
    C → shared
    D → shared
    F → new (reasoning-specific)
```

---

# 6. Three-Way Merge

## 6.1 Merge Definition

`[REV-MRG-1]` Merge menggabungkan dua branches menjadi satu revision.

`[REV-MRG-2]` Merge MUST menggunakan three-way merge pada level Cell.

## 6.2 Three-Way Merge Algorithm

```pseudo
function merge(branch_a: RevisionId, branch_b: RevisionId) -> Result<RevisionId, MergeConflict>:
    // 1. Find common ancestor (base)
    base = find_common_ancestor(branch_a, branch_b)
    
    // 2. Resolve effective graphs
    base_graph = resolve(base)
    graph_a = resolve(branch_a)
    graph_b = resolve(branch_b)
    
    // 3. Three-way merge for each Cell
    merged_cells = []
    conflicts = []
    
    all_cell_ids = union(base_graph.cells, graph_a.cells, graph_b.cells)
    
    for cell_id in all_cell_ids:
        base_cell = base_graph.get(cell_id)
        cell_a = graph_a.get(cell_id)
        cell_b = graph_b.get(cell_id)
        
        result = three_way_merge_cell(base_cell, cell_a, cell_b)
        
        match result:
            case Merged(cell):
                merged_cells.append(cell)
            case Conflict(conflict):
                conflicts.append(conflict)
    
    // 4. If conflicts, return error
    if conflicts is not empty:
        return Err(MergeConflict { conflicts })
    
    // 5. Create merge revision
    merge_rev = Revision {
        id: compute_merge_id(branch_a, branch_b),
        parents: [branch_a, branch_b],  // Multiple parents
        changed_cells: merged_cells,
        // ...
    }
    
    atomic_commit(merge_rev)
    
    return Ok(merge_rev.id)
```

## 6.3 Cell-Level Three-Way Merge

`[REV-MRG-3]` Three-way merge untuk setiap Cell:

```pseudo
function three_way_merge_cell(base, a, b) -> MergeResult:
    // Case 1: Both same as base → use base
    if a == base and b == base:
        return Merged(base)
    
    // Case 2: A changed, B same as base → use A
    if a != base and b == base:
        return Merged(a)
    
    // Case 3: B changed, A same as base → use B
    if a == base and b != base:
        return Merged(b)
    
    // Case 4: Both changed to same value → use either
    if a == b:
        return Merged(a)
    
    // Case 5: Both changed to different values → CONFLICT
    if a != base and b != base and a != b:
        return Conflict(MergeConflict {
            cell_id: base.id,
            base: base,
            branch_a: a,
            branch_b: b,
        })
    
    // Case 6: Cell added in A, not in base or B → use A
    if base is None and a is not None and b is None:
        return Merged(a)
    
    // Case 7: Cell added in B, not in base or A → use B
    if base is None and a is None and b is not None:
        return Merged(b)
    
    // Case 8: Cell added in both → check if same
    if base is None and a is not None and b is not None:
        if a == b:
            return Merged(a)
        else:
            return Conflict(...)
    
    // Case 9: Cell deleted in A, not changed in B → delete
    if a is None and b == base:
        return Merged(None)  // Delete
    
    // Case 10: Cell deleted in B, not changed in A → delete
    if b is None and a == base:
        return Merged(None)  // Delete
    
    // Case 11: Cell deleted in one, modified in other → CONFLICT
    if (a is None and b != base) or (b is None and a != base):
        return Conflict(...)
```

## 6.4 Merge Invariants

| ID | Invariant |
|---|---|
| REV-MRG-INV-1 | Merge MUST menggunakan three-way merge |
| REV-MRG-INV-2 | Merge MUST pada level Cell |
| REV-MRG-INV-3 | Merge MUST menemukan common ancestor |
| REV-MRG-INV-4 | Merge dengan conflict MUST gagal |
| REV-MRG-INV-5 | Merge revision MUST memiliki multiple parents |

---

# 7. Conflict Resolution

## 7.1 Conflict Detection

`[REV-CFL-1]` Conflict terdeteksi ketika kedua branches memodifikasi Cell yang sama dengan cara berbeda.

`[REV-CFL-2]` Conflict MUST dilaporkan secara eksplisit.

## 7.2 Conflict Structure

```rust
struct MergeConflict {
    conflicts: Vec<CellConflict>,
}

struct CellConflict {
    cell_id: CellId,
    base: Option<Cell>,
    branch_a: Option<Cell>,
    branch_b: Option<Cell>,
    conflict_type: ConflictType,
}

enum ConflictType {
    BothModified,      // Both changed to different values
    ModifyDelete,      // One modified, other deleted
    BothAdded,         // Both added different Cells
}
```

## 7.3 Conflict Resolution Strategies

`[REV-CFL-3]` Conflict resolution strategies:

| Strategy | Deskripsi | Use Case |
|---|---|---|
| Manual | User memilih secara manual | Conflict kompleks |
| PreferA | Pilih branch A | A lebih authoritative |
| PreferB | Pilih branch B | B lebih authoritative |
| PreferLatest | Pilih revision lebih baru | Temporal precedence |
| MergeSemantics | Gabungkan secara semantic | Cells dapat digabung |
| CreateNew | Buat Cell baru dari kedua | Kompromi |

## 7.4 Conflict Resolution Protocol

```pseudo
function resolve_conflicts(conflicts: MergeConflict, strategy: ResolutionStrategy) -> Result<Vec<Cell>, ResolutionError>:
    resolved = []
    
    for conflict in conflicts.conflicts:
        match strategy:
            case Manual:
                // Prompt user
                choice = prompt_user(conflict)
                resolved.append(choice)
            
            case PreferA:
                resolved.append(conflict.branch_a)
            
            case PreferB:
                resolved.append(conflict.branch_b)
            
            case PreferLatest:
                latest = max_by_timestamp(conflict.branch_a, conflict.branch_b)
                resolved.append(latest)
            
            case MergeSemantics:
                merged = semantic_merge(conflict)
                if merged is None:
                    return Err(ResolutionError::CannotMerge(conflict))
                resolved.append(merged)
            
            case CreateNew:
                new_cell = create_from_both(conflict.branch_a, conflict.branch_b)
                resolved.append(new_cell)
    
    return Ok(resolved)
```

## 7.5 Conflict Resolution Invariants

| ID | Invariant |
|---|---|
| REV-CFL-INV-1 | Conflict MUST dilaporkan eksplisit |
| REV-CFL-INV-2 | Conflict MUST NOT di-resolve secara silent |
| REV-CFL-INV-3 | Conflict resolution MUST deterministic |
| REV-CFL-INV-4 | Conflict resolution MUST menghasilkan revision baru |
| REV-CFL-INV-5 | Unresolved conflict MUST membatalkan merge |

---

# 8. Rollback

## 8.1 Rollback Definition

`[REV-RB-1]` Rollback mengubah active revision ke revision sebelumnya.

`[REV-RB-2]` Rollback MUST NOT menghapus revision yang sudah ada.

## 8.2 Rollback Protocol

```pseudo
function rollback(target: RevisionId) -> Result<()>:
    // 1. Validate target exists
    if not revision_exists(target):
        return Err(Error::RevisionNotFound)
    
    // 2. Validate target is ancestor or valid revision
    if not is_valid_rollback_target(target):
        return Err(Error::InvalidRollbackTarget)
    
    // 3. Set active revision
    set_active_revision(target)
    
    // 4. Invalidate resolution cache
    invalidate_resolution_cache()
    
    // 5. Rebuild effective graph
    rebuild_effective_graph(target)
    
    return Ok(())
```

## 8.3 Rollback Example

```text
Rev 0 → Rev 1 → Rev 2 (broken) → Rev 3

Rollback to Rev 1:
  Active revision = Rev 1
  Rev 2, Rev 3 tetap ada (immutable)
  Tidak ada data yang dihapus
  
  Effective graph = resolve(Rev 1)
```

## 8.4 Rollback Invariants

| ID | Invariant |
|---|---|
| REV-RB-INV-1 | Rollback MUST NOT menghapus revision |
| REV-RB-INV-2 | Rollback MUST mengubah active revision |
| REV-RB-INV-3 | Rollback MUST invalidate resolution cache |
| REV-RB-INV-4 | Rollback MUST O(1) (hanya pointer change) |
| REV-RB-INV-5 | Rollback MUST reversible |

---

# 9. Garbage Collection Interaction

## 9.1 GC Overview

`[REV-GC-1]` GC menghapus Tiles yang tidak lagi reachable dari revision roots.

`[REV-GC-2]` GC MUST berbasis reachability, bukan reference counting.

## 9.2 GC Algorithm

```pseudo
function gc() -> GcReport:
    // 1. Identify active revision roots
    roots = get_active_revision_roots()
    
    // 2. Reachability traversal
    reachable_tiles = set()
    reachable_cells = set()
    
    for root in roots:
        walk_revision(root, |rev| {
            for cell_id in rev.changed_cells:
                reachable_cells.add(cell_id)
                for tile_id in get_tiles(cell_id):
                    reachable_tiles.add(tile_id)
        })
    
    // 3. Mark phase: identify all Tiles in store
    all_tiles = list_all_tiles()
    
    // 4. Sweep phase: identify unreferenced Tiles
    unreferenced = all_tiles - reachable_tiles
    
    // 5. Grace period check
    to_reclaim = filter(unreferenced, |tile| {
        tile.unreferenced_since < now() - grace_period
    })
    
    // 6. Reclaim
    for tile_id in to_reclaim:
        reclaim_tile(tile_id)
    
    return GcReport {
        reachable_tiles: reachable_tiles.len(),
        unreferenced_tiles: unreferenced.len(),
        reclaimed_tiles: to_reclaim.len(),
        bytes_reclaimed: sum(to_reclaim.sizes),
    }
```

## 9.3 GC and Revisions

`[REV-GC-3]` GC MUST NOT menghapus Tile yang reachable dari revision root mana pun.

`[REV-GC-4]` Revision yang dihapus (deleted branch) MAY membuat Tiles menjadi unreachable.

`[REV-GC-5]` GC MUST memiliki grace period untuk mencegah penghapusan prematur.

## 9.4 GC Grace Period

`[REV-GC-6]` Default grace period: 7 hari.

`[REV-GC-7]` Grace period MUST configurable.

## 9.5 GC Invariants

| ID | Invariant |
|---|---|
| REV-GC-INV-1 | GC MUST berbasis reachability |
| REV-GC-INV-2 | GC MUST NOT menghapus Tile reachable |
| REV-GC-INV-3 | GC MUST memiliki grace period |
| REV-GC-INV-4 | GC MUST two-phase (mark-sweep) |
| REV-GC-INV-5 | GC MUST NOT mengganggu active revisions |

---

# 10. Learning Isolation

## 10.1 Definition

`[REV-ISO-1]` Learning isolation menjamin bahwa learning update hanya mempengaruhi Cell yang berubah.

`[REV-ISO-2]` Learning isolation adalah mekanisme utama untuk mencegah catastrophic forgetting.

## 10.2 Isolation Mechanism

`[REV-ISO-3]` Learning isolation dicapai melalui:

1. **Cell immutability**: Cell lama tidak dimodifikasi
2. **Content addressing**: Cell baru memiliki identity baru
3. **Revision delta**: Hanya Cell yang berubah yang dicatat
4. **Reference sharing**: Cell yang tidak berubah tetap direfer

## 10.3 Isolation Example

```text
Before learning:
  Cell A (v1) ──► Tiles [T1, T2, T3]
  Cell B (v1) ──► Tiles [T4, T5]
  Cell C (v1) ──► Tiles [T6]

Learning: Refine Cell B

After learning:
  Cell A (v1) ──► Tiles [T1, T2, T3]  (unchanged)
  Cell B (v1) ──► Tiles [T4, T5]       (old, still exists)
  Cell B (v2) ──► Tiles [T7, T8]       (new)
  Cell C (v1) ──► Tiles [T6]           (unchanged)

Revision delta:
  changed_cells: [Cell B]
  cells_replaced: [(Cell B v1, Cell B v2)]
  
  Cell A: unchanged → still referenced
  Cell C: unchanged → still referenced
```

## 10.4 Isolation Invariants

| ID | Invariant |
|---|---|
| REV-ISO-INV-1 | Learning MUST hanya mempengaruhi Cell yang berubah |
| REV-ISO-INV-2 | Cell yang tidak berubah MUST tetap direfer |
| REV-ISO-INV-3 | Cell lama MUST NOT dimodifikasi |
| REV-ISO-INV-4 | Learning cost MUST O(affected_cells) |
| REV-ISO-INV-5 | Learning MUST NOT menyebabkan side effects ke Cell lain |

---

# 11. Specialization

## 11.1 Definition

`[REV-SPC-1]` Specialization membuat domain-specific variant tanpa duplikasi base Cells.

`[REV-SPC-2]` Specialization menggunakan branching + CellCreate/CellRefine.

## 11.2 Specialization Protocol

```pseudo
function specialize(base: RevisionId, domain: String, training_data: DataSet) -> RevisionId:
    // 1. Create branch
    branch = branch(base, domain)
    
    // 2. Set branch as active
    set_active_revision(branch)
    
    // 3. Learn from domain data
    for batch in training_data:
        // Identify what needs to change
        changes = analyze_batch(batch)
        
        // Apply learning updates
        for change in changes:
            match change:
                case NewKnowledge(spec):
                    cell_create(spec, context)
                case RefineKnowledge(cell_id, updates):
                    cell_refine(cell_id, updates, context)
                case NewMemory(entry):
                    memory_write(entry, context)
    
    // 4. Return specialization revision
    return active_revision()
```

## 11.3 Specialization Storage

`[REV-SPC-3]` Specialization storage:

```text
Base Model (Rev 0):
  Cells: A, B, C, D, E, F, G, H
  Tiles: T1-T100 (100 tiles)
  Storage: 100 tiles

Coding Specialization (Rev N):
  Base Cells: A, B, C, D, E, F, G, H (shared, no copy)
  New Cells: X, Y (coding-specific)
  New Tiles: T101-T110 (10 tiles)
  Additional storage: 10 tiles only

Reasoning Specialization (Rev M):
  Base Cells: A, B, C, D, E, F, G, H (shared, no copy)
  New Cells: Z (reasoning-specific)
  New Tiles: T111-T115 (5 tiles)
  Additional storage: 5 tiles only
```

## 11.4 Multiple Specializations

`[REV-SPC-4]` Multiple specializations MAY share base Cells.

```text
Base (Rev 0)
    │
    ├── Coding (Rev N):
    │     Base Cells: shared
    │     Coding Cells: X, Y
    │
    ├── Reasoning (Rev M):
    │     Base Cells: shared
    │     Reasoning Cells: Z
    │
    └── Medical (Rev K):
          Base Cells: shared
          Medical Cells: W
```

## 11.5 Specialization Invariants

| ID | Invariant |
|---|---|
| REV-SPC-INV-1 | Specialization MUST NOT menyalin base Cells |
| REV-SPC-INV-2 | Specialization MUST menggunakan branching |
| REV-SPC-INV-3 | Specialization storage MUST O(domain_delta) |
| REV-SPC-INV-4 | Multiple specializations MUST share base Cells |
| REV-SPC-INV-5 | Specialization MUST reversible (rollback) |

---

# 12. Catastrophic-Forgetting Prevention

## 12.1 Definition

`[REV-CFP-1]` Catastrophic forgetting adalah hilangnya knowledge lama saat belajar knowledge baru.

`[REV-CFP-2]` CNWS MUST mencegah catastrophic forgetting melalui learning isolation.

## 12.2 Prevention Mechanisms

`[REV-CFP-3]` Mekanisme pencegahan catastrophic forgetting:

| Mechanism | Deskripsi |
|---|---|
| Cell immutability | Cell lama tidak dimodifikasi |
| Learning isolation | Hanya Cell relevan yang berubah |
| Revision history | Knowledge lama tetap accessible |
| Verification | Post-learning verification |
| Rollback | Dapat kembali ke state sebelumnya |

## 12.3 Post-Learning Verification

`[REV-CFP-4]` Setelah learning, sistem MUST memverifikasi bahwa knowledge lama tetap berfungsi.

```pseudo
function verify_no_forgetting(before: RevisionId, after: RevisionId, test_suite: TestSuite) -> VerificationResult:
    // 1. Run test suite on before revision
    before_results = run_tests(before, test_suite)
    
    // 2. Run test suite on after revision
    after_results = run_tests(after, test_suite)
    
    // 3. Compare results
    regressions = []
    for test in test_suite:
        if before_results[test].passed and not after_results[test].passed:
            regressions.append(test)
    
    if regressions is not empty:
        return VerificationResult::Failed(regressions)
    
    return VerificationResult::Passed
```

## 12.4 Verification Actions

`[REV-CFP-5]` Jika verification gagal:

| Action | Deskripsi |
|---|---|
| Reject learning | Batalkan revision |
| Rollback | Kembali ke revision sebelumnya |
| Alert | Laporkan ke user |
| Retry with isolation | Coba lagi dengan isolasi lebih ketat |

## 12.5 Catastrophic-Forgetting Prevention Invariants

| ID | Invariant |
|---|---|
| REV-CFP-INV-1 | Learning MUST NOT memodifikasi Cell yang tidak relevan |
| REV-CFP-INV-2 | Post-learning verification MUST dilakukan |
| REV-CFP-INV-3 | Regression MUST terdeteksi |
| REV-CFP-INV-4 | Failed verification MUST membatalkan learning |
| REV-CFP-INV-5 | Knowledge lama MUST tetap accessible |

---

# 13. Affected-Cell Accounting

## 13.1 Definition

`[REV-ACC-1]` Affected-cell accounting tracking Cell mana yang berubah dalam setiap learning update.

`[REV-ACC-2]` Learning cost MUST O(affected_cells), bukan O(total_cells).

## 13.2 Accounting Structure

```rust
struct AffectedCellAccounting {
    revision_id: RevisionId,
    total_cells: u64,
    affected_cells: u64,
    affected_ratio: f64,
    
    // Breakdown by update type
    cells_created: u64,
    cells_refined: u64,
    cells_replaced: u64,
    memory_added: u64,
    routing_updated: u64,
    compositions_added: u64,
    
    // Cost accounting
    estimated_cost: CostBreakdown,
}

struct CostBreakdown {
    storage_bytes: u64,
    compute_flops: u64,
    io_bytes: u64,
    wall_time_us: u64,
}
```

## 13.3 Accounting Protocol

```pseudo
function account_affected_cells(revision: Revision) -> AffectedCellAccounting:
    total_cells = count_total_cells()
    
    affected = revision.changed_cells.len()
    + revision.changed_memory.len()
    + revision.changed_routing.len()
    + revision.changed_compositions.len()
    
    ratio = affected / total_cells
    
    return AffectedCellAccounting {
        revision_id: revision.id,
        total_cells: total_cells,
        affected_cells: affected,
        affected_ratio: ratio,
        cells_created: count_created(revision),
        cells_refined: count_refined(revision),
        cells_replaced: count_replaced(revision),
        memory_added: revision.changed_memory.len(),
        routing_updated: revision.changed_routing.len(),
        compositions_added: revision.changed_compositions.len(),
        estimated_cost: estimate_cost(revision),
    }
```

## 13.4 Accounting Invariants

| ID | Invariant |
|---|---|
| REV-ACC-INV-1 | Affected cells MUST tracked |
| REV-ACC-INV-2 | Learning cost MUST O(affected_cells) |
| REV-ACC-INV-3 | Affected ratio MUST dilaporkan |
| REV-ACC-INV-4 | Cost breakdown MUST tersedia |
| REV-ACC-INV-5 | Accounting MUST tersedia untuk observability |

## 13.5 Accounting Example

```text
Model: 100,000 Cells total

Learning update: Refine 50 Cells for coding task

Affected-Cell Accounting:
  total_cells: 100,000
  affected_cells: 50
  affected_ratio: 0.0005 (0.05%)
  
  cells_created: 0
  cells_refined: 50
  cells_replaced: 50
  memory_added: 10
  routing_updated: 50
  compositions_added: 2
  
  Cost:
    storage_bytes: 250 MB (50 cells × 5 MB avg)
    compute_flops: 10 GFLOP (training)
    io_bytes: 500 MB
    wall_time_us: 30,000,000 (30 seconds)

Compare to full retraining:
  affected_cells: 100,000 (100%)
  cost: ~2000× higher
```

---

# 14. Revision Lifecycle

## 14.1 Revision State Machine

```text
┌──────────┐   create   ┌──────────┐   commit   ┌──────────┐
│  DRAFT   │──────────►│ STAGED   │──────────►│ COMMITTED│
└──────────┘           └──────────┘           └────┬─────┘
                                                   │
                                    ┌──────────────┼──────────────┐
                                    │              │              │
                                    ▼              ▼              ▼
                              ┌──────────┐  ┌──────────┐  ┌──────────┐
                              │  ACTIVE  │  │SUPERSEDED│  │ DELETED  │
                              └──────────┘  └──────────┘  └──────────┘
                                    │              │              │
                                    └──────────────┴──────────────┘
                                                   │
                                                   ▼
                                          ┌──────────┐
                                          │ ARCHIVED │
                                          └──────────┘
```

## 14.2 Revision States

| State | Deskripsi |
|---|---|
| DRAFT | Revision sedang dibuat, belum committed |
| STAGED | Revision siap commit, menunggu atomic commit |
| COMMITTED | Revision committed, immutable |
| ACTIVE | Revision aktif saat ini |
| SUPERSEDED | Revision tidak lagi aktif (ada revision lebih baru) |
| DELETED | Revision ditandai untuk dihapus (masih ada untuk GC) |
| ARCHIVED | Revision diarsipkan, tidak aktif |

## 14.3 Revision Lifecycle Transitions

`[REV-LIFE-1]` Revision lifecycle transitions:

| From | To | Trigger |
|---|---|---|
| DRAFT | STAGED | Changes complete |
| STAGED | COMMITTED | Atomic commit successful |
| COMMITTED | ACTIVE | set_active_revision() |
| ACTIVE | SUPERSEDED | New revision committed |
| COMMITTED | DELETED | delete_revision() |
| SUPERSEDED | ARCHIVED | archive_revision() |
| DELETED | (GC) | GC reclaims if unreachable |

## 14.4 Revision Lifecycle Invariants

| ID | Invariant |
|---|---|
| REV-LIFE-INV-1 | COMMITTED revision MUST immutable |
| REV-LIFE-INV-2 | ACTIVE revision MUST hanya satu |
| REV-LIFE-INV-3 | DELETED revision MUST NOT dihapus fisik sampai GC |
| REV-LIFE-INV-4 | Lifecycle transitions MUST valid |

---

# 15. Error Handling

## 15.1 Revision Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-REV-NOTFOUND` | Revision not found |
| `CNWS-E-REV-INVALID` | Invalid revision |
| `CNWS-E-REV-CONFLICT` | Merge conflict |
| `CNWS-E-REV-COMMIT` | Commit failed |
| `CNWS-E-REV-ROLLBACK` | Rollback failed |
| `CNWS-E-REV-GC` | GC failed |
| `CNWS-E-REV-BRANCH` | Branch failed |
| `CNWS-E-REV-MERGE` | Merge failed |

## 15.2 Learning Error Codes

| Code | Meaning |
|---|---|
| `CNWS-E-LRN-CELLCREATE` | CellCreate failed |
| `CNWS-E-LRN-CELLREFINE` | CellRefine failed |
| `CNWS-E-LRN-MEMORYWRITE` | MemoryWrite failed |
| `CNWS-E-LRN-ROUTING` | RoutingUpdate failed |
| `CNWS-E-LRN-VERIFICATION` | Post-learning verification failed |
| `CNWS-E-LRN-FORGETTING` | Catastrophic forgetting detected |

## 15.3 Error Severity

| Severity | Examples | Action |
|---|---|---|
| Fatal | REV-COMMIT, LRN-FORGETTING | Abort, require intervention |
| Recoverable | REV-CONFLICT, LRN-VERIFICATION | Retry or manual resolution |
| Warning | REV-GC partial | Log and continue |

---

# 16. Final Revision & Learning Contract

## 16.1 Ringkasan Keputusan Revision & Learning

| ID | Keputusan |
|---|---|
| REV-F01 | Revision adalah learning-integrated versioning, bukan Git-like. |
| REV-F02 | Setiap learning update menghasilkan revision baru. |
| REV-F03 | Revision delta pada level Cell/Tile. |
| REV-F04 | Revision immutable setelah committed. |
| REV-F05 | Revision lineage membentuk DAG. |
| REV-F06 | Model efektif adalah hasil resolusi revision + ancestors. |
| REV-F07 | Resolusi DAG di-cache untuk O(1) runtime lookup. |
| REV-F08 | CellCreate menambahkan Cell baru. |
| REV-F09 | CellRefine membuat Cell version baru. |
| REV-F10 | MemoryWrite menambahkan memory entry. |
| REV-F11 | RoutingUpdate mengubah routing. |
| REV-F12 | CompositionCache menyimpan composition pattern. |
| REV-F13 | Branching tidak menyalin Cells. |
| REV-F14 | Merge menggunakan three-way merge pada level Cell. |
| REV-F15 | Conflict resolution pada level Cell/Tile. |
| REV-F16 | Rollback tidak menghapus revision. |
| REV-F17 | GC berbasis reachability dari revision roots. |
| REV-F18 | Learning isolation: hanya Cell yang berubah yang dimodifikasi. |
| REV-F19 | Specialization tanpa duplikasi base Cells. |
| REV-F20 | Catastrophic forgetting dicegah melalui learning isolation. |

## 16.2 Revision & Learning Invariants

| ID | Invariant |
|---|---|
| REV-INV-1 | Revision MUST immutable setelah committed. |
| REV-INV-2 | Revision commit MUST atomic. |
| REV-INV-3 | Revision MUST menyimpan delta, bukan full state. |
| REV-INV-4 | Setiap learning update MUST menghasilkan revision baru. |
| REV-INV-5 | Learning cost MUST O(affected_cells). |
| REV-INV-6 | Branching MUST NOT menyalin Cells. |
| REV-INV-7 | Merge MUST menggunakan three-way merge. |
| REV-INV-8 | Conflict MUST dilaporkan eksplisit. |
| REV-INV-9 | Rollback MUST NOT menghapus revision. |
| REV-INV-10 | GC MUST berbasis reachability. |
| REV-INV-11 | GC MUST NOT menghapus Tile reachable. |
| REV-INV-12 | Learning MUST hanya mempengaruhi Cell yang berubah. |
| REV-INV-13 | Cell yang tidak berubah MUST tetap direfer. |
| REV-INV-14 | Specialization MUST tanpa duplikasi base Cells. |
| REV-INV-15 | Post-learning verification MUST dilakukan. |
| REV-INV-16 | Catastrophic forgetting MUST dicegah. |
| REV-INV-17 | Affected-cell accounting MUST tracked. |
| REV-INV-18 | Revision lifecycle MUST dipatuhi. |
| REV-INV-19 | ACTIVE revision MUST hanya satu. |
| REV-INV-20 | Revision DAG adalah learning-integrated versioning. |

## 16.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Revision & Learning final dan mengikat** untuk CNWS. Ia mendefinisikan bagaimana Revision DAG dan Learning terintegrasi, dari revision creation hingga learning isolation, dari branching hingga catastrophic-forgetting prevention.

Revision DAG CNWS **bukan Git-like versioning**. Ia adalah **learning-integrated versioning** di mana setiap learning update menghasilkan revision baru, dan setiap revision merepresentasikan state intelligence yang immutable.

Seluruh implementasi Revision Manager, Learning Engine, GC, dan Specialization CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan Revision & Learning yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN REVISION & LEARNING SPECIFICATION**
