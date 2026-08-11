# CNWS
## Compatibility & Migration Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Compatibility & Migration Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (COMPATIBILITY SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS .cd Format Spec; CNWS Cell & Schema Spec |
| Hulu ke | Implementasi migration tooling, upgrade procedures, legacy converters |
| Otoritas | Spesifikasi tunggal untuk compatibility dan migration CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract     .cd Format Spec      Compatibility & Migration Spec   Implementation
─────────────────────    ────────────────     ──────────────────────────────   ─────────────
Version invariants   ──► format_version   ──► Compatibility model          ──► Migration Engine
Schema stability         Segment layout        Migration procedures            Version Detector
Backward compat          Cell schema           Legacy migration                Schema Migrator
                         Revision format       Forward incompatibility         Legacy Converter
```

`[COMPAT-DOC-1]` Dokumen ini mendefinisikan **bagaimana format CNWS berkembang dan bagaimana migrasi antar versi dilakukan**.

`[COMPAT-DOC-2]` Format CNWS akan berkembang. Dokumen ini menjamin bahwa evolusi tersebut terkelola, aman, dan tidak menyebabkan data loss.

`[COMPAT-DOC-3]` Jika terjadi konflik dengan spesifikasi lain untuk hal behavior, spesifikasi tersebut menang. Untuk hal compatibility rules dan migration procedures, dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-COMPAT-01 | Format version menggunakan semver (major.minor.patch). |
| DF-COMPAT-02 | Backward compatibility: reader MUST menerima minor ≤ versi didukung. |
| DF-COMPAT-03 | Forward incompatibility: reader MUST menolak major lebih tinggi. |
| DF-COMPAT-04 | Migration MUST atomic (berhasil penuh atau gagal penuh). |
| DF-COMPAT-05 | Migration MUST dapat di-rollback. |
| DF-COMPAT-06 | Legacy CNWS-X migration via `cnws migrate --from cnws-x`. |
| DF-COMPAT-07 | Legacy LATTICE migration via `cnws migrate --from lattice`. |
| DF-COMPAT-08 | Deprecation: minimum 2 minor versions notice. |
| DF-COMPAT-09 | Schema migration MUST deterministic. |
| DF-COMPAT-10 | Migration MUST diverifikasi dengan integrity check. |
| DF-COMPAT-11 | CellType additions MUST backward compatible. |
| DF-COMPAT-12 | CellType removals MUST major version bump. |
| DF-COMPAT-13 | Revision format MUST backward compatible untuk minor. |
| DF-COMPAT-14 | Migration tooling MUST tersedia untuk setiap major version transition. |

---

# 1. Executive Summary

## 1.1 Compatibility Philosophy

`[COMPAT-EXEC-1]` Prinsip compatibility CNWS:

1. **Stable core**: format inti (BLAKE3-256, Tile immutability, .cd structure) stabil.
2. **Controlled evolution**: perubahan melalui versioning yang terkelola.
3. **No data loss**: migrasi MUST NOT menyebabkan data loss.
4. **Backward compatible**: versi baru dapat membaca data versi lama (dalam major yang sama).
5. **Forward incompatible**: versi lama tidak dapat membaca data versi baru (major lebih tinggi).
6. **Explicit migration**: migrasi antar major version eksplisit dan terkontrol.

## 1.2 Version Landscape

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS VERSION LANDSCAPE                    │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  CNWS v1.x (Current Stable)                                 │
│  ├── v1.0.0  Initial release                               │
│  ├── v1.1.0  Minor additions (backward compatible)         │
│  ├── v1.2.0  Minor additions (backward compatible)         │
│  └── ...                                                   │
│                                                             │
│  CNWS v2.x (Future Major)                                   │
│  ├── v2.0.0  Breaking changes (migration required)         │
│  └── ...                                                   │
│                                                             │
│  Legacy Formats                                             │
│  ├── CNWS-X v0.x  Pre-unification infrastructure           │
│  └── LATTICE v0.x  Pre-unification intelligence            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 1.3 Compatibility Guarantees

`[COMPAT-EXEC-2]` Jaminan compatibility:

| Guarantee | Scope |
|---|---|
| Read backward compatibility | Dalam major version yang sama |
| Write stability | Canonical serialization deterministik |
| Migration safety | Atomic, rollback-able, verified |
| Data integrity | BLAKE3-256 verification sebelum dan sesudah migrasi |
| Legacy support | CNWS-X dan LATTICE migration paths |

---

# 2. Version Model

## 2.1 Version Dimensions

`[COMPAT-VER-1]` CNWS memiliki beberapa dimensi version:

| Dimension | Location | Purpose |
|---|---|---|
| Format Version | SUPERBLOCK, MANIFEST.cd | .cd file format |
| Schema Version | MANIFEST.cd | Manifest schema |
| Cell Schema Version | Cell metadata | Cell structure |
| Revision Format Version | Revision files | Revision structure |
| API Version | Runtime API | Programmatic interface |
| Tool Version | CLI, SDK | Tooling |

## 2.2 Format Version

`[COMPAT-VER-2]` Format version menggunakan **semver** (major.minor.patch).

```rust
struct FormatVersion {
    major: u32,    // Breaking changes
    minor: u32,    // Backward-compatible additions
    patch: u32,    // Bug fixes
}
```

`[COMPAT-VER-3]` Format version disimpan di:

1. **SUPERBLOCK**: `version_major`, `version_minor` (binary)
2. **MANIFEST.cd**: `format_version` (string "major.minor.patch")

```json
{
  "format_version": "1.0.0",
  ...
}
```

## 2.3 Version Semantics

`[COMPAT-VER-4]` Version semantics:

| Change Type | Version Bump | Compatibility |
|---|---|---|
| Bug fix, clarification | patch (x.y.Z) | Fully compatible |
| New optional field | minor (x.Y.0) | Backward compatible |
| New CellType | minor (x.Y.0) | Backward compatible |
| New metadata field | minor (x.Y.0) | Backward compatible |
| Remove field | major (X.0.0) | Breaking |
| Change field semantics | major (X.0.0) | Breaking |
| Change hash algorithm | major (X.0.0) | Breaking |
| Change Tile layout | major (X.0.0) | Breaking |

## 2.4 Version Detection

`[COMPAT-VER-5]` Version detection saat membuka store:

```pseudo
function detect_version(store_path) -> FormatVersion:
    // Read SUPERBLOCK
    superblock = read_superblock(store_path)
    
    // Validate magic
    if superblock.magic != "CNWSSB01":
        return Err(Error::InvalidSuperblock)
    
    // Extract version
    version = FormatVersion {
        major: superblock.version_major,
        minor: superblock.version_minor,
        patch: 0,  // patch not in SUPERBLOCK
    }
    
    // Read MANIFEST.cd for patch version
    manifest = read_manifest(store_path)
    version.patch = parse_patch(manifest.format_version)
    
    return version
```

## 2.5 Version Invariants

| ID | Invariant |
|---|---|
| COMPAT-VER-INV-1 | Format version MUST semver |
| COMPAT-VER-INV-2 | Version MUST tersimpan di SUPERBLOCK dan MANIFEST.cd |
| COMPAT-VER-INV-3 | Version MUST terdeteksi saat open |
| COMPAT-VER-INV-4 | Version mismatch MUST menghasilkan error eksplisit |

---

# 3. Compatibility Model

## 3.1 Backward Compatibility

`[COMPAT-MODEL-1]` **Backward compatibility**: versi baru dapat membaca data versi lama.

```text
Reader v1.2.0 membaca data v1.0.0  → OK (backward compatible)
Reader v1.2.0 membaca data v1.1.0  → OK (backward compatible)
Reader v1.2.0 membaca data v1.2.0  → OK (same version)
```

`[COMPAT-MODEL-2]` Backward compatibility MUST dijamin dalam major version yang sama.

## 3.2 Forward Incompatibility

`[COMPAT-MODEL-3]` **Forward incompatibility**: versi lama tidak dapat membaca data versi baru.

```text
Reader v1.0.0 membaca data v1.1.0  → MAY (jika tidak ada field baru yang required)
Reader v1.0.0 membaca data v2.0.0  → MUST reject (forward incompatible)
Reader v1.2.0 membaca data v2.0.0  → MUST reject (forward incompatible)
```

`[COMPAT-MODEL-4]` Reader MUST menolak data dengan major version lebih tinggi.

## 3.3 Compatibility Matrix

`[COMPAT-MODEL-5]` Compatibility matrix:

| Reader \ Data | v1.0 | v1.1 | v1.2 | v2.0 |
|---|---|---|---|---|
| **v1.0** | ✅ | ⚠️ | ⚠️ | ❌ |
| **v1.1** | ✅ | ✅ | ⚠️ | ❌ |
| **v1.2** | ✅ | ✅ | ✅ | ❌ |
| **v2.0** | ✅* | ✅* | ✅* | ✅ |

Legend:
- ✅ Fully compatible
- ⚠️ Partial (new fields ignored)
- ❌ Rejected (forward incompatible)
- ✅* Requires migration

## 3.4 Compatibility Rules

### 3.4.1 Reader Rules

`[COMPAT-MODEL-6]` Reader rules:

```pseudo
function check_read_compatibility(reader_version, data_version) -> CompatibilityResult:
    // Major version check
    if data_version.major > reader_version.major:
        return Reject(ForwardIncompatible {
            data_version,
            reader_version,
            message: "Data version is newer than reader. Upgrade required."
        })
    
    if data_version.major < reader_version.major:
        // Older major version: may need migration
        return RequiresMigration {
            from: data_version,
            to: reader_version,
        }
    
    // Same major version
    if data_version.minor > reader_version.minor:
        // Newer minor: partial compatibility
        return PartialCompatible {
            unknown_fields_will_be_ignored: true,
        }
    
    // Compatible
    return Compatible
```

### 3.4.2 Writer Rules

`[COMPAT-MODEL-7]` Writer rules:

```pseudo
function check_write_compatibility(writer_version, target_version) -> CompatibilityResult:
    // Writer MUST NOT write data with version higher than itself
    if target_version > writer_version:
        return Reject(CannotWriteNewerVersion)
    
    // Writer MUST write its own version
    return Compatible {
        written_version: writer_version,
    }
```

## 3.5 Unknown Field Handling

`[COMPAT-MODEL-8]` Unknown field handling:

| Scenario | Action |
|---|---|
| Reader encounters unknown JSON field | Ignore, preserve in round-trip |
| Reader encounters unknown CellType | Reject Cell, log warning |
| Reader encounters unknown metadata key | Ignore, preserve |
| Reader encounters unknown binary structure | Reject with error |

`[COMPAT-MODEL-9]` Unknown fields SHOULD dipertahankan saat round-trip untuk memungkinkan future compatibility.

## 3.6 Compatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-MODEL-INV-1 | Backward compatibility MUST dalam major yang sama |
| COMPAT-MODEL-INV-2 | Forward incompatibility MUST enforced |
| COMPAT-MODEL-INV-3 | Major version lebih tinggi MUST ditolak |
| COMPAT-MODEL-INV-4 | Unknown fields SHOULD dipertahankan |
| COMPAT-MODEL-INV-5 | Compatibility check MUST saat open |

---

# 4. .cd Version Migration

## 4.1 Migration Triggers

`[COMPAT-MIG-1]` Migration dipicu ketika:

1. Store version < Reader version (major berbeda)
2. Explicit migration command
3. Upgrade procedure

## 4.2 Migration Detection

```pseudo
function detect_migration_needed(store_path) -> MigrationInfo:
    store_version = detect_version(store_path)
    reader_version = current_version()
    
    if store_version.major < reader_version.major:
        return MigrationInfo {
            needed: true,
            from: store_version,
            to: reader_version,
            migration_path: compute_migration_path(store_version, reader_version),
        }
    
    return MigrationInfo { needed: false }
