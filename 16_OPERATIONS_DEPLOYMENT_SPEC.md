# CNWS
## Operations & Deployment Specification

| Field | Value |
|---|---|
| Dokumen | CNWS Operations & Deployment Specification |
| Status | **FINAL, NORMATIF, MENGIKAT (OPERATIONS SPECIFICATION)** |
| Versi | 1.0.0 |
| Tanggal | 2026-08-11 |
| Hilir dari | Seluruh spesifikasi CNWS |
| Hulu ke | Deployment procedures, SRE runbook, infrastructure automation |
| Otoritas | Spesifikasi tunggal untuk operasional dan deployment CNWS |

---

# 0. Document Control

## 0.1 Posisi Dokumen

```text
All CNWS Specs              Operations Spec              Deployment
──────────────────          ────────────────────         ──────────────
.cd Format Spec         ──► Store initialization     ──► Infra automation
Reliability Spec            Hardware requirements        CI/CD pipelines
Observability Spec          Configuration                Monitoring setup
Security Spec               Upgrade/downgrade            SRE runbook
Performance Spec            Backup/restore               Disaster recovery
API Spec                    Monitoring
                            Failure handling
```

`[OPS-DOC-1]` Dokumen ini mendefinisikan **bagaimana CNWS dijalankan sebagai sistem nyata** dalam lingkungan produksi.

`[OPS-DOC-2]` Dokumen ini mencakup seluruh lifecycle operasional: installation → configuration → operation → upgrade → backup → restore → failure handling.

`[OPS-DOC-3]` Jika terjadi konflik dengan spesifikasi lain untuk hal behavior, spesifikasi tersebut menang. Untuk hal operational procedures, dokumen ini menang.

## 0.2 Interpretasi Normatif

Kata kunci **MUST**, **MUST NOT**, **SHOULD**, **MAY** diinterpretasikan sesuai RFC 2119.

## 0.3 Keputusan Engineering Final (DF)

| ID | Keputusan Final |
|---|---|
| DF-OPS-01 | Configuration menggunakan format TOML. |
| DF-OPS-02 | Store initialization via `cnws init`. |
| DF-OPS-03 | Minimum RAM: 16 GB untuk runtime, 8 GB untuk conversion. |
| DF-OPS-04 | Filesystem recommendation: XFS atau ext4 untuk `.cd` store. |
| DF-OPS-05 | NVMe I/O scheduler: `none` atau `mq-deadline`. |
| DF-OPS-06 | GPU configuration via CNWS config, bukan environment variables. |
| DF-OPS-07 | Backup strategy: full weekly + incremental daily. |
| DF-OPS-08 | Upgrade MUST backward-compatible untuk minor version. |
| DF-OPS-09 | Downgrade MUST eksplisit dan logged. |
| DF-OPS-10 | Migration MUST atomic (berhasil penuh atau gagal penuh). |
| DF-OPS-11 | Monitoring via OpenMetrics endpoint (Prometheus-compatible). |
| DF-OPS-12 | Health check endpoint MUST tersedia. |
| DF-OPS-13 | Operational runbook MUST terdokumentasi. |
| DF-OPS-14 | Remote storage menggunakan S3-compatible API. |
| DF-OPS-15 | Store directory MUST dedicated filesystem atau mount. |

---

# 1. Executive Summary

## 1.1 Operational Philosophy

`[OPS-EXEC-1]` Prinsip operasional CNWS:

1. **Simple deployment**: satu binary, satu store, satu konfigurasi.
2. **Predictable operations**: setiap operasi memiliki prosedur yang terdefinisi.
3. **Safe upgrades**: upgrade tidak boleh menyebabkan data loss.
4. **Recoverable**: setiap failure memiliki recovery procedure.
5. **Observable**: setiap aspek sistem dapat dimonitor.
6. **Automatable**: setiap operasi dapat diotomatisasi.

## 1.2 Deployment Topology

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS DEPLOYMENT                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              CNWS Node (Single Machine)              │   │
│  │                                                     │   │
│  │   ┌─────────────┐  ┌─────────────┐                │   │
│  │   │ CNWS Binary │  │ Config File │                │   │
│  │   └──────┬──────┘  └─────────────┘                │   │
│  │          │                                          │   │
│  │          ▼                                          │   │
│  │   ┌─────────────────────────────────────────┐      │   │
│  │   │           .cd Store (NVMe)              │      │   │
│  │   │                                         │      │   │
│  │   │   SUPERBLOCK  MANIFEST.cd  segments/    │      │   │
│  │   │   journal/    index/       memory/      │      │   │
│  │   │   revisions/  lattice/     meta/        │      │   │
│  │   └─────────────────────────────────────────┘      │   │
│  │                                                     │   │
│  │   ┌──────────┐  ┌──────────┐  ┌──────────┐       │   │
│  │   │ GPU 0    │  │ GPU 1    │  │ ...      │       │   │
│  │   └──────────┘  └──────────┘  └──────────┘       │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  Optional:                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Remote Storage (S3-compatible)              │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │         Monitoring (Prometheus + Grafana)           │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

---

# 2. System Requirements

## 2.1 Hardware Requirements

`[OPS-HW-1]` Minimum dan recommended hardware:

### 2.1.1 Minimum Requirements (Development)

| Component | Minimum | Notes |
|---|---|---|
| CPU | 4 cores | x86-64 atau ARM64 |
| RAM | 16 GB | Untuk runtime kecil |
| Storage | 100 GB NVMe | Untuk store kecil |
| GPU | Optional | Untuk GPU acceleration |
| Network | 1 Gbps | Untuk remote storage |

### 2.1.2 Recommended Requirements (Production)

| Component | Recommended | Notes |
|---|---|---|
| CPU | 16+ cores | High single-thread performance |
| RAM | 64+ GB | Tergantung model size |
| Storage | 2+ TB NVMe | Gen4 recommended |
| GPU | 1+ (24+ GB VRAM) | Untuk inference |
| Network | 10+ Gbps | Untuk remote operations |

### 2.1.3 Large-Scale Requirements

| Component | Large Scale | Notes |
|---|---|---|
| CPU | 64+ cores | Multi-socket OK |
| RAM | 256+ GB | Untuk model > 100 GB |
| Storage | 16+ TB NVMe | RAID atau distributed |
| GPU | 4+ (80 GB VRAM each) | Multi-GPU inference |
| Network | 25+ Gbps | High-bandwidth operations |

## 2.2 Software Requirements

`[OPS-HW-2]` Software requirements:

