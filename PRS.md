# CNWS
## Product Requirements Specification (PRS)

| Field | Value |
|---|---|
| Dokumen | CNWS Product Requirements Specification |
| Status | **FINAL, NORMATIF, MENGIKAT** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract (Final Technical & Architecture Specification) |
| Hulu ke | Implementasi, testing, validation, acceptance |
| Otoritas | Single Source of Truth untuk kebutuhan produk CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen dalam Rantai Spesifikasi

```text
Engineering Contract          Product Requirements          Implementation
(Architecture & Technical)    Specification (PRS)           (Code & Tests)
─────────────────────         ─────────────────────         ─────────────
HOW it is built        ───►   WHAT it must deliver   ───►   Build & verify
Final architecture            Target use cases               Conformant code
Data model                    Capability matrix              Conformance tests
Binary format                 Functional requirements        Acceptance tests
Invariants                    Performance targets            Benchmarks
                              Resource targets
                              Acceptance criteria
                              Product boundaries
```

`[DOC-1]` PRS ini MUST diturunkan dari Engineering Contract dan MUST NOT bertentangan dengannya.

`[DOC-2]` Jika terjadi konflik antara PRS dan Engineering Contract, Engineering Contract menang.

`[DOC-3]` PRS ini mendefinisikan **apa** yang harus dipenuhi, bukan **bagaimana** membangunnya.

`[DOC-4]` Seluruh requirement dalam PRS ini MUST terukur dan dapat diverifikasi.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **REQUIRED**, **SHALL**, **SHALL NOT**, **SHOULD**, **SHOULD NOT**, **RECOMMENDED**, **MAY**, dan **OPTIONAL** diinterpretasikan sesuai RFC 2119.

## 0.3 ID Requirement

| Prefix | Makna |
|---|---|
| `UC-*` | Use Case |
| `CAP-*` | Capability |
| `FR-*` | Functional Requirement |
| `WF-*` | Workflow |
| `PT-*` | Performance Target |
| `RT-*` | Resource Target |
| `AC-*` | Acceptance Criteria |
| `PB-*` | Product Boundary |

---

# 1. Executive Summary

## 1.1 Definisi Produk

**CNWS** adalah canonical intelligence infrastructure yang menyediakan tiga kemampuan produk utama:

1. **Conversion**: mengubah checkpoint LLM besar dari berbagai format eksternal menjadi representasi canonical yang terstruktur dan independen dari format sumber.
2. **Runtime**: menyediakan selective, adaptive, dan bounded-memory weight loading untuk eksekusi model besar.
3. **Versioning**: menyediakan incremental revision, branching, specialization, dan cross-model sharing tanpa duplikasi penuh.

## 1.2 Nilai Produk

| Masalah yang Dipecahkan | Nilai CNWS |
|---|---|
| Model terlalu besar untuk RAM | Streaming conversion + selective loading |
| Terikat pada format checkpoint tertentu | Zero Format Coupling |
| Duplikasi storage untuk setiap revision | Content-addressed deduplication |
| Fine-tuning membutuhkan copy penuh | Incremental Tile-level delta |
| MoE memuat seluruh expert | Selective expert loading |
| Tidak ada integrity verification | BLAKE3-256 content addressing |
| Runtime bergantung pada format sumber | Canonical manifest `.cd` |

## 1.3 Persona Pengguna

| Persona | Deskripsi | Kebutuhan Utama |
|---|---|---|
| ML Engineer | Mengonversi dan men-deploy model | Conversion andal, runtime cepat |
| Infra Engineer | Mengelola storage dan deployment | Deduplication, integrity, efisiensi storage |
| Researcher | Fine-tuning dan specialization | Branching, rollback, versioning |
| Platform Team | Membangun platform inference | API stabil, selective loading, budgeting |

---

# 2. Target Use Cases

## 2.1 Use Case Matrix

