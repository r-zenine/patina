# HNSW-RS

A Rust implementation of the HNSW (Hierarchical Navigable Small World) algorithm for approximate nearest neighbor search.

## Benchmarks

This project includes comparative benchmarks against popular HNSW implementations to validate performance and correctness.

### Running Basic Benchmarks

To run benchmarks with just the hnsw-rs implementation:

```bash
# Download SIFT dataset first
./scripts/fetch_sift.sh

# Run basic benchmarks
cargo bench --bench comparative_baseline
```

### Running Comparative Benchmarks

To compare against external libraries (hnsw_rs crate and Faiss), you need to install additional dependencies and use the `bench-compare` feature.

#### Prerequisites for Faiss

On macOS with Homebrew:

```bash
# Install Faiss C library
brew install faiss

# Set environment variables for linking
export LIBRARY_PATH=/opt/homebrew/lib:$LIBRARY_PATH
export LD_LIBRARY_PATH=/opt/homebrew/lib:$LD_LIBRARY_PATH
export DYLD_LIBRARY_PATH=/opt/homebrew/lib:$DYLD_LIBRARY_PATH
```

On Ubuntu/Debian:

```bash
# Install Faiss
sudo apt-get update
sudo apt-get install libfaiss-dev

# Set environment variables
export LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LIBRARY_PATH
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:$LD_LIBRARY_PATH
```

#### Running Full Comparative Benchmarks

```bash
# Make sure SIFT data is downloaded
./scripts/fetch_sift.sh

# Run comparative benchmarks (includes hnsw_rs crate and Faiss)
cargo bench --bench comparative_baseline --features bench-compare
```

### Benchmark Details

The comparative benchmark (`comparative_baseline`) tests:

- **Insert performance**: Time to insert a single vector into a 4,000-vector index
- **Search performance**: Time to find k=10 nearest neighbors with ef=200

**Libraries compared:**
- `hnsw-rs` (this implementation)
- `hnsw_rs` (external Rust crate) - requires `bench-compare` feature
- `Faiss` (industry standard C++ library) - requires `bench-compare` feature and system installation

**Dataset:**
- SIFT 128-dimensional vectors (standard computer vision benchmark)
- 4,000 seed vectors + 200 test vectors for insertion
- 200 query vectors for search testing

### Benchmark Results

Recent results on Apple Silicon (M-series) show:

- **hnsw-rs**: ~5μs insert, ~750ns search
- **external hnsw_rs**: ~400μs insert, ~280μs search (79x and 373x slower respectively)
- **Faiss**: Competitive search performance, but insert benchmarks need methodology fixes

### Troubleshooting

**Faiss linking errors:**
```bash
# Verify Faiss installation
ls /opt/homebrew/lib/libfaiss*  # macOS
ls /usr/lib/x86_64-linux-gnu/libfaiss*  # Linux

# Check if environment variables are set
echo $LIBRARY_PATH
echo $LD_LIBRARY_PATH
```

**Missing SIFT data:**
```bash
# Download dataset
./scripts/fetch_sift.sh

# Verify files exist
ls -la data/sift/
```

**Build errors with external dependencies:**
- Try running without `--features bench-compare` first to test basic functionality
- Ensure Faiss C library is properly installed before enabling comparative features

## Development

See `CLAUDE.md` for detailed development guidelines and project architecture information.