| Component | Requirement | Notes |
|---|---|---|
| OS | Linux 5.15+ | Ubuntu 22.04+, RHEL 9+ |
| Filesystem | XFS atau ext4 | Untuk `.cd` store |
| CUDA | 12.0+ | Untuk GPU support |
| ROCm | 6.0+ | Untuk AMD GPU (optional) |
| glibc | 2.35+ | — |
| Kernel modules | — | Tidak ada yang khusus |

## 2.3 GPU Configuration

`[OPS-GPU-1]` GPU requirements:

| GPU Tier | VRAM | Compute Capability | Use Case |
|---|---|---|---|
| Entry | 12 GB | 8.0+ | Small models, development |
| Standard | 24 GB | 8.6+ | Medium models |
| Professional | 40-80 GB | 9.0+ | Large models, production |
| Data Center | 80+ GB | 9.0+ | XL models, multi-GPU |

`[OPS-GPU-2]` GPU detection:

```bash
# Check GPU availability
cnws diag gpu-detect

# Expected output:
# GPU 0: NVIDIA RTX 4090 (24 GB, CC 8.9, FP8: yes)
# GPU 1: NVIDIA A100 (80 GB, CC 8.0, FP8: no)
```

## 2.4 NVMe Requirements

`[OPS-NVME-1]` NVMe requirements:

| Metric | Minimum | Recommended |
|---|---|---|
| Sequential read | 3,000 MB/s | 7,000 MB/s |
| Sequential write | 2,000 MB/s | 5,000 MB/s |
| Random read IOPS | 500K | 1M+ |
| Latency | < 100 μs | < 50 μs |
| Endurance | 1 DWPD | 3+ DWPD |

`[OPS-NVME-2]` Filesystem recommendations:

| Filesystem | Status | Notes |
|---|---|---|
| XFS | RECOMMENDED | Best untuk large files |
| ext4 | SUPPORTED | Good general purpose |
| Btrfs | NOT RECOMMENDED | Copy-on-write overhead |
| ZFS | NOT RECOMMENDED | Overhead untuk use case ini |
| tmpfs | FORBIDDEN | Tidak persistent |

---

# 3. Installation

## 3.1 Installation Methods

`[OPS-INST-1]` CNWS dapat diinstal melalui:

| Method | Status | Use Case |
|---|---|---|
| Package manager | RECOMMENDED | Production |
| Binary download | SUPPORTED | Quick setup |
| From source | SUPPORTED | Development |
| Container image | MAY | Containerized deployment |

## 3.2 Package Manager Installation

### 3.2.1 Debian/Ubuntu

```bash
# Add CNWS repository
curl -fsSL https://packages.cnws.dev/gpg | sudo gpg --dearmor -o /usr/share/keyrings/cnws.gpg
echo "deb [signed-by=/usr/share/keyrings/cnws.gpg] https://packages.cnws.dev/apt stable main" | \
    sudo tee /etc/apt/sources.list.d/cnws.list

# Install
sudo apt update
sudo apt install cnws

# Verify
cnws --version
# Expected: cnws 1.0.0
```

### 3.2.2 RHEL/CentOS/Fedora

```bash
# Add CNWS repository
sudo dnf config-manager --add-repo https://packages.cnws.dev/rpm/cnws.repo

# Install
sudo dnf install cnws

# Verify
cnws --version
```

### 3.2.3 Arch Linux

```bash
# Install from AUR
yay -S cnws

# Verify
cnws --version
```

## 3.3 Binary Installation

```bash
# Download binary
curl -fsSL https://releases.cnws.dev/cnws-1.0.0-linux-amd64.tar.gz | \
    sudo tar -xz -C /usr/local/bin

# Verify checksum
sha256sum /usr/local/bin/cnws
# Compare with published checksum

# Verify
cnws --version
```

## 3.4 From Source

```bash
# Clone repository
git clone https://github.com/cnws/cnws.git
cd cnws

# Build (requires Rust 1.75+)
cargo build --release

# Install
sudo cp target/release/cnws /usr/local/bin/

# Verify
cnws --version
```

## 3.5 Post-Installation Verification

`[OPS-INST-2]` Setelah instalasi, MUST jalankan verifikasi:

```bash
# System check
cnws diag system-check

# Expected output:
# ✓ CPU: 16 cores detected
# ✓ RAM: 64 GB available
# ✓ NVMe: /dev/nvme0n1 (7000 MB/s sequential read)
# ✓ Filesystem: XFS supported
# ✓ GPU: NVIDIA RTX 4090 detected (24 GB VRAM)
# ✓ CUDA: 12.4 available
# ✓ Permissions: OK
# 
# System check: PASSED
```

## 3.6 Installation Invariants

| ID | Invariant |
|---|---|
| OPS-INST-INV-1 | Installation MUST idempotent |
| OPS-INST-INV-2 | Installation MUST NOT memodifikasi existing store |
| OPS-INST-INV-3 | Version MUST terverifikasi setelah install |
| OPS-INST-INV-4 | System check MUST lulus sebelum production use |

---

# 4. Store Initialization

## 4.1 Store Creation

`[OPS-STORE-1]` Store dibuat dengan `cnws init`:

```bash
# Create new store
cnws init /data/model.cd \
    --model-id "example-org/model-70b" \
    --gpu-budget 16GB \
    --cpu-budget 32GB

# Expected output:
# Initializing CNWS store at /data/model.cd
# ✓ Created SUPERBLOCK
# ✓ Created MANIFEST.cd
# ✓ Created directory structure
# ✓ Initialized journal
# ✓ Store initialized successfully
# 
# Store ID: b3:store_7f3a8e...
# Model ID: example-org/model-70b
```

## 4.2 Store Directory Structure

`[OPS-STORE-2]` Store initialization MUST membuat struktur berikut:

```text
/data/model.cd/
├── SUPERBLOCK                  # 4096 bytes
├── LOCK                        # Advisory lock
├── MANIFEST.cd                 # Root manifest
├── MANIFEST.cd.prev            # Previous manifest
├── cnws.toml                   # Store configuration
│
├── journal/
│   └── commit.wal              # Write-ahead log
│
├── staging/                    # Staging area
│
├── index/
│   ├── cells.idx
│   ├── tiles.idx
│   ├── memory.idx
│   └── routing.idx
│
├── segments/                   # Tile storage
│
├── lattice/
│   ├── graph.cd
│   ├── compositions.cd
│   └── routing_policy.cd
│
├── memory/
│   ├── episodic/
│   ├── semantic/
│   ├── procedural/
│   └── index.cd
│
├── revisions/                  # Revision objects
│
├── corrupt/                    # Quarantine
│
└── meta/
    ├── provenance/
    └── routing_stats/
```

## 4.3 Store Configuration File

`[OPS-STORE-3]` Store configuration disimpan di `cnws.toml` dalam store directory:

