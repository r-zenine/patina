use hnsw::{HnswIndex, HnswParams, persistence::BaseIndex, persistence::IndexWriter};
use tempfile::TempDir;

const GRAPH_CHUNK_SIZE: usize = 4096;
const VECTOR_CHUNK_SIZE: usize = 4096;

#[test]
fn test_serialize_and_load_index() {
    // Create a temporary directory for output
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("index");

    // Create and populate index
    let params = HnswParams::default();
    let max_connections = params.max_connections;
    let max_connections_level0 = params.max_connections_level0;
    let mut index: HnswIndex<u32, 4, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> =
        HnswIndex::new_euclidean(params);

    // Insert some test vectors
    let vectors = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    let mut node_ids = Vec::new();
    for (i, vec) in vectors.iter().enumerate() {
        let key = i as u32;
        let node_id = index.insert(key, *vec).expect("Failed to insert vector");
        node_ids.push(node_id);
    }

    // Verify index was populated
    assert_eq!(index.len(), 4, "Index should have 4 nodes");
    assert!(index.entry_point().is_some(), "Entry point should be set");

    // Serialize index to disk
    IndexWriter::write(&index, &base_path).expect("Failed to serialize index");

    // Verify files were created
    assert!(
        base_path.join("main.tape").exists(),
        "Graph tape file should exist"
    );
    assert!(
        base_path.join("main.vectors").exists(),
        "Vector tape file should exist"
    );
    assert!(
        base_path.join("main.offsets").exists(),
        "Offsets file should exist"
    );
    assert!(
        base_path.join("main.meta").exists(),
        "Metadata file should exist"
    );

    // Load index from disk
    let loaded_index = BaseIndex::open(&base_path).expect("Failed to load index");

    // Verify metadata
    assert_eq!(
        loaded_index.node_count(),
        4,
        "Loaded index should have 4 nodes"
    );
    assert!(
        loaded_index.entry_point().is_some(),
        "Loaded index should have entry point"
    );
    assert_eq!(
        loaded_index.max_connections(),
        max_connections,
        "Max connections should match"
    );
    assert_eq!(
        loaded_index.max_connections_level0(),
        max_connections_level0,
        "Max connections level0 should match"
    );

    // Verify offsets
    let offsets = loaded_index.offsets();
    assert!(!offsets.is_empty(), "Offsets should not be empty");

    // Verify we can read back a vector
    let vec0 = loaded_index.get_vector::<4>(&node_ids[0]);
    assert!(
        vec0.is_some(),
        "Should be able to read back first vector from mmap"
    );
    if let Some(v) = vec0 {
        assert_eq!(v[0], 1.0, "First element should be 1.0");
        assert_eq!(v[1], 0.0, "Second element should be 0.0");
    }

    // Verify we can read other vectors
    let vec1 = loaded_index.get_vector::<4>(&node_ids[1]);
    assert!(vec1.is_some(), "Should be able to read second vector");
    if let Some(v) = vec1 {
        assert_eq!(v[1], 1.0, "Second element should be 1.0");
    }

    println!("✓ Serialization and loading test passed");
}

#[test]
fn test_empty_index_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("empty_index");

    // Create empty index
    let params = HnswParams::default();
    let index: HnswIndex<u32, 4, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> =
        HnswIndex::new_euclidean(params);

    // Serialize empty index
    IndexWriter::write(&index, &base_path).expect("Failed to serialize empty index");

    // Load empty index
    let loaded_index = BaseIndex::open(&base_path).expect("Failed to load empty index");

    assert_eq!(
        loaded_index.node_count(),
        0,
        "Loaded empty index should have 0 nodes"
    );
    assert!(
        loaded_index.entry_point().is_none(),
        "Empty index should have no entry point"
    );

    println!("✓ Empty index serialization test passed");
}

#[test]
fn test_single_node_index() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("single_node");

    // Create index with single node
    let params = HnswParams::default();
    let mut index: HnswIndex<u32, 4, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> =
        HnswIndex::new_euclidean(params);

    let vector = [1.0, 2.0, 3.0, 4.0];
    let node_id = index
        .insert(42u32, vector)
        .expect("Failed to insert vector");

    // Serialize
    IndexWriter::write(&index, &base_path).expect("Failed to serialize");

    // Load
    let loaded_index = BaseIndex::open(&base_path).expect("Failed to load");

    assert_eq!(loaded_index.node_count(), 1, "Should have 1 node");
    assert_eq!(
        loaded_index.entry_point(),
        Some(node_id),
        "Entry point should be the single node"
    );

    // Verify vector
    let loaded_vec = loaded_index.get_vector::<4>(&node_id);
    assert!(loaded_vec.is_some(), "Should be able to read vector");
    if let Some(v) = loaded_vec {
        assert_eq!(v, &vector, "Vector should match original");
    }

    println!("✓ Single node index test passed");
}

#[test]
fn test_vector_persistence_round_trip() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("vectors");

    let params = HnswParams::default();
    let mut index: HnswIndex<u32, 4, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> =
        HnswIndex::new_euclidean(params);

    // Create and insert vectors with specific values
    let vectors = [
        [0.5, 0.5, 0.5, 0.5],
        [1.0, 2.0, 3.0, 4.0],
        [-1.0, -2.0, -3.0, -4.0],
        [0.1, 0.2, 0.3, 0.4],
    ];

    let mut node_ids = Vec::new();
    for (i, vec) in vectors.iter().enumerate() {
        let key = (1000 + i) as u32;
        let node_id = index.insert(key, *vec).expect("Failed to insert");
        node_ids.push(node_id);
    }

    // Serialize
    IndexWriter::write(&index, &base_path).expect("Failed to serialize");

    // Load
    let loaded_index = BaseIndex::open(&base_path).expect("Failed to load");

    // Verify all vectors match
    for (i, &node_id) in node_ids.iter().enumerate() {
        let loaded_vec = loaded_index
            .get_vector::<4>(&node_id)
            .unwrap_or_else(|| panic!("Should be able to read vector {i}"));
        assert_eq!(loaded_vec, &vectors[i], "Vector {i} should match");
    }

    println!("✓ Vector persistence round-trip test passed");
}