```

## 4.3 Migration Path

`[COMPAT-MIG-2]` Migration path untuk multi-version jumps:

```text
v1.0 → v1.1 → v1.2 → v2.0 → v2.1

Migration from v1.0 to v2.1:
  Step 1: v1.0 → v1.1 (minor migration)
  Step 2: v1.1 → v1.2 (minor migration)
  Step 3: v1.2 → v2.0 (major migration)
  Step 4: v2.0 → v2.1 (minor migration)
```

`[COMPAT-MIG-3]` Migration MUST dilakukan step-by-step untuk multi-version jumps.

## 4.4 Migration Procedure

```pseudo
function migrate_store(store_path, target_version) -> MigrationResult:
    // Step 1: Pre-migration validation
    validate_store(store_path)?
    current_version = detect_version(store_path)
    
    // Step 2: Backup
    backup_path = create_backup(store_path)
    
    // Step 3: Compute migration path
    migration_path = compute_migration_path(current_version, target_version)
    
    // Step 4: Apply migrations step-by-step
    for migration in migration_path:
        log("Applying migration: {} → {}", migration.from, migration.to)
        
        // Apply migration
        apply_migration(store_path, migration)?
        
        // Verify after each step
        verify_store(store_path)?
        
        // Update version
        update_version(store_path, migration.to)?
    
    // Step 5: Final verification
    final_verification(store_path)?
    
    // Step 6: Cleanup (optional, after verification)
    // Keep backup for safety
    
    return MigrationResult {
        from: current_version,
        to: target_version,
        steps_applied: migration_path.len(),
        backup_path: backup_path,
        success: true,
    }
```

## 4.5 Atomic Migration

`[COMPAT-MIG-4]` Migration MUST atomic:

```text
Migration State Machine:
  ┌──────────┐   start   ┌──────────┐
  │ ORIGINAL │─────────►│ MIGRATING│
  └──────────┘          └────┬─────┘
                             │
                    ┌────────┼────────┐
                    │        │        │
                    ▼        ▼        ▼
              ┌──────────┐ ┌────┐ ┌──────────┐
              │ MIGRATED │ │FAIL│ │ ROLLBACK │
              └──────────┘ └────┘ └──────────┘
                    │                  │
                    ▼                  ▼
              ┌──────────┐      ┌──────────┐
              │ VERIFIED │      │ ORIGINAL │
              └──────────┘      └──────────┘
```

`[COMPAT-MIG-5]` Jika migration gagal:

1. Stop migration.
2. Restore dari backup.
3. Report error.
4. Store kembali ke state original.

## 4.6 Migration Commands

```bash
# Check if migration needed
cnws migrate check /data/model.cd

# Dry-run migration (show what would be done)
cnws migrate /data/model.cd --dry-run

# Perform migration
cnws migrate /data/model.cd --to 2.0.0

# Verify migration
cnws migrate verify /data/model.cd

# Rollback migration (if failed)
cnws migrate rollback /data/model.cd --backup /backup/pre-migration
```

## 4.7 Migration Invariants

| ID | Invariant |
|---|---|
| COMPAT-MIG-INV-1 | Migration MUST atomic |
| COMPAT-MIG-INV-2 | Migration MUST dapat di-rollback |
| COMPAT-MIG-INV-3 | Migration MUST step-by-step untuk multi-version |
| COMPAT-MIG-INV-4 | Migration MUST diverifikasi setelah setiap step |
| COMPAT-MIG-INV-5 | Backup MUST dibuat sebelum migration |
| COMPAT-MIG-INV-6 | Migration MUST deterministic |

---

# 5. Schema Migration

## 5.1 Manifest Schema Migration

`[COMPAT-SCH-1]` Manifest schema migration untuk perubahan struktur MANIFEST.cd.

### 5.1.1 Schema Version Tracking

```json
{
  "format_version": "1.0.0",
  "schema_version": "1.0.0",
  ...
}
```

### 5.1.2 Schema Migration Rules

| Change Type | Migration Action |
|---|---|
| Add optional field | No migration needed (default value) |
| Add required field | Migration required (compute value) |
| Remove field | Migration required (remove from all records) |
| Rename field | Migration required (rename in all records) |
| Change field type | Migration required (transform values) |

### 5.1.3 Schema Migration Example

```pseudo
function migrate_manifest_schema_v1_to_v2(manifest) -> Manifest:
    // v2 adds "memory" section
    if "memory" not in manifest:
        manifest["memory"] = {
            "episodic_entries": 0,
            "semantic_entries": 0,
            "procedural_entries": 0,
            "working_memory_bound_bytes": 268435456,
        }
    
    // v2 adds "routing" section
    if "routing" not in manifest:
        manifest["routing"] = {
            "index_dimensions": 512,
            "index_structure": "HNSW",
            "routing_policy_version": 0,
        }
    
    // v2 renames "columns" to "cells"
    if "columns" in manifest:
        manifest["cells"] = manifest.pop("columns")
    
    // Update schema version
    manifest["schema_version"] = "2.0.0"
    
    return manifest
```

## 5.2 Cell Schema Migration

`[COMPAT-SCH-2]` Cell schema migration untuk perubahan struktur Cell.

### 5.2.1 CellType Migration

```pseudo
function migrate_cell_type_v1_to_v2(cell) -> Cell:
    // v2 adds new CellTypes
    // No migration needed for existing Cells
    
    // v2 changes CUSTOM CellType format
    if cell.cell_type == CUSTOM:
        // Ensure type_string follows new format
        cell.metadata.custom_type = normalize_custom_type(cell.metadata.custom_type)
    
    return cell
```

### 5.2.2 Metadata Migration

```pseudo
function migrate_cell_metadata_v1_to_v2(cell) -> Cell:
    // v2 adds "index_vector" field
    if "index_vector" not in cell:
        cell["index_vector"] = compute_default_index_vector(cell)
    
    // v2 adds "importance_score" field
    if "importance_score" not in cell.metadata:
        cell.metadata["importance_score"] = 0.5
    
    return cell