```toml
# /data/model.cd/cnws.toml

[store]
model_id = "example-org/model-70b"
created_at = "2026-08-11T00:00:00Z"
format_version = "1.0.0"

[cache]
gpu_budget_bytes = 17179869184    # 16 GB
cpu_budget_bytes = 34359738368    # 32 GB
eviction_policy = "lru_by_priority"
prefetch_policy = "dependency_aware"
prefetch_depth = 2

[budget]
max_flops = 10000000000           # 10 GFLOP
max_bytes_moved = 1073741824      # 1 GB
max_wall_time_us = 1000000        # 1 second
working_memory_bytes = 268435456  # 256 MB

[runtime]
deterministic_mode = true
accuracy_policy = "balanced"
min_depth = 3
max_depth = 25

[conversion]
buffer_size_bytes = 2147483648    # 2 GB
tile_size_target_bytes = 134217728  # 128 MB
compression = "zstd3"

[gc]
grace_period_days = 7
auto_gc = false

[observability]
metrics_enabled = true
metrics_port = 9090
log_level = "info"
log_file = "/var/log/cnws/cnws.log"

[remote]
enabled = false
# endpoint = "https://s3.example.com"
# bucket = "cnws-store"
```

## 4.4 Store Initialization Options

```bash
cnws init --help

# Usage: cnws init [OPTIONS] <PATH>
# 
# Options:
#   --model-id <ID>              Model identifier (required)
#   --gpu-budget <SIZE>          GPU cache budget (default: 8GB)
#   --cpu-budget <SIZE>          CPU cache budget (default: 16GB)
#   --working-memory <SIZE>      Working memory limit (default: 256MB)
#   --compression <CODEC>        Default compression (default: zstd3)
#   --tile-size <SIZE>           Target tile size (default: 128MB)
#   --no-remote                  Disable remote storage
#   --config-template            Generate config template only
#   --dry-run                    Show what would be created
```

## 4.5 Store Initialization Invariants

| ID | Invariant |
|---|---|
| OPS-STORE-INV-1 | Store init MUST membuat struktur lengkap |
| OPS-STORE-INV-2 | Store init MUST idempotent |
| OPS-STORE-INV-3 | Store init MUST NOT overwrite existing store |
| OPS-STORE-INV-4 | Store config MUST tersimpan dalam store |
| OPS-STORE-INV-5 | Store init MUST mencatat timestamp |

---

# 5. Configuration

## 5.1 Configuration Hierarchy

`[OPS-CFG-1]` Configuration hierarchy (highest to lowest priority):

```text
1. Command-line flags          (highest priority)
2. Environment variables
3. Store config (cnws.toml)
4. System config (/etc/cnws/config.toml)
5. Built-in defaults           (lowest priority)
```

## 5.2 Configuration File Format

`[OPS-CFG-2]` Configuration MUST menggunakan TOML.

### 5.2.1 System Configuration

```toml
# /etc/cnws/config.toml

[defaults]
log_level = "info"
metrics_enabled = true
deterministic_mode = true

[hardware]
# Auto-detect if not specified
# gpu_count = 1
# nvme_device = "/dev/nvme0n1"

[network]
# For remote storage
timeout_seconds = 60
retry_count = 3
retry_backoff_ms = 1000

[security]
allow_symlinks = false
max_allocation_bytes = 1099511627776  # 1 TB
restricted_unpickler = true

[telemetry]
export_format = "prometheus"
metrics_port = 9090
traces_enabled = false
traces_sampling_rate = 0.01
```

### 5.2.2 Environment Variables

`[OPS-CFG-3]` Environment variables menggunakan prefix `CNWS_`:

| Variable | Description | Example |
|---|---|---|
| `CNWS_LOG_LEVEL` | Log level | `CNWS_LOG_LEVEL=debug` |
| `CNWS_METRICS_PORT` | Metrics port | `CNWS_METRICS_PORT=9091` |
| `CNWS_GPU_BUDGET` | GPU budget | `CNWS_GPU_BUDGET=16GB` |
| `CNWS_CPU_BUDGET` | CPU budget | `CNWS_CPU_BUDGET=32GB` |
| `CNWS_STORE_PATH` | Default store path | `CNWS_STORE_PATH=/data/model.cd` |
| `CNWS_CONFIG` | Config file path | `CNWS_CONFIG=/etc/cnws/config.toml` |

## 5.3 Configuration Validation

`[OPS-CFG-4]` Configuration MUST divalidasi saat load:

```bash
# Validate configuration
cnws config validate /data/model.cd

# Expected output:
# Validating configuration...
# ✓ Store config: valid
# ✓ Cache config: valid
# ✓ Budget config: valid
# ✓ Runtime config: valid
# 
# Configuration: VALID
```

## 5.4 Configuration Commands

```bash
# Show current configuration
cnws config show /data/model.cd

# Show specific section
cnws config show /data/model.cd --section cache

# Set configuration value
cnws config set /data/model.cd cache.gpu_budget_bytes 17179869184

# Reset to defaults
cnws config reset /data/model.cd --section cache

# Export configuration
cnws config export /data/model.cd > config-backup.toml

# Import configuration
cnws config import /data/model.cd < config-backup.toml
```

## 5.5 Configuration Invariants

| ID | Invariant |
|---|---|
| OPS-CFG-INV-1 | Configuration MUST TOML format |
| OPS-CFG-INV-2 | Configuration MUST divalidasi saat load |
| OPS-CFG-INV-3 | Invalid configuration MUST menghasilkan error |
| OPS-CFG-INV-4 | Configuration hierarchy MUST dipatuhi |
| OPS-CFG-INV-5 | Store config MUST tersimpan dalam store |

---

# 6. GPU Configuration

## 6.1 GPU Detection

`[OPS-GPU-1]` GPU detection otomatis saat startup:

```bash
# Detect GPUs
cnws diag gpu-detect

# Output:
# Detected 2 GPUs:
# 
# GPU 0:
#   Model: NVIDIA RTX 4090
#   VRAM: 24 GB (25769803776 bytes)
#   Compute Capability: 8.9
#   FP8 Support: yes
#   Driver: 550.54.15
#   CUDA: 12.4
# 
# GPU 1:
#   Model: NVIDIA A100-SXM4-80GB
#   VRAM: 80 GB (85899345920 bytes)
#   Compute Capability: 8.0
#   FP8 Support: no
#   Driver: 550.54.15
#   CUDA: 12.4
```

## 6.2 GPU Budget Configuration

`[OPS-GPU-2]` GPU budget configuration:

```toml
# In cnws.toml

[gpu]
# GPU selection (auto-detect if not specified)
# device_ids = [0, 1]

# VRAM budget per GPU
# If not specified: 80% of VRAM
budget_percent = 80

# Reserved VRAM for runtime (not for Tiles)
reserved_bytes = 2147483648  # 2 GB

# FP8 enablement
fp8_enabled = "auto"  # auto, true, false

# Multi-GPU strategy
multi_gpu_strategy = "round_robin"  # round_robin, manual, auto
```

## 6.3 Multi-GPU Configuration

`[OPS-GPU-3]` Multi-GPU configuration:

```toml
[gpu.multi]
strategy = "round_robin"

# Manual assignment (if strategy = "manual")
[[gpu.multi.assignments]]
gpu_id = 0
cell_types = ["ATTENTION_Q_PROJ", "ATTENTION_K_PROJ", "ATTENTION_V_PROJ"]

[[gpu.multi.assignments]]
gpu_id = 1
cell_types = ["MLP_GATE", "MLP_UP", "MLP_DOWN"]
```

## 6.4 GPU Configuration Commands

```bash
# Show GPU status
cnws gpu status

# Set GPU budget
cnws gpu set-budget --gpu 0 --bytes 16GB

# Enable/disable FP8
cnws gpu set-fp8 --gpu 0 --enabled true

# Show GPU utilization
cnws gpu utilization
```

## 6.5 GPU Invariants

| ID | Invariant |
|---|---|
| OPS-GPU-INV-1 | GPU detection MUST otomatis |
| OPS-GPU-INV-2 | GPU budget MUST configurable |
| OPS-GPU-INV-3 | GPU budget MUST hard-enforced |
| OPS-GPU-INV-4 | Multi-GPU MUST didukung |
| OPS-GPU-INV-5 | GPU failure MUST NOT crash system |

---

# 7. NVMe Layout

## 7.1 Directory Layout

`[OPS-NVME-1]` Recommended NVMe layout:

```text
/nvme/
├── cnws-stores/              # CNWS stores
│   ├── model-70b.cd/
│   ├── model-13b.cd/
│   └── ...
│
├── cnws-cache/               # Optional: separate cache
│   └── ...
│
└── cnws-temp/                # Temporary files
    └── ...
```

## 7.2 Filesystem Mount Options

`[OPS-NVME-2]` Recommended mount options:

```bash
# /etc/fstab entry
/dev/nvme0n1 /nvme xfs noatime,nodiratime,discard=async 0 0

# Or for ext4:
/dev/nvme0n1 /nvme ext4 noatime,nodiratime,discard=async 0 0
```

`[OPS-NVME-3]` Mount options explanation:

| Option | Purpose |
|---|---|
| `noatime` | Disable access time updates (reduce writes) |
| `nodiratime` | Disable directory access time |
| `discard=async` | Enable TRIM for SSD longevity |

## 7.3 I/O Scheduler

`[OPS-NVME-4]` I/O scheduler configuration:

```bash
# Check current scheduler
cat /sys/block/nvme0n1/queue/scheduler
# Expected: none mq-deadline kyber [bfq]

# Set to 'none' for NVMe (recommended)
echo "none" | sudo tee /sys/block/nvme0n1/queue/scheduler

# Or 'mq-deadline' for mixed workloads
echo "mq-deadline" | sudo tee /sys/block/nvme0n1/queue/scheduler

# Make persistent
echo 'ACTION=="add|change", KERNEL=="nvme*", ATTR{queue/scheduler}="none"' | \
    sudo tee /etc/udev/rules.d/60-nvme-scheduler.rules
```

## 7.4 Read-Ahead Configuration

`[OPS-NVME-5]` Read-ahead tuning:

```bash
# Check current read-ahead
blockdev --getra /dev/nvme0n1
# Default: 256 (128 KB)

# Set read-ahead (in 512-byte sectors)
# For large sequential reads: 4096 (2 MB)
sudo blockdev --setra 4096 /dev/nvme0n1

# Make persistent
echo 'ACTION=="add|change", KERNEL=="nvme*", ATTR{bdi/read_ahead_kb}="2048"' | \
    sudo tee /etc/udev/rules.d/60-nvme-readahead.rules
```

## 7.5 NVMe Health Monitoring

`[OPS-NVME-6]` NVMe health monitoring:

```bash
# Check NVMe health
sudo nvme smart-log /dev/nvme0n1

# Key metrics to monitor:
# - temperature
# - available_spare (should be > 10%)
# - percentage_used (should be < 90%)
# - media_errors (should be 0)
```

## 7.6 NVMe Invariants

| ID | Invariant |
|---|---|
| OPS-NVME-INV-1 | Store MUST pada NVMe atau SSD |
| OPS-NVME-INV-2 | Filesystem MUST XFS atau ext4 |
| OPS-NVME-INV-3 | I/O scheduler SHOULD `none` untuk NVMe |
| OPS-NVME-INV-4 | NVMe health MUST dimonitor |
| OPS-NVME-INV-5 | Store MUST NOT pada network filesystem (NFS, CIFS) |

---

# 8. Remote Storage

## 8.1 Remote Storage Configuration

`[OPS-REMOTE-1]` Remote storage untuk:
- Backup
- Cross-machine sharing
- Disaster recovery

```toml
# In cnws.toml

[remote]
enabled = true
endpoint = "https://s3.example.com"
bucket = "cnws-store"
prefix = "model-70b"
region = "us-east-1"

# Authentication
auth_method = "env"  # env, config, iam_role
# access_key_id = "..."     # if auth_method = "config"
# secret_access_key = "..."  # if auth_method = "config"

# Performance
multipart_threshold_mb = 100
multipart_chunk_size_mb = 64
max_concurrent_transfers = 8

# Caching
cache_remote_tiles = true
remote_cache_size_gb = 100
```

## 8.2 Remote Storage Operations

```bash
# Configure remote storage
cnws remote configure /data/model.cd \
    --endpoint https://s3.example.com \
    --bucket cnws-store

# Test connection
cnws remote test /data/model.cd

# Sync to remote
cnws remote sync /data/model.cd --direction push

# Sync from remote
cnws remote sync /data/model.cd --direction pull

# List remote contents
cnws remote list /data/model.cd
```

## 8.3 Remote Storage Invariants

| ID | Invariant |
|---|---|
| OPS-REMOTE-INV-1 | Remote storage MUST S3-compatible |
| OPS-REMOTE-INV-2 | Remote transfer MUST menggunakan integrity verification |
| OPS-REMOTE-INV-3 | Remote sync MUST atomic per Tile |
| OPS-REMOTE-INV-4 | Remote credentials MUST NOT disimpan plaintext |
| OPS-REMOTE-INV-5 | Remote failure MUST NOT crash local operations |

---

# 9. Upgrade

## 9.1 Version Compatibility

`[OPS-UPG-1]` Version compatibility rules:

