# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Jika Anda menemukan kerentanan keamanan, silakan laporkan dengan aman melalui:

1. **Email**: security@example.com
2. **GitHub Security Advisory**: Buat advisory privat di repository

Jangan buat issue publik untuk kerentanan keamanan.

## Security Considerations

CNWS dirancang dengan prinsip keamanan berikut:

### Content Addressing
- Semua identitas menggunakan BLAKE3-256 hash
- Collision resistance melalui cryptographic hash
- Tamper detection melalui hash verification

### Immutability
- Data tidak bisa diubah setelah ditulis
- Revision DAG mencegah modifikasi history
- Write-once semantics

### Access Control
- Store-level access control
- Memory type-based access policies
- Admin API untuk operasi sensitif

### Data Protection
- Compression untuk data at rest
- Integrity verification via BLAKE3-256
- Quarantine untuk tile korup

### Threat Model
Lihat [docs/specs/11-security-threat-model.md](docs/specs/11-security-threat-model.md) untuk analisis threat model lengkap.
