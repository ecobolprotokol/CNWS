# Contributing to CNWS Documentation

## How to Contribute

1. Fork repository
2. Buat branch baru: `git checkout -b docs/update-xxx`
3. Edit dokumentasi
4. Commit: `git commit -am 'docs: update xxx'`
5. Push: `git push origin docs/update-xxx`
6. Buat Pull Request

## Documentation Standards

- Gunakan Markdown untuk semua dokumen
- Sertakan contoh kode dimana relevan
- Update link jika memindahkan/rename file
- Test semua code examples

## File Structure

```
docs/
├── README.md              # Index dokumentasi
├── GETTING_STARTED.md     # Panduan mulai cepat
├── ARCHITECTURE.md        # Gambaran arsitektur
├── DESIGN.md              # Design decisions
├── API.md                 # API reference
├── CONTRIBUTING.md        # Panduan kontribusi
└── specs/                 # Spesifikasi teknis
    ├── 01-engineering-contract.md
    ├── 02-product-requirements.md
    └── ...
```
