# Contributing to CNWS

Terima kasih atas minat Anda untuk berkontribusi pada CNWS (Canonical Neural Weight System)!

## Prerequisites

- Rust 1.75.0 atau lebih baru
- Git
- Editor dengan Rust support (VS Code + rust-analyzer recommended)

## Setup Development Environment

```bash
# Clone repository
git clone https://github.com/example/cnws.git
cd cnws

# Install Rust toolchain
rustup toolchain install 1.75.0
rustup default 1.75.0
rustup component add rustfmt clippy

# Build workspace
cargo build --workspace

# Run tests
cargo test --workspace

# Run conformance tests
cargo run --bin cnws-conformance
```

## Development Workflow

1. **Buat branch baru** untuk setiap fitur atau bug fix:
   ```bash
   git checkout -b feature/my-new-feature
   # atau
   git checkout -b fix/bug-description
   ```

2. **Buat perubahan** dengan mengikuti coding standards:
   - Gunakan `cargo fmt` untuk formatting
   - Gunakan `cargo clippy` untuk linting
   - Tulis unit tests untuk setiap module baru
   - Update dokumentasi jika diperlukan

3. **Commit perubahan**:
   ```bash
   git add .
   git commit -m "feat: add new feature"
   ```

4. **Push dan buat Pull Request**:
   ```bash
   git push origin feature/my-new-feature
   ```

## Coding Standards

### Error Handling
- Gunakan `CnwsError` untuk semua error
- Setiap error harus memiliki error code (CNWS-E-*)
- Implementasikan `is_fatal()`, `is_recoverable()`, `is_transient()`

### Content Addressing
- Semua identitas harus menggunakan BLAKE3-256
- Gunakan `Blake3Hash` type untuk semua hash
- Implementasikan `hash()` dan `hash_streaming()`

### Serialization
- Gunakan `bincode` untuk binary serialization
- Gunakan `serde` untuk JSON serialization
- Semua struct harus implement `Serialize` dan `Deserialize`

### Testing
- Setiap module harus memiliki unit tests
- Integration tests di `tests/`
- Conformance tests di `cnws-conformance`
- Properties tests menggunakan `proptest`

### Documentation
- Dokumentasikan semua public API
- Gunakan `///` untuk documentation comments
- Include examples dalam dokumentasi

## Commit Message Convention

Gunakan [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` - Fitur baru
- `fix:` - Bug fix
- `docs:` - Dokumentasi
- `style:` - Formatting, missing semicolons, etc
- `refactor:` - Refactoring kode
- `perf:` - Performance improvements
- `test:` - Menambah tests
- `chore:` - Maintenance tasks

## Pull Request Process

1. Pastikan semua tests pass:
   ```bash
   cargo test --workspace
   cargo run --bin cnws-conformance
   ```

2. Pastikan clippy tidak ada warning:
   ```bash
   cargo clippy --workspace -- -D warnings
   ```

3. Pastikan kode terformat:
   ```bash
   cargo fmt --all
   ```

4. Update CHANGELOG.md jika diperlukan

5. Buat PR dengan deskripsi yang jelas

## Code Review

- Semua PR harus direview oleh minimal 1 maintainer
- Review berfokus pada:
  - Correctness
  - Performance
  - Security
  - Maintainability
  - Test coverage

## Questions?

Jika ada pertanyaan, silakan buat issue di GitHub atau hubungi maintainer.