| ID | Use Case | Prioritas | Status |
|---|---|---|---|
| UC-01 | Konversi checkpoint Safetensors ke canonical `.cd` | P0 | MUST |
| UC-02 | Konversi checkpoint GGUF ke canonical `.cd` | P0 | MUST |
| UC-03 | Konversi checkpoint PyTorch ke canonical `.cd` | P0 | MUST |
| UC-04 | Konversi checkpoint custom ke canonical `.cd` | P1 | SHOULD |
| UC-05 | Selective loading weight untuk inference | P0 | MUST |
| UC-06 | Adaptive representation selection berdasarkan hardware | P0 | MUST |
| UC-07 | MoE selective expert loading | P0 | MUST |
| UC-08 | Fine-tuning menghasilkan revision incremental | P0 | MUST |
| UC-09 | Branching specialization tanpa copy model | P0 | MUST |
| UC-10 | Merging dua specialization branch | P1 | SHOULD |
| UC-11 | Rollback ke revision sebelumnya | P0 | MUST |
| UC-12 | Cross-model Tile deduplication | P0 | MUST |
| UC-13 | Integrity verification Tile sebelum eksekusi | P0 | MUST |
| UC-14 | Garbage collection Tile unreferenced | P1 | SHOULD |
| UC-15 | Streaming conversion dengan bounded memory | P0 | MUST |
| UC-16 | Multi-hardware deployment (GPU/CPU/NVMe) | P0 | MUST |
| UC-17 | Remote Tile loading dari object storage | P2 | MAY |
| UC-18 | Continual learning / incremental update | P1 | SHOULD |
| UC-19 | Concurrent inference dengan shared model store | P1 | SHOULD |
| UC-20 | Model provenance tracking | P1 | SHOULD |

## 2.2 Use Case Detail

### UC-01: Konversi Safetensors

```text
Aktor: ML Engineer
Precondition: File Safetensors tersedia di disk
Trigger: Perintah import CNWS
Flow:
  1. CNWS membaca header Safetensors
  2. CNWS memetakan tensor ke semantic CellId
  3. CNWS melakukan streaming read per tensor
  4. CNWS memecah tensor menjadi Tiles
  5. CNWS menghitung BLAKE3-256 untuk setiap Tile
  6. CNWS melakukan deduplication terhadap Tile Registry
  7. CNWS menulis Tile baru ke segment
  8. CNWS membangun manifest dan commit
Postcondition: Model tersedia sebagai canonical `.cd`
Invariant: Peak RAM tidak bergantung pada ukuran model
```

### UC-05: Selective Loading

```text
Aktor: Execution Engine (sistem)
Precondition: Model `.cd` tersedia, runtime aktif
Trigger: Permintaan weight untuk Cell tertentu
Flow:
  1. Runtime menerima permintaan Cell ID
  2. Runtime resolve Cell → TileRef
  3. Runtime cek cache hierarchy (GPU → CPU → NVMe)
  4. Jika miss, runtime load Tile dari segment
  5. Runtime verifikasi BLAKE3-256
  6. Runtime tempatkan Tile di cache level sesuai
  7. Runtime mengembalikan handle Tile
Postcondition: Weight tersedia untuk eksekusi
Invariant: Hanya Tile yang dibutuhkan yang di-load
```

### UC-08: Fine-tuning Incremental

```text
Aktor: Researcher
Precondition: Base revision tersedia
Trigger: Fine-tuning selesai, commit revision baru
Flow:
  1. Sistem mengidentifikasi Cell yang berubah
  2. Sistem generate Tile baru untuk Cell berubah
  3. Sistem hitung BLAKE3-256 Tile baru
  4. Sistem deduplikasi dengan Tile existing
  5. Sistem buat revision baru dengan delta
  6. Tile tidak berubah direferensikan dari ancestor
Postcondition: Revision baru tersedia tanpa full copy
Invariant: Storage delta << ukuran model penuh
```

---

# 3. Capability Matrix

## 3.1 Capability Overview

| ID | Capability | Kategori | Prioritas | Status |
|---|---|---|---|---|
| CAP-01 | Streaming-First Conversion | Conversion | P0 | MUST |
| CAP-02 | Multi-format Import (Safetensors/GGUF/PyTorch) | Conversion | P0 | MUST |
| CAP-03 | Custom Format Import | Conversion | P1 | SHOULD |
| CAP-04 | Canonical Normalization | Conversion | P0 | MUST |
| CAP-05 | Tile Planning & Splitting | Conversion | P0 | MUST |
| CAP-06 | BLAKE3-256 Content Addressing | Storage | P0 | MUST |
| CAP-07 | Immutable Tile Storage | Storage | P0 | MUST |
| CAP-08 | Segment-based Physical Storage | Storage | P0 | MUST |
| CAP-09 | Tile Deduplication | Storage | P0 | MUST |
| CAP-10 | Compression (zstd) | Storage | P1 | SHOULD |
| CAP-11 | Multiple Representations | Storage | P1 | SHOULD |
| CAP-12 | Canonical Manifest (.cd) | Storage | P0 | MUST |
| CAP-13 | Zero Format Coupling | Runtime | P0 | MUST |
| CAP-14 | Selective Cell Loading | Runtime | P0 | MUST |
| CAP-15 | Adaptive Representation Selection | Runtime | P0 | MUST |
| CAP-16 | Cache Hierarchy (GPU/CPU/NVMe) | Runtime | P0 | MUST |
| CAP-17 | Asynchronous Tile Loading | Runtime | P0 | MUST |
| CAP-18 | Prefetch Engine | Runtime | P1 | SHOULD |
| CAP-19 | MoE Selective Expert Loading | Runtime | P0 | MUST |
| CAP-20 | Memory Budget Enforcement | Runtime | P0 | MUST |
| CAP-21 | Integrity Verification | Integrity | P0 | MUST |
| CAP-22 | Corruption Detection & Quarantine | Integrity | P0 | MUST |
| CAP-23 | Revision DAG | Versioning | P0 | MUST |
| CAP-24 | Branching | Versioning | P0 | MUST |
| CAP-25 | Merging | Versioning | P1 | SHOULD |
| CAP-26 | Rollback | Versioning | P0 | MUST |
| CAP-27 | Tile-level Delta | Versioning | P0 | MUST |
| CAP-28 | Garbage Collection | Versioning | P1 | SHOULD |
| CAP-29 | Cross-model Tile Sharing | Versioning | P1 | SHOULD |
| CAP-30 | Provenance Tracking | Metadata | P1 | SHOULD |