```

## 5.3 Tile Schema Migration

`[COMPAT-SCH-3]` Tile schema migration untuk perubahan struktur Tile.

```pseudo
function migrate_tile_schema_v1_to_v2(tile) -> Tile:
    // v2 adds "representation_count" field
    if "representation_count" not in tile:
        tile["representation_count"] = len(tile.representations)
    
    // v2 changes compression encoding
    if tile.compression in OLD_COMPRESSION_CODES:
        tile.compression = map_to_new_code(tile.compression)
    
    return tile
```

## 5.4 Index Schema Migration

`[COMPAT-SCH-4]` Index schema migration untuk perubahan struktur index files.

```pseudo
function migrate_index_v1_to_v2(index_path) -> Index:
    // v2 changes index entry size
    old_entries = read_index_v1(index_path)
    new_entries = []
    
    for entry in old_entries:
        new_entry = convert_entry_v1_to_v2(entry)
        new_entries.append(new_entry)
    
    write_index_v2(index_path, new_entries)
    
    return new_entries
```

## 5.5 Schema Migration Invariants

| ID | Invariant |
|---|---|
| COMPAT-SCH-INV-1 | Schema migration MUST deterministic |
| COMPAT-SCH-INV-2 | Schema migration MUST preserve data |
| COMPAT-SCH-INV-3 | Schema migration MUST diverifikasi |
| COMPAT-SCH-INV-4 | Unknown fields SHOULD dipertahankan |
| COMPAT-SCH-INV-5 | Schema version MUST tracked |

---

# 6. Cell Schema Compatibility

## 6.1 CellType Compatibility

`[COMPAT-CELL-1]` CellType compatibility rules:

| Change | Compatibility | Version Bump |
|---|---|---|
| Add new CellType | Backward compatible | minor |
| Remove CellType | Breaking | major |
| Rename CellType | Breaking | major |
| Change CellType discriminant | Breaking | major |
| Add CellType metadata field | Backward compatible | minor |

### 6.1.1 CellType Addition

```pseudo
function add_cell_type(new_type: CellType) -> Result:
    // New CellType MUST use unused discriminant
    if new_type.discriminant in existing_discriminants:
        return Err(Error::DiscriminantConflict)
    
    // New CellType MUST be in reserved range or CUSTOM
    if new_type.discriminant < 0x80 and new_type.discriminant not in reserved_range:
        return Err(Error::InvalidDiscriminant)
    
    // Add to registry
    register_cell_type(new_type)
    
    return Ok
```

### 6.1.2 CellType Deprecation

`[COMPAT-CELL-2]` CellType deprecation:

```text
Deprecation timeline:
  v1.2.0: CellType marked as deprecated (warning on use)
  v1.3.0: CellType still works (warning)
  v2.0.0: CellType removed (migration required)
```

`[COMPAT-CELL-3]` Deprecation MUST memberikan minimum 2 minor versions notice.

## 6.2 Cell Metadata Compatibility

`[COMPAT-CELL-4]` Cell metadata compatibility:

| Change | Compatibility |
|---|---|
| Add optional metadata field | Backward compatible |
| Add required metadata field | Breaking (major) |
| Remove metadata field | Breaking (major) |
| Change metadata field type | Breaking (major) |
| Add attribute key | Backward compatible |
| Remove attribute key | Breaking (major) |

## 6.3 Index Vector Compatibility

`[COMPAT-CELL-5]` Index vector compatibility:

| Change | Compatibility |
|---|---|
| Change dimensions | Breaking (major) |
| Change metric | Breaking (major) |
| Add index vector to Cells without | Backward compatible (minor) |

## 6.4 Cell Compatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-CELL-INV-1 | CellType addition MUST backward compatible |
| COMPAT-CELL-INV-2 | CellType removal MUST major version |
| COMPAT-CELL-INV-3 | Deprecation MUST 2 minor versions notice |
| COMPAT-CELL-INV-4 | Cell metadata addition MUST backward compatible |
| COMPAT-CELL-INV-5 | Index vector dimension change MUST major version |

---

# 7. Revision Compatibility

## 7.1 Revision Format Compatibility

`[COMPAT-REV-1]` Revision format compatibility:

| Change | Compatibility | Version Bump |
|---|---|---|
| Add optional revision field | Backward compatible | minor |
| Add required revision field | Breaking | major |
| Remove revision field | Breaking | major |
| Change revision field type | Breaking | major |
| Add new change type | Backward compatible | minor |

## 7.2 Cross-Version Revision Resolution

`[COMPAT-REV-2]` Cross-version revision resolution:

```pseudo
function resolve_cross_version_revisions(revisions) -> EffectiveGraph:
    // Sort revisions by version
    sorted = sort_by_version(revisions)
    
    // Apply migrations as needed
    effective_graph = EffectiveGraph::new()
    
    for revision in sorted:
        if revision.version < current_version:
            // Migrate revision to current format
            migrated_revision = migrate_revision(revision, current_version)
            effective_graph.apply(migrated_revision)
        else:
            effective_graph.apply(revision)
    
    return effective_graph
```

## 7.3 Revision DAG Migration

`[COMPAT-REV-3]` Revision DAG migration:

```pseudo
function migrate_revision_dag(dag, from_version, to_version) -> DAG:
    // Migrate each revision node
    for revision in dag.nodes:
        revision = migrate_revision(revision, from_version, to_version)
    
    // Rebuild DAG edges
    dag.rebuild_edges()
    
    // Verify DAG integrity
    verify_dag(dag)?
    
    return dag