| From | To | Compatible | Action |
|---|---|---|---|
| 1.0.x | 1.0.y | YES | Direct upgrade |
| 1.0.x | 1.1.x | YES | Direct upgrade (backward compatible) |
| 1.0.x | 2.0.x | NO | Migration required |
| 1.1.x | 1.0.x | MAY | Downgrade (explicit) |

## 9.2 Pre-Upgrade Checklist

`[OPS-UPG-2]` Sebelum upgrade, MUST:

```bash
# 1. Check current version
cnws --version

# 2. Check store integrity
cnws diag integrity /data/model.cd

# 3. Backup store
cnws backup create /data/model.cd --target /backup/model.cd-backup

# 4. Check disk space
df -h /data

# 5. Check release notes
# Review changelog for breaking changes

# 6. Stop active workloads
cnws runtime stop /data/model.cd
```

## 9.3 Upgrade Procedure

```bash
# 1. Stop CNWS service
sudo systemctl stop cnws

# 2. Backup (if not done)
cnws backup create /data/model.cd --target /backup/pre-upgrade

# 3. Upgrade binary
sudo apt update
sudo apt upgrade cnws

# Or for binary installation:
# curl -fsSL https://releases.cnws.dev/cnws-1.1.0-linux-amd64.tar.gz | \
#     sudo tar -xz -C /usr/local/bin

# 4. Verify new version
cnws --version
# Expected: cnws 1.1.0

# 5. Run migration (if needed)
cnws migrate /data/model.cd --from 1.0.0 --to 1.1.0

# 6. Verify store
cnws diag integrity /data/model.cd

# 7. Start service
sudo systemctl start cnws

# 8. Verify operation
cnws diag health /data/model.cd
```

## 9.4 Upgrade Rollback

`[OPS-UPG-3]` Jika upgrade gagal:

```bash
# 1. Stop service
sudo systemctl stop cnws

# 2. Restore backup
cnws backup restore /backup/pre-upgrade --target /data/model.cd

# 3. Downgrade binary
sudo apt install cnws=1.0.0

# 4. Start service
sudo systemctl start cnws

# 5. Verify
cnws diag health /data/model.cd
```

## 9.5 Upgrade Invariants

| ID | Invariant |
|---|---|
| OPS-UPG-INV-1 | Upgrade MUST backward-compatible untuk minor version |
| OPS-UPG-INV-2 | Backup MUST dilakukan sebelum upgrade |
| OPS-UPG-INV-3 | Upgrade MUST dapat di-rollback |
| OPS-UPG-INV-4 | Store integrity MUST diverifikasi setelah upgrade |
| OPS-UPG-INV-5 | Breaking changes MUST memerlukan major version |

---

# 10. Downgrade

## 10.1 Downgrade Policy

`[OPS-DWN-1]` Downgrade policy:

| Scenario | Allowed | Notes |
|---|---|---|
| Patch downgrade (1.0.1 → 1.0.0) | YES | Safe |
| Minor downgrade (1.1.0 → 1.0.0) | MAY | Requires verification |
| Major downgrade (2.0.0 → 1.0.0) | NO | Not supported |

## 10.2 Downgrade Procedure

```bash
# 1. Verify downgrade is supported
cnws version check-downgrade --from 1.1.0 --to 1.0.0

# 2. Backup current state
cnws backup create /data/model.cd --target /backup/pre-downgrade

# 3. Stop service
sudo systemctl stop cnws

# 4. Install older version
sudo apt install cnws=1.0.0

# 5. Verify store compatibility
cnws diag integrity /data/model.cd

# 6. Start service
sudo systemctl start cnws

# 7. Verify operation
cnws diag health /data/model.cd

# 8. Log downgrade
cnws audit log "Downgraded from 1.1.0 to 1.0.0"
```

## 10.3 Downgrade Invariants

| ID | Invariant |
|---|---|
| OPS-DWN-INV-1 | Downgrade MUST eksplisit |
| OPS-DWN-INV-2 | Downgrade MUST logged |
| OPS-DWN-INV-3 | Major version downgrade MUST NOT didukung |
| OPS-DWN-INV-4 | Store compatibility MUST diverifikasi |

---

# 11. Migration

## 11.1 Migration Types

`[OPS-MIG-1]` Migration types:

| Type | Description | Complexity |
|---|---|---|
| Cross-machine | Move store to different machine | Medium |
| Cross-filesystem | Move store to different filesystem | Low |
| Format version | Upgrade store format version | High |
| Model merge | Merge multiple stores | High |

## 11.2 Cross-Machine Migration

```bash
# On source machine:
# 1. Verify store
cnws diag integrity /data/model.cd

# 2. Export store
cnws export /data/model.cd --target /backup/model.cd-export

# 3. Transfer (rsync recommended)
rsync -avz --progress /backup/model.cd-export/ target-machine:/data/model.cd/

# On target machine:
# 4. Verify transfer
cnws diag integrity /data/model.cd

# 5. Update configuration if needed
cnws config show /data/model.cd

# 6. Test operation
cnws runtime test /data/model.cd
```

## 11.3 Format Version Migration

```bash
# Check current format version
cnws store info /data/model.cd | grep format_version

# Migrate to new format version
cnws migrate /data/model.cd --to-format 2.0.0

# Migration will:
# 1. Backup current store
# 2. Convert format
# 3. Verify integrity
# 4. Update SUPERBLOCK
```

## 11.4 Migration Invariants

| ID | Invariant |
|---|---|
| OPS-MIG-INV-1 | Migration MUST atomic |
| OPS-MIG-INV-2 | Migration MUST dapat di-rollback |
| OPS-MIG-INV-3 | Migration MUST memverifikasi integrity |
| OPS-MIG-INV-4 | Migration MUST logged |
| OPS-MIG-INV-5 | Cross-machine migration MUST menggunakan checksum verification |

---

# 12. Backup

## 12.1 Backup Strategy

`[OPS-BKP-1]` Backup strategy:

| Type | Frequency | Retention | Description |
|---|---|---|---|
| Full backup | Weekly | 4 weeks | Complete store copy |
| Incremental backup | Daily | 7 days | Changed Tiles only |
| Remote backup | Continuous | Configurable | Sync to remote storage |

## 12.2 Full Backup

```bash
# Create full backup
cnws backup create /data/model.cd \
    --target /backup/model.cd-full-$(date +%Y%m%d) \
    --type full \
    --verify

# Expected output:
# Creating full backup...
# ✓ Verified store integrity
# ✓ Copied SUPERBLOCK
# ✓ Copied MANIFEST.cd
# ✓ Copied 256 segments (1.2 TB)
# ✓ Copied revisions
# ✓ Copied memory
# ✓ Verified backup integrity
# 
# Backup complete: /backup/model.cd-full-20260811
# Size: 1.2 TB
# Duration: 45 minutes
# Checksum: b3:backup_7f3a8e...
```