## 3.2 Capability Dependency

```text
CAP-01 (Streaming Conversion)
  ├── CAP-02/03/04 (Import & Normalize)
  ├── CAP-05 (Tile Planning)
  └── CAP-06 (BLAKE3)

CAP-06 (BLAKE3)
  ├── CAP-07 (Immutable Storage)
  ├── CAP-09 (Deduplication)
  ├── CAP-21 (Integrity)
  └── CAP-23 (Revision DAG)

CAP-12 (Canonical Manifest)
  ├── CAP-13 (Zero Format Coupling)
  ├── CAP-14 (Selective Loading)
  └── CAP-15 (Adaptive Representation)

CAP-14 (Selective Loading)
  ├── CAP-16 (Cache Hierarchy)
  ├── CAP-17 (Async Loading)
  ├── CAP-19 (MoE Selective)
  └── CAP-20 (Memory Budget)
```

---

# 4. Functional Requirements

## 4.1 Conversion Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-C01 | Sistem MUST dapat mengimpor checkpoint Safetensors. | P0 |
| FR-C02 | Sistem MUST dapat mengimpor checkpoint GGUF. | P0 |
| FR-C03 | Sistem MUST dapat mengimpor checkpoint PyTorch. | P0 |
| FR-C04 | Sistem SHOULD dapat mengimpor checkpoint custom melalui adapter. | P1 |
| FR-C05 | Sistem MUST memproses weight secara streaming incremental. | P0 |
| FR-C06 | Sistem MUST menggunakan bounded buffer selama conversion. | P0 |
| FR-C07 | Sistem MUST memetakan tensor sumber ke semantic CellId. | P0 |
| FR-C08 | Sistem MUST memecah Cell menjadi Tiles berdasarkan ukuran target. | P0 |
| FR-C09 | Sistem MUST menghitung BLAKE3-256 untuk setiap Tile. | P0 |
| FR-C10 | Sistem MUST melakukan deduplication terhadap Tile Registry. | P0 |
| FR-C11 | Sistem MUST menyimpan Tile ke segment immutable. | P0 |
| FR-C12 | Sistem MUST membangun canonical manifest setelah conversion. | P0 |
| FR-C13 | Sistem MUST melakukan atomic commit untuk manifest. | P0 |
| FR-C14 | Sistem MUST menolak checkpoint yang mengandung kode executable. | P0 |
| FR-C15 | Sistem SHOULD menyimpan provenance source checkpoint. | P1 |

## 4.2 Storage Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-S01 | Sistem MUST menyimpan Tile dalam format immutable. | P0 |
| FR-S02 | Sistem MUST menggunakan BLAKE3-256 sebagai Tile identity. | P0 |
| FR-S03 | Sistem MUST menyimpan Tile dalam segment, bukan satu file per Tile. | P0 |
| FR-S04 | Sistem MUST menyediakan segment index untuk lookup O(1). | P0 |
| FR-S05 | Sistem MUST mendukung alignment Tile payload minimum 4 KiB. | P0 |
| FR-S06 | Sistem SHOULD mendukung compression zstd. | P1 |
| FR-S07 | Sistem SHOULD mendukung multiple representations per Tile. | P1 |
| FR-S08 | Sistem MUST menyimpan canonical manifest sebagai source of truth. | P0 |
| FR-S09 | Sistem MUST mendukung global Tile pool untuk cross-model sharing. | P1 |
| FR-S10 | Sistem MUST menyimpan Tile metadata (shape, dtype, offset, size). | P0 |