```

## 7.4 Revision Compatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-REV-INV-1 | Revision format MUST backward compatible untuk minor |
| COMPAT-REV-INV-2 | Cross-version resolution MUST didukung |
| COMPAT-REV-INV-3 | Revision DAG migration MUST preserve history |
| COMPAT-REV-INV-4 | Revision immutability MUST dipertahankan |

---

# 8. Backward Compatibility Rules

## 8.1 What Is Guaranteed

`[COMPAT-BWD-1]` Backward compatibility guarantees:

| Guarantee | Scope |
|---|---|
| Read old data | Dalam major version yang sama |
| Write old format | Tidak dijamin (writer menulis versi sendiri) |
| API compatibility | Minor version additions |
| CLI compatibility | Minor version additions |
| Config compatibility | Minor version additions |

## 8.2 What Is Not Guaranteed

`[COMPAT-BWD-2]` Yang tidak dijamin:

| Non-Guarantee | Reason |
|---|---|
| Write backward compatibility | Writer selalu menulis versi terbaru |
| Cross-major compatibility | Major version breaking |
| Deprecated feature support | Deprecated features dihapus di major |
| Internal format stability | Internal structures dapat berubah |

## 8.3 Deprecation Policy

`[COMPAT-BWD-3]` Deprecation policy:

```text
Deprecation Lifecycle:
  
  v1.0.0: Feature introduced
  v1.2.0: Feature deprecated (warning issued)
  v1.3.0: Feature still works (warning)
  v2.0.0: Feature removed (migration required)
  
  Minimum notice: 2 minor versions
```

`[COMPAT-BWD-4]` Deprecation warning MUST mencakup:

1. Feature yang deprecated
2. Versi deprecation
3. Versi removal
4. Migration path
5. Alternative

## 8.4 Backward Compatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-BWD-INV-1 | Backward compatibility MUST dalam major yang sama |
| COMPAT-BWD-INV-2 | Deprecation MUST 2 minor versions notice |
| COMPAT-BWD-INV-3 | Deprecated features MUST warning |
| COMPAT-BWD-INV-4 | Removal MUST major version |

---

# 9. Forward Incompatibility

## 9.1 What Breaks

`[COMPAT-FWD-1]` Forward incompatibility terjadi ketika:

| Scenario | Result |
|---|---|
| Reader v1.x membaca data v2.x | MUST reject |
| Reader v1.x membaca CellType baru | MAY reject Cell |
| Reader v1.x membaca field baru | Ignore (jika optional) |
| Reader v1.x membaca revision format baru | MUST reject |

## 9.2 Detection

`[COMPAT-FWD-2]` Forward incompatibility detection:

```pseudo
function detect_forward_incompatibility(reader_version, data_version) -> Result:
    if data_version.major > reader_version.major:
        return Err(ForwardIncompatible {
            data_version,
            reader_version,
            message: format!(
                "Data version {}.{}.{} is newer than reader version {}.{}.{}. \
                 Upgrade CNWS to read this data.",
                data_version.major, data_version.minor, data_version.patch,
                reader_version.major, reader_version.minor, reader_version.patch
            ),
            upgrade_required: true,
        })
    
    return Ok
```

## 9.3 Rejection Policy

`[COMPAT-FWD-3]` Rejection policy:

```text
Forward incompatible data detected:
  │
  ├── Log error with details
  ├── Return explicit error to user
  ├── Suggest upgrade path
  └── Do NOT attempt partial read
```

`[COMPAT-FWD-4]` Forward incompatible data MUST NOT dibaca secara partial.

## 9.4 Forward Incompatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-FWD-INV-1 | Major version lebih tinggi MUST ditolak |
| COMPAT-FWD-INV-2 | Rejection MUST eksplisit dengan pesan jelas |
| COMPAT-FWD-INV-3 | Partial read MUST NOT dilakukan |
| COMPAT-FWD-INV-4 | Upgrade path MUST disarankan |

---

# 10. CNWS v1 → v2 Migration

## 10.1 Migration Overview

`[COMPAT-V12-1]` CNWS v1 → v2 migration adalah major version migration.

```text
CNWS v1.x                          CNWS v2.x
─────────────────────              ─────────────────────
format_version: 1.x.x    ──────►   format_version: 2.0.0
schema_version: 1.x.x    ──────►   schema_version: 2.0.0
Cell schema v1           ──────►   Cell schema v2
Revision format v1       ──────►   Revision format v2
Index format v1          ──────►   Index format v2
```

## 10.2 Migration Plan

`[COMPAT-V12-2]` Migration plan:

```text
Phase 1: Preparation
  ├── Backup store
  ├── Validate store integrity
  └── Check disk space

Phase 2: Schema Migration
  ├── Migrate MANIFEST.cd schema
  ├── Migrate Cell schemas
  ├── Migrate Tile schemas
  └── Migrate index formats

Phase 3: Revision Migration
  ├── Migrate revision format
  ├── Rebuild revision DAG
  └── Verify revision resolution

Phase 4: Verification
  ├── Verify all Cell hashes
  ├── Verify all Tile hashes
  ├── Verify manifest hash
  └── Verify revision DAG integrity

Phase 5: Finalization
  ├── Update SUPERBLOCK version
  ├── Update MANIFEST.cd version
  └── Final integrity check
```

## 10.3 Migration Tooling

```bash
# Check migration readiness
cnws migrate check /data/model.cd --to 2.0.0

# Dry-run migration
cnws migrate /data/model.cd --to 2.0.0 --dry-run

# Perform migration
cnws migrate /data/model.cd --to 2.0.0

# Verify migration
cnws migrate verify /data/model.cd

