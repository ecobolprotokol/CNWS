# CNWS
## Reliability, Recovery & Disaster Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Reliability, Recovery & Disaster Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (RELIABILITY SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS .cd Format Spec; CNWS DAS; CNWS Security Spec |
| Hulu ke | Implementasi Recovery Subsystem, WAL, Quarantine, Repair, Rollback |
| Otoritas | Spesifikasi tunggal untuk seluruh failure behavior dan recovery CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    .cd Format Spec     Reliability & Recovery Spec    Implementation
─────────────────────   ────────────────    ────────────────────────────   ─────────────
Crash-safe commit     ──► WAL format     ──► Crash consistency model     ──► Recovery Subsystem
Atomic commit           Segment layout       Failure catalog                WAL Manager
Integrity verification  Journal              Recovery procedures            Quarantine Manager
GC reachability         SUPERBLOCK           Repair strategies              Repair Engine
                                              Rollback protocols             Rollback Manager
                                              Recovery guarantees            Recovery Tester
```

`[REL-DOC-1]` Dokumen ini mendefinisikan **behavior CNWS ketika sesuatu rusak** dan **bagaimana sistem pulih**.

`[REL-DOC-2]` Dokumen ini menjawab pertanyaan: "Apa yang terjadi jika X rusak, dan bagaimana kita pulih?"

`[REL-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

`[REL-DOC-4]` Jika terjadi konflik dengan .cd Format Spec untuk hal WAL format dan atomic commit, .cd Format Spec menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-REL-01 | Crash consistency model: WAL + atomic rename + fsync. |
| DF-REL-02 | Recovery MUST idempotent. |
| DF-REL-03 | Recovery MUST NOT kehilangan committed data. |
| DF-REL-04 | Uncommitted data MAY hilang saat crash. |
| DF-REL-05 | Quarantine untuk corrupted/malicious data. |
| DF-REL-06 | Repair strategy: replica → remote → re-import. |
| DF-REL-07 | Rollback tidak menghapus revision. |
| DF-REL-08 | Recovery guarantees: RPO = 0 untuk committed data, RTO < 60 detik. |
| DF-REL-09 | Recovery testing MUST otomatis dan repeatable. |
| DF-REL-10 | Partial disk failure MUST ditangani per-segment. |
| DF-REL-11 | Missing Tile MUST terdeteksi melalui index verification. |
| DF-REL-12 | Recovery MUST memiliki timeout dan fallback. |

---

# 1. Executive Summary

## 1.1 Reliability Philosophy

`[REL-EXEC-1]` CNWS mengadopsi prinsip **"assume failure"** — setiap komponen diasumsikan dapat gagal, dan sistem MUST dapat pulih.

`[REL-EXEC-2]` Prinsip reliability CNWS:

1. **Crash-safe**: crash kapan pun tidak menghasilkan state inconsistent.
2. **Idempotent recovery**: recovery dapat dijalankan berulang kali dengan hasil sama.
3. **No committed data loss**: data yang sudah committed MUST NOT hilang.
4. **Graceful degradation**: kegagalan sebagian tidak menghentikan seluruh sistem.
5. **Explicit failure**: kegagalan MUST dilaporkan eksplisit, bukan silent corruption.

## 1.2 Failure Categories

| Category | Examples | Impact |
|---|---|---|
| Process crash | OOM, panic, kill | Interrupted operation |
| Storage corruption | Bit rot, bad sectors | Corrupted data |
| Missing data | Deleted files, disk failure | Unavailable data |
| Partial write | Crash during write | Incomplete data |
| Manifest corruption | Tampered, corrupted | Invalid metadata |
| Disk failure | Hardware failure | Unavailable storage |

## 1.3 Recovery Objectives

`[REL-EXEC-3]` Recovery objectives:

| Metric | Target | Description |
|---|---|---|
| RPO (Recovery Point Objective) | 0 untuk committed data | Tidak ada committed data yang hilang |
| RTO (Recovery Time Objective) | < 60 detik | Waktu untuk pulih dari crash |
| Data integrity | 100% terverifikasi | BLAKE3 verification |
| Service availability | Degraded mode | Sistem tetap beroperasi meskipun degraded |

---

# 2. Failure Model

## 2.1 Failure Mode Catalog

`[REL-FM-1]` Catalog failure modes CNWS:

| ID | Failure Mode | Category | Severity | Frequency |
|---|---|---|---|---|
| FM-01 | Process crash during conversion | Process | High | Medium |
| FM-02 | Process crash during commit | Process | Critical | Low |
| FM-03 | Process crash during Tile write | Process | High | Medium |
| FM-04 | Segment corruption (bit rot) | Storage | High | Medium |
| FM-05 | Segment corruption (partial write) | Storage | High | Low |
| FM-06 | Manifest corruption | Storage | Critical | Low |
| FM-07 | SUPERBLOCK corruption | Storage | Critical | Low |
| FM-08 | Missing Tile | Storage | High | Low |
| FM-09 | Missing segment | Storage | High | Low |
| FM-10 | Partial disk failure | Hardware | Critical | Low |
| FM-11 | Full disk failure | Hardware | Critical | Low |
| FM-12 | WAL corruption | Storage | High | Low |
| FM-13 | Index corruption | Storage | Medium | Medium |
| FM-14 | Network failure during remote fetch | Network | Medium | Medium |
| FM-15 | OOM during conversion | Resource | High | Medium |
| FM-16 | OOM during runtime | Resource | High | Medium |

## 2.2 Failure Severity Levels

| Severity | Description | Response |
|---|---|---|
| Critical | System cannot continue | Immediate recovery required |
| High | Operation failed, system degraded | Recovery required |
| Medium | Operation degraded | Retry or workaround |
| Low | Minor issue | Log and continue |

## 2.3 Failure Detection

`[REL-FM-2]` Failure detection mechanisms:

| Detection | Method |
|---|---|
| Crash detection | Process exit code, watchdog |
| Corruption detection | BLAKE3 verification |
| Missing data detection | Index lookup failure |
| Partial write detection | Size mismatch, checksum |
| Manifest corruption | Hash mismatch with SUPERBLOCK |
| Disk failure | I/O error, SMART status |

---

# 3. Crash Consistency

## 3.1 Crash Consistency Model

`[REL-CC-1]` CNWS menggunakan **WAL + atomic rename + fsync** untuk crash consistency.

`[REL-CC-2]` Invariant crash consistency:

```text
At any point in time, the store is in one of:
  1. Last committed state (before operation)
  2. New committed state (after operation)
  
NEVER:
  - Partial state
  - Inconsistent state
  - Corrupted state
```

## 3.2 Crash Points

`[REL-CC-3]` Crash dapat terjadi di berbagai titik:

```text
Operation Timeline:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. Write staging    2. Journal    3. Rename    4. SUPERBLOCK│
  │    manifest           append        manifest     update      │
  └─────────────────────────────────────────────────────────────┘
  
  Crash points:
    A: Before staging write      → No change
    B: After staging write       → Staging exists, no commit
    C: After journal append      → Journal has intent
    D: After rename              → New manifest active
    E: After SUPERBLOCK update   → Fully committed
```

## 3.3 Crash Point Recovery

| Crash Point | State | Recovery Action |
|---|---|---|
| A | No change | None needed |
| B | Staging exists | Clean up staging |
| C | Journal has intent | Replay or rollback |
| D | New manifest active | Complete SUPERBLOCK update |
| E | Fully committed | None needed |

## 3.4 Crash Consistency Invariants

| ID | Invariant |
|---|---|
| REL-CC-INV-1 | Store MUST selalu dalam state konsisten |
| REL-CC-INV-2 | Crash MUST NOT menghasilkan partial commit |
| REL-CC-INV-3 | Recovery MUST idempotent |
| REL-CC-INV-4 | Committed data MUST NOT hilang |
| REL-CC-INV-5 | Uncommitted data MAY hilang |

---

# 4. Write-Ahead Log (WAL)

## 4.1 WAL Purpose

`[REL-WAL-1]` WAL (Write-Ahead Log) mencatat intent sebelum perubahan dilakukan.

`[REL-WAL-2]` WAL memungkinkan recovery setelah crash dengan replay atau rollback.

## 4.2 WAL Location

`[REL-WAL-3]` WAL disimpan di `journal/commit.wal`.

`[REL-WAL-4]` WAL MUST di-fsync setelah setiap append.

## 4.3 WAL Record Format

Dari .cd Format Specification:

```text
Offset  Size  Field                  Type
──────  ────  ─────────────────────  ───────────
0x00    8     magic                  [u8; 8]     "CNWSWAL1"
0x08    8     record_id              u64
0x10    8     timestamp_ns           u64
0x18    1     record_type            u8
0x19    7     padding                [u8; 7]
0x20    32    manifest_hash          Blake3Hash
0x40    8     manifest_size          u64
0x48    8     staging_offset         u64
0x50    32    prev_manifest_hash     Blake3Hash
0x70    8     flags                  u64
0x78    ...   payload                (variable)
```

## 4.4 WAL Record Types

| Type | Value | Description |
|---|---|---|
| BEGIN_COMMIT | 0x01 | Mulai commit |
| STAGE_MANIFEST | 0x02 | Manifest ditulis ke staging |
| RENAME_MANIFEST | 0x03 | Manifest di-rename |
| UPDATE_SUPERBLOCK | 0x04 | SUPERBLOCK di-update |
| COMMIT_COMPLETE | 0x05 | Commit selesai |
| ROLLBACK | 0x06 | Rollback dilakukan |
| BEGIN_CONVERSION | 0x07 | Mulai conversion |
| CONVERSION_PROGRESS | 0x08 | Progress conversion |
| CONVERSION_COMPLETE | 0x09 | Conversion selesai |

## 4.5 WAL Protocol

```pseudo
function wal_append(record_type, payload):
    record = WalRecord {
        magic: "CNWSWAL1",
        record_id: next_record_id(),
        timestamp_ns: now(),
        record_type: record_type,
        payload: payload,
    }
    
    // Serialize record
    bytes = serialize(record)
    
    // Append to WAL
    wal_file.append(bytes)
    
    // Fsync to ensure durability
    wal_file.fsync()
    
    return record.record_id
```

`[REL-WAL-5]` WAL append MUST diikuti fsync.

`[REL-WAL-6]` WAL MUST di-append sebelum perubahan aktual.

## 4.6 WAL Truncation

`[REL-WAL-7]` WAL MAY di-truncate setelah COMMIT_COMPLETE.

`[REL-WAL-8]` WAL truncation MUST aman (tidak menghapus record yang dibutuhkan).

```pseudo
function wal_truncate():
    // Find last COMMIT_COMPLETE
    last_complete = find_last_commit_complete()
    
    if last_complete is None:
        return  // Nothing to truncate
    
    // Truncate up to last complete
    wal_file.truncate(last_complete.offset)
    wal_file.fsync()
```

## 4.7 WAL Recovery

```pseudo
function wal_recover() -> RecoveryAction:
    records = read_wal_records()
    
    if records is empty:
        return RecoveryAction::None
    
    last_record = records[-1]
    
    match last_record.record_type:
        case BEGIN_COMMIT:
            // Commit started but not completed
            return RecoveryAction::RollbackCommit
        
        case STAGE_MANIFEST:
            // Manifest staged but not renamed
            return RecoveryAction::CleanupStaging
        
        case RENAME_MANIFEST:
            // Manifest renamed but SUPERBLOCK not updated
            return RecoveryAction::CompleteSuperblock
        
        case UPDATE_SUPERBLOCK:
            // SUPERBLOCK updated but commit not marked complete
            return RecoveryAction::MarkComplete
        
        case COMMIT_COMPLETE:
            // Fully committed
            return RecoveryAction::None
        
        case BEGIN_CONVERSION:
            // Conversion started but not completed
            return RecoveryAction::CleanupConversion
        
        case CONVERSION_COMPLETE:
            // Conversion completed
            return RecoveryAction::None
```

## 4.8 WAL Invariants

| ID | Invariant |
|---|---|
| REL-WAL-INV-1 | WAL MUST di-append sebelum perubahan |
| REL-WAL-INV-2 | WAL append MUST diikuti fsync |
| REL-WAL-INV-3 | WAL MUST dapat di-replay |
| REL-WAL-INV-4 | WAL recovery MUST idempotent |
| REL-WAL-INV-5 | WAL MUST NOT dihapus sebelum COMMIT_COMPLETE |

---

# 5. Atomic Commit Protocol

## 5.1 Protocol Overview

`[REL-AC-1]` Atomic commit protocol memastikan commit bersifat all-or-nothing.

## 5.2 Protocol Steps

```pseudo
function atomic_commit(manifest: Manifest) -> Result<CommitReceipt>:
    // Step 1: Canonicalize manifest
    canonical = canonicalize(manifest)
    
    // Step 2: Compute manifest hash
    manifest_hash = blake3_256(canonical)
    
    // Step 3: Write staging manifest
    staging_path = "staging/manifest-" + hex(manifest_hash) + ".cd"
    write_file(staging_path, canonical)
    fsync(staging_path)
    
    // Step 4: Append WAL (intent)
    wal_append(BEGIN_COMMIT, {
        manifest_hash: manifest_hash,
        prev_manifest_hash: current_manifest_hash(),
    })
    
    // Step 5: Append WAL (staging complete)
    wal_append(STAGE_MANIFEST, {
        staging_path: staging_path,
        manifest_hash: manifest_hash,
    })
    
    // Step 6: Rename staging to MANIFEST.cd
    rename(staging_path, "MANIFEST.cd")
    fsync_directory(".")
    
    // Step 7: Append WAL (rename complete)
    wal_append(RENAME_MANIFEST, {
        manifest_hash: manifest_hash,
    })
    
    // Step 8: Update SUPERBLOCK
    superblock = read_superblock()
    superblock.manifest_hash = manifest_hash
    superblock.manifest_size = len(canonical)
    superblock.last_modified_ns = now()
    
    write_superblock_tmp(superblock)
    fsync("SUPERBLOCK.tmp")
    rename("SUPERBLOCK.tmp", "SUPERBLOCK")
    fsync_directory(".")
    
    // Step 9: Append WAL (superblock updated)
    wal_append(UPDATE_SUPERBLOCK, {
        manifest_hash: manifest_hash,
    })
    
    // Step 10: Append WAL (commit complete)
    wal_append(COMMIT_COMPLETE, {
        manifest_hash: manifest_hash,
    })
    
    // Step 11: Cleanup
    remove(staging_path)  // If still exists
    
    return CommitReceipt {
        manifest_hash: manifest_hash,
        committed_at: now(),
    }
```

## 5.3 Atomicity Guarantee

`[REL-AC-2]` Atomicity dijamin oleh:

1. **Staging**: manifest baru ditulis ke staging dulu
2. **WAL**: intent dicatat sebelum perubahan
3. **Atomic rename**: rename adalah operasi atomic di POSIX
4. **Fsync**: durability dijamin dengan fsync

## 5.4 Crash Recovery for Atomic Commit

```text
Crash during atomic commit:
    │
    ▼
Read WAL
    │
    ├── No records → No recovery needed
    │
    ├── BEGIN_COMMIT only → Rollback (no changes made)
    │
    ├── STAGE_MANIFEST → Cleanup staging
    │
    ├── RENAME_MANIFEST → Complete SUPERBLOCK update
    │
    ├── UPDATE_SUPERBLOCK → Mark commit complete
    │
    └── COMMIT_COMPLETE → No recovery needed
```

## 5.5 Atomic Commit Invariants

| ID | Invariant |
|---|---|
| REL-AC-INV-1 | Commit MUST atomic |
| REL-AC-INV-2 | Commit MUST menggunakan WAL |
| REL-AC-INV-3 | Commit MUST menggunakan atomic rename |
| REL-AC-INV-4 | Commit MUST fsync pada titik kritis |
| REL-AC-INV-5 | Crash during commit MUST recoverable |

---

# 6. Interrupted Conversion Recovery

## 6.1 Conversion State

`[REL-IC-1]` Conversion memiliki state yang dapat di-track:

```text
Conversion State:
  ┌─────────────────────────────────────────────────────────────┐
  │ 1. Format Detection → 2. Validation → 3. Reading →         │
  │ 4. Normalization → 5. Tiling → 6. Hashing →                │
  │ 7. Dedup → 8. Writing → 9. Manifest Build → 10. Commit     │
  └─────────────────────────────────────────────────────────────┘
```

## 6.2 Conversion WAL

`[REL-IC-2]` Conversion MUST menggunakan WAL untuk tracking progress.

```pseudo
function conversion_with_wal(source, target):
    // Begin conversion
    wal_append(BEGIN_CONVERSION, {
        source: source,
        target: target,
        started_at: now(),
    })
    
    // Process tensors
    for tensor in tensor_stream(source):
        // Normalize
        cell = normalize(tensor)
        
        // Tile
        tiles = plan_tiles(cell)
        
        // Hash and write
        for tile in tiles:
            tile_id = hash_tile(tile)
            write_tile(tile_id, tile)
            
            // Progress record
            wal_append(CONVERSION_PROGRESS, {
                tensor_name: tensor.name,
                tiles_written: tiles.len(),
            })
    
    // Build manifest
    manifest = build_manifest()
    
    // Commit
    atomic_commit(manifest)
    
    // Complete
    wal_append(CONVERSION_COMPLETE, {
        tiles_total: total_tiles,
        completed_at: now(),
    })
```

## 6.3 Interrupted Conversion Recovery

```pseudo
function recover_interrupted_conversion():
    // Read WAL
    records = read_wal_records()
    
    // Find conversion records
    conversion_records = filter(records, is_conversion_record)
    
    if conversion_records is empty:
        return  // No conversion in progress
    
    last_record = conversion_records[-1]
    
    match last_record.record_type:
        case BEGIN_CONVERSION:
            // Conversion started but no progress
            cleanup_conversion_artifacts()
        
        case CONVERSION_PROGRESS:
            // Conversion in progress
            // Option 1: Resume from last progress
            // Option 2: Restart from beginning
            resume_or_restart_conversion(last_record)
        
        case CONVERSION_COMPLETE:
            // Conversion completed but commit may not have completed
            verify_and_complete_commit()
```

## 6.4 Conversion Artifacts Cleanup

`[REL-IC-3]` Jika conversion dibatalkan atau gagal, artifacts MUST dibersihkan.

```pseudo
function cleanup_conversion_artifacts():
    // Remove staging files
    remove_staging_files()
    
    // Remove partially written segments
    remove_partial_segments()
    
    // Remove temporary files
    remove_temp_files()
    
    // Update WAL
    wal_append(ROLLBACK, {
        reason: "conversion_cleanup",
    })
```

## 6.5 Interrupted Conversion Invariants

| ID | Invariant |
|---|---|
| REL-IC-INV-1 | Conversion MUST menggunakan WAL |
| REL-IC-INV-2 | Interrupted conversion MUST recoverable |
| REL-IC-INV-3 | Conversion artifacts MUST dibersihkan jika gagal |
| REL-IC-INV-4 | Conversion MUST atomic (berhasil penuh atau gagal penuh) |
| REL-IC-INV-5 | Partial .cd MUST NOT dihasilkan |

---

# 7. Interrupted Revision Recovery

## 7.1 Revision State

`[REL-IR-1]` Revision commit mengikuti atomic commit protocol.

`[REL-IR-2]` Interrupted revision recovery menggunakan WAL.

## 7.2 Interrupted Revision Recovery

```pseudo
function recover_interrupted_revision():
    // Read WAL
    records = read_wal_records()
    
    // Find revision commit records
    commit_records = filter(records, is_commit_record)
    
    if commit_records is empty:
        return  // No commit in progress
    
    last_record = commit_records[-1]
    
    match last_record.record_type:
        case BEGIN_COMMIT:
            // Commit started but staging not complete
            // No changes made, nothing to recover
            log("Interrupted commit, no changes made")
        
        case STAGE_MANIFEST:
            // Staging complete but rename not done
            cleanup_staging()
        
        case RENAME_MANIFEST:
            // Rename complete but SUPERBLOCK not updated
            complete_superblock_update(last_record.manifest_hash)
        
        case UPDATE_SUPERBLOCK:
            // SUPERBLOCK updated but commit not marked complete
            mark_commit_complete(last_record.manifest_hash)
        
        case COMMIT_COMPLETE:
            // Fully committed
            log("Commit already complete")
```

## 7.3 Learning Update Recovery

`[REL-IR-3]` Learning updates (CellCreate, CellRefine, dll.) yang interrupted MUST di-recover.

```pseudo
function recover_interrupted_learning():
    // Check for partially applied learning updates
    pending_updates = get_pending_learning_updates()
    
    for update in pending_updates:
        match update.state:
            case WRITING_TILES:
                // Tiles partially written
                cleanup_partial_tiles(update)
            
            case UPDATING_ROUTING:
                // Routing partially updated
                rollback_routing_update(update)
            
            case COMMITTING:
                // Revision commit in progress
                recover_interrupted_revision()
```

## 7.4 Interrupted Revision Invariants

| ID | Invariant |
|---|---|
| REL-IR-INV-1 | Interrupted revision MUST recoverable |
| REL-IR-INV-2 | Recovery MUST menggunakan WAL |
| REL-IR-INV-3 | Partial revision MUST NOT committed |
| REL-IR-INV-4 | Recovery MUST idempotent |
| REL-IR-INV-5 | Committed revision MUST NOT hilang |

---

# 8. Corrupted Segment Recovery

## 8.1 Segment Corruption Detection

`[REL-CS-1]` Segment corruption terdeteksi melalui:

1. **Header validation**: magic, version, field consistency
2. **Index hash verification**: BLAKE3-256 of segment index
3. **Tile hash verification**: BLAKE3-256 of Tile payloads
4. **Trailer validation**: magic, index hash match

## 8.2 Segment Corruption Types

| Type | Detection | Severity |
|---|---|---|
| Header corruption | Magic mismatch | Critical |
| Index corruption | Index hash mismatch | High |
| Tile corruption | Tile hash mismatch | High |
| Trailer corruption | Trailer magic mismatch | Medium |
| Truncation | File size < expected | High |

## 8.3 Segment Recovery Protocol

```pseudo
function recover_corrupted_segment(segment_id) -> RecoveryResult:
    // Step 1: Identify corruption
    corruption = diagnose_segment(segment_id)
    
    // Step 2: Quarantine corrupted segment
    quarantine_segment(segment_id)
    
    // Step 3: Identify affected Tiles
    affected_tiles = get_tiles_in_segment(segment_id)
    
    // Step 4: Attempt recovery for each Tile
    for tile_id in affected_tiles:
        recovery = recover_tile(tile_id)
        
        match recovery:
            case RecoveredFromReplica:
                log("Tile recovered from replica")
            
            case RecoveredFromRemote:
                log("Tile recovered from remote")
            
            case RecoveredFromReimport:
                log("Tile recovered from re-import")
            
            case Unrecoverable:
                log_error("Tile unrecoverable: {}", tile_id)
                mark_tile_unrecoverable(tile_id)
    
    // Step 5: Rebuild segment if possible
    if all_tiles_recovered(affected_tiles):
        rebuild_segment(segment_id, affected_tiles)
    
    return RecoveryResult {
        segment_id: segment_id,
        tiles_affected: affected_tiles.len(),
        tiles_recovered: count_recovered(affected_tiles),
        tiles_unrecoverable: count_unrecoverable(affected_tiles),
    }
```

## 8.4 Segment Quarantine

`[REL-CS-2]` Corrupted segment MUST dikarantina.

```pseudo
function quarantine_segment(segment_id):
    // Move to quarantine directory
    source = "segments/segment-" + format(segment_id) + ".cd"
    target = "corrupt/segment-" + format(segment_id) + ".quarantine"
    
    rename(source, target)
    
    // Record quarantine metadata
    quarantine_record = {
        segment_id: segment_id,
        quarantined_at: now(),
        reason: "corruption_detected",
        corruption_details: diagnose_segment(segment_id),
    }
    
    write_quarantine_record(quarantine_record)
    
    // Log security event
    log_security_event(SecurityEvent::SegmentCorruption {
        segment_id,
        details: quarantine_record,
    })
```

## 8.5 Segment Recovery Invariants

| ID | Invariant |
|---|---|
| REL-CS-INV-1 | Segment corruption MUST terdeteksi |
| REL-CS-INV-2 | Corrupted segment MUST dikarantina |
| REL-CS-INV-3 | Affected Tiles MUST diidentifikasi |
| REL-CS-INV-4 | Recovery MUST dicoba sebelum error fatal |
| REL-CS-INV-5 | Unrecoverable Tiles MUST dilaporkan eksplisit |

---

# 9. Corrupted Manifest Recovery

## 9.1 Manifest Corruption Detection

`[REL-CM-1]` Manifest corruption terdeteksi melalui:

1. **Hash verification**: BLAKE3-256 of MANIFEST.cd vs SUPERBLOCK.manifest_hash
2. **JSON validation**: parse dan schema validation
3. **Content validation**: field consistency

## 9.2 Manifest Corruption Types

| Type | Detection | Severity |
|---|---|---|
| Hash mismatch | BLAKE3 verification | Critical |
| Invalid JSON | Parse failure | Critical |
| Schema violation | Field validation | High |
| Truncation | Size mismatch | Critical |
| Missing fields | Schema validation | High |

## 9.3 Manifest Recovery Protocol

```pseudo
function recover_corrupted_manifest() -> RecoveryResult:
    // Step 1: Detect corruption
    manifest_bytes = read_file("MANIFEST.cd")
    actual_hash = blake3_256(manifest_bytes)
    expected_hash = read_superblock().manifest_hash
    
    if actual_hash == expected_hash:
        return RecoveryResult::NotCorrupted
    
    // Step 2: Try MANIFEST.cd.prev
    prev_bytes = read_file("MANIFEST.cd.prev")
    prev_hash = blake3_256(prev_bytes)
    
    if prev_hash == read_superblock().prev_manifest_hash:
        // Restore from prev
        rename("MANIFEST.cd.prev", "MANIFEST.cd")
        update_superblock(prev_hash)
        return RecoveryResult::RestoredFromPrev
    
    // Step 3: Try staged manifests
    staged = list_staged_manifests()
    for staging in staged:
        staging_hash = blake3_256(read_file(staging))
        if staging_hash == expected_hash:
            rename(staging, "MANIFEST.cd")
            return RecoveryResult::RestoredFromStaging
    
    // Step 4: Try WAL replay
    wal_manifest = replay_wal_for_manifest()
    if wal_manifest is not None:
        write_file("MANIFEST.cd", wal_manifest)
        return RecoveryResult::RestoredFromWal
    
    // Step 5: Rebuild from segments
    rebuilt = rebuild_manifest_from_segments()
    if rebuilt is not None:
        write_file("MANIFEST.cd", rebuilt)
        return RecoveryResult::RebuiltFromSegments
    
    // Step 6: Unrecoverable
    return RecoveryResult::Unrecoverable
```

## 9.4 Manifest Recovery Invariants

| ID | Invariant |
|---|---|
| REL-CM-INV-1 | Manifest corruption MUST terdeteksi |
| REL-CM-INV-2 | Recovery MUST mencoba MANIFEST.cd.prev |
| REL-CM-INV-3 | Recovery MUST mencoba staged manifests |
| REL-CM-INV-4 | Recovery MUST mencoba WAL replay |
| REL-CM-INV-5 | Unrecoverable manifest MUST dilaporkan fatal |

---

# 10. Missing Tile Recovery

## 10.1 Missing Tile Detection

`[REL-MT-1]` Missing Tile terdeteksi melalui:

1. **Index lookup**: Tile ID tidak ada di index
2. **Segment read**: offset tidak valid atau data tidak ada
3. **Manifest verification**: Tile reference tidak dapat di-resolve

## 10.2 Missing Tile Recovery Protocol

```pseudo
function recover_missing_tile(tile_id) -> RecoveryResult:
    // Step 1: Check if Tile exists in index
    if tile_exists_in_index(tile_id):
        // Index says it exists, check actual location
        location = lookup_tile(tile_id)
        
        if segment_exists(location.segment_id):
            // Segment exists, Tile may be corrupted
            return recover_corrupted_tile(tile_id, location)
        else:
            // Segment missing
            return recover_missing_segment(location.segment_id)
    
    // Step 2: Check replicas
    if has_replica(tile_id):
        replica_data = read_replica(tile_id)
        if verify_tile(tile_id, replica_data):
            write_tile(tile_id, replica_data)
            return RecoveryResult::RecoveredFromReplica
    
    // Step 3: Check remote source
    if has_remote_source(tile_id):
        remote_data = fetch_remote(tile_id)
        if verify_tile(tile_id, remote_data):
            write_tile(tile_id, remote_data)
            return RecoveryResult::RecoveredFromRemote
    
    // Step 4: Check if Tile can be re-imported
    if can_reimport(tile_id):
        reimported = reimport_tile(tile_id)
        if verify_tile(tile_id, reimported):
            write_tile(tile_id, reimported)
            return RecoveryResult::RecoveredFromReimport
    
    // Step 5: Check if Tile exists in another revision
    if exists_in_other_revision(tile_id):
        revision_data = read_from_revision(tile_id)
        if verify_tile(tile_id, revision_data):
            write_tile(tile_id, revision_data)
            return RecoveryResult::RecoveredFromRevision
    
    // Step 6: Unrecoverable
    return RecoveryResult::Unrecoverable
```

## 10.3 Missing Tile Invariants

| ID | Invariant |
|---|---|
| REL-MT-INV-1 | Missing Tile MUST terdeteksi |
| REL-MT-INV-2 | Recovery MUST mencoba replica |
| REL-MT-INV-3 | Recovery MUST mencoba remote source |
| REL-MT-INV-4 | Recovery MUST mencoba re-import |
| REL-MT-INV-5 | Unrecoverable Tile MUST dilaporkan eksplisit |

---

# 11. Partial Disk Failure Recovery

## 11.1 Partial Disk Failure Detection

`[REL-PD-1]` Partial disk failure terdeteksi melalui:

1. **I/O errors**: read/write failures
2. **Checksum errors**: BLAKE3 mismatches
3. **SMART status**: disk health monitoring
4. **File system errors**: metadata corruption

## 11.2 Partial Disk Failure Scope

`[REL-PD-2]` Partial disk failure dapat mempengaruhi:

| Scope | Impact |
|---|---|
| Single Tile | One Tile corrupted |
| Single segment | Multiple Tiles corrupted |
| Multiple segments | Many Tiles corrupted |
| Index files | Lookup failures |
| Manifest | Metadata corruption |
| WAL | Recovery capability impaired |

## 11.3 Partial Disk Failure Recovery

```pseudo
function recover_partial_disk_failure() -> RecoveryResult:
    // Step 1: Assess damage
    damage = assess_disk_damage()
    
    // Step 2: Quarantine affected regions
    for region in damage.affected_regions:
        quarantine_region(region)
    
    // Step 3: Identify affected Tiles
    affected_tiles = identify_affected_tiles(damage)
    
    // Step 4: Recover Tiles
    recovered = 0
    unrecoverable = 0
    
    for tile_id in affected_tiles:
        result = recover_missing_tile(tile_id)
        
        match result:
            case Recovered:
                recovered += 1
            case Unrecoverable:
                unrecoverable += 1
    
    // Step 5: Rebuild indexes
    rebuild_indexes()
    
    // Step 6: Verify manifest
    verify_manifest()
    
    // Step 7: Report
    return RecoveryResult {
        affected_tiles: affected_tiles.len(),
        recovered: recovered,
        unrecoverable: unrecoverable,
        disk_health: get_disk_health(),
    }
```

## 11.4 Degraded Mode

`[REL-PD-3]` Jika sebagian data unrecoverable, sistem MAY beroperasi dalam degraded mode.

```pseudo
function enter_degraded_mode(unrecoverable_tiles):
    // Mark unrecoverable Tiles
    for tile_id in unrecoverable_tiles:
        mark_tile_unavailable(tile_id)
    
    // Identify affected Cells
    affected_cells = get_cells_with_tiles(unrecoverable_tiles)
    
    // Mark affected Cells as unavailable
    for cell_id in affected_cells:
        mark_cell_degraded(cell_id)
    
    // Continue operation with available Cells
    set_degraded_mode(true)
    
    // Alert operator
    alert_operator("Operating in degraded mode", {
        unrecoverable_tiles: unrecoverable_tiles.len(),
        affected_cells: affected_cells.len(),
    })
```

## 11.5 Partial Disk Failure Invariants

| ID | Invariant |
|---|---|
| REL-PD-INV-1 | Partial disk failure MUST terdeteksi |
| REL-PD-INV-2 | Affected regions MUST dikarantina |
| REL-PD-INV-3 | Recovery MUST dicoba untuk setiap Tile |
| REL-PD-INV-4 | Degraded mode MAY jika sebagian unrecoverable |
| REL-PD-INV-5 | Disk health MUST dimonitor |

---

# 12. Unified Recovery Procedure

## 12.1 Recovery Entry Point

`[REL-REC-1]` Recovery MUST memiliki entry point tunggal.

```pseudo
function recover() -> RecoveryReport:
    report = RecoveryReport::new()
    
    // Phase 1: WAL recovery
    wal_result = wal_recover()
    report.wal = wal_result
    
    // Phase 2: Manifest verification
    manifest_result = verify_and_recover_manifest()
    report.manifest = manifest_result
    
    // Phase 3: SUPERBLOCK verification
    superblock_result = verify_and_recover_superblock()
    report.superblock = superblock_result
    
    // Phase 4: Segment verification
    segment_result = verify_all_segments()
    report.segments = segment_result
    
    // Phase 5: Index verification
    index_result = verify_and_rebuild_indexes()
    report.indexes = index_result
    
    // Phase 6: Tile verification (sampling)
    tile_result = verify_tile_sample()
    report.tiles = tile_result
    
    // Phase 7: Revision DAG verification
    revision_result = verify_revision_dag()
    report.revisions = revision_result
    
    // Phase 8: Final consistency check
    consistency_result = final_consistency_check()
    report.consistency = consistency_result
    
    return report
```

## 12.2 Recovery State Machine

```text
┌──────────┐   start    ┌──────────┐
│  IDLE    │───────────►│ASSESSING │
└──────────┘            └────┬─────┘
                             │ damage assessed
                             ▼
┌──────────┐   complete ┌──────────┐
│RECOVERED │◄───────────│RECOVERING│
└──────────┘            └────┬─────┘
                             │ unrecoverable
                             ▼
                        ┌──────────┐
                        │ DEGRADED │
                        └────┬─────┘
                             │ fatal
                             ▼
                        ┌──────────┐
                        │  FAILED  │
                        └──────────┘
```

## 12.3 Recovery Idempotency

`[REL-REC-2]` Recovery MUST idempotent.

```text
recover() → State A
recover() → State A  (same result)
recover() → State A  (same result)
```

## 12.4 Recovery Timeout

`[REL-REC-3]` Recovery MUST memiliki timeout.

`[REL-REC-4]` Default recovery timeout: 300 detik.

`[REL-REC-5]` Jika timeout, recovery MUST melaporkan progress dan MAY dilanjutkan nanti.

## 12.5 Recovery Invariants

| ID | Invariant |
|---|---|
| REL-REC-INV-1 | Recovery MUST memiliki entry point tunggal |
| REL-REC-INV-2 | Recovery MUST idempotent |
| REL-REC-INV-3 | Recovery MUST memiliki timeout |
| REL-REC-INV-4 | Recovery MUST melaporkan progress |
| REL-REC-INV-5 | Recovery MUST NOT kehilangan committed data |

---

# 13. Quarantine Protocol

## 13.1 Quarantine Purpose

`[REL-QR-1]` Quarantine mengisolasi data corrupted atau malicious untuk mencegah penggunaan.

## 13.2 Quarantine Triggers

| Trigger | Action |
|---|---|
| BLAKE3 mismatch | Quarantine Tile |
| Segment corruption | Quarantine segment |
| Manifest tampering | Quarantine manifest |
| Malicious content detected | Quarantine dan alert |
| Repeated read errors | Quarantine region |

## 13.3 Quarantine Directory

`[REL-QR-2]` Quarantine directory: `corrupt/`

```text
corrupt/
├── <tile-id>.quarantine           # Quarantined Tiles
├── segment-<id>.quarantine        # Quarantined segments
├── manifest.quarantine            # Quarantined manifest
└── quarantine_log.json            # Quarantine log
```

## 13.4 Quarantine Record

```json
{
  "quarantine_id": "q-000001",
  "item_type": "tile",
  "item_id": "b3:7f3a8e...",
  "quarantined_at": "2026-08-11T12:00:00Z",
  "reason": "blake3_mismatch",
  "details": {
    "expected_hash": "b3:7f3a8e...",
    "actual_hash": "b3:9c2b1f...",
    "location": "segments/segment-000001.cd",
    "offset": 1048576
  },
  "recovery_attempts": [],
  "status": "quarantined"
}
```

## 13.5 Quarantine Protocol

```pseudo
function quarantine(item_type, item_id, reason, details):
    // Step 1: Move to quarantine
    source = get_item_path(item_type, item_id)
    target = "corrupt/" + item_id + ".quarantine"
    rename(source, target)
    
    // Step 2: Record quarantine
    record = QuarantineRecord {
        quarantine_id: next_quarantine_id(),
        item_type: item_type,
        item_id: item_id,
        quarantined_at: now(),
        reason: reason,
        details: details,
        recovery_attempts: [],
        status: "quarantined",
    }
    
    write_quarantine_record(record)
    
    // Step 3: Log security event
    log_security_event(SecurityEvent::Quarantine {
        item_type,
        item_id,
        reason,
    })
    
    // Step 4: Alert if critical
    if is_critical(reason):
        alert_operator("Critical quarantine", record)
```

## 13.6 Quarantine Invariants

| ID | Invariant |
|---|---|
| REL-QR-INV-1 | Corrupted data MUST dikarantina |
| REL-QR-INV-2 | Quarantine MUST recorded |
| REL-QR-INV-3 | Quarantined data MUST NOT dieksekusi |
| REL-QR-INV-4 | Quarantine MUST logged |
| REL-QR-INV-5 | Quarantine MUST reversible (untuk recovery) |

---

# 14. Repair Procedures

## 14.1 Repair Strategy

`[REL-RP-1]` Repair strategy berdasarkan prioritas:

| Priority | Strategy | Description |
|---|---|---|
| 1 | Replica | Pulihkan dari replica lokal |
| 2 | Remote | Pulihkan dari remote source |
| 3 | Re-import | Pulihkan dengan re-import dari source |
| 4 | Rebuild | Rebuild dari komponen lain |
| 5 | Manual | Intervensi manual |

## 14.2 Tile Repair

```pseudo
function repair_tile(tile_id) -> RepairResult:
    // Strategy 1: Replica
    if has_local_replica(tile_id):
        data = read_local_replica(tile_id)
        if verify_tile(tile_id, data):
            write_tile(tile_id, data)
            return RepairResult::RepairedFromReplica
    
    // Strategy 2: Remote
    if has_remote_source(tile_id):
        data = fetch_remote(tile_id)
        if verify_tile(tile_id, data):
            write_tile(tile_id, data)
            return RepairResult::RepairedFromRemote
    
    // Strategy 3: Re-import
    if can_reimport(tile_id):
        data = reimport_tile(tile_id)
        if verify_tile(tile_id, data):
            write_tile(tile_id, data)
            return RepairResult::RepairedFromReimport
    
    // Strategy 4: Rebuild from other representations
    if has_other_representations(tile_id):
        data = rebuild_from_representations(tile_id)
        if verify_tile(tile_id, data):
            write_tile(tile_id, data)
            return RepairResult::RepairedFromRepresentations
    
    // Strategy 5: Manual intervention required
    return RepairResult::ManualInterventionRequired
```

## 14.3 Segment Repair

```pseudo
function repair_segment(segment_id) -> RepairResult:
    // Step 1: Identify affected Tiles
    affected_tiles = get_tiles_in_segment(segment_id)
    
    // Step 2: Repair each Tile
    repaired = 0
    failed = 0
    
    for tile_id in affected_tiles:
        result = repair_tile(tile_id)
        
        match result:
            case Repaired:
                repaired += 1
            case Failed:
                failed += 1
    
    // Step 3: Rebuild segment
    if failed == 0:
        rebuild_segment(segment_id, affected_tiles)
        return RepairResult::SegmentRepaired
    
    // Step 4: Partial repair
    return RepairResult::PartialRepair {
        repaired: repaired,
        failed: failed,
    }
```

## 14.4 Index Repair

```pseudo
function repair_indexes() -> RepairResult:
    // Step 1: Rebuild Cell index from manifest
    rebuild_cell_index()
    
    // Step 2: Rebuild Tile index from segments
    rebuild_tile_index()
    
    // Step 3: Rebuild Memory index from memory segments
    rebuild_memory_index()
    
    // Step 4: Rebuild Routing index from routing data
    rebuild_routing_index()
    
    // Step 5: Verify indexes
    verify_all_indexes()
    
    return RepairResult::IndexesRepaired
```

## 14.5 Repair Invariants

| ID | Invariant |
|---|---|
| REL-RP-INV-1 | Repair MUST mengikuti priority strategy |
| REL-RP-INV-2 | Repair MUST verifikasi setelah restore |
| REL-RP-INV-3 | Failed repair MUST dilaporkan eksplisit |
| REL-RP-INV-4 | Repair MUST idempotent |
| REL-RP-INV-5 | Manual intervention MUST documented |

---

# 15. Rollback Procedures

## 15.1 Rollback Definition

`[REL-RB-1]` Rollback mengubah active revision ke revision sebelumnya.

`[REL-RB-2]` Rollback MUST NOT menghapus revision yang sudah ada.

## 15.2 Rollback Protocol

```pseudo
function rollback(target_revision) -> Result<()>:
    // Step 1: Validate target
    if not revision_exists(target_revision):
        return Err(Error::RevisionNotFound)
    
    if not is_valid_rollback_target(target_revision):
        return Err(Error::InvalidRollbackTarget)
    
    // Step 2: Log rollback intent
    wal_append(ROLLBACK, {
        from: active_revision(),
        to: target_revision,
        initiated_at: now(),
    })
    
    // Step 3: Set active revision
    set_active_revision(target_revision)
    
    // Step 4: Invalidate caches
    invalidate_resolution_cache()
    invalidate_tile_cache()
    
    // Step 5: Rebuild effective graph
    rebuild_effective_graph(target_revision)
    
    // Step 6: Verify
    verify_active_revision()
    
    // Step 7: Log completion
    log_event("Rollback completed", {
        from: previous_revision,
        to: target_revision,
        completed_at: now(),
    })
    
    return Ok(())
```

## 15.3 Rollback Verification

`[REL-RB-3]` Setelah rollback, sistem MUST memverifikasi:

1. Active revision sesuai target
2. Effective graph dapat di-resolve
3. Tiles yang dibutuhkan tersedia
4. Manifest konsisten

## 15.4 Rollback Invariants

| ID | Invariant |
|---|---|
| REL-RB-INV-1 | Rollback MUST NOT menghapus revision |
| REL-RB-INV-2 | Rollback MUST logged |
| REL-RB-INV-3 | Rollback MUST diverifikasi |
| REL-RB-INV-4 | Rollback MUST O(1) |
| REL-RB-INV-5 | Rollback MUST reversible |

---

# 16. Recovery Guarantees

## 16.1 RPO (Recovery Point Objective)

`[REL-GUAR-1]` RPO untuk committed data: **0** (tidak ada data loss).

`[REL-GUAR-2]` RPO untuk uncommitted data: **last WAL record** (data sejak WAL record terakhir mungkin hilang).

## 16.2 RTO (Recovery Time Objective)

`[REL-GUAR-3]` RTO untuk crash recovery: **< 60 detik**.

`[REL-GUAR-4]` RTO untuk corruption recovery: **tergantung severity**, target < 5 menit.

`[REL-GUAR-5]` RTO untuk disk failure recovery: **tergantung extent**, target < 30 menit.

## 16.3 Data Integrity Guarantee

`[REL-GUAR-6]` Seluruh data yang di-serve MUST terverifikasi BLAKE3-256.

`[REL-GUAR-7]` Data yang gagal verifikasi MUST NOT di-serve.

## 16.4 Availability Guarantee

`[REL-GUAR-8]` Sistem MUST dapat beroperasi dalam degraded mode jika sebagian data unavailable.

`[REL-GUAR-9]` Degraded mode MUST dilaporkan eksplisit ke operator.

## 16.5 Recovery Guarantees Summary

| Guarantee | Target |
|---|---|
| RPO (committed data) | 0 |
| RPO (uncommitted data) | Last WAL record |
| RTO (crash) | < 60 detik |
| RTO (corruption) | < 5 menit |
| RTO (disk failure) | < 30 menit |
| Data integrity | 100% BLAKE3 verified |
| Availability | Degraded mode supported |

---

# 17. Recovery Testing

## 17.1 Testing Requirements

`[REL-TEST-1]` Recovery MUST diuji secara otomatis dan repeatable.

## 17.2 Test Categories

| Category | Tests |
|---|---|
| Crash recovery | Kill process at various points, verify recovery |
| WAL recovery | Corrupt/truncate WAL, verify recovery |
| Manifest recovery | Corrupt manifest, verify recovery |
| Segment recovery | Corrupt segment, verify recovery |
| Tile recovery | Delete/corrupt Tiles, verify recovery |
| Disk failure | Simulate disk failure, verify recovery |
| Rollback | Perform rollback, verify state |
| Idempotency | Run recovery multiple times, verify same result |

## 17.3 Crash Injection Tests

```pseudo
test_crash_during_commit():
    // Start commit
    start_commit()
    
    // Kill process at various points
    for crash_point in [BEFORE_STAGING, AFTER_STAGING, AFTER_RENAME, AFTER_SUPERBLOCK]:
        inject_crash(crash_point)
        
        // Restart and recover
        restart()
        result = recover()
        
        // Verify
        assert result.status == RECOVERED
        assert store_is_consistent()
        assert no_data_loss()
```

## 17.4 Corruption Injection Tests

```pseudo
test_segment_corruption():
    // Corrupt a segment
    corrupt_segment(segment_id, corruption_type=BIT_FLIP)
    
    // Attempt to load
    result = load_segment(segment_id)
    
    // Verify detection
    assert result.status == CORRUPTION_DETECTED
    
    // Recover
    recovery = recover_corrupted_segment(segment_id)
    
    // Verify recovery
    assert recovery.tiles_recovered > 0
```

## 17.5 Recovery Test Invariants

| ID | Invariant |
|---|---|
| REL-TEST-INV-1 | Recovery tests MUST otomatis |
| REL-TEST-INV-2 | Recovery tests MUST repeatable |
| REL-TEST-INV-3 | Recovery tests MUST mencakup semua failure modes |
| REL-TEST-INV-4 | Recovery tests MUST menjalankan idempotency check |
| REL-TEST-INV-5 | Recovery test failures MUST memblokir release |

---

# 18. Final Reliability Contract

## 18.1 Ringkasan Keputusan Reliability

| ID | Keputusan |
|---|---|
| REL-F01 | Crash consistency: WAL + atomic rename + fsync. |
| REL-F02 | Recovery MUST idempotent. |
| REL-F03 | Recovery MUST NOT kehilangan committed data. |
| REL-F04 | Uncommitted data MAY hilang saat crash. |
| REL-F05 | Quarantine untuk corrupted/malicious data. |
| REL-F06 | Repair strategy: replica → remote → re-import → rebuild → manual. |
| REL-F07 | Rollback tidak menghapus revision. |
| REL-F08 | RPO = 0 untuk committed data. |
| REL-F09 | RTO < 60 detik untuk crash recovery. |
| REL-F10 | Partial disk failure ditangani per-segment. |
| REL-F11 | Missing Tile terdeteksi melalui index verification. |
| REL-F12 | Recovery memiliki timeout dan fallback. |
| REL-F13 | Degraded mode supported. |
| REL-F14 | Recovery testing MUST otomatis. |
| REL-F15 | WAL MUST di-fsync setelah append. |
| REL-F16 | Atomic commit MUST menggunakan staging + rename. |
| REL-F17 | Manifest recovery: prev → staging → WAL → rebuild. |
| REL-F18 | Segment corruption MUST dikarantina. |
| REL-F19 | Tile recovery: replica → remote → re-import. |
| REL-F20 | Index MUST dapat di-rebuild. |

## 18.2 Reliability Invariants

| ID | Invariant |
|---|---|
| REL-INV-1 | Store MUST selalu dalam state konsisten. |
| REL-INV-2 | Crash MUST NOT menghasilkan partial commit. |
| REL-INV-3 | Recovery MUST idempotent. |
| REL-INV-4 | Committed data MUST NOT hilang. |
| REL-INV-5 | WAL MUST di-append sebelum perubahan. |
| REL-INV-6 | WAL append MUST diikuti fsync. |
| REL-INV-7 | Atomic commit MUST menggunakan staging + rename. |
| REL-INV-8 | Corrupted data MUST dikarantina. |
| REL-INV-9 | Quarantined data MUST NOT dieksekusi. |
| REL-INV-10 | Missing Tile MUST terdeteksi. |
| REL-INV-11 | Recovery MUST mencoba replica sebelum error. |
| REL-INV-12 | Recovery MUST mencoba remote sebelum error. |
| REL-INV-13 | Unrecoverable data MUST dilaporkan eksplisit. |
| REL-INV-14 | Rollback MUST NOT menghapus revision. |
| REL-INV-15 | Degraded mode MUST dilaporkan. |
| REL-INV-16 | Recovery MUST memiliki timeout. |
| REL-INV-17 | Recovery testing MUST otomatis. |
| REL-INV-18 | Index MUST dapat di-rebuild. |
| REL-INV-19 | Manifest MUST dapat di-recover. |
| REL-INV-20 | Segment MUST dapat di-recover. |

## 18.3 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Reliability, Recovery & Disaster final dan mengikat** untuk CNWS. Ia mendefinisikan behavior sistem ketika sesuatu rusak, dari crash consistency hingga disaster recovery, dari WAL hingga quarantine, dari repair hingga rollback.

CNWS dirancang dengan prinsip **"assume failure"** — setiap komponen dapat gagal, dan sistem MUST dapat pulih. Recovery adalah bagian integral dari arsitektur, bukan afterthought.

Seluruh implementasi Recovery Subsystem, WAL Manager, Quarantine Manager, Repair Engine, dan Rollback Manager CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan reliability yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN RELIABILITY, RECOVERY & DISASTER SPECIFICATION**