## 4.3 Runtime Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-R01 | Sistem MUST dapat resolve Cell berdasarkan semantic ID. | P0 |
| FR-R02 | Sistem MUST dapat load subset Tile dari sebuah Cell. | P0 |
| FR-R03 | Sistem MUST memilih representation berdasarkan hardware dan workload. | P0 |
| FR-R04 | Sistem MUST menggunakan cache hierarchy GPU/CPU/NVMe. | P0 |
| FR-R05 | Sistem MUST mendukung asynchronous Tile loading. | P0 |
| FR-R06 | Sistem SHOULD melakukan prefetch berdasarkan dependency graph. | P1 |
| FR-R07 | Sistem MUST hanya memuat expert yang dipilih router untuk MoE. | P0 |
| FR-R08 | Sistem MUST melakukan deduplikasi expert IDs dalam batch. | P0 |
| FR-R09 | Sistem MUST enforce hard memory budget. | P0 |
| FR-R10 | Sistem MUST melakukan admission control sebelum load Tile baru. | P0 |
| FR-R11 | Sistem MUST melakukan eviction berdasarkan byte capacity. | P0 |
| FR-R12 | Sistem MUST memverifikasi BLAKE3-256 Tile sebelum eksekusi. | P0 |
| FR-R13 | Sistem MUST menolak Tile yang gagal integrity check. | P0 |
| FR-R14 | Sistem MUST NOT memahami format checkpoint eksternal di runtime. | P0 |
| FR-R15 | Sistem SHOULD mendukung remote Tile loading. | P2 |

## 4.4 Versioning Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-V01 | Sistem MUST membuat revision immutable untuk setiap perubahan. | P0 |
| FR-V02 | Sistem MUST menyimpan delta pada level Tile, bukan full model. | P0 |
| FR-V03 | Sistem MUST mereferensikan Tile tidak berubah dari ancestor. | P0 |
| FR-V04 | Sistem MUST mendukung branching tanpa copy fisik. | P0 |
| FR-V05 | Sistem SHOULD mendukung merging dengan three-way merge. | P1 |
| FR-V06 | Sistem MUST mendukung rollback ke revision sebelumnya. | P0 |
| FR-V07 | Sistem MUST mendukung multiple parents dalam revision DAG. | P1 |
| FR-V08 | Sistem MUST melakukan GC berbasis reachability dari revision roots. | P1 |
| FR-V09 | Sistem MUST NOT menghapus Tile yang masih reachable. | P0 |
| FR-V10 | Sistem MUST menyimpan revision metadata (author, timestamp, message). | P0 |

## 4.5 Integrity Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-I01 | Sistem MUST menggunakan BLAKE3-256 untuk Tile integrity. | P0 |
| FR-I02 | Sistem MUST menggunakan BLAKE3-256 untuk manifest integrity. | P0 |
| FR-I03 | Sistem MUST mendeteksi same-ID-different-payload sebagai fatal error. | P0 |
| FR-I04 | Sistem MUST mengarantina Tile korup ke direktori `corrupt/`. | P0 |
| FR-I05 | Sistem MUST melakukan streaming hash tanpa buffer duplikat. | P0 |

## 4.6 Concurrency Requirements

| ID | Requirement | Prioritas |
|---|---|---|
| FR-X01 | Sistem MUST mendukung multiple reader concurrent. | P0 |
| FR-X02 | Sistem MUST menggunakan single-writer untuk commit. | P0 |
| FR-X03 | Sistem MUST menggunakan advisory lock untuk write operations. | P0 |
| FR-X04 | Sistem MUST thread-safe untuk concurrent Cell reads. | P0 |

---

# 5. User & System Workflows

## 5.1 Workflow Import Model

```text
┌─────────────┐
│ User        │
│ menyediakan │
│ checkpoint  │
└──────┬──────┘
       │
       ▼
┌─────────────────────┐
│ cnws import \       │
│   --source model.safetensors \
│   --target model.cd │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Format Detection    │
│ & Validation        │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Streaming Read      │
│ (bounded buffer)    │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Normalize to        │
│ Semantic CellId     │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Tile Planning       │
│ & Splitting         │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ BLAKE3-256 Hash     │
│ (streaming)         │
└──────┬──────────────┘
       │
       ├── duplicate ──► reuse Tile
       │
       └── new ──► write Tile
              │
              ▼
┌─────────────────────┐
│ Segment Write       │
│ (aligned, immutable)│
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ Manifest Build      │
│ & Atomic Commit     │
└──────┬──────────────┘
       │
       ▼
┌─────────────────────┐
│ model.cd ready      │
└─────────────────────┘
```