# Rollback if needed
cnws migrate rollback /data/model.cd --backup /backup/pre-v2
```

## 10.4 Migration Validation

`[COMPAT-V12-3]` Migration validation checklist:

| Check | Method |
|---|---|
| All Cell hashes valid | BLAKE3 verification |
| All Tile hashes valid | BLAKE3 verification |
| Manifest hash valid | BLAKE3 verification |
| Revision DAG intact | DAG traversal |
| Index consistent | Index verification |
| Version updated | SUPERBLOCK check |
| No data loss | Cell/Tile count comparison |

## 10.5 v1 → v2 Migration Invariants

| ID | Invariant |
|---|---|
| COMPAT-V12-INV-1 | Migration MUST atomic |
| COMPAT-V12-INV-2 | Migration MUST diverifikasi |
| COMPAT-V12-INV-3 | Migration MUST dapat di-rollback |
| COMPAT-V12-INV-4 | Data integrity MUST preserved |
| COMPAT-V12-INV-5 | Revision history MUST preserved |

---

# 11. Legacy CNWS-X Migration

## 11.1 CNWS-X → CNWS Mapping

`[COMPAT-CX-1]` CNWS-X adalah predecessor CNWS untuk infrastructure layer.

```text
CNWS-X Concept              CNWS Concept
─────────────────────       ─────────────────────
Column                  ──► Cell (weight type)
Tile                    ──► Tile (unchanged)
Columnar Model Graph    ──► Cell Graph
model.cd                ──► model.cd (extended)
Revision DAG            ──► Revision DAG (extended)
BLAKE3-256              ──► BLAKE3-256 (unchanged)
Segment                 ──► Segment (unchanged)
```

## 11.2 Column → Cell Migration

`[COMPAT-CX-2]` Column → Cell migration:

```pseudo
function migrate_column_to_cell(column: CNWSXColumn) -> CNWSCell:
    cell = Cell {
        id: column.id,  // Same identity
        cell_type: map_column_type_to_cell_type(column.semantic_type),
        version: CellVersion { major: 1, minor: 0, patch: 0 },
        input_schema: infer_input_schema(column),
        output_schema: infer_output_schema(column),
        tiles: column.tiles,  // Same TileRefs
        index_vector: compute_index_vector(column),
        dependencies: column.dependencies,
        metadata: migrate_column_metadata(column.metadata),
        representations: column.representations,
    }
    
    return cell
```

### 11.2.1 ColumnType → CellType Mapping

| CNWS-X ColumnType | CNWS CellType |
|---|---|
| EMBEDDING | CellType::Embedding |
| ATTENTION_Q_PROJ | CellType::AttentionQProj |
| ATTENTION_K_PROJ | CellType::AttentionKProj |
| ATTENTION_V_PROJ | CellType::AttentionVProj |
| ATTENTION_OUT | CellType::AttentionOut |
| MLP_GATE | CellType::MlpGate |
| MLP_UP | CellType::MlpUp |
| MLP_DOWN | CellType::MlpDown |
| EXPERT_GATE | CellType::ExpertGate |
| EXPERT_ROUTE | CellType::ExpertRoute |
| EXPERT_WEIGHT | CellType::ExpertWeight |
| LAYERNORM_WEIGHT | CellType::LayerNormWeight |
| LAYERNORM_BIAS | CellType::LayerNormBias |
| LM_HEAD | CellType::LmHead |
| VISION_ENCODER | CellType::VisionEncoder |
| CUSTOM(String) | CellType::Custom(String) |

## 11.3 Manifest Migration

`[COMPAT-CX-3]` CNWS-X manifest → CNWS manifest:

```pseudo
function migrate_cnwsx_manifest(cnwsx_manifest) -> CNWSManifest:
    cnws_manifest = CNWSManifest {
        format_version: "1.0.0",
        model_id: cnwsx_manifest.model_id,
        
        // Rename "columns" to "cells"
        cells: [migrate_column_to_cell(c) for c in cnwsx_manifest.columns],
        
        // Keep dependency graph
        dependency_graph: cnwsx_manifest.dependencies,
        
        // Keep architecture
        architecture: cnwsx_manifest.architecture,
        
        // Add new sections (empty)
        memory: default_memory_section(),
        routing: default_routing_section(),
        
        // Keep segments
        segments: cnwsx_manifest.segments,
        
        // Keep revision
        revision: migrate_revision(cnwsx_manifest.revision),
        
        // Keep provenance
        provenance: cnwsx_manifest.provenance,
    }
    
    return cnws_manifest
```

## 11.4 CNWS-X Migration Command

```bash
# Migrate CNWS-X store to CNWS
cnws migrate --from cnws-x /data/cnws-x-model.cd --to /data/cnws-model.cd

# Options:
#   --from cnws-x          Source format (CNWS-X)
#   --to <path>            Target path
#   --verify               Verify after migration
#   --dry-run              Show what would be done
```

## 11.5 CNWS-X Migration Invariants

| ID | Invariant |
|---|---|
| COMPAT-CX-INV-1 | Column → Cell migration MUST preserve identity |
| COMPAT-CX-INV-2 | Tile references MUST preserved |
| COMPAT-CX-INV-3 | BLAKE3-256 hashes MUST preserved |
| COMPAT-CX-INV-4 | Revision history MUST preserved |
| COMPAT-CX-INV-5 | Migration MUST diverifikasi |

---

# 12. Legacy LATTICE Migration

## 12.1 LATTICE → CNWS Mapping

`[COMPAT-LAT-1]` LATTICE adalah predecessor CNWS untuk intelligence layer.

```text
LATTICE Concept             CNWS Concept
─────────────────────       ─────────────────────
Cell                    ──► Cell (unchanged)
Memory Cell             ──► Memory Cell (CellType 0x20-0x2F)
Routing Cell            ──► Routing Cell (CellType 0x30-0x3F)
Composition Cell        ──► Composition Cell (CellType 0x40-0x4F)
Computation Cell        ──► Computation Cell (CellType 0x50-0x5F)
WorkingState            ──► WorkingState (runtime, not persisted)
Memory System           ──► Memory System (extended)
Routing Engine          ──► Routing Engine (extended)
```

## 12.2 Memory Migration

`[COMPAT-LAT-2]` LATTICE memory → CNWS memory:

```pseudo
function migrate_lattice_memory(lattice_memory) -> CNWSMemory:
    cnws_memory = CNWSMemory {
        episodic: [],
        semantic: [],
        procedural: [],
    }
    
    for entry in lattice_memory.entries:
        memory_cell = Cell {
            id: entry.id,
            cell_type: map_memory_type_to_cell_type(entry.memory_type),
            key_vector: entry.key_vector,
            value_payload: entry.value_payload,
            metadata: migrate_memory_metadata(entry.metadata),
        }
        
        match entry.memory_type:
            case Episodic:
                cnws_memory.episodic.append(memory_cell)
            case Semantic:
                cnws_memory.semantic.append(memory_cell)
            case Procedural:
                cnws_memory.procedural.append(memory_cell)
    
    return cnws_memory