## 12.3 Incremental Backup

```bash
# Create incremental backup
cnws backup create /data/model.cd \
    --target /backup/model.cd-incr-$(date +%Y%m%d) \
    --type incremental \
    --since "2026-08-10T00:00:00Z"

# Expected output:
# Creating incremental backup...
# ✓ Found 12 changed Tiles
# ✓ Copied 12 Tiles (1.5 GB)
# ✓ Copied 2 new revisions
# ✓ Verified backup integrity
# 
# Backup complete: /backup/model.cd-incr-20260811
# Size: 1.5 GB
# Duration: 2 minutes
```

## 12.4 Backup Verification

`[OPS-BKP-2]` Backup MUST diverifikasi:

```bash
# Verify backup
cnws backup verify /backup/model.cd-full-20260811

# Expected output:
# Verifying backup...
# ✓ SUPERBLOCK valid
# ✓ MANIFEST.cd hash matches
# ✓ All segments valid
# ✓ All Tiles verified
# 
# Backup: VALID
```

## 12.5 Backup Automation

```bash
# Cron job for daily incremental backup
# /etc/cron.d/cnws-backup

# Daily incremental backup at 2 AM
0 2 * * * cnws backup create /data/model.cd --target /backup/incr-$(date +\%Y\%m\%d) --type incremental

# Weekly full backup on Sunday at 3 AM
0 3 * * 0 cnws backup create /data/model.cd --target /backup/full-$(date +\%Y\%m\%d) --type full --verify
```

## 12.6 Backup Invariants

| ID | Invariant |
|---|---|
| OPS-BKP-INV-1 | Backup MUST diverifikasi |
| OPS-BKP-INV-2 | Backup MUST atomic |
| OPS-BKP-INV-3 | Backup MUST dapat di-restore |
| OPS-BKP-INV-4 | Backup retention MUST dipatuhi |
| OPS-BKP-INV-5 | Backup failure MUST alert |

---

# 13. Restore

## 13.1 Full Restore

```bash
# Restore from full backup
cnws backup restore /backup/model.cd-full-20260811 \
    --target /data/model.cd \
    --verify

# Expected output:
# Restoring from backup...
# ✓ Verified backup integrity
# ✓ Restored SUPERBLOCK
# ✓ Restored MANIFEST.cd
# ✓ Restored 256 segments
# ✓ Restored revisions
# ✓ Restored memory
# ✓ Verified restored store
# 
# Restore complete: /data/model.cd
# Duration: 50 minutes
```

## 13.2 Incremental Restore

```bash
# Restore incremental backup on top of full backup
cnws backup restore /backup/model.cd-incr-20260811 \
    --target /data/model.cd \
    --type incremental

# This applies incremental changes to existing store
```

## 13.3 Point-in-Time Restore

```bash
# Restore to specific revision
cnws backup restore /backup/model.cd-full-20260811 \
    --target /data/model.cd \
    --revision "b3:rev42..."

# This restores store and sets active revision
```

## 13.4 Partial Restore

```bash
# Restore specific Tiles only
cnws backup restore /backup/model.cd-full-20260811 \
    --target /data/model.cd \
    --tiles "b3:tile1...,b3:tile2..."

# Restore specific Cells only
cnws backup restore /backup/model.cd-full-20260811 \
    --target /data/model.cd \
    --cells "b3:cell1...,b3:cell2..."
```

## 13.5 Restore Verification

`[OPS-RST-1]` Setelah restore, MUST verifikasi:

```bash
# Verify restored store
cnws diag integrity /data/model.cd

# Test operation
cnws runtime test /data/model.cd

# Compare with backup checksum
cnws backup checksum /backup/model.cd-full-20260811
cnws store checksum /data/model.cd
```

## 13.6 Restore Invariants

| ID | Invariant |
|---|---|
| OPS-RST-INV-1 | Restore MUST diverifikasi |
| OPS-RST-INV-2 | Restore MUST atomic |
| OPS-RST-INV-3 | Restore MUST dapat dibatalkan |
| OPS-RST-INV-4 | Restored store MUST lulus integrity check |
| OPS-RST-INV-5 | Restore MUST logged |

---

# 14. Monitoring

## 14.1 Monitoring Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                    CNWS MONITORING                           │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  CNWS Node                                                  │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  CNWS Process                                       │   │
│  │  ┌─────────────┐  ┌─────────────┐                 │   │
│  │  │ Metrics     │  │ Health      │                 │   │
│  │  │ Endpoint    │  │ Endpoint    │                 │   │
│  │  │ :9090       │  │ :9091/health│                 │   │
│  │  └──────┬──────┘  └──────┬──────┘                 │   │
│  └─────────┼────────────────┼─────────────────────────┘   │
│            │                │                               │
└────────────┼────────────────┼───────────────────────────────┘
             │                │
             ▼                ▼
┌─────────────────────────────────────────────────────────────┐
│              MONITORING STACK                                │
│                                                             │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                │
│  │Prometheus│  │ Grafana  │  │Alertmanager│               │
│  └──────────┘  └──────────┘  └──────────┘                │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 14.2 Metrics Endpoint

`[OPS-MON-1]` Metrics endpoint MUST tersedia:

```bash
# Check metrics endpoint
curl http://localhost:9090/metrics

# Sample output:
# cnws_runtime_cell_resolve_total{cell_type="ATTENTION_Q_PROJ",status="success"} 42
# cnws_runtime_tile_load_duration_seconds_bucket{le="0.001"} 100
# cnws_cache_hit_ratio{level="gpu"} 0.94
# cnws_storage_tile_count_total 65536
# ...
```

## 14.3 Health Check Endpoint

`[OPS-MON-2]` Health check endpoint MUST tersedia:

```bash
# Health check
curl http://localhost:9091/health

# Response:
{
  "status": "healthy",
  "timestamp": "2026-08-11T12:34:56Z",
  "version": "1.0.0",
  "uptime_seconds": 86400
}
```

## 14.4 Prometheus Configuration

```yaml
# prometheus.yml

scrape_configs:
  - job_name: 'cnws'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
    metrics_path: /metrics

  - job_name: 'cnws-health'
    static_configs:
      - targets: ['localhost:9091']
    scrape_interval: 30s
    metrics_path: /health
```

## 14.5 Alerting Rules

`[OPS-MON-3]` Recommended alerting rules:

```yaml
# cnws-alerts.yml

groups:
  - name: cnws-alerts
    rules:
      - alert: CNWSStoreCorrupted
        expr: cnws_integrity_errors_total > 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "CNWS store corruption detected"
      
      - alert: CNWSCacheHitRateLow
        expr: cnws_cache_hit_ratio < 0.7
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CNWS cache hit rate below 70%"
      
      - alert: CNWSMemoryBudgetExceeded
        expr: cnws_memory_working_size_bytes > cnws_memory_working_budget_bytes
        for: 1m
        labels:
          severity: warning
        annotations:
          summary: "CNWS working memory budget exceeded"
      
      - alert: CNWSDiskSpaceLow
        expr: cnws_storage_disk_free_bytes < 10737418240  # 10 GB
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "CNWS disk space below 10 GB"
      
      - alert: CNWSProcessDown
        expr: up{job="cnws"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "CNWS process is down"
```

## 14.6 Grafana Dashboard

`[OPS-MON-4]` Grafana dashboard SHOULD tersedia untuk:

| Dashboard | Panels |
|---|---|
| CNWS Overview | Store size, Cell count, Tile count, revision count |
| CNWS Runtime | Active Cells, cache hit rate, bytes moved |
| CNWS Performance | Latency P50/P95/P99, throughput |
| CNWS Storage | Disk usage, segment count, GC status |
| CNWS Errors | Error rate, error types, corruption events |

## 14.7 Monitoring Invariants

| ID | Invariant |
|---|---|
| OPS-MON-INV-1 | Metrics endpoint MUST tersedia |
| OPS-MON-INV-2 | Health check MUST tersedia |
| OPS-MON-INV-3 | Alerting rules SHOULD terdefinisi |
| OPS-MON-INV-4 | Monitoring MUST NOT signifikan mempengaruhi performa |
| OPS-MON-INV-5 | Metrics MUST sesuai Observability Spec |

---

# 15. Operational Failure Handling

## 15.1 Failure Classification

`[OPS-FAIL-1]` Failure classification:

| Severity | Description | Response Time |
|---|---|---|
| Critical | System down, data loss risk | Immediate |
| High | Degraded operation | < 1 hour |
| Medium | Performance degradation | < 4 hours |
| Low | Minor issue | < 24 hours |

## 15.2 Common Failure Scenarios

### 15.2.1 Disk Full

```text
Symptoms:
  - Write operations fail
  - cnws_errors_total{code="CNWS-E-STORE"} increasing
  - Disk usage > 95%

Diagnosis:
  cnws diag store-status /data/model.cd
  df -h /data

Resolution:
  1. Identify large files: du -sh /data/model.cd/*
  2. Run GC: cnws gc /data/model.cd --dry-run
  3. If safe: cnws gc /data/model.cd
  4. Remove old backups: rm /backup/old-*
  5. Expand disk if needed

Prevention:
  - Monitor disk usage
  - Alert at 80% usage
  - Configure GC schedule
```

### 15.2.2 Tile Corruption

```text
Symptoms:
  - cnws_integrity_errors_total increasing
  - Tile load failures
  - Corruption events in logs

Diagnosis:
  cnws diag integrity /data/model.cd
  cnws diag corruption-status

Resolution:
  1. Identify corrupted Tiles: cnws diag corruption-status
  2. Attempt recovery: cnws repair /data/model.cd --tiles <tile-ids>
  3. If recovery fails: restore from backup
  4. Investigate root cause (disk health, bit rot)

Prevention:
  - Regular integrity checks
  - Monitor NVMe health
  - Use ECC RAM
```

### 15.2.3 OOM (Out of Memory)

```text
Symptoms:
  - Process killed by OOM killer
  - cnws_memory_working_size_bytes near budget
  - System logs show OOM events

Diagnosis:
  dmesg | grep -i oom
  cnws diag memory-status (if process running)

Resolution:
  1. Reduce cache budget: cnws config set cache.cpu_budget_bytes <lower>
  2. Reduce working memory: cnws config set budget.working_memory_bytes <lower>
  3. Add more RAM
  4. Use smaller model or fewer concurrent operations

Prevention:
  - Configure budgets conservatively
  - Monitor memory usage
  - Alert at 90% memory usage
```

### 15.2.4 GPU Failure

```text
Symptoms:
  - GPU operations fail
  - CUDA errors in logs
  - nvidia-smi shows errors

Diagnosis:
  nvidia-smi
  cnws diag gpu-status

Resolution:
  1. Check GPU health: nvidia-smi -q
  2. Reset GPU if possible: nvidia-smi --gpu-reset
  3. Fall back to CPU mode: cnws config set gpu.enabled false
  4. Replace GPU if hardware failure

Prevention:
  - Monitor GPU temperature
  - Monitor GPU utilization
  - Alert on GPU errors
```

### 15.2.5 Process Crash

```text
Symptoms:
  - CNWS process not running
  - systemd shows failed status

Diagnosis:
  systemctl status cnws
  journalctl -u cnws --since "1 hour ago"

Resolution:
  1. Check logs for crash reason
  2. Run recovery: cnws recover /data/model.cd
  3. Restart service: systemctl restart cnws
  4. Verify health: cnws diag health /data/model.cd

Prevention:
  - Configure systemd restart policy
  - Monitor process health
  - Regular backups
```

## 15.3 Systemd Service Configuration

`[OPS-FAIL-2]` Recommended systemd service:

```ini
# /etc/systemd/system/cnws.service

[Unit]
Description=CNWS Runtime Service
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=cnws
Group=cnws
WorkingDirectory=/data/model.cd
ExecStart=/usr/local/bin/cnws runtime serve /data/model.cd
ExecStop=/usr/local/bin/cnws runtime stop /data/model.cd

# Restart policy
Restart=on-failure
RestartSec=10
StartLimitBurst=5
StartLimitIntervalSec=60

# Resource limits
LimitNOFILE=1048576
LimitMEMLOCK=infinity

# Security hardening
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/data/model.cd
ReadWritePaths=/var/log/cnws

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=cnws

[Install]
WantedBy=multi-user.target
```

## 15.4 Operational Runbook

`[OPS-FAIL-3]` Runbook untuk common operations:

```bash
#!/bin/bash
# CNWS Operational Runbook

# === Daily Operations ===

# Check health
cnws diag health /data/model.cd

# Check disk space
df -h /data

# Check metrics
curl -s http://localhost:9090/metrics | grep cnws_

# === Weekly Operations ===

# Full backup
cnws backup create /data/model.cd --target /backup/full-$(date +%Y%m%d) --type full

# Integrity check
cnws diag integrity /data/model.cd

# GC (if needed)
cnws gc /data/model.cd --dry-run

# === Monthly Operations ===

# NVMe health check
sudo nvme smart-log /dev/nvme0n1

# Review logs
journalctl -u cnws --since "30 days ago" | grep -i error

# Review metrics trends
# (via Grafana)

# === Emergency Procedures ===

# Stop service
sudo systemctl stop cnws

# Emergency backup
cnws backup create /data/model.cd --target /backup/emergency-$(date +%s) --type full

# Restore from backup
cnws backup restore /backup/latest --target /data/model.cd

# Start service
sudo systemctl start cnws
```