`[WF-01]` Workflow import MUST berhasil tanpa memerlukan RAM sebesar model.

`[WF-02]` Workflow import MUST menghasilkan `.cd` yang valid dan complete.

## 5.2 Workflow Inference Selective Loading

```text
┌──────────────────┐
│ Execution Engine │
│ request Cell X   │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Runtime Resolver │
│ resolve CellId   │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Tile Selector    │
│ determine needed │
│ Tiles            │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Representation   │
│ Selector         │
└──────┬───────────┘
       │
       ▼
┌──────────────────────────────────┐
│ Cache Lookup                     │
│ GPU → CPU RAM → NVMe → Remote    │
└──────┬───────────────────────────┘
       │
       ├── hit ──► return Tile handle
       │
       └── miss ──► load from segment
                       │
                       ▼
                ┌──────────────┐
                │ BLAKE3 verify│
                └──────┬───────┘
                       │
                       ├── valid ──► cache & return
                       │
                       └── invalid ──► quarantine & error
```

`[WF-03]` Workflow loading MUST hanya memuat Tile yang dibutuhkan.

`[WF-04]` Workflow loading MUST memverifikasi integrity sebelum eksekusi.

## 5.3 Workflow Fine-tuning / Specialization

```text
┌──────────────────┐
│ Base Revision    │
│ (Revision 0)     │
└──────┬───────────┘
       │
       ├── branch("coding")
       │
       ▼
┌──────────────────┐
│ Fine-tuning      │
│ (only some Cells │
│  change)         │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Identify changed │
│ Cells            │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Generate new     │
│ Tiles for        │
│ changed Cells    │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Dedup check      │
│ against registry │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Create Revision 1│
│ delta:           │
│  - changed Tiles │
│  - unchanged →   │
│    ref ancestor  │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Commit Revision  │
│ (immutable)      │
└──────────────────┘
```

`[WF-05]` Workflow specialization MUST NOT menyalin seluruh model.

`[WF-06]` Tile yang tidak berubah MUST direferensikan dari ancestor.

## 5.4 Workflow Branching & Merging

```text
Base (Rev 0)
    │
    ├── branch A ──► Rev A1 ──► Rev A2
    │
    └── branch B ──► Rev B1

merge(A2, B1):
    │
    ▼
Three-way merge:
    if A == Base: use B
    if B == Base: use A
    if A == B:    use A
    else:         CONFLICT
    │
    ▼
Merge Revision (Rev M)
    parents: [A2, B1]
```

`[WF-07]` Merge MUST menggunakan three-way merge pada level Cell/Tile.

`[WF-08]` Conflict MUST dilaporkan secara eksplisit kepada pengguna.

## 5.5 Workflow Rollback

```text
Rev 0 → Rev 1 → Rev 2 (broken) → Rev 3

Rollback:
  set_active_revision(Rev 1)

Hasil:
  Active = Rev 1
  Rev 2, Rev 3 tetap ada (immutable)
  Tidak ada data yang dihapus
```

`[WF-09]` Rollback MUST NOT menghapus revision yang sudah ada.

## 5.6 Workflow Garbage Collection

```text
┌──────────────────┐
│ Identify active  │
│ revision roots   │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Reachability     │
│ traversal        │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Mark referenced  │
│ Tiles            │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Sweep            │
│ unreferenced     │
│ Tiles            │
└──────┬───────────┘
       │
       ▼
┌──────────────────┐
│ Reclaim storage  │
└──────────────────┘
```

`[WF-10]` GC MUST NOT menghapus Tile yang reachable dari revision root mana pun.

---

# 6. Performance Targets

## 6.1 Conversion Performance

| ID | Metric | Target | Priority |
|---|---|---|---|
| PT-C01 | Peak RAM conversion | ≤ 4 GiB untuk model hingga 1 TB | P0 |
| PT-C02 | Conversion throughput | ≥ 500 MB/s sequential read | P1 |
| PT-C03 | Hashing overhead | ≤ 10% dari total conversion time | P1 |
| PT-C04 | Deduplication check | O(1) per Tile | P0 |
| PT-C05 | Manifest build time | ≤ 5 detik untuk model 100K Tiles | P1 |

## 6.2 Runtime Performance

