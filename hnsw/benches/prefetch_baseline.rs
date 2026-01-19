use std::fs::File;
use std::hint::black_box;
use std::io::{self, BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use hnsw_rs::{HnswIndex, HnswParams, select_best_euclidean};

const GRAPH_CHUNK_SIZE: usize = 4096;
const VECTOR_CHUNK_SIZE: usize = 4096;
const DATA_DIR: &str = "/Users/ryad/workspace/hnsw-rs/data/sift";
const BASE_FILE: &str = "sift_base.fvecs";
const QUERY_FILE: &str = "sift_query.fvecs";

const DIM: usize = 128;
const SEED_COUNT: usize = 4_000;
const INSERT_SAMPLE_SIZE: usize = 200;
const SEARCH_SAMPLE_SIZE: usize = 200;
const SEARCH_K: usize = 10;
const SEARCH_EF: usize = 200;

fn load_fvecs_array<const D: usize>(path: &Path, limit: usize) -> Result<Vec<[f32; D]>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut reader = BufReader::new(file);
    let mut vectors = Vec::new();
    let mut dim_buf = [0u8; 4];

    loop {
        match reader.read_exact(&mut dim_buf) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e).with_context(|| format!("read dim from {}", path.display())),
        }

        let dim = i32::from_le_bytes(dim_buf) as usize;
        if dim != D {
            return Err(anyhow!(
                "dimension mismatch in {}: expected {}, got {}",
                path.display(),
                D,
                dim
            ));
        }

        let mut raw = vec![0u8; D * 4];
        reader
            .read_exact(&mut raw)
            .with_context(|| format!("read payload from {}", path.display()))?;

        let mut vector = [0f32; D];
        for (i, chunk) in raw.chunks_exact(4).enumerate() {
            vector[i] = f32::from_le_bytes(chunk.try_into().unwrap());
        }
        vectors.push(vector);

        if vectors.len() == limit {
            break;
        }
    }

    Ok(vectors)
}

fn load_sift_data() -> Result<(Vec<[f32; DIM]>, Vec<[f32; DIM]>)> {
    let base_path = Path::new(DATA_DIR).join(BASE_FILE);
    let query_path = Path::new(DATA_DIR).join(QUERY_FILE);

    if !base_path.exists() || !query_path.exists() {
        anyhow::bail!("SIFT files not found. Run scripts/fetch_sift.sh first.");
    }

    let base = load_fvecs_array::<DIM>(&base_path, SEED_COUNT + INSERT_SAMPLE_SIZE)
        .with_context(|| format!("load base from {}", base_path.display()))?;
    let queries = load_fvecs_array::<DIM>(&query_path, SEARCH_SAMPLE_SIZE)
        .with_context(|| format!("load queries from {}", query_path.display()))?;
    Ok((base, queries))
}

fn build_seed_index(
    vectors: &[[f32; DIM]],
) -> HnswIndex<u32, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE> {
    let mut index = HnswIndex::<u32, DIM, GRAPH_CHUNK_SIZE, VECTOR_CHUNK_SIZE>::new(
        HnswParams::default(),
        select_best_euclidean(),
    );

    for (i, vector) in vectors.iter().enumerate() {
        index
            .insert(i as u32, *vector)
            .unwrap_or_else(|err| panic!("seed insert failed for key {i}: {err}"));
    }

    index
}

fn benchmark_sift(c: &mut Criterion) {
    let (base_vectors, queries) = load_sift_data().expect("failed to load sift data");
    let seed_vectors = &base_vectors[..SEED_COUNT];
    let insert_vectors = &base_vectors[SEED_COUNT..SEED_COUNT + INSERT_SAMPLE_SIZE];

    let mut group = c.benchmark_group("sift-incremental");

    // Benchmark individual insert operations with consistent index size
    group.bench_function(BenchmarkId::new("insert", "single"), |b| {
        b.iter_batched(
            || build_seed_index(seed_vectors), // Setup: fresh 4K index each time
            |mut index| {
                let vec = insert_vectors[0]; // Use same vector for consistency
                index.insert(SEED_COUNT as u32, vec).unwrap();
                black_box(index)
            },
            BatchSize::SmallInput,
        )
    });

    // Benchmark individual search operations
    group.bench_function(BenchmarkId::new("search", "single"), |b| {
        let index = build_seed_index(seed_vectors);
        b.iter(|| {
            let query = &queries[0]; // Use same query for consistency
            let results = index.search_with_ef(query, SEARCH_K, SEARCH_EF);
            black_box(results)
        })
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_sift(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