```

## 12.3 Routing Migration

`[COMPAT-LAT-3]` LATTICE routing → CNWS routing:

```pseudo
function migrate_lattice_routing(lattice_routing) -> CNWSRouting:
    cnws_routing = CNWSRouting {
        policy: migrate_routing_policy(lattice_routing.policy),
        statistics: migrate_routing_statistics(lattice_routing.statistics),
        index: migrate_routing_index(lattice_routing.index),
    }
    
    return cnws_routing
```

## 12.4 Composition Migration

`[COMPAT-LAT-4]` LATTICE composition → CNWS composition:

```pseudo
function migrate_lattice_compositions(lattice_compositions) -> Vec<Cell>:
    cnws_compositions = []
    
    for pattern in lattice_compositions:
        composition_cell = Cell {
            id: pattern.id,
            cell_type: CellType::CompositionPattern,
            cell_sequence: pattern.cell_ids,
            execution_mode: pattern.mode,
            metadata: migrate_composition_metadata(pattern.metadata),
        }
        
        cnws_compositions.append(composition_cell)
    
    return cnws_compositions
```

## 12.5 LATTICE Migration Command

```bash
# Migrate LATTICE store to CNWS
cnws migrate --from lattice /data/lattice-model.cd --to /data/cnws-model.cd

# Options:
#   --from lattice         Source format (LATTICE)
#   --to <path>            Target path
#   --verify               Verify after migration
#   --dry-run              Show what would be done
```

## 12.6 LATTICE Migration Invariants

| ID | Invariant |
|---|---|
| COMPAT-LAT-INV-1 | Memory entries MUST preserved |
| COMPAT-LAT-INV-2 | Routing data MUST preserved |
| COMPAT-LAT-INV-3 | Composition patterns MUST preserved |
| COMPAT-LAT-INV-4 | BLAKE3-256 hashes MUST preserved |
| COMPAT-LAT-INV-5 | Migration MUST diverifikasi |

---

# 13. Migration Tooling

## 13.1 CLI Migration Commands

`[COMPAT-TOOL-1]` Migration CLI commands:

```bash
# Check migration status
cnws migrate check <store-path>

# Dry-run migration
cnws migrate <store-path> --dry-run

# Perform migration
cnws migrate <store-path> --to <version>

# Migrate from legacy format
cnws migrate --from <format> <source-path> --to <target-path>

# Verify migration
cnws migrate verify <store-path>

# Rollback migration
cnws migrate rollback <store-path> --backup <backup-path>

# Show migration history
cnws migrate history <store-path>
```

## 13.2 Migration API

`[COMPAT-TOOL-2]` Migration API untuk programmatic access:

```rust
trait MigrationEngine {
    // Check if migration needed
    fn check_migration_needed(
        &self,
        store_path: &Path,
    ) -> Result<MigrationInfo, CnwsError>;
    
    // Dry-run migration
    fn dry_run_migration(
        &self,
        store_path: &Path,
        target_version: FormatVersion,
    ) -> Result<MigrationPlan, CnwsError>;
    
    // Perform migration
    fn migrate(
        &self,
        store_path: &Path,
        target_version: FormatVersion,
    ) -> Result<MigrationResult, CnwsError>;
    
    // Verify migration
    fn verify_migration(
        &self,
        store_path: &Path,
    ) -> Result<VerificationReport, CnwsError>;
    
    // Rollback migration
    fn rollback_migration(
        &self,
        store_path: &Path,
        backup_path: &Path,
    ) -> Result<(), CnwsError>;
}
```

## 13.3 Migration Validation Tools

`[COMPAT-TOOL-3]` Migration validation tools:

```bash
# Validate store integrity
cnws diag integrity <store-path>

# Compare before/after migration
cnws migrate compare <before-path> <after-path>

# Show migration diff
cnws migrate diff <before-path> <after-path>
```

## 13.4 Migration Tooling Invariants

| ID | Invariant |
|---|---|
| COMPAT-TOOL-INV-1 | Migration tooling MUST tersedia untuk setiap major transition |
| COMPAT-TOOL-INV-2 | Migration tooling MUST mendukung dry-run |
| COMPAT-TOOL-INV-3 | Migration tooling MUST mendukung rollback |
| COMPAT-TOOL-INV-4 | Migration tooling MUST mendukung verification |

---

# 14. Migration Testing

## 14.1 Migration Test Suite

`[COMPAT-TEST-1]` Migration test suite MUST mencakup:

| Test Category | Description |
|---|---|
| Version detection | Detect version correctly |
| Compatibility check | Check compatibility correctly |
| Schema migration | Migrate schemas correctly |
| Data preservation | Preserve all data |
| Integrity verification | Verify hashes after migration |
| Rollback | Rollback correctly |
| Multi-version migration | Migrate across multiple versions |
| Legacy migration | Migrate from CNWS-X and LATTICE |

## 14.2 Golden File Migration Tests

`[COMPAT-TEST-2]` Golden file migration tests:

```text
Test: CNWS-MIG-0001
  Input: golden/v1.0.0/store.cd
  Migration: v1.0.0 → v1.1.0
  Expected: golden/v1.1.0/store.cd
  Verify: byte-identical after migration

Test: CNWS-MIG-0002
  Input: golden/v1.0.0/store.cd
  Migration: v1.0.0 → v2.0.0
  Expected: golden/v2.0.0/store.cd
  Verify: byte-identical after migration

Test: CNWS-MIG-0003
  Input: golden/cnws-x/store.cd
  Migration: cnws-x → cnws v1.0.0
  Expected: golden/cnws/from-cnws-x.cd
  Verify: semantic equivalence
```

## 14.3 Rollback Tests

`[COMPAT-TEST-3]` Rollback tests:

```text
Test: CNWS-MIG-ROLLBACK-0001
  Steps:
    1. Create store at v1.0.0
    2. Migrate to v2.0.0
    3. Simulate failure during migration
    4. Rollback
    5. Verify store is back to v1.0.0
  Expected: Store identical to original v1.0.0