| ID | Metric | Target | Priority |
|---|---|---|---|
| PT-R01 | Cell resolve latency | ≤ 1 μs setelah manifest loaded | P0 |
| PT-R02 | Tile lookup latency | ≤ 10 μs melalui index | P0 |
| PT-R03 | Cache hit rate (steady state) | ≥ 90% untuk workload inference tipikal | P1 |
| PT-R04 | Tile load latency (NVMe → CPU) | ≤ 5 ms per 128 MiB Tile | P1 |
| PT-R05 | Tile load latency (CPU → GPU) | ≤ 2 ms per 128 MiB Tile | P1 |
| PT-R06 | Manifest load time | ≤ 30 detik untuk model 100K Tiles | P1 |
| PT-R07 | MoE expert load overhead | ≤ 5 ms per expert miss | P1 |
| PT-R08 | Integrity verification throughput | ≥ 1 GB/s streaming BLAKE3 | P1 |

## 6.3 Versioning Performance

| ID | Metric | Target | Priority |
|---|---|---|---|
| PT-V01 | Revision commit latency | ≤ 1 detik untuk delta 100 Tiles | P0 |
| PT-V02 | Revision resolution (startup) | ≤ 10 detik untuk DAG depth 100 | P1 |
| PT-V03 | Branch creation | ≤ 100 ms (no physical copy) | P0 |
| PT-V04 | GC mark phase | ≤ 60 detik untuk 1M Tiles | P1 |
| PT-V05 | Rollback | ≤ 1 detik (switch active revision) | P0 |

---

# 7. Resource Targets

## 7.1 Memory Targets

| ID | Resource | Target | Priority |
|---|---|---|---|
| RT-M01 | Peak RAM during conversion | ≤ 4 GiB (configurable) | P0 |
| RT-M02 | Runtime manifest memory | ≤ 2 GiB untuk model 100K Cells | P0 |
| RT-M03 | GPU VRAM cache budget | Configurable, default ≤ 80% VRAM | P0 |
| RT-M04 | CPU RAM cache budget | Configurable, default ≤ 50% available RAM | P0 |
| RT-M05 | Working set bounded | Tidak bergantung pada ukuran model | P0 |

## 7.2 Storage Targets

| ID | Resource | Target | Priority |
|---|---|---|---|
| RT-S01 | Segment size | 32 GiB target (configurable) | P1 |
| RT-S02 | Tile size default | 128 MiB (range 32-256 MiB) | P1 |
| RT-S03 | Deduplication ratio (revision) | ≥ 90% Tile reuse untuk fine-tuning tipikal | P0 |
| RT-S04 | Deduplication ratio (cross-model) | ≥ 50% untuk model dengan arsitektur sama | P1 |
| RT-S05 | Storage overhead (manifest + index) | ≤ 1% dari total Tile size | P1 |
| RT-S06 | Revision delta storage | O(changed_tiles), bukan O(model_size) | P0 |

## 7.3 Compute Targets

| ID | Resource | Target | Priority |
|---|---|---|---|
| RT-X01 | BLAKE3 hashing CPU usage | ≤ 2 core selama conversion | P1 |
| RT-X02 | Runtime resolver CPU | ≤ 1 core untuk resolution logic | P1 |
| RT-X03 | Concurrent inference threads | Configurable | P1 |

---

# 8. Acceptance Criteria

## 8.1 Conversion Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-C01 | Import Safetensors 70B berhasil dengan peak RAM ≤ 4 GiB | Benchmark test |
| AC-C02 | Import GGUF 7B berhasil dan menghasilkan `.cd` valid | Functional test |
| AC-C03 | Import PyTorch 13B berhasil tanpa eksekusi kode | Security test |
| AC-C04 | Hasil conversion deterministik (Tile ID sama untuk input sama) | Determinism test |
| AC-C05 | Conversion dapat di-resume setelah interupsi | Recovery test |
| AC-C06 | Checkpoint korup terdeteksi dan ditolak | Negative test |

## 8.2 Storage Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-S01 | Tile immutable setelah ditulis (tidak bisa dimodifikasi) | Immutability test |
| AC-S02 | Tile ID = BLAKE3-256(canonical payload) terverifikasi | Hash verification test |
| AC-S03 | Dua Tile identik menghasilkan satu physical copy | Dedup test |
| AC-S04 | Segment index memungkinkan lookup O(1) | Performance test |
| AC-S05 | Manifest valid dan dapat di-load setelah commit | Functional test |
| AC-S06 | Atomic commit tidak menghasilkan partial manifest | Crash recovery test |

