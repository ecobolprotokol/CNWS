# CNWS
## Security & Threat Model

| Field | Value |
|---|---|
| Dokumen | CNWS Security & Threat Model |
| Status | **FINAL, NORMATIF, MENGIKAT (SECURITY SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | CNWS Engineering Contract; CNWS Conversion Spec; CNWS .cd Format Spec |
| Hulu ke | Implementasi Security Layer, Validators, Sandbox, Incident Response |
| Otoritas | Threat model tunggal dan spesifikasi keamanan untuk seluruh CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
Engineering Contract    Conversion Spec     Security & Threat Model     Implementation
─────────────────────   ────────────────    ────────────────────────    ─────────────
SEC-1..SEC-9          ──► Restricted     ──► Trust boundaries        ──► Security Layer
Integrity invariants      unpickling         Threat catalog               Validators
BLAKE3 verification       Validation         Attack scenarios             Sandbox
                          Malformed          Mitigations                  Incident Response
                          handling           Detection                    Audit
                                             Response
```

`[SEC-DOC-1]` Dokumen ini adalah **threat model lengkap dan spesifikasi keamanan** untuk CNWS.

`[SEC-DOC-2]` Engineering Contract memiliki security invariant; dokumen ini memperluasnya menjadi threat model yang komprehensif dengan skenario serangan, mitigasi, deteksi, dan respons.

`[SEC-DOC-3]` Jika terjadi konflik dengan Engineering Contract, Engineering Contract menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-SEC-01 | Trust boundaries: External (untrusted) → Conversion (semi-trusted) → Store (trusted) → Runtime (trusted). |
| DF-SEC-02 | Seluruh checkpoint eksternal diperlakukan sebagai UNTRUSTED. |
| DF-SEC-03 | PyTorch importer MUST menggunakan restricted unpickler dengan allowlist. |
| DF-SEC-04 | BLAKE3-256 verification MUST sebelum eksekusi Tile. |
| DF-SEC-05 | Path traversal MUST ditolak di semua layer. |
| DF-SEC-06 | Resource limits MUST di-enforce di conversion dan runtime. |
| DF-SEC-07 | Manifest integrity MUST diverifikasi saat load. |
| DF-SEC-08 | Version downgrade MUST ditolak kecuali eksplisit diizinkan. |
| DF-SEC-09 | Security incidents MUST logged dan reported. |
| DF-SEC-10 | Quarantine MUST untuk Tile korup atau malicious. |
| DF-SEC-11 | Conversion layer MUST sandboxed. |
| DF-SEC-12 | Remote sources MUST menggunakan integrity verification. |

---

# 1. Executive Summary

## 1.1 Security Posture CNWS

`[SEC-EXEC-1]` CNWS mengadopsi **zero-trust posture** terhadap seluruh input eksternal.

`[SEC-EXEC-2]` Prinsip keamanan CNWS:

1. **Never trust external input**: seluruh checkpoint, remote source, dan user input diperlakukan sebagai potentially malicious.
2. **Verify before use**: BLAKE3-256 verification sebelum eksekusi.
3. **Least privilege**: setiap komponen hanya memiliki akses yang diperlukan.
4. **Defense in depth**: multiple layers of protection.
5. **Fail safe**: kegagalan menghasilkan state aman, bukan state berbahaya.
6. **Audit everything**: seluruh security-relevant events logged.

## 1.2 Security Objectives

| Objective | Deskripsi |
|---|---|
| Confidentiality | Mencegah unauthorized access ke model dan data |
| Integrity | Mencegah unauthorized modification ke Cell, Tile, Manifest |
| Availability | Mencegah denial of service |
| Authenticity | Memastikan origin dari Cell dan Tile |
| Non-repudiation | Tracking perubahan melalui Revision DAG dan provenance |

## 1.3 Threat Model Scope

`[SEC-EXEC-3]` Threat model ini mencakup:

| In Scope | Out of Scope |
|---|---|
| Malicious checkpoint | Physical security (data center) |
| Malicious Cell | Network-level attacks (TLS, DDoS infrastructure) |
| Corrupted Tile | Operating system vulnerabilities |
| Malicious remote source | Hardware attacks (rowhammer, etc.) |
| Path traversal | Side-channel attacks (Spectre, Meltdown) |
| Resource exhaustion | Supply chain attacks on dependencies |
| Parser attacks | — |
| Manifest tampering | — |
| Replay/version attacks | — |
| Unsafe importer behavior | — |

---

# 2. Trust Boundaries

## 2.1 Trust Boundary Diagram

```text
┌─────────────────────────────────────────────────────────────────────┐
│                        EXTERNAL WORLD (UNTRUSTED)                   │
│                                                                     │
│   Checkpoint Files    Remote Sources    User Input                  │
│   (Safetensors,       (Object Store,    (CLI, API)                 │
│    GGUF, PyTorch)      CDN, Network)                                │
│                                                                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                    ══════════╪══════════════  TRUST BOUNDARY 1
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                   CONVERSION LAYER (SEMI-TRUSTED)                   │
│                                                                     │
│   Format Readers      Normalizers      Validators                   │
│   (parse external     (map to Cell)    (validate before             │
│    formats)                             processing)                 │
│                                                                     │
│   Security: Sandbox, Restricted Unpickler, Bounds Checks            │
│                                                                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                    ══════════╪══════════════  TRUST BOUNDARY 2
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                   CNWS STORE (TRUSTED)                              │
│                                                                     │
│   .cd Store           Segments         Manifest                     │
│   (canonical)         (Tiles)          (MANIFEST.cd)                │
│                                                                     │
│   Security: BLAKE3 Integrity, Atomic Commit, Access Control         │
│                                                                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                    ══════════╪══════════════  TRUST BOUNDARY 3
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                   CNWS RUNTIME (TRUSTED)                            │
│                                                                     │
│   Execution Engine    Cache Manager    Memory System                │
│                                                                     │
│   Security: Budget Enforcement, Integrity Verification              │
│                                                                     │
└─────────────────────────────┬───────────────────────────────────────┘
                              │
                    ══════════╪══════════════  TRUST BOUNDARY 4
                              │
┌─────────────────────────────▼───────────────────────────────────────┐
│                   EXECUTION WORLD (TRUSTED)                         │
│                                                                     │
│   CPU / GPU / NVMe / Accelerators                                   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

## 2.2 Trust Boundary Definitions

| Boundary | From | To | Trust Transition | Security Controls |
|---|---|---|---|---|
| TB-1 | External World | Conversion Layer | Untrusted → Semi-trusted | Validation, Sandbox, Restricted Parsing |
| TB-2 | Conversion Layer | CNWS Store | Semi-trusted → Trusted | BLAKE3 Hash, Atomic Commit |
| TB-3 | CNWS Store | Runtime | Trusted → Trusted | Integrity Verification |
| TB-4 | Runtime | Execution | Trusted → Trusted | Budget Enforcement |

## 2.3 Trust Boundary Rules

`[SEC-TB-1]` Data yang melewati TB-1 MUST divalidasi dan disanitasi.

`[SEC-TB-2]` Data yang melewati TB-2 MUST di-hash dengan BLAKE3-256.

`[SEC-TB-3]` Data yang melewati TB-3 MUST diverifikasi integritasnya.

`[SEC-TB-4]` Tidak ada komponen di Trusted Zone yang boleh membaca langsung dari External World tanpa melalui Conversion Layer.

## 2.4 Trust Boundary Invariants

| ID | Invariant |
|---|---|
| SEC-TB-INV-1 | External input MUST melalui Conversion Layer |
| SEC-TB-INV-2 | Conversion Layer MUST sandboxed |
| SEC-TB-INV-3 | Store MUST hanya menerima data yang sudah di-hash |
| SEC-TB-INV-4 | Runtime MUST memverifikasi integrity sebelum eksekusi |
| SEC-TB-INV-5 | Tidak ada bypass trust boundary |

---

# 3. Threat Model Overview

## 3.1 STRIDE Analysis

`[SEC-STRIDE-1]` Threat model menggunakan STRIDE framework:

| Category | Threat | CNWS Impact |
|---|---|---|
| **S**poofing | Fake checkpoint, fake remote source | Malicious content masuk ke store |
| **T**ampering | Modify Tile, manifest, revision | Integrity violation |
| **R**epudiation | Deny changes | Audit trail hilang |
| **I**nformation Disclosure | Leak model weights | Confidentiality violation |
| **D**enial of Service | Resource exhaustion | Availability violation |
| **E**levation of Privilege | Execute code via checkpoint | System compromise |

## 3.2 Threat Catalog

| ID | Threat | Category | Severity | Likelihood |
|---|---|---|---|---|
| THR-01 | Malicious checkpoint with code execution | EoP | Critical | Medium |
| THR-02 | Malicious checkpoint with parser exploit | EoP | Critical | Medium |
| THR-03 | Corrupted Tile | Tampering | High | Medium |
| THR-04 | Manifest tampering | Tampering | Critical | Low |
| THR-05 | Path traversal via tensor name | Info Disclosure | High | Medium |
| THR-06 | Resource exhaustion via large allocation | DoS | High | High |
| THR-07 | Decompression bomb | DoS | High | Medium |
| THR-08 | Malicious remote source | Spoofing | High | Medium |
| THR-09 | Man-in-the-middle on remote fetch | Tampering | High | Low |
| THR-10 | Version downgrade attack | Tampering | Medium | Low |
| THR-11 | Replay attack on manifest | Tampering | Medium | Low |
| THR-12 | Malicious Cell injection | Tampering | High | Low |
| THR-13 | Unsafe importer behavior | EoP | Critical | Medium |
| THR-14 | Integer overflow in parser | EoP | High | Low |
| THR-15 | Infinite loop in parser | DoS | Medium | Low |

## 3.3 Attack Surface

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS ATTACK SURFACE                       │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  External Input Surface:                                    │
│    ├── Checkpoint files (Safetensors, GGUF, PyTorch)        │
│    ├── Remote Tile sources                                  │
│    ├── Custom importer plugins                              │
│    └── User-provided configuration                          │
│                                                             │
│  Parsing Surface:                                           │
│    ├── Format headers                                       │
│    ├── Tensor metadata                                      │
│    ├── JSON manifests                                       │
│    └── Pickle streams (PyTorch)                             │
│                                                             │
│  Storage Surface:                                           │
│    ├── Segment files                                        │
│    ├── Index files                                          │
│    ├── Manifest files                                       │
│    └── Journal files                                        │
│                                                             │
│  Network Surface:                                           │
│    ├── Remote Tile fetch                                    │
│    ├── Model registry access                                │
│    └── Telemetry export                                     │
│                                                             │
│  Runtime Surface:                                           │
│    ├── Cell loading                                         │
│    ├── Cache operations                                     │
│    └── Memory allocation                                    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

# 4. Malicious Checkpoint

## 4.1 Threat Description

`[SEC-MC-1]` Malicious checkpoint adalah checkpoint yang dirancang untuk:
- Mengeksekusi kode arbitrer saat import
- Menyebabkan crash atau hang
- Menghasilkan Cell/Tile yang korup
- Menyebabkan resource exhaustion

## 4.2 Attack Scenarios

### 4.2.1 PyTorch Pickle Code Execution

```text
Attack:
  Attacker creates PyTorch checkpoint with malicious __reduce__ method
  
  torch.save({
      "__reduce__": lambda: os.system("rm -rf /")
  }, "malicious.pt")

Impact:
  Arbitrary code execution during import
  
Mitigation:
  Restricted unpickler with allowlist
```

`[SEC-MC-2]` PyTorch importer MUST menggunakan restricted unpickler (dari Conversion Spec §7).

`[SEC-MC-3]` PyTorch importer MUST menolak semua class yang tidak ada di allowlist.

### 4.2.2 Malformed Header

```text
Attack:
  Attacker creates checkpoint with invalid header:
  - Header length = 0xFFFFFFFFFFFFFFFF (overflow)
  - Header length = 0 (empty)
  - Header contains invalid JSON

Impact:
  Parser crash, buffer overflow
  
Mitigation:
  Header validation, bounds checking
```

`[SEC-MC-4]` Header validation MUST memeriksa:
- Header length dalam range wajar (< 100 MiB)
- JSON valid dan parseable
- Required fields present

### 4.2.3 Tensor Name Injection

```text
Attack:
  Attacker creates checkpoint with tensor name:
  "../../../etc/passwd"
  "| rm -rf /"
  "$(malicious command)"

Impact:
  Path traversal, command injection
  
Mitigation:
  Tensor name sanitization, CellId grammar enforcement
```

`[SEC-MC-5]` Tensor names MUST divalidasi terhadap CellId grammar.

`[SEC-MC-6]` Tensor names MUST NOT mengandung path separators, shell metacharacters, atau control characters.

### 4.2.4 Oversized Metadata

```text
Attack:
  Attacker creates checkpoint with metadata claiming:
  - shape = [2^63, 2^63, 2^63]
  - dtype = F32
  
  This would require 2^189 bytes allocation.

Impact:
  Memory exhaustion, OOM crash
  
Mitigation:
  Shape validation, allocation limits
```

`[SEC-MC-7]` Shape validation MUST menolak shape yang akan menyebabkan allocation > configurable limit.

`[SEC-MC-8]` Default maximum allocation per tensor: 1 TiB.

## 4.3 Malicious Checkpoint Mitigations

| Mitigation | Requirement |
|---|---|
| Restricted unpickler | MUST untuk PyTorch |
| Header validation | MUST untuk semua format |
| Tensor name sanitization | MUST untuk semua format |
| Shape validation | MUST untuk semua format |
| Allocation limits | MUST untuk semua format |
| Sandbox execution | SHOULD untuk conversion |
| Timeout | MUST untuk parsing |

## 4.4 Malicious Checkpoint Detection

`[SEC-MC-9]` Detection mechanisms:

| Detection | Method |
|---|---|
| Invalid header | Parse validation |
| Oversized allocation | Pre-allocation check |
| Malicious tensor name | Grammar validation |
| Pickle code execution | Allowlist enforcement |
| Corrupted data | BLAKE3 verification |

## 4.5 Malicious Checkpoint Invariants

| ID | Invariant |
|---|---|
| SEC-MC-INV-1 | Checkpoint eksternal MUST diperlakukan sebagai untrusted |
| SEC-MC-INV-2 | PyTorch importer MUST menggunakan restricted unpickler |
| SEC-MC-INV-3 | Header MUST divalidasi sebelum parsing |
| SEC-MC-INV-4 | Tensor names MUST disanitasi |
| SEC-MC-INV-5 | Shape MUST divalidasi sebelum allocation |
| SEC-MC-INV-6 | Malicious checkpoint MUST ditolak dengan error eksplisit |

---

# 5. Malicious Cell

## 5.1 Threat Description

`[SEC-MCELL-1]` Malicious Cell adalah Cell yang dirancang untuk:
- Menyebabkan eksekusi yang salah
- Menghasilkan output yang berbahaya
- Menyebabkan resource exhaustion saat eksekusi

## 5.2 Attack Scenarios

### 5.2.1 Cell with Crafted Index Vector

```text
Attack:
  Attacker creates Cell with index vector designed to:
  - Always match any query (high similarity to everything)
  - Cause Cell to be selected for all inputs

Impact:
  Cell selalu dipilih, menghasilkan output salah
  
Mitigation:
  Index vector validation, anomaly detection
```

`[SEC-MCELL-2]` Index vector MUST divalidasi:
- Dimensions sesuai konfigurasi
- Values dalam range wajar (tidak NaN, tidak Inf)
- Normalized jika menggunakan cosine metric

### 5.2.2 Cell with Invalid Dependencies

```text
Attack:
  Attacker creates Cell with circular dependencies:
  Cell A → Cell B → Cell C → Cell A

Impact:
  Infinite loop saat dependency resolution
  
Mitigation:
  Dependency graph cycle detection
```

`[SEC-MCELL-3]` Dependency graph MUST acyclic.

`[SEC-MCELL-4]` Cycle detection MUST dilakukan saat Cell creation.

### 5.2.3 Cell with Oversized Payload

```text
Attack:
  Attacker creates Cell with payload claiming to be small
  but actually expands to huge size during decompression.

Impact:
  Decompression bomb, memory exhaustion
  
Mitigation:
  Decompression ratio limits, size validation
```

`[SEC-MCELL-5]` Decompression ratio MUST dibatasi (default: max 100:1).

`[SEC-MCELL-6]` Decompressed size MUST diverifikasi sebelum allocation.

## 5.3 Malicious Cell Mitigations

| Mitigation | Requirement |
|---|---|
| Index vector validation | MUST |
| Dependency cycle detection | MUST |
| Payload size validation | MUST |
| Decompression ratio limit | MUST |
| Cell schema validation | MUST |
| BLAKE3 verification | MUST |

## 5.4 Malicious Cell Invariants

| ID | Invariant |
|---|---|
| SEC-MCELL-INV-1 | Cell MUST divalidasi saat creation |
| SEC-MCELL-INV-2 | Index vector MUST valid |
| SEC-MCELL-INV-3 | Dependency graph MUST acyclic |
| SEC-MCELL-INV-4 | Payload size MUST sesuai deklarasi |
| SEC-MCELL-INV-5 | Decompression ratio MUST dibatasi |

---

# 6. Corrupted Tile

## 6.1 Threat Description

`[SEC-CT-1]` Corrupted Tile adalah Tile yang payload-nya tidak sesuai dengan BLAKE3-256 identity.

`[SEC-CT-2]` Korupsi dapat disebabkan oleh:
- Bit rot (storage degradation)
- Partial write (crash during write)
- Intentional corruption (attacker)
- Storage media failure

## 6.2 Attack Scenarios

### 6.2.1 Bit Rot

```text
Attack/Event:
  Storage media degrades over time
  One or more bits flip in Tile payload

Impact:
  Tile payload tidak sesuai dengan BLAKE3-256 identity
  
Detection:
  BLAKE3 verification saat load
  
Mitigation:
  Quarantine Tile, reload from replica
```

### 6.2.2 Partial Write

```text
Attack/Event:
  System crash during Tile write
  Tile partially written

Impact:
  Tile truncated, BLAKE3 mismatch
  
Detection:
  BLAKE3 verification, size check
  
Mitigation:
  Atomic write, journal recovery
```

### 6.2.3 Intentional Corruption

```text
Attack:
  Attacker with storage access modifies Tile payload

Impact:
  Tile payload berubah, BLAKE3 mismatch
  
Detection:
  BLAKE3 verification
  
Mitigation:
  Quarantine, alert, audit
```

## 6.3 Corrupted Tile Detection

`[SEC-CT-3]` Tile MUST diverifikasi BLAKE3-256:

```pseudo
function verify_tile(tile_id: Blake3Hash, payload: Vec<u8>) -> Result<()>:
    actual_hash = blake3_256(payload)
    
    if actual_hash != tile_id:
        // Corruption detected
        quarantine_tile(tile_id, payload)
        log_security_event(SecurityEvent::TileCorruption {
            tile_id,
            expected_hash: tile_id,
            actual_hash,
        })
        return Err(Error::TileCorruption)
    
    return Ok(())
```

## 6.4 Corrupted Tile Response

`[SEC-CT-4]` Corrupted Tile response protocol:

```text
Corruption detected
    │
    ▼
Quarantine Tile
    │
    ├── Move to corrupt/ directory
    ├── Record corruption metadata
    └── Alert operator
    │
    ▼
Attempt Recovery
    │
    ├── Check replica (if available)
    ├── Check remote source (if available)
    └── Check previous revision
    │
    ▼
Recovery Result
    │
    ├── Success → Replace corrupted Tile
    └── Failure → Report unrecoverable error
```

## 6.5 Corrupted Tile Invariants

| ID | Invariant |
|---|---|
| SEC-CT-INV-1 | Tile MUST diverifikasi BLAKE3-256 sebelum eksekusi |
| SEC-CT-INV-2 | Corrupted Tile MUST dikarantina |
| SEC-CT-INV-3 | Corruption MUST logged sebagai security event |
| SEC-CT-INV-4 | Corrupted Tile MUST NOT dieksekusi |
| SEC-CT-INV-5 | Recovery MUST dicoba sebelum error fatal |

---

# 7. Malicious Remote Source

## 7.1 Threat Description

`[SEC-MRS-1]` Malicious remote source adalah remote storage yang menyediakan Tile yang salah atau malicious.

## 7.2 Attack Scenarios

### 7.2.1 Tile Substitution

```text
Attack:
  Attacker controls remote source
  Provides Tile with correct ID but wrong payload

Impact:
  Tile dengan payload salah dieksekusi
  
Detection:
  BLAKE3 verification (ID tidak cocok dengan payload)
  
Mitigation:
  Reject Tile, alert, use local copy
```

`[SEC-MRS-2]` Remote Tile MUST diverifikasi BLAKE3-256 sebelum digunakan.

### 7.2.2 Man-in-the-Middle

```text
Attack:
  Attacker intercepts network traffic
  Modifies Tile in transit

Impact:
  Tile corrupted in transit
  
Detection:
  BLAKE3 verification
  
Mitigation:
  TLS for transport, BLAKE3 verification
```

`[SEC-MRS-3]` Remote fetch SHOULD menggunakan TLS.

`[SEC-MRS-4]` BLAKE3 verification MUST dilakukan terlepas dari transport security.

### 7.2.3 Denial of Service

```text
Attack:
  Attacker controls remote source
  Responds very slowly or not at all

Impact:
  Tile loading hangs
  
Mitigation:
  Timeout, fallback to local
```

`[SEC-MRS-5]` Remote fetch MUST memiliki timeout.

`[SEC-MRS-6]` Remote fetch failure MUST fallback ke local copy jika tersedia.

## 7.3 Malicious Remote Source Mitigations

| Mitigation | Requirement |
|---|---|
| BLAKE3 verification | MUST |
| TLS transport | SHOULD |
| Timeout | MUST |
| Fallback to local | MUST (jika tersedia) |
| Remote source allowlist | SHOULD |
| Audit logging | MUST |

## 7.4 Malicious Remote Source Invariants

| ID | Invariant |
|---|---|
| SEC-MRS-INV-1 | Remote Tile MUST diverifikasi BLAKE3-256 |
| SEC-MRS-INV-2 | Remote fetch MUST memiliki timeout |
| SEC-MRS-INV-3 | BLAKE3 verification MUST terlepas dari transport |
| SEC-MRS-INV-4 | Remote source failure MUST NOT crash system |
| SEC-MRS-INV-5 | Remote fetch MUST logged |

---

# 8. Path Traversal

## 8.1 Threat Description

`[SEC-PT-1]` Path traversal attack menggunakan path manipulasi untuk mengakses file di luar directory yang diizinkan.

## 8.2 Attack Scenarios

### 8.2.1 Tensor Name Path Traversal

```text
Attack:
  Checkpoint contains tensor named:
  "../../../etc/passwd"
  "..\\..\\windows\\system32\\config"
  "/absolute/path/to/file"

Impact:
  System attempts to read/write outside .cd directory
  
Mitigation:
  Tensor name sanitization, CellId grammar enforcement
```

`[SEC-PT-2]` Tensor names MUST divalidasi terhadap CellId grammar (dari Cell & Schema Spec).

`[SEC-PT-3]` Tensor names MUST NOT mengandung:
- Path separators: `/`, `\`
- Parent directory: `..`
- Null bytes: `\0`
- Control characters

### 8.2.2 Symbolic Link Attack

```text
Attack:
  Attacker creates symbolic link inside .cd directory:
  model.cd/segments/segment-000001.cd → /etc/passwd

Impact:
  System reads/writes to /etc/passwd
  
Mitigation:
  Symlink detection, reject symlinks in .cd
```

`[SEC-PT-4]`.cd directory MUST NOT mengandung symbolic links.

`[SEC-PT-5]` System MUST mendeteksi dan menolak symbolic links.

### 8.2.3 Absolute Path Injection

```text
Attack:
  Manifest contains absolute path:
  "path": "/etc/shadow"

Impact:
  System reads sensitive file
  
Mitigation:
  Path validation, relative paths only
```

`[SEC-PT-6]` Semua path dalam `.cd` MUST relative terhadap `.cd` root.

`[SEC-PT-7]` Absolute paths MUST ditolak.

## 8.3 Path Traversal Mitigations

| Mitigation | Requirement |
|---|---|
| CellId grammar validation | MUST |
| Path separator rejection | MUST |
| Symlink detection | MUST |
| Relative path enforcement | MUST |
| Directory canonicalization | SHOULD |
| Access control | MUST |

## 8.4 Path Traversal Invariants

| ID | Invariant |
|---|---|
| SEC-PT-INV-1 | Tensor names MUST NOT mengandung path separators |
| SEC-PT-INV-2 | .cd MUST NOT mengandung symbolic links |
| SEC-PT-INV-3 | Semua path MUST relative |
| SEC-PT-INV-4 | Absolute paths MUST ditolak |
| SEC-PT-INV-5 | Path traversal attempt MUST logged |

---

# 9. Resource Exhaustion

## 9.1 Threat Description

`[SEC-RE-1]` Resource exhaustion attack bertujuan menghabiskan resource sistem (memory, disk, CPU, network).

## 9.2 Attack Scenarios

### 9.2.1 Memory Exhaustion via Large Allocation

```text
Attack:
  Checkpoint declares tensor with shape [2^32, 2^32]
  System attempts to allocate 2^64 × dtype_size bytes

Impact:
  OOM crash
  
Mitigation:
  Shape validation, allocation limits
```

`[SEC-RE-2]` Shape MUST divalidasi sebelum allocation.

`[SEC-RE-3]` Maximum allocation per tensor MUST dibatasi (default: 1 TiB).

### 9.2.2 Decompression Bomb

```text
Attack:
  Compressed Tile: 1 MB
  Decompressed size: 100 GB (ratio 100000:1)

Impact:
  Memory exhaustion during decompression
  
Mitigation:
  Decompression ratio limit, streaming decompression
```

`[SEC-RE-4]` Decompression ratio MUST dibatasi (default: max 100:1).

`[SEC-RE-5]` Decompression MUST streaming untuk bounded memory.

### 9.2.3 Disk Exhaustion

```text
Attack:
  Attacker imports many unique Tiles
  Fills up disk

Impact:
  Disk full, system unable to write
  
Mitigation:
  Disk quota, admission control
```

`[SEC-RE-6]` Disk usage MUST dimonitor.

`[SEC-RE-7]` Import MUST gagal dengan error eksplisit jika disk quota exceeded.

### 9.2.4 CPU Exhaustion via Complex Parsing

```text
Attack:
  Checkpoint with deeply nested structure
  Causes exponential parsing time

Impact:
  CPU exhaustion, hang
  
Mitigation:
  Parsing depth limit, timeout
```

`[SEC-RE-8]` Parsing depth MUST dibatasi (default: max 100 levels).

`[SEC-RE-9]` Parsing MUST memiliki timeout.

## 9.3 Resource Limits

`[SEC-RE-10]` Resource limits default:

| Resource | Default Limit | Configurable |
|---|---|---|
| Max allocation per tensor | 1 TiB | YES |
| Max decompression ratio | 100:1 | YES |
| Max parsing depth | 100 | YES |
| Max parsing timeout | 300 seconds | YES |
| Max disk usage | 90% of capacity | YES |
| Max memory usage (conversion) | 4 GiB | YES |
| Max network timeout | 60 seconds | YES |

## 9.4 Resource Exhaustion Invariants

| ID | Invariant |
|---|---|
| SEC-RE-INV-1 | Resource limits MUST di-enforce |
| SEC-RE-INV-2 | Resource exhaustion MUST menghasilkan error eksplisit |
| SEC-RE-INV-3 | Resource usage MUST dimonitor |
| SEC-RE-INV-4 | Resource limits MUST configurable |
| SEC-RE-INV-5 | Resource exhaustion MUST NOT crash system |

---

# 10. Parser Attacks

## 10.1 Threat Description

`[SEC-PA-1]` Parser attacks mengeksploitasi kerentanan dalam parser format checkpoint.

## 10.2 Attack Scenarios

### 10.2.1 Integer Overflow

```text
Attack:
  Header contains:
  tensor_count = 0xFFFFFFFFFFFFFFFF
  
  Parser computes: tensor_count × sizeof(TensorInfo) → overflow

Impact:
  Buffer overflow, memory corruption
  
Mitigation:
  Integer overflow checks, safe arithmetic
```

`[SEC-PA-2]` Semua arithmetic pada parsed values MUST menggunakan checked arithmetic.

`[SEC-PA-3]` Integer overflow MUST menghasilkan error, bukan wrap-around.

### 10.2.2 Infinite Loop

```text
Attack:
  Checkpoint with circular reference in metadata
  Parser enters infinite loop

Impact:
  CPU exhaustion, hang
  
Mitigation:
  Loop detection, timeout
```

`[SEC-PA-4]` Parser MUST memiliki loop detection.

`[SEC-PA-5]` Parser MUST memiliki timeout.

### 10.2.3 Stack Overflow via Deep Nesting

```text
Attack:
  JSON manifest with 10000 levels of nesting
  
  {"a": {"b": {"c": ... }}}

Impact:
  Stack overflow, crash
  
Mitigation:
  Nesting depth limit
```

`[SEC-PA-6]` JSON parsing MUST membatasi nesting depth (default: max 100).

### 10.2.4 Buffer Over-read

```text
Attack:
  Header declares data size larger than actual file
  Parser reads beyond file boundary

Impact:
  Segfault, information leak
  
Mitigation:
  Bounds checking, file size validation
```

`[SEC-PA-7]` Parser MUST melakukan bounds checking pada setiap read.

`[SEC-PA-8]` Declared size MUST diverifikasi terhadap actual file size.

## 10.3 Parser Security Requirements

| Requirement | Status |
|---|---|
| Checked arithmetic | MUST |
| Bounds checking | MUST |
| Nesting depth limit | MUST |
| Parsing timeout | MUST |
| Input size validation | MUST |
| Fuzzing tested | SHOULD |

## 10.4 Parser Attacks Invariants

| ID | Invariant |
|---|---|
| SEC-PA-INV-1 | Parser MUST menggunakan checked arithmetic |
| SEC-PA-INV-2 | Parser MUST melakukan bounds checking |
| SEC-PA-INV-3 | Parser MUST memiliki nesting depth limit |
| SEC-PA-INV-4 | Parser MUST memiliki timeout |
| SEC-PA-INV-5 | Parser MUST validated terhadap malformed input |

---

# 11. Manifest Tampering

## 11.1 Threat Description

`[SEC-MT-1]` Manifest tampering adalah modifikasi unauthorized terhadap MANIFEST.cd, SUPERBLOCK, atau segment index.

## 11.2 Attack Scenarios

### 11.2.1 MANIFEST.cd Modification

```text
Attack:
  Attacker with file system access modifies MANIFEST.cd:
  - Change Tile references to point to malicious Tiles
  - Remove security-relevant metadata
  - Add unauthorized Cells

Impact:
  Runtime loads malicious Tiles
  
Detection:
  BLAKE3 verification of manifest (stored in SUPERBLOCK)
  
Mitigation:
  Verify manifest hash, reject if mismatch
```

`[SEC-MT-2]` MANIFEST.cd hash MUST diverifikasi saat load.

`[SEC-MT-3]` MANIFEST.cd hash disimpan di SUPERBLOCK.

### 11.2.2 SUPERBLOCK Modification

```text
Attack:
  Attacker modifies SUPERBLOCK:
  - Change manifest_hash to match tampered manifest
  - Change version to trigger downgrade

Impact:
  Tampered manifest accepted
  
Detection:
  SUPERBLOCK integrity check (magic, version, reserved)
  
Mitigation:
  Validate SUPERBLOCK structure, version check
```

`[SEC-MT-4]` SUPERBLOCK MUST divalidasi saat load:
- Magic bytes correct
- Version dalam range yang didukung
- Reserved fields zero

### 11.2.3 Segment Index Tampering

```text
Attack:
  Attacker modifies segment index:
  - Point Tile to wrong offset
  - Point Tile to malicious payload

Impact:
  Runtime reads wrong data
  
Detection:
  BLAKE3 verification of segment index (stored in segment header)
  
Mitigation:
  Verify segment index hash
```

`[SEC-MT-5]` Segment index hash MUST diverifikasi saat segment load.

## 11.3 Manifest Integrity Verification

```pseudo
function verify_manifest_integrity() -> Result<()>:
    // 1. Read SUPERBLOCK
    superblock = read_superblock()
    
    // 2. Validate SUPERBLOCK
    if superblock.magic != "CNWSSB01":
        return Err(Error::InvalidSuperblock)
    
    if superblock.version_major > SUPPORTED_MAJOR:
        return Err(Error::UnsupportedVersion)
    
    // 3. Read MANIFEST.cd
    manifest_bytes = read_file("MANIFEST.cd")
    
    // 4. Compute manifest hash
    actual_hash = blake3_256(manifest_bytes)
    
    // 5. Verify hash
    if actual_hash != superblock.manifest_hash:
        return Err(Error::ManifestTampered)
    
    // 6. Parse and validate manifest
    manifest = parse_manifest(manifest_bytes)
    validate_manifest(manifest)?
    
    return Ok(())
```

## 11.4 Manifest Tampering Invariants

| ID | Invariant |
|---|---|
| SEC-MT-INV-1 | MANIFEST.cd hash MUST diverifikasi saat load |
| SEC-MT-INV-2 | SUPERBLOCK MUST divalidasi saat load |
| SEC-MT-INV-3 | Segment index hash MUST diverifikasi |
| SEC-MT-INV-4 | Manifest tampering MUST menghasilkan error fatal |
| SEC-MT-INV-5 | Manifest tampering MUST logged sebagai security event |

---

# 12. Replay / Version Attacks

## 12.1 Threat Description

`[SEC-RV-1]` Replay/version attacks memanipulasi revision history untuk rollback ke state yang vulnerable atau replay manifest lama.

## 12.2 Attack Scenarios

### 12.2.1 Version Downgrade

```text
Attack:
  Attacker modifies SUPERBLOCK to claim older format version
  that has known vulnerabilities

Impact:
  System uses vulnerable code path
  
Mitigation:
  Version validation, reject unsupported versions
```

`[SEC-RV-2]` Version downgrade MUST ditolak kecuali eksplisit diizinkan.

`[SEC-RV-3]` Minimum supported version MUST dikonfigurasi.

### 12.2.2 Manifest Replay

```text
Attack:
  Attacker saves old MANIFEST.cd
  Later replaces current MANIFEST.cd with old one

Impact:
  System uses outdated manifest
  
Detection:
  Manifest hash mismatch with SUPERBLOCK
  
Mitigation:
  Verify manifest hash, check revision number
```

`[SEC-RV-4]` Manifest replay MUST terdeteksi melalui hash verification.

### 12.2.3 Revision Rollback Attack

```text
Attack:
  Attacker forces rollback to revision with known vulnerability

Impact:
  System uses vulnerable revision
  
Mitigation:
  Rollback authorization, audit logging
```

`[SEC-RV-5]` Rollback MUST logged sebagai security event.

`[SEC-RV-6]` Rollback ke revision dengan known vulnerabilities MUST ditolak atau memerlukan explicit authorization.

## 12.3 Version Security

`[SEC-RV-7]` Version security rules:

| Rule | Requirement |
|---|---|
| Version validation | MUST |
| Minimum version enforcement | MUST |
| Downgrade rejection | MUST (default) |
| Downgrade authorization | MAY (explicit) |
| Version audit logging | MUST |

## 12.4 Replay/Version Attacks Invariants

| ID | Invariant |
|---|---|
| SEC-RV-INV-1 | Version MUST divalidasi |
| SEC-RV-INV-2 | Version downgrade MUST ditolak (default) |
| SEC-RV-INV-3 | Manifest replay MUST terdeteksi |
| SEC-RV-INV-4 | Rollback MUST logged |
| SEC-RV-INV-5 | Version attacks MUST menghasilkan error eksplisit |

---

# 13. Unsafe Importer Behavior

## 13.1 Threat Description

`[SEC-UI-1]` Unsafe importer behavior adalah importer yang melakukan operasi berbahaya selama conversion.

## 13.2 Attack Scenarios

### 13.2.1 Code Execution

```text
Attack:
  Custom importer executes arbitrary code:
  - Spawns subprocess
  - Loads dynamic library
  - Evaluates script

Impact:
  System compromise
  
Mitigation:
  Importer sandbox, capability restriction
```

`[SEC-UI-2]` Importer MUST NOT mengeksekusi kode dari checkpoint.

`[SEC-UI-3]` Custom importer SHOULD dijalankan dalam sandbox.

### 13.2.2 Network Access

```text
Attack:
  Importer makes unauthorized network requests:
  - Exfiltrate data
  - Download malicious payload

Impact:
  Data leak, malware injection
  
Mitigation:
  Network restriction, egress filtering
```

`[SEC-UI-4]` Importer MUST NOT melakukan network access tanpa explicit authorization.

### 13.2.3 File System Access Outside Sandbox

```text
Attack:
  Importer reads/writes files outside .cd directory:
  - Read sensitive files
  - Write to system directories

Impact:
  Information disclosure, system compromise
  
Mitigation:
  File system sandbox, path restriction
```

`[SEC-UI-5]` Importer MUST hanya mengakses file dalam scope yang diizinkan.

`[SEC-UI-6]` Importer MUST NOT mengakses file di luar source checkpoint dan target `.cd`.

## 13.3 Importer Security Requirements

| Requirement | Status |
|---|---|
| No code execution from checkpoint | MUST |
| No unauthorized network access | MUST |
| File system sandbox | SHOULD |
| Resource limits | MUST |
| Timeout | MUST |
| Audit logging | MUST |

## 13.4 Unsafe Importer Invariants

| ID | Invariant |
|---|---|
| SEC-UI-INV-1 | Importer MUST NOT mengeksekusi kode dari checkpoint |
| SEC-UI-INV-2 | Importer MUST NOT network access tanpa authorization |
| SEC-UI-INV-3 | Importer MUST hanya akses file dalam scope |
| SEC-UI-INV-4 | Importer MUST memiliki resource limits |
| SEC-UI-INV-5 | Importer MUST memiliki timeout |

---

# 14. Security Response Requirements

## 14.1 Security Event Classification

`[SEC-RESP-1]` Security events diklasifikasikan berdasarkan severity:

| Severity | Description | Response Time |
|---|---|---|
| Critical | System compromise, data breach | Immediate |
| High | Integrity violation, unauthorized access | < 1 hour |
| Medium | Suspicious activity, policy violation | < 24 hours |
| Low | Informational, minor anomaly | < 7 days |

## 14.2 Security Event Types

| Event Type | Severity | Description |
|---|---|---|
| `TILE_CORRUPTION` | High | BLAKE3 mismatch detected |
| `MANIFEST_TAMPERING` | Critical | Manifest hash mismatch |
| `SUPERBLOCK_TAMPERING` | Critical | SUPERBLOCK validation failed |
| `PATH_TRAVERSAL_ATTEMPT` | High | Path traversal detected |
| `RESOURCE_EXHAUSTION` | Medium | Resource limit exceeded |
| `UNSAFE_IMPORTER` | Critical | Importer attempted forbidden operation |
| `MALICIOUS_CHECKPOINT` | Critical | Checkpoint rejected due to security |
| `REMOTE_TILE_MISMATCH` | High | Remote Tile failed verification |
| `VERSION_DOWNGRADE` | Medium | Version downgrade attempted |
| `ROLLBACK` | Low | Revision rollback performed |

## 14.3 Security Response Protocol

```text
Security Event Detected
    │
    ▼
┌─────────────────────────────────────────────────────────┐
│ 1. DETECTION                                            │
│    - Identify event type                                │
│    - Classify severity                                  │
│    - Capture evidence                                   │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ 2. CONTAINMENT                                          │
│    - Quarantine affected component                      │
│    - Stop affected operation                            │
│    - Prevent propagation                                │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ 3. ERADICATION                                          │
│    - Remove malicious/corrupted data                    │
│    - Fix vulnerability                                  │
│    - Update security controls                           │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ 4. RECOVERY                                             │
│    - Restore from known-good state                      │
│    - Verify integrity                                   │
│    - Resume operation                                   │
└─────────────────────┬───────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────────┐
│ 5. REPORTING                                            │
│    - Log security event                                 │
│    - Notify operator                                    │
│    - Update audit trail                                 │
└─────────────────────────────────────────────────────────┘
```

## 14.4 Security Logging

`[SEC-RESP-2]` Security events MUST logged dengan informasi:

| Field | Required |
|---|---|
| `timestamp` | MUST |
| `event_type` | MUST |
| `severity` | MUST |
| `source` | MUST |
| `description` | MUST |
| `affected_component` | MUST |
| `action_taken` | MUST |
| `evidence` | SHOULD |
| `operator` | SHOULD |

## 14.5 Security Audit Trail

`[SEC-RESP-3]` Audit trail MUST mencakup:

1. Seluruh security events
2. Seluruh revision changes
3. Seluruh Tile quarantine
4. Seluruh manifest verification failures
5. Seluruh importer rejections

`[SEC-RESP-4]` Audit trail MUST immutable.

`[SEC-RESP-5]` Audit trail MUST retained minimum 1 tahun.

## 14.6 Security Response Invariants

| ID | Invariant |
|---|---|
| SEC-RESP-INV-1 | Security events MUST logged |
| SEC-RESP-INV-2 | Critical events MUST immediate response |
| SEC-RESP-INV-3 | Quarantine MUST untuk corrupted/malicious data |
| SEC-RESP-INV-4 | Audit trail MUST immutable |
| SEC-RESP-INV-5 | Security response MUST terdokumentasi |

---

# 15. Security Invariants Summary

## 15.1 Complete Security Invariants

| ID | Invariant |
|---|---|
| SEC-INV-1 | External input MUST diperlakukan sebagai untrusted. |
| SEC-INV-2 | PyTorch importer MUST menggunakan restricted unpickler. |
| SEC-INV-3 | BLAKE3-256 verification MUST sebelum eksekusi Tile. |
| SEC-INV-4 | Path traversal MUST ditolak di semua layer. |
| SEC-INV-5 | Resource limits MUST di-enforce. |
| SEC-INV-6 | Manifest integrity MUST diverifikasi saat load. |
| SEC-INV-7 | Version downgrade MUST ditolak (default). |
| SEC-INV-8 | Security incidents MUST logged dan reported. |
| SEC-INV-9 | Quarantine MUST untuk Tile korup atau malicious. |
| SEC-INV-10 | Conversion layer MUST sandboxed. |
| SEC-INV-11 | Remote sources MUST menggunakan integrity verification. |
| SEC-INV-12 | Importer MUST NOT mengeksekusi kode dari checkpoint. |
| SEC-INV-13 | Importer MUST NOT network access tanpa authorization. |
| SEC-INV-14 | Tensor names MUST disanitasi. |
| SEC-INV-15 | Shape MUST divalidasi sebelum allocation. |
| SEC-INV-16 | Decompression ratio MUST dibatasi. |
| SEC-INV-17 | Dependency graph MUST acyclic. |
| SEC-INV-18 | Parser MUST menggunakan checked arithmetic. |
| SEC-INV-19 | Parser MUST memiliki timeout. |
| SEC-INV-20 | Audit trail MUST immutable. |

---

# 16. Final Security Contract

## 16.1 Ringkasan Keputusan Security

| ID | Keputusan |
|---|---|
| SEC-F01 | Trust boundaries: External → Conversion → Store → Runtime → Execution. |
| SEC-F02 | Seluruh checkpoint eksternal diperlakukan sebagai UNTRUSTED. |
| SEC-F03 | PyTorch importer MUST menggunakan restricted unpickler dengan allowlist. |
| SEC-F04 | BLAKE3-256 verification MUST sebelum eksekusi Tile. |
| SEC-F05 | Path traversal MUST ditolak di semua layer. |
| SEC-F06 | Resource limits MUST di-enforce di conversion dan runtime. |
| SEC-F07 | Manifest integrity MUST diverifikasi saat load. |
| SEC-F08 | Version downgrade MUST ditolak kecuali eksplisit diizinkan. |
| SEC-F09 | Security incidents MUST logged dan reported. |
| SEC-F10 | Quarantine MUST untuk Tile korup atau malicious. |
| SEC-F11 | Conversion layer MUST sandboxed. |
| SEC-F12 | Remote sources MUST menggunakan integrity verification. |
| SEC-F13 | Importer MUST NOT mengeksekusi kode dari checkpoint. |
| SEC-F14 | Importer MUST NOT network access tanpa authorization. |
| SEC-F15 | Tensor names MUST disanitasi terhadap CellId grammar. |
| SEC-F16 | Shape validation MUST sebelum allocation. |
| SEC-F17 | Decompression ratio MUST dibatasi (default 100:1). |
| SEC-F18 | Dependency graph MUST acyclic. |
| SEC-F19 | Parser MUST menggunakan checked arithmetic. |
| SEC-F20 | Parser MUST memiliki timeout dan nesting depth limit. |

## 16.2 Security Testing Requirements

`[SEC-TEST-1]` Security testing MUST mencakup:

| Test Category | Requirement |
|---|---|
| Malicious checkpoint tests | MUST |
| Path traversal tests | MUST |
| Resource exhaustion tests | MUST |
| Parser fuzzing | SHOULD |
| Manifest tampering tests | MUST |
| Remote source verification tests | MUST |
| Version downgrade tests | MUST |
| Importer sandbox tests | MUST |
| Security logging tests | MUST |

## 16.3 Pernyataan Penutup

Dokumen ini adalah **Security & Threat Model final dan mengikat** untuk CNWS. Ia mendefinisikan trust boundaries, threat catalog, attack scenarios, mitigations, detection mechanisms, dan response requirements untuk seluruh aspek keamanan CNWS.

Seluruh implementasi Security Layer, Validators, Sandbox, dan Incident Response CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan security yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN SECURITY & THREAT MODEL**