Test: CNWS-MIG-ROLLBACK-0002
  Steps:
    1. Create store at v1.0.0
    2. Migrate to v1.1.0
    3. Rollback
    4. Verify store is back to v1.0.0
  Expected: Store identical to original v1.0.0
```

## 14.4 Migration Testing Invariants

| ID | Invariant |
|---|---|
| COMPAT-TEST-INV-1 | Migration tests MUST otomatis |
| COMPAT-TEST-INV-2 | Golden file tests MUST untuk setiap migration |
| COMPAT-TEST-INV-3 | Rollback tests MUST untuk setiap migration |
| COMPAT-TEST-INV-4 | Migration test failures MUST memblokir release |

---

# 15. Final Compatibility Contract

## 15.1 Ringkasan Keputusan Compatibility

| ID | Keputusan |
|---|---|
| COMPAT-F01 | Format version menggunakan semver. |
| COMPAT-F02 | Backward compatibility: minor ≤ versi didukung. |
| COMPAT-F03 | Forward incompatibility: major lebih tinggi ditolak. |
| COMPAT-F04 | Migration atomic. |
| COMPAT-F05 | Migration dapat di-rollback. |
| COMPAT-F06 | Legacy CNWS-X migration via `cnws migrate --from cnws-x`. |
| COMPAT-F07 | Legacy LATTICE migration via `cnws migrate --from lattice`. |
| COMPAT-F08 | Deprecation: 2 minor versions notice. |
| COMPAT-F09 | Schema migration deterministic. |
| COMPAT-F10 | Migration diverifikasi dengan integrity check. |
| COMPAT-F11 | CellType additions backward compatible. |
| COMPAT-F12 | CellType removals major version. |
| COMPAT-F13 | Revision format backward compatible untuk minor. |
| COMPAT-F14 | Migration tooling untuk setiap major transition. |
| COMPAT-F15 | Unknown fields dipertahankan saat round-trip. |
| COMPAT-F16 | Forward incompatible data ditolak eksplisit. |
| COMPAT-F17 | Migration step-by-step untuk multi-version. |
| COMPAT-F18 | Backup sebelum migration. |
| COMPAT-F19 | Migration history tracked. |
| COMPAT-F20 | Migration tests wajib. |

## 15.2 Compatibility Invariants

| ID | Invariant |
|---|---|
| COMPAT-INV-1 | Format version MUST semver. |
| COMPAT-INV-2 | Backward compatibility MUST dalam major yang sama. |
| COMPAT-INV-3 | Forward incompatibility MUST enforced. |
| COMPAT-INV-4 | Major version lebih tinggi MUST ditolak. |
| COMPAT-INV-5 | Migration MUST atomic. |
| COMPAT-INV-6 | Migration MUST dapat di-rollback. |
| COMPAT-INV-7 | Migration MUST diverifikasi. |
| COMPAT-INV-8 | Backup MUST sebelum migration. |
| COMPAT-INV-9 | Deprecation MUST 2 minor versions notice. |
| COMPAT-INV-10 | CellType addition MUST backward compatible. |
| COMPAT-INV-11 | CellType removal MUST major version. |
| COMPAT-INV-12 | Schema migration MUST deterministic. |
| COMPAT-INV-13 | Unknown fields SHOULD dipertahankan. |
| COMPAT-INV-14 | Legacy migration paths MUST tersedia. |
| COMPAT-INV-15 | Migration tooling MUST tersedia. |
| COMPAT-INV-16 | Migration tests MUST otomatis. |
| COMPAT-INV-17 | Golden file tests MUST untuk setiap migration. |
| COMPAT-INV-18 | Rollback tests MUST untuk setiap migration. |
| COMPAT-INV-19 | Revision history MUST preserved. |
| COMPAT-INV-20 | Data integrity MUST preserved. |

## 15.3 Compatibility Matrix Summary

| Reader \ Data | v1.0 | v1.1 | v1.2 | v2.0 | CNWS-X | LATTICE |
|---|---|---|---|---|---|---|
| **v1.0** | ✅ | ⚠️ | ⚠️ | ❌ | 🔧 | 🔧 |
| **v1.1** | ✅ | ✅ | ⚠️ | ❌ | 🔧 | 🔧 |
| **v1.2** | ✅ | ✅ | ✅ | ❌ | 🔧 | 🔧 |
| **v2.0** | ✅* | ✅* | ✅* | ✅ | 🔧 | 🔧 |

Legend:
- ✅ Fully compatible
- ⚠️ Partial (unknown fields ignored)
- ❌ Rejected (forward incompatible)
- ✅* Requires migration
- 🔧 Requires legacy migration tool

## 15.4 Migration Path Summary

| From | To | Migration Command |
|---|---|---|
| CNWS-X | CNWS v1.0 | `cnws migrate --from cnws-x` |
| LATTICE | CNWS v1.0 | `cnws migrate --from lattice` |
| CNWS v1.0 | CNWS v1.1 | `cnws migrate --to 1.1.0` |
| CNWS v1.1 | CNWS v1.2 | `cnws migrate --to 1.2.0` |
| CNWS v1.x | CNWS v2.0 | `cnws migrate --to 2.0.0` |
| CNWS v2.0 | CNWS v2.1 | `cnws migrate --to 2.1.0` |

## 15.5 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Compatibility & Migration final dan mengikat** untuk CNWS. Ia mendefinisikan bagaimana format CNWS berkembang, bagaimana compatibility dijaga, dan bagaimana migrasi antar versi dilakukan dengan aman.

Format CNWS akan berkembang. Dokumen ini menjamin bahwa evolusi tersebut terkelola, aman, dan tidak menyebabkan data loss. Setiap perubahan format melalui versioning yang terkontrol, setiap migrasi atomic dan dapat di-rollback, dan setiap legacy format memiliki migration path yang jelas.

Seluruh implementasi migration tooling, upgrade procedures, dan legacy converters CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan compatibility atau migration yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN COMPATIBILITY & MIGRATION SPECIFICATION**