## 8.3 Runtime Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-R01 | Cell resolve O(1) setelah manifest loaded | Performance test |
| AC-R02 | Hanya Tile yang dibutuhkan yang di-load | Tracing test |
| AC-R03 | Representation selection sesuai hardware profile | Functional test |
| AC-R04 | Cache eviction tidak melebihi budget | Budget enforcement test |
| AC-R05 | MoE hanya memuat expert yang dipilih | MoE test |
| AC-R06 | Tile korup terdeteksi dan dikarantina | Corruption test |
| AC-R07 | Runtime tidak memiliki dependency format eksternal | Code inspection + test |
| AC-R08 | Full-model residency tidak diperlukan | Memory profiling test |

## 8.4 Versioning Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-V01 | Revision baru hanya menyimpan delta | Storage measurement test |
| AC-V02 | Branch tidak menyalin Tile fisik | Storage measurement test |
| AC-V03 | Rollback mengembalikan state revision target | Functional test |
| AC-V04 | Merge three-way menghasilkan hasil benar | Merge test |
| AC-V05 | GC tidak menghapus Tile reachable | GC safety test |
| AC-V06 | Tile shared antar revision hanya disimpan sekali | Dedup verification test |

## 8.5 Integrity Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-I01 | BLAKE3 mismatch terdeteksi sebagai korupsi | Corruption injection test |
| AC-I02 | Same-ID-different-payload menghasilkan fatal error | Negative test |
| AC-I03 | Streaming hash tidak membutuhkan buffer duplikat | Memory profiling test |
| AC-I04 | Manifest integrity terverifikasi saat load | Integrity test |

## 8.6 Conformance Acceptance

| ID | Criteria | Verification Method |
|---|---|---|
| AC-F01 | Seluruh conformance test suite lulus | Automated test suite |
| AC-F02 | Tidak ada invariant `FAC-*` yang dilanggar | Invariant verification |
| AC-F03 | Interoperability: `.cd` dapat dibaca oleh implementasi independen | Interop test |
| AC-F04 | Error model sesuai spesifikasi | Error injection test |

---

# 9. Product Boundaries (Batasan Produk)

## 9.1 In Scope

| Item | Status |
|---|---|
| Conversion checkpoint ke canonical `.cd` | In scope |
| Canonical storage dan Tile management | In scope |
| Selective & adaptive runtime loading | In scope |
| Revision DAG dan versioning | In scope |
| Integrity verification | In scope |
| Deduplication | In scope |
| Cache hierarchy management | In scope |
| MoE selective loading | In scope |

## 9.2 Out of Scope

| Item | Status | Alasan |
|---|---|---|
| Training / optimizer | Out of scope | CNWS adalah infrastructure, bukan training framework |
| Inference kernel / operator | Out of scope | CNWS menyediakan weight, bukan compute kernel |
| Model architecture design | Out of scope | CNWS menerima arsitektur apa pun |
| Tokenizer | Out of scope | Bukan bagian weight infrastructure |
| Distributed training scheduler | Out of scope | Di luar inti produk |
| Serving / API gateway | Out of scope | Layer di atas CNWS |
| Encryption at rest (mandatory) | Out of scope | MAY sebagai optional layer |
| Model compression / quantization training | Out of scope | CNWS menyimpan representation yang sudah ada |

## 9.3 Constraints

| ID | Constraint |
|---|---|
| PB-01 | CNWS MUST NOT mengeksekusi kode dari checkpoint. |
| PB-02 | CNWS MUST NOT mengubah semantic model saat conversion. |
| PB-03 | CNWS MUST NOT bergantung pada satu vendor hardware tertentu. |
| PB-04 | CNWS MUST NOT memerlukan jaringan untuk operasi lokal dasar. |
| PB-05 | CNWS MUST NOT mengubah Tile yang sudah immutable. |
| PB-06 | CNWS MUST NOT menggunakan SHA-256 sebagai identity primitive. |
| PB-07 | CNWS MUST NOT menyimpan state runtime ephemeral dalam `.cd` secara mandatory. |

## 9.4 Assumptions

| ID | Assumption |
|---|---|
| PB-A01 | Storage lokal (NVMe/SSD) tersedia untuk `.cd` store. |
| PB-A02 | Filesystem mendukung atomic rename dan fsync. |
| PB-A03 | Hardware mendukung BLAKE3 (atau software fallback tersedia). |
| PB-A04 | Checkpoint sumber dapat dibaca secara sequential. |

---

# 10. Hubungan Engineering Contract → PRS → Implementation

## 10.1 Traceability Matrix

