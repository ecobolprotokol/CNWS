//! Unit tests for Core Types
//! Tests Blake3Hash, CellType, DataType, Cell, Tile, and reference types
//!
//! Spec Ref: 05-cell-schema.md

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
        assert_eq!(hash.as_bytes().len(), 32);
        let hash2 = Blake3Hash::hash(data);
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_blake3hash_hex_roundtrip() {
        let data = b"test data";
        let hash1 = Blake3Hash::hash(data);
        let hex = hash1.to_hex();
        let hash2 = Blake3Hash::from_hex(&hex).unwrap();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_blake3hash_display() {
        let hash = Blake3Hash::default();
        let display = format!("{}", hash);
        assert_eq!(display.len(), 64);
    }

    #[test]
    fn test_blake3hash_different_inputs() {
        let h1 = Blake3Hash::hash(b"foo");
        let h2 = Blake3Hash::hash(b"bar");
        assert_ne!(h1, h2);
    }

    // ============================================================================
    // CellType Tests (spec §3 - all 57 types)
    // ============================================================================

    #[test]
    fn test_celltype_weight_category() {
        let weight_types = [
            CellType::Embedding,
            CellType::AttentionQProj,
            CellType::AttentionKProj,
            CellType::AttentionVProj,
            CellType::AttentionOut,
            CellType::MlpGate,
            CellType::MlpUp,
            CellType::MlpDown,
            CellType::ExpertGate,
            CellType::ExpertRoute,
            CellType::ExpertWeight,
            CellType::LayerNormWeight,
            CellType::LayerNormBias,
            CellType::LmHead,
            CellType::VisionEncoder,
            CellType::ConvWeight,
            CellType::NormScale,
            CellType::NormBias,
            CellType::Positional,
            CellType::ResidualGate,
        ];
        for ct in weight_types {
            assert!(ct.is_weight(), "Expected weight type: {:?}", ct);
            assert_eq!(ct.category(), CellTypeCategory::Weight);
        }
    }

    #[test]
    fn test_celltype_memory_category() {
        let memory_types = [
            CellType::MemoryEpisodic,
            CellType::MemorySemantic,
            CellType::MemoryProcedural,
            CellType::MemoryWorking,
            CellType::MemoryConsolidated,
            CellType::MemoryAssociation,
        ];
        for ct in memory_types {
            assert!(ct.is_memory(), "Expected memory type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_routing_category() {
        let routing_types = [
            CellType::RoutingPolicy,
            CellType::RoutingStatistics,
            CellType::RoutingIndex,
            CellType::RoutingAssociation,
            CellType::RoutingThreshold,
        ];
        for ct in routing_types {
            assert!(ct.is_routing(), "Expected routing type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_composition_category() {
        let composition_types = [
            CellType::CompositionPattern,
            CellType::CompositionTemplate,
            CellType::CompositionMacro,
            CellType::CompositionSequence,
            CellType::CompositionParallel,
            CellType::CompositionConditional,
            CellType::CompositionIterative,
        ];
        for ct in composition_types {
            assert!(ct.is_composition(), "Expected composition type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_computation_category() {
        let computation_types = [
            CellType::TransformModule,
            CellType::EncodeModule,
            CellType::DecodeModule,
            CellType::NormalizeModule,
            CellType::ActivationModule,
            CellType::PoolingModule,
            CellType::AttentionModule,
            CellType::ConvolutionModule,
            CellType::RecurrentModule,
        ];
        for ct in computation_types {
            assert!(ct.is_computation(), "Expected computation type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_control_category() {
        let control_types = [
            CellType::HaltCondition,
            CellType::BudgetPolicy,
            CellType::BranchCondition,
            CellType::LoopControl,
            CellType::ErrorHandler,
        ];
        for ct in control_types {
            assert!(ct.is_control(), "Expected control type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_meta_category() {
        let meta_types = [
            CellType::Provenance,
            CellType::Configuration,
            CellType::Statistics,
            CellType::Annotation,
            CellType::Validation,
        ];
        for ct in meta_types {
            assert!(ct.is_meta(), "Expected meta type: {:?}", ct);
        }
    }

    #[test]
    fn test_celltype_custom_category() {
        assert!(CellType::Custom.is_meta() == false);
        assert_eq!(CellType::Custom.category(), CellTypeCategory::Custom);
    }

    #[test]
    fn test_celltype_roundtrip_all() {
        let all_types = [
            CellType::Embedding, CellType::AttentionQProj, CellType::AttentionKProj,
            CellType::AttentionVProj, CellType::AttentionOut,
            CellType::MlpGate, CellType::MlpUp, CellType::MlpDown,
            CellType::ExpertGate, CellType::ExpertRoute, CellType::ExpertWeight,
            CellType::LayerNormWeight, CellType::LayerNormBias, CellType::LmHead,
            CellType::VisionEncoder, CellType::ConvWeight,
            CellType::NormScale, CellType::NormBias, CellType::Positional, CellType::ResidualGate,
            CellType::MemoryEpisodic, CellType::MemorySemantic, CellType::MemoryProcedural,
            CellType::MemoryWorking, CellType::MemoryConsolidated, CellType::MemoryAssociation,
            CellType::RoutingPolicy, CellType::RoutingStatistics, CellType::RoutingIndex,
            CellType::RoutingAssociation, CellType::RoutingThreshold,
            CellType::CompositionPattern, CellType::CompositionTemplate, CellType::CompositionMacro,
            CellType::CompositionSequence, CellType::CompositionParallel,
            CellType::CompositionConditional, CellType::CompositionIterative,
            CellType::TransformModule, CellType::EncodeModule, CellType::DecodeModule,
            CellType::NormalizeModule, CellType::ActivationModule, CellType::PoolingModule,
            CellType::AttentionModule, CellType::ConvolutionModule, CellType::RecurrentModule,
            CellType::HaltCondition, CellType::BudgetPolicy, CellType::BranchCondition,
            CellType::LoopControl, CellType::ErrorHandler,
            CellType::Provenance, CellType::Configuration, CellType::Statistics,
            CellType::Annotation, CellType::Validation,
            CellType::Custom,
        ];

        for ct in all_types {
            let byte_val = u8::from(ct);
            let recovered = CellType::try_from(byte_val).unwrap();
            assert_eq!(ct, recovered, "Roundtrip failed for {:?}", ct);
            assert_eq!(ct.name(), recovered.name());
        }
    }

    #[test]
    fn test_celltype_reserved_is_rejected() {
        let reserved = [0x15, 0x16, 0x1A, 0x1F, 0x26, 0x35, 0x47, 0x59, 0x65, 0x75, 0x80, 0xAA, 0xFE];
        for val in reserved {
            assert!(CellType::try_from(val).is_err(), "Expected error for reserved: 0x{:02x}", val);
        }
    }

    #[test]
    fn test_celltype_name() {
        assert_eq!(CellType::Embedding.name(), "EMBEDDING");
        assert_eq!(CellType::LmHead.name(), "LM_HEAD");
        assert_eq!(CellType::Custom.name(), "CUSTOM");
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
    fn test_datatype_is_float() {
        assert!(DataType::F32.is_float());
        assert!(DataType::F16.is_float());
        assert!(DataType::BF16.is_float());
        assert!(!DataType::I32.is_float());
    }

    #[test]
    fn test_datatype_is_integer() {
        assert!(DataType::I32.is_integer());
        assert!(DataType::U64.is_integer());
        assert!(!DataType::F32.is_integer());
    }

    #[test]
    fn test_datatype_widening() {
        assert!(DataType::F16.can_widen_to(&DataType::F32));
        assert!(DataType::BF16.can_widen_to(&DataType::F32));
        assert!(DataType::I8.can_widen_to(&DataType::I32));
        assert!(DataType::I32.can_widen_to(&DataType::I64));
        assert!(!DataType::F32.can_widen_to(&DataType::I32));
    }

    #[test]
    fn test_datatype_conversions() {
        let types = [
            DataType::F32, DataType::F16, DataType::I8, DataType::I64, DataType::Bool,
        ];
        for dt in types {
            let byte_val = u8::from(dt);
            let recovered = DataType::try_from(byte_val).unwrap();
            assert_eq!(dt, recovered);
        }
    }

    // ============================================================================
    // Schema Tests (spec §4)
    // ============================================================================

    #[test]
    fn test_schema_empty() {
        let schema = Schema::empty();
        assert_eq!(schema.kind, SchemaKind::Empty);
    }

    #[test]
    fn test_schema_tensor() {
        let schema = Schema::tensor(vec![4096, 4096], DataType::F16);
        assert_eq!(schema.kind, SchemaKind::Tensor);
        let ts = schema.tensor.as_ref().unwrap();
        assert_eq!(ts.shape, vec![4096, 4096]);
        assert_eq!(ts.dtype, DataType::F16);
        assert_eq!(ts.layout, TensorLayout::RowMajor);
    }

    #[test]
    fn test_schema_scalar() {
        let schema = Schema::scalar(DataType::F32);
        assert_eq!(schema.kind, SchemaKind::Scalar);
    }

    #[test]
    fn test_schema_compatibility() {
        let a = Schema::tensor(vec![10], DataType::F32);
        let b = Schema::tensor(vec![10], DataType::F32);
        assert!(a.is_compatible_with(&b));

        let c = Schema::tensor(vec![10], DataType::F16);
        assert!(c.is_compatible_with(&a)); // F16 can widen to F32

        let d = Schema::scalar(DataType::F32);
        assert!(!a.is_compatible_with(&d)); // Different kinds
    }

    // ============================================================================
    // Dependency Tests (spec §5)
    // ============================================================================

    #[test]
    fn test_dependency_types() {
        let hash = Blake3Hash::hash(b"target");
        let d = Dependency::data(hash);
        assert!(d.is_hard());
        assert_eq!(d.dep_type, DependencyType::Data);

        let d = Dependency::control(hash);
        assert!(d.is_hard());
        assert_eq!(d.dep_type, DependencyType::Control);

        let d = Dependency::execution_order(hash);
        assert!(d.is_hard());
        assert_eq!(d.dep_type, DependencyType::ExecutionOrder);

        let d = Dependency::prefetch_hint(hash);
        assert!(!d.is_hard());
        assert_eq!(d.dep_type, DependencyType::PrefetchHint);

        let d = Dependency::semantic(hash);
        assert!(!d.is_hard());
        assert_eq!(d.dep_type, DependencyType::Semantic);
    }

    #[test]
    fn test_dependency_metadata() {
        let hash = Blake3Hash::hash(b"target");
        let mut d = Dependency::data(hash);
        d.metadata.strength = 0.5;
        d.metadata.conditional = true;
        d.metadata.condition = Some("x > 0".to_string());
        assert_eq!(d.metadata.strength, 0.5);
    }

    // ============================================================================
    // Cell Lifecycle Tests (spec §7)
    // ============================================================================

    #[test]
    fn test_cell_lifecycle() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![10]);
        assert!(cell.is_live());

        cell.cell_metadata.lifecycle = CellLifecycle::Deprecated;
        assert!(cell.is_deprecated());

        cell.cell_metadata.lifecycle = CellLifecycle::Tombstone;
        assert!(cell.is_tombstone());
    }

    // ============================================================================
    // Cell Tests
    // ============================================================================

    #[test]
    fn test_cell_creation() {
        let cell = Cell::new(CellType::Embedding, DataType::F32, vec![10, 20, 30]);
        assert_eq!(cell.cell_type, CellType::Embedding);
        assert_eq!(cell.data_type, DataType::F32);
        assert_eq!(cell.num_elements, 10 * 20 * 30);
        assert_eq!(cell.uncompressed_size, 10 * 20 * 30 * 4);
        assert_eq!(cell.compression, Compression::None);
        assert!(cell.is_live());
    }

    #[test]
    fn test_cell_with_compression() {
        let mut cell = Cell::new(CellType::ExpertWeight, DataType::F16, vec![100, 100]);
        cell = cell.with_compression(Compression::Zstd, 5000);
        assert_eq!(cell.compression, Compression::Zstd);
        assert_eq!(cell.compressed_size, 5000);
        assert!(cell.compression_ratio() > 0.0);
    }

    #[test]
    fn test_cell_add_children() {
        let mut cell = Cell::new(CellType::CompositionSequence, DataType::I32, vec![1]);
        let child1 = Blake3Hash::default();
        let child2 = Blake3Hash::hash(b"child2");
        cell.add_child(child1);
        cell.add_child(child2);
        assert_eq!(cell.children.len(), 2);
    }

    #[test]
    fn test_cell_add_dependencies() {
        let mut cell = Cell::new(CellType::AttentionQProj, DataType::F32, vec![4096, 4096]);
        let target = Blake3Hash::hash(b"embedding");
        cell.add_dependency(Dependency::data(target));
        cell.add_dependency(Dependency::execution_order(Blake3Hash::hash(b"k_proj")));

        assert_eq!(cell.dependencies.len(), 2);
        assert_eq!(cell.hard_dependencies().len(), 2);
    }

    #[test]
    fn test_cell_prefetch_hints() {
        let mut cell = Cell::new(CellType::ExpertWeight, DataType::F32, vec![10]);
        cell.add_dependency(Dependency::prefetch_hint(Blake3Hash::hash(b"expert8")));
        assert_eq!(cell.prefetch_hints().len(), 1);
    }

    #[test]
    fn test_cell_tiles() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![100, 100]);
        let tile = Blake3Hash::hash(b"tile");
        cell.add_tile(tile);
        assert_eq!(cell.tiles.len(), 1);
    }

    #[test]
    fn test_cell_representations() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![100, 100]);
        cell.add_representation(RepresentationRef {
            hash: Blake3Hash::hash(b"f16_repr"),
            dtype: DataType::F16,
            shape: vec![100, 100],
            compression: Compression::Zstd,
            size: 20000,
        });
        assert_eq!(cell.representations.len(), 1);
    }

    #[test]
    fn test_cell_compute_id() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![10]);
        let id = cell.compute_id().unwrap();
        assert_ne!(id, Blake3Hash::default());
        assert_eq!(cell.id, id);
    }

    #[test]
    fn test_cell_metadata_custom() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![10]);
        cell.metadata.insert("layer".to_string(), "attention_0".to_string());
        assert_eq!(cell.metadata.len(), 1);
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
    fn test_tile_compute_id() {
        let location = TileLocation {
            segment_idx: 0, tile_offset: 0, byte_offset: 0, size: 0,
            compression: Compression::None,
        };
        let mut tile = Tile::new(location);
        let data = b"tile data";
        let id = tile.compute_id(data).unwrap();
        assert_ne!(id, Blake3Hash::default());
        assert_eq!(tile.size, data.len() as u64);
    }

    // ============================================================================
    // IndexVector Tests
    // ============================================================================

    #[test]
    fn test_indexvector_creation() {
        let idx_vec = IndexVector::new(DEFAULT_INDEX_DIMENSIONS);
        assert_eq!(idx_vec.dimensions, DEFAULT_INDEX_DIMENSIONS);
        assert!(idx_vec.is_empty());
    }

    #[test]
    fn test_indexvector_similarity() {
        let mut a = IndexVector::new(512);
        a.add_entry(0, vec![1, 0, 0]);
        a.add_entry(1, vec![0, 1, 0]);
        a.norm = 1.0;

        let mut b = IndexVector::new(512);
        b.add_entry(0, vec![1, 0, 0]);
        b.add_entry(1, vec![0, 1, 0]);
        b.norm = 1.0;

        let sim = a.cosine_similarity(&b);
        assert!(sim > 0.0);
    }

    // ============================================================================
    // TensorPatterns Tests
    // ============================================================================

    #[test]
    fn test_tensor_patterns_infer() {
        assert_eq!(TensorPatterns::infer_cell_type("model.embed_tokens.weight"), CellType::Embedding);
        assert_eq!(TensorPatterns::infer_cell_type("model.layer.0.self_attn.q_proj.weight"), CellType::AttentionQProj);
        assert_eq!(TensorPatterns::infer_cell_type("model.layer.0.self_attn.k_proj.weight"), CellType::AttentionKProj);
        assert_eq!(TensorPatterns::infer_cell_type("model.layer.0.mlp.gate_proj.weight"), CellType::MlpGate);
        assert_eq!(TensorPatterns::infer_cell_type("lm_head.weight"), CellType::LmHead);
        assert_eq!(TensorPatterns::infer_cell_type("model.layer.0.input_layernorm.weight"), CellType::LayerNormWeight);
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_cell_serde() {
        let mut cell = Cell::new(CellType::Embedding, DataType::F32, vec![10, 20]);
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
            segment_idx: 1, tile_offset: 50, byte_offset: 1024, size: 4096,
            compression: Compression::None,
        };
        let tile = Tile::new(location);
        let json = serde_json::to_string(&tile).unwrap();
        let recovered: Tile = serde_json::from_str(&json).unwrap();
        assert_eq!(tile.location, recovered.location);
    }

    // ============================================================================
    // Constants Tests
    // ============================================================================

    #[test]
    fn test_constants() {
        assert_eq!(TILE_SIZE, 4 * 1024 * 1024);
        assert_eq!(SUPERBLOCK_SIZE, 4096);
        assert_eq!(SEGMENT_HEADER_SIZE, 4096);
        assert_eq!(MEMORY_INDEX_ENTRY_SIZE, 104);
        assert_eq!(DEFAULT_INDEX_DIMENSIONS, 512);
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
}
