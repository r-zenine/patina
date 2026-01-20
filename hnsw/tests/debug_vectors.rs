use hnsw::{HnswIndex, HnswParams, persistence::BaseIndex, persistence::IndexWriter};
use tempfile::TempDir;

const GRAPH_CHUNK_SIZE: usize = 4096;
const VECTOR_CHUNK_SIZE: usize = 4096;

#[test]
fn debug_vector_serialization() {
    let temp_dir = TempDir::new().unwrap();
    let base_path = temp_dir.path().join("debug");

    let params = HnswParams::default();
    let mut index: HnswIndex<u32, 4, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> =
        HnswIndex::new_euclidean(params);

    let vector = [1.0, 2.0, 3.0, 4.0];
    let node_id = index.insert(42u32, vector).expect("Failed to insert");

    println!("Node ID: {:?}", node_id);
    println!("Vector: {:?}", vector);

    // Check offsets before serialization
    let offsets = index.get_vector_offsets().expect("Failed to get offsets");
    println!("Vector offsets before serialization: {:?}", offsets);
    if node_id.to_usize() < offsets.len() {
        println!("Vector offset for node: {}", offsets[node_id.to_usize()]);
    }

    // Check what's in the vector before serialization
    let stored_vec = index.get_node_vector(node_id);
    println!("Stored vector in memory: {:?}", stored_vec);

    // Serialize
    IndexWriter::write(&index, &base_path).expect("Failed to serialize");
    println!("Serialized to: {:?}", base_path);

    // Check file sizes
    let tape_size = std::fs::metadata(base_path.join("main.tape")).map(|m| m.len());
    let vectors_size = std::fs::metadata(base_path.join("main.vectors")).map(|m| m.len());
    let offsets_size = std::fs::metadata(base_path.join("main.offsets")).map(|m| m.len());
    println!(
        "File sizes - tape: {:?}, vectors: {:?}, offsets: {:?}",
        tape_size, vectors_size, offsets_size
    );

    // Load offsets file manually
    let offsets_path = base_path.join("main.offsets");
    let offsets_data = std::fs::read(&offsets_path).expect("Failed to read offsets");
    println!("Offsets file size: {} bytes", offsets_data.len());
    if offsets_data.len() >= 14 {
        let count = u64::from_le_bytes([
            offsets_data[6],
            offsets_data[7],
            offsets_data[8],
            offsets_data[9],
            offsets_data[10],
            offsets_data[11],
            offsets_data[12],
            offsets_data[13],
        ]) as usize;
        println!("Offset count: {}", count);
        for i in 0..count.min(3) {
            let pos = 14 + i * 8;
            if pos + 8 <= offsets_data.len() {
                let offset_bytes: [u8; 8] = offsets_data[pos..pos + 8].try_into().unwrap();
                let offset = usize::from_le_bytes(offset_bytes);
                println!("  Offset[{}]: {}", i, offset);
            }
        }
    }

    // Load and verify
    println!("\n--- Loading ---");
    let loaded_index = BaseIndex::open(&base_path).expect("Failed to load");
    println!(
        "Loaded index - entry point: {:?}, node_count: {}",
        loaded_index.entry_point(),
        loaded_index.node_count()
    );

    let loaded_offsets = loaded_index.offsets();
    println!("Loaded offsets: {:?}", loaded_offsets);

    let loaded_vec = loaded_index.get_vector::<4>(&node_id);
    println!("Loaded vector: {:?}", loaded_vec);
    if let Some(v) = loaded_vec {
        println!("Vector matches: {}", v == &vector);
    }
}