## 15.5 Failure Handling Invariants

| ID | Invariant |
|---|---|
| OPS-FAIL-INV-1 | Failure MUST diklasifikasikan |
| OPS-FAIL-INV-2 | Runbook MUST terdokumentasi |
| OPS-FAIL-INV-3 | Service MUST memiliki restart policy |
| OPS-FAIL-INV-4 | Emergency procedures MUST terdefinisi |
| OPS-FAIL-INV-5 | Failure MUST logged dan alert |

---

# 16. Security Operations

## 16.1 Access Control

`[OPS-SEC-1]` File permissions:

```bash
# Store directory
chmod 750 /data/model.cd
chown cnws:cnws /data/model.cd

# Configuration files
chmod 640 /data/model.cd/cnws.toml
chmod 640 /etc/cnws/config.toml

# Log files
chmod 640 /var/log/cnws/cnws.log

# Backup files
chmod 600 /backup/*
```

## 16.2 User Configuration

```bash
# Create dedicated user
sudo useradd --system --shell /usr/sbin/nologin cnws

# Add to required groups
sudo usermod -aG video cnws  # For GPU access
sudo usermod -aG disk cnws   # For NVMe access (if needed)
```

## 16.3 Audit Logging

`[OPS-SEC-2]` Audit events MUST logged:

| Event | Logged |
|---|---|
| Store open/close | YES |
| Configuration changes | YES |
| Revision commits | YES |
| Backup/restore | YES |
| Security events | YES |
| User authentication | YES |

## 16.4 Security Operations Invariants

| ID | Invariant |
|---|---|
| OPS-SEC-INV-1 | Store MUST memiliki restricted permissions |
| OPS-SEC-INV-2 | Dedicated user SHOULD digunakan |
| OPS-SEC-INV-3 | Audit events MUST logged |
| OPS-SEC-INV-4 | Backup files MUST encrypted (jika remote) |
| OPS-SEC-INV-5 | Credentials MUST NOT disimpan plaintext |

---

# 17. Final Operations Contract

## 17.1 Ringkasan Keputusan Operations

| ID | Keputusan |
|---|---|
| OPS-F01 | Configuration menggunakan TOML. |
| OPS-F02 | Store init via `cnws init`. |
| OPS-F03 | Minimum RAM: 16 GB runtime, 8 GB conversion. |
| OPS-F04 | Filesystem: XFS atau ext4. |
| OPS-F05 | I/O scheduler: `none` atau `mq-deadline`. |
| OPS-F06 | GPU config via CNWS config. |
| OPS-F07 | Backup: full weekly + incremental daily. |
| OPS-F08 | Upgrade backward-compatible untuk minor. |
| OPS-F09 | Downgrade eksplisit dan logged. |
| OPS-F10 | Migration atomic. |
| OPS-F11 | Monitoring via OpenMetrics. |
| OPS-F12 | Health check endpoint tersedia. |
| OPS-F13 | Runbook terdokumentasi. |
| OPS-F14 | Remote storage S3-compatible. |
| OPS-F15 | Store dedicated filesystem. |
| OPS-F16 | Installation idempotent. |
| OPS-F17 | Store init idempotent. |
| OPS-F18 | Backup diverifikasi. |
| OPS-F19 | Restore diverifikasi. |
| OPS-F20 | Failure classified dan runbook tersedia. |

## 17.2 Operations Invariants

| ID | Invariant |
|---|---|
| OPS-INV-1 | Installation MUST idempotent. |
| OPS-INV-2 | Store init MUST membuat struktur lengkap. |
| OPS-INV-3 | Configuration MUST TOML. |
| OPS-INV-4 | Configuration MUST divalidasi. |
| OPS-INV-5 | GPU budget MUST hard-enforced. |
| OPS-INV-6 | Store MUST pada NVMe/SSD. |
| OPS-INV-7 | Upgrade MUST backward-compatible (minor). |
| OPS-INV-8 | Backup MUST diverifikasi. |
| OPS-INV-9 | Restore MUST diverifikasi. |
| OPS-INV-10 | Migration MUST atomic. |
| OPS-INV-11 | Monitoring endpoint MUST tersedia. |
| OPS-INV-12 | Health check MUST tersedia. |
| OPS-INV-13 | Failure MUST diklasifikasikan. |
| OPS-INV-14 | Runbook MUST terdokumentasi. |
| OPS-INV-15 | Security permissions MUST restricted. |
| OPS-INV-16 | Audit events MUST logged. |
| OPS-INV-17 | Service MUST memiliki restart policy. |
| OPS-INV-18 | Emergency procedures MUST terdefinisi. |
| OPS-INV-19 | Backup retention MUST dipatuhi. |
| OPS-INV-20 | Remote credentials MUST NOT plaintext. |

## 17.3 Operations Checklist

### Pre-Deployment Checklist

- [ ] Hardware requirements terpenuhi
- [ ] Software requirements terpenuhi
- [ ] NVMe filesystem configured (XFS/ext4)
- [ ] I/O scheduler configured
- [ ] CNWS installed dan verified
- [ ] Store initialized
- [ ] Configuration validated
- [ ] GPU detected dan configured
- [ ] Monitoring configured
- [ ] Backup strategy defined
- [ ] Runbook documented
- [ ] Security permissions set

### Post-Deployment Checklist

- [ ] Health check passing
- [ ] Metrics endpoint accessible
- [ ] First import successful
- [ ] Integrity check passing
- [ ] Backup created dan verified
- [ ] Monitoring alerts configured
- [ ] Logs accessible
- [ ] Documentation updated

## 17.4 Pernyataan Penutup

Dokumen ini adalah **spesifikasi Operations & Deployment final dan mengikat** untuk CNWS. Ia mendefinisikan bagaimana CNWS dijalankan sebagai sistem nyata, dari installation hingga failure handling, dari configuration hingga monitoring, dari backup hingga restore.

Operasional CNWS dirancang untuk **predictable, safe, dan recoverable**. Setiap operasi memiliki prosedur yang terdefinisi, setiap failure memiliki runbook, dan setiap aspek sistem dapat dimonitor.

Seluruh deployment CNWS MUST conformant terhadap spesifikasi ini.

Tidak ada keputusan operations yang tersisa sebagai open question. Seluruh keputusan telah ditetapkan final dalam dokumen ini.

**AKHIR DOKUMEN OPERATIONS & DEPLOYMENT SPECIFICATION**