```text
Engineering Contract              PRS                         Implementation
────────────────────              ───                         ──────────────
FAC-1  (Cell fundamental)    ───► CAP-06, FR-C07        ───► Cell data structure
FAC-3  (Tile storage)        ───► CAP-07, FR-S01        ───► Tile writer
FAC-5  (BLAKE3 identity)     ───► CAP-06, FR-C09        ───► Hash module
FAC-8  (.cd source of truth) ───► CAP-12, FR-S08        ───► Manifest module
FAC-11 (Zero Format Coupling)───► CAP-13, FR-R14        ───► Runtime boundary
FAC-12 (Streaming conversion)───► CAP-01, FR-C05        ───► Pipeline module
FAC-13 (Content-based select)───► CAP-14, FR-R01        ───► Resolver
FAC-16 (Memory content-addr) ───► CAP-16, FR-R04        ───► Cache manager
FAC-24 (Revision DAG)        ───► CAP-23, FR-V01        ───► Revision module
FAC-26 (No full-model load)  ───► CAP-14, FR-R02        ───► Tile selector
FAC-30 (Active param < 10%)  ───► PT-R07, AC-R05        ───► MoE loader
FAC-32 (GC reachability)     ───► CAP-28, FR-V08        ───► GC module
FAC-33 (Integrity verify)    ───► CAP-21, FR-R12        ───► Verify module
```

## 10.2 Alur Verifikasi

```text
Engineering Contract
        │
        │ mendefinisikan invariants & architecture
        ▼
      PRS
        │
        │ mendefinisikan requirements & acceptance criteria
        ▼
  Implementation
        │
        │ menghasilkan code & tests
        ▼
  Conformance Tests
        │
        │ memverifikasi terhadap PRS acceptance criteria
        ▼
  Acceptance Report
        │
        │ mengkonfirmasi conformant terhadap Engineering Contract
        ▼
  Release
```

`[TRACE-1]` Setiap requirement PRS MUST dapat ditelusuri ke Engineering Contract.

`[TRACE-2]` Setiap acceptance criteria MUST memiliki test yang sesuai.

`[TRACE-3]` Setiap invariant Engineering Contract MUST diverifikasi oleh minimal satu test.

---

# 11. Final Product Contract

## 11.1 Ringkasan Komitmen Produk

| ID | Komitmen Produk |
|---|---|
| PC-01 | CNWS MUST dapat mengonversi Safetensors, GGUF, dan PyTorch ke canonical `.cd`. |
| PC-02 | CNWS MUST melakukan conversion dengan bounded memory. |
| PC-03 | CNWS MUST menggunakan BLAKE3-256 untuk seluruh content addressing. |
| PC-04 | CNWS MUST menyimpan Tile secara immutable dalam segment. |
| PC-05 | CNWS MUST melakukan deduplication pada level Tile. |
| PC-06 | CNWS MUST menyediakan runtime yang tidak bergantung pada format checkpoint. |
| PC-07 | CNWS MUST mendukung selective loading pada granularity Tile. |
| PC-08 | CNWS MUST mendukung adaptive representation selection. |
| PC-09 | CNWS MUST mendukung cache hierarchy GPU/CPU/NVMe. |
| PC-10 | CNWS MUST mendukung MoE selective expert loading. |
| PC-11 | CNWS MUST enforce hard memory budget. |
| PC-12 | CNWS MUST memverifikasi integrity Tile sebelum eksekusi. |
| PC-13 | CNWS MUST menyediakan revision DAG dengan Tile-level delta. |
| PC-14 | CNWS MUST mendukung branching tanpa copy fisik. |
| PC-15 | CNWS MUST mendukung rollback tanpa menghapus revision. |
| PC-16 | CNWS MUST melakukan GC berbasis reachability. |
| PC-17 | CNWS MUST mendukung concurrent readers. |
| PC-18 | CNWS MUST menggunakan single-writer untuk commit. |
| PC-19 | CNWS MUST lulus seluruh conformance test sebelum release. |
| PC-20 | CNWS MUST memenuhi seluruh acceptance criteria P0. |

## 11.2 Pernyataan Penutup

Dokumen PRS ini adalah **spesifikasi kebutuhan produk final dan mengikat** untuk CNWS. Ia mendefinisikan **apa** yang harus dipenuhi oleh implementasi CNWS, diturunkan langsung dari Engineering Contract.

Seluruh requirement P0 MUST dipenuhi sebelum CNWS dianggap conformant. Requirement P1 SHOULD dipenuhi untuk release produksi. Requirement P2 MAY dipenuhi sebagai enhancement.

Tidak ada kebutuhan produk yang tersisa sebagai open question. Seluruh requirement telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN PRS**
