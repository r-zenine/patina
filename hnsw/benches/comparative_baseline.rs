use std::fs::File;
use std::hint::black_box;
use std::io::{self, BufReader, Read};
use std::path::Path;

use anyhow::{Context, Result, anyhow};
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use hnsw_rs::{HnswIndex, HnswParams, select_best_euclidean};

#[cfg(feature = "bench-compare")]
use external_hnsw;
#[cfg(feature = "bench-compare")]
use faiss;

const GRAPH_CHUNK_SIZE: usize = 4096;
const VECTOR_CHUNK_SIZE: usize = 4096;
const DATA_DIR: &str = "data/sift";
const BASE_FILE: &str = "sift_base.fvecs";
const QUERY_FILE: &str = "sift_query.fvecs";

const DIM: usize = 128;
const SEED_COUNT: usize = 4_000;
const INSERT_SAMPLE_SIZE: usize = 200;
const SEARCH_SAMPLE_SIZE: usize = 200;
const SEARCH_K: usize = 10;
const SEARCH_EF: usize = 200;
const M: usize = 16; // HNSW parameter for maximum connections

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

// Our implementation
fn build_hnsw_rs_index(
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

#[cfg(feature = "bench-compare")]
fn build_external_hnsw_index(vectors: &[[f32; DIM]]) -> external_hnsw::hnsw::Hnsw<f32, external_hnsw::prelude::DistL2> {
    use external_hnsw::prelude::DistL2;
    use external_hnsw::hnsw::Hnsw;

    let nb_layer = 16.min((vectors.len() as f32).ln() as usize);
    let mut hnsw = Hnsw::<f32, DistL2>::new(
        M,              // max_nb_connection
        vectors.len(),  // nb_elem
        nb_layer,       // nb_layer
        200,            // ef_c (construction parameter)
        DistL2{},       // distance function
    );

    // Insert vectors
    let data_for_insertion: Vec<(Vec<f32>, usize)> = vectors
        .iter()
        .enumerate()
        .map(|(i, v)| (v.to_vec(), i))
        .collect();

    let data_refs: Vec<(&Vec<f32>, usize)> = data_for_insertion
        .iter()
        .map(|(v, i)| (v, *i))
        .collect();

    hnsw.parallel_insert(&data_refs);
    hnsw.set_searching_mode(true);
    hnsw
}

#[cfg(feature = "bench-compare")]
fn build_faiss_index(vectors: &[[f32; DIM]]) -> Result<faiss::index::IndexImpl> {
    use faiss::{Index, index_factory, MetricType};

    // Create HNSW index (not flat!)
    let description = format!("HNSW{}", M);
    println!("Creating Faiss index with description: {}", description);
    let mut index = index_factory(DIM as u32, &description, MetricType::L2)?;

    // Flatten vectors for Faiss
    let flat_vectors: Vec<f32> = vectors.iter().flat_map(|v| v.iter()).copied().collect();

    index.train(&flat_vectors)?;
    index.add(&flat_vectors)?;

    Ok(index)
}

fn benchmark_implementations(c: &mut Criterion) {
    let (base_vectors, queries) = load_sift_data().expect("failed to load sift data");
    let seed_vectors = &base_vectors[..SEED_COUNT];
    let insert_vectors = &base_vectors[SEED_COUNT..SEED_COUNT + INSERT_SAMPLE_SIZE];

    let mut group = c.benchmark_group("hnsw-comparison");

    // Benchmark our implementation
    group.bench_function(BenchmarkId::new("insert", "hnsw-rs"), |b| {
        b.iter_batched(
            || build_hnsw_rs_index(seed_vectors),
            |mut index| {
                let vec = insert_vectors[0];
                index.insert(SEED_COUNT as u32, vec).unwrap();
                black_box(index)
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function(BenchmarkId::new("search", "hnsw-rs"), |b| {
        let index = build_hnsw_rs_index(seed_vectors);
        b.iter(|| {
            let query = &queries[0];
            let results = index.search_with_ef(query, SEARCH_K, SEARCH_EF);
            black_box(results)
        })
    });

    #[cfg(feature = "bench-compare")]
    {
        // Benchmark external hnswlib implementation
        group.bench_function(BenchmarkId::new("insert", "external-hnsw"), |b| {
            b.iter_batched(
                || build_external_hnsw_index(seed_vectors),
                |mut index| {
                    let vec = &insert_vectors[0][..];
                    index.insert((vec, SEED_COUNT));
                    black_box(index)
                },
                BatchSize::SmallInput,
            )
        });

        group.bench_function(BenchmarkId::new("search", "external-hnsw"), |b| {
            let index = build_external_hnsw_index(seed_vectors);
            b.iter(|| {
                let query = &queries[0][..];
                let results = index.search(query, SEARCH_K, SEARCH_EF);
                black_box(results)
            })
        });

        // Benchmark Faiss implementation
        match build_faiss_index(seed_vectors) {
            Ok(mut index) => {
                use faiss::Index;
                println!("Faiss index created successfully, ntotal: {}", index.ntotal());

                // Faiss insert benchmark
                group.bench_function(BenchmarkId::new("insert", "faiss"), |b| {
                    use faiss::Index;
                    b.iter_batched(
                        || {
                            // Create fresh index for each iteration
                            build_faiss_index(seed_vectors).expect("Failed to create Faiss index")
                        },
                        |mut index| {
                            let vec = &insert_vectors[0][..];
                            match index.add(vec) {
                                Ok(_) => black_box(index),
                                Err(e) => panic!("Faiss insert failed: {}", e),
                            }
                        },
                        BatchSize::SmallInput,
                    )
                });

                // Faiss search benchmark
                group.bench_function(BenchmarkId::new("search", "faiss"), |b| {
                    use faiss::Index;
                    b.iter(|| {
                        let query = &queries[0][..];
                        match index.search(query, SEARCH_K) {
                            Ok(results) => {
                                // Ensure we actually got results
                                assert_eq!(results.labels.len(), SEARCH_K);
                                black_box(results)
                            }
                            Err(e) => {
                                panic!("Faiss search failed: {}", e);
                            }
                        }
                    })
                });
            }
            Err(e) => {
                println!("Failed to create Faiss index: {}", e);
            }
        }
    }

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    benchmark_implementations(c);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);