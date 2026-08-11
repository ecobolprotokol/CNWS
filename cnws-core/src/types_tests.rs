//! Unit tests for Phase 1 Core Types
//! Tests Blake3Hash, CellType, DataType, Cell, Tile, and reference types

#[cfg(test)]
mod tests {
    use crate::types::*;

    // ============================================================================
    // Blake3Hash Tests
    // ============================================================================

    #[test]
    fn test_blake3hash_compute() {
        let data = b"hello world";
        let hash = Blake3Hash::hash(data);
        
        // Verify hash is 32 bytes
        assert_eq!(hash.as_bytes().len(), 32);
        
        // Same data should produce same hash
        let hash2 = Blake3Hash::hash(data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_blake3hash_hex_roundtrip() {
        let data = b"test data";
        let hash1 = Blake3Hash::hash(data);
        let hex = hash1.to_hex();
        
        // Should convert back correctly
        let hash2 = Blake3Hash::from_hex(&hex).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3hash_display() {
        let hash = Blake3Hash::default();
        let display = format!("{}", hash);
        assert_eq!(display.len(), 64); // 32 bytes * 2 hex chars per byte
    }

    // ============================================================================
    // CellType Tests
    // ============================================================================

    #[test]
    fn test_celltype_conversions() {
        // Test round-trip conversion for all cell types
        let types = [
            CellType::Tensor,
            CellType::Attention,
            CellType::FFN,
            CellType::Weight,
            CellType::Custom,
        ];
        
        for cell_type in types {
            let byte_val = u8::from(cell_type);
            let recovered = CellType::try_from(byte_val).unwrap();
            assert_eq!(cell_type, recovered);
        }
    }

    // ============================================================================
    // DataType Tests
    // ============================================================================

    #[test]
    fn test_datatype_size() {
        assert_eq!(DataType::F32.size(), 4);
        assert_eq!(DataType::F16.size(), 2);
        assert_eq!(DataType::I8.size(), 1);
        assert_eq!(DataType::I64.size(), 8);
        assert_eq!(DataType::Bool.size(), 1);
    }

    #[test]
    fn test_datatype_conversions() {
        let types = [
            DataType::F32,
            DataType::F16,
            DataType::I8,
            DataType::I64,
            DataType::Bool,
        ];
        
        for dt in types {
            let byte_val = u8::from(dt);
            let recovered = DataType::try_from(byte_val).unwrap();
            assert_eq!(dt, recovered);
        }
    }

    // ============================================================================
    // Cell Tests
    // ============================================================================

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new(CellType::Tensor, DataType::F32, vec![10, 20, 30]);
        
        assert_eq!(cell.cell_type, CellType::Tensor);
        assert_eq!(cell.data_type, DataType::F32);
        assert_eq!(cell.num_elements, 10 * 20 * 30);
        assert_eq!(cell.uncompressed_size, 10 * 20 * 30 * 4); // F32 = 4 bytes
        assert_eq!(cell.compression, Compression::None);
    }

    #[test]
    fn test_cell_with_compression() {
        let mut cell = Cell::new(CellType::Weight, DataType::F16, vec![100, 100]);
        cell = cell.with_compression(Compression::Zstd, 5000);
        
        assert_eq!(cell.compression, Compression::Zstd);
        assert_eq!(cell.compressed_size, 5000);
    }

    #[test]
    fn test_cell_add_children() {
        let mut cell = Cell::new(CellType::Merge, DataType::I32, vec![1]);
        let child1 = Blake3Hash::default();
        let child2 = Blake3Hash::hash(b"child2");
        
        cell.add_child(child1);
        cell.add_child(child2);
        
        assert_eq!(cell.children.len(), 2);
        assert_eq!(cell.children[0], child1);
        assert_eq!(cell.children[1], child2);
    }

    #[test]
    fn test_cell_compute_id() {
        let mut cell = Cell::new(CellType::Tensor, DataType::F32, vec![10]);
        let id = cell.compute_id().unwrap();
        
        // ID should be set
        assert_ne!(id, Blake3Hash::default());
        assert_eq!(cell.id, id);
    }

    #[test]
    fn test_cell_metadata() {
        let mut cell = Cell::new(CellType::Tensor, DataType::F32, vec![10]);
        cell.metadata.insert("layer".to_string(), "attention_0".to_string());
        cell.metadata.insert("model".to_string(), "llama-7b".to_string());
        
        assert_eq!(cell.metadata.len(), 2);
        assert_eq!(cell.metadata.get("layer"), Some(&"attention_0".to_string()));
    }

    // ============================================================================
    // Tile Tests
    // ============================================================================

    #[test]
    fn test_tile_creation() {
        let location = TileLocation {
            segment_idx: 0,
            tile_offset: 100,
            byte_offset: 4_194_304,
            size: 4_194_304,
            compression: Compression::None,
        };
        
        let tile = Tile::new(location);
        assert_eq!(tile.location, location);
        assert_eq!(tile.size, 0);
        assert_eq!(tile.dedup_count, 1);
    }

    #[test]
    fn test_tile_add_cells() {
        let location = TileLocation {
            segment_idx: 0,
            tile_offset: 0,
            byte_offset: 0,
            size: 0,
            compression: Compression::None,
        };
        let mut tile = Tile::new(location);
        
        let cell1 = Blake3Hash::default();
        let cell2 = Blake3Hash::hash(b"cell2");
        
        tile.add_cell(cell1);
        tile.add_cell(cell2);
        
        assert_eq!(tile.cell_ids.len(), 2);
    }

    #[test]
    fn test_tile_compute_id() {
        let location = TileLocation {
            segment_idx: 0,
            tile_offset: 0,
            byte_offset: 0,
            size: 0,
            compression: Compression::None,
        };
        let mut tile = Tile::new(location);
        
        let data = b"tile data";
        let id = tile.compute_id(data).unwrap();
        
        assert_ne!(id, Blake3Hash::default());
        assert_eq!(tile.size, data.len() as u64);
    }

    // ============================================================================
    // CellRef Tests
    // ============================================================================

    #[test]
    fn test_cellref_creation() {
        let id = Blake3Hash::hash(b"cell");
        let cell_ref = CellRef::new(id);
        
        assert_eq!(cell_ref.id, id);
        assert_eq!(cell_ref.tile_location, None);
    }

    #[test]
    fn test_cellref_with_location() {
        let id = Blake3Hash::hash(b"cell");
        let location = TileLocation {
            segment_idx: 1,
            tile_offset: 50,
            byte_offset: 10_485_760,
            size: 4_194_304,
            compression: Compression::None,
        };
        
        let cell_ref = CellRef::with_location(id, location);
        
        assert_eq!(cell_ref.id, id);
        assert_eq!(cell_ref.tile_location, Some(location));
    }

    // ============================================================================
    // TileRef Tests
    // ============================================================================

    #[test]
    fn test_tileref_creation() {
        let id = Blake3Hash::hash(b"tile");
        let location = TileLocation {
            segment_idx: 0,
            tile_offset: 0,
            byte_offset: 0,
            size: 4_194_304,
            compression: Compression::None,
        };
        
        let tile_ref = TileRef::new(id, location, 4_194_304);
        
        assert_eq!(tile_ref.id, id);
        assert_eq!(tile_ref.location, location);
        assert_eq!(tile_ref.size, 4_194_304);
    }

    // ============================================================================
    // IndexVector Tests
    // ============================================================================

    #[test]
    fn test_indexvector_creation() {
        let idx_vec = IndexVector::new(768); // Common transformer dimension
        
        assert_eq!(idx_vec.dimensions, 768);
        assert_eq!(idx_vec.values.len(), 0);
        assert_eq!(idx_vec.norm, 0.0);
    }

    #[test]
    fn test_indexvector_add_entries() {
        let mut idx_vec = IndexVector::new(768);
        
        idx_vec.add_entry(0, vec![1, 0, 0, 0]);
        idx_vec.add_entry(5, vec![0, 1, 0, 0]);
        idx_vec.add_entry(10, vec![0, 0, 1, 0]);
        
        assert_eq!(idx_vec.values.len(), 3);
        assert_eq!(idx_vec.values[0].index, 0);
        assert_eq!(idx_vec.values[1].index, 5);
        assert_eq!(idx_vec.values[2].index, 10);
    }

    // ============================================================================
    // Metadata Tests
    // ============================================================================

    #[test]
    fn test_metadata_creation() {
        let metadata = Metadata::new("user@example.com".to_string());
        
        assert_eq!(metadata.owner, "user@example.com");
        assert_eq!(metadata.version, Version::current());
        assert!(metadata.created_at > 0);
        assert_eq!(metadata.provenance, None);
    }

    #[test]
    fn test_metadata_with_provenance() {
        let mut metadata = Metadata::new("owner".to_string());
        metadata.provenance = Some(Provenance {
            source_model: "llama-7b".to_string(),
            import_format: "safetensors".to_string(),
            import_timestamp: 1690000000,
            revision: "v1.0".to_string(),
        });
        
        let prov = metadata.provenance.as_ref().unwrap();
        assert_eq!(prov.source_model, "llama-7b");
        assert_eq!(prov.import_format, "safetensors");
    }

    // ============================================================================
    // Version Tests
    // ============================================================================

    #[test]
    fn test_version_display() {
        let version = Version::new(1, 2, 3);
        assert_eq!(format!("{}", version), "1.2.3");
    }

    #[test]
    fn test_version_current() {
        let current = Version::current();
        assert_eq!(current.major, 1);
        assert_eq!(current.minor, 0);
        assert_eq!(current.patch, 0);
    }

    // ============================================================================
    // Constants Tests
    // ============================================================================

    #[test]
    fn test_constants() {
        assert_eq!(TILE_SIZE, 4 * 1024 * 1024); // 4 MB
        assert_eq!(SUPERBLOCK_SIZE, 4096);
        assert_eq!(SEGMENT_HEADER_SIZE, 4096);
        assert_eq!(MEMORY_INDEX_ENTRY_SIZE, 104);
    }

    #[test]
    fn test_magic_bytes() {
        assert_eq!(SUPERBLOCK_MAGIC, b"CNWSSB01");
        assert_eq!(SEGMENT_MAGIC, b"CNWSSEG1");
        assert_eq!(INDEX_MAGIC, b"CNWSIDX1");
        assert_eq!(MEMORY_MAGIC, b"CNWSMEM1");
        assert_eq!(REVISION_MAGIC, b"CNWSREV1");
        assert_eq!(MANIFEST_MAGIC, b"CNWSMAN1");
    }

    // ============================================================================
    // Compression Tests
    // ============================================================================

    #[test]
    fn test_compression_conversions() {
        let types = [
            Compression::None,
            Compression::Zstd,
            Compression::Lz4,
            Compression::Brotli,
        ];
        
        for comp in types {
            let byte_val = u8::from(comp);
            let recovered = Compression::try_from(byte_val).unwrap();
            assert_eq!(comp, recovered);
        }
    }

    // ============================================================================
    // MemoryType Tests
    // ============================================================================

    #[test]
    fn test_memorytype_conversions() {
        let types = [
            MemoryType::Episodic,
            MemoryType::Semantic,
            MemoryType::Procedural,
            MemoryType::Working,
            MemoryType::LongTerm,
        ];
        
        for mt in types {
            let byte_val = u8::from(mt);
            let recovered = MemoryType::try_from(byte_val).unwrap();
            assert_eq!(mt, recovered);
        }
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_blake3hash_serde() {
        let hash = Blake3Hash::hash(b"test");
        let json = serde_json::to_string(&hash).unwrap();
        let recovered: Blake3Hash = serde_json::from_str(&json).unwrap();
        assert_eq!(hash, recovered);
    }

    #[test]
    fn test_cell_serde() {
        let mut cell = Cell::new(CellType::Tensor, DataType::F32, vec![10, 20]);
        cell.compute_id().unwrap();
        
        let json = serde_json::to_string(&cell).unwrap();
        let recovered: Cell = serde_json::from_str(&json).unwrap();
        
        assert_eq!(cell.cell_type, recovered.cell_type);
        assert_eq!(cell.data_type, recovered.data_type);
        assert_eq!(cell.shape, recovered.shape);
    }

    #[test]
    fn test_tile_serde() {
        let location = TileLocation {
            segment_idx: 1,
            tile_offset: 50,
            byte_offset: 1024,
            size: 4096,
            compression: Compression::None,
        };
        let tile = Tile::new(location);
        
        let json = serde_json::to_string(&tile).unwrap();
        let recovered: Tile = serde_json::from_str(&json).unwrap();
        
        assert_eq!(tile.location, recovered.location);
        assert_eq!(tile.cell_ids, recovered.cell_ids);
    }
}
