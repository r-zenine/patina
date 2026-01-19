# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library project called `hnsw-rs` - a learning exercise to reimplement the USearch HNSW (Hierarchical Navigable Small World) algorithm. The goal is to understand both HNSW algorithms and advanced search optimization techniques.

## Reference Implementation

The reference implementation is USearch (located at `/Users/ryad/workspace/usearch/`), a highly optimized C++ HNSW implementation. Key architectural insights from USearch:

### Core HNSW Algorithm
- **Multi-level graph**: Hierarchical structure with exponentially fewer nodes at higher levels
- **Greedy search**: Start from entry point, descend through levels to find local minima
- **Graph construction**: Insert nodes with bidirectional connections using heuristic pruning
- **Level assignment**: Exponential distribution based on connectivity parameter

### Memory Layout Optimization
- **Tape allocation**: Sequential memory layout for cache efficiency
- **Node structure**: `[key][level][neighbors_level0][neighbors_level1]...`
- **Custom integers**: 40-bit addressing for >4B entries while saving memory
- **Memory alignment**: Cache line alignment and packed structures

### Performance Techniques
- **SIMD optimizations**: Hardware-accelerated distance calculations
- **Prefetching**: Manual prefetching of neighbor nodes during traversal
- **Lock-free operations**: Atomic counters and bitset-based fine-grained locking
- **Template specialization**: Zero-cost abstractions for different data types

### Key Design Patterns
- **Allocator separation**: Different strategies for metadata vs data
- **Error handling**: Result-based propagation without exceptions
- **Type system**: Support for f32, f64, f16, i8, and binary vectors
- **Configuration**: Immutable config with sensible defaults and validation

## Working Mode and Expectations

**This is a learning-focused project.** The primary goal is educational rather than production. Guidelines for interaction:

### Code Generation Policy
- **Guidance over code**: Provide architectural guidance, algorithm explanations, and implementation strategies rather than writing implementation code
- **Test generation**: Actively generate comprehensive test suites ported from USearch to validate correctness
- **Examples**: Provide small code examples to illustrate concepts when helpful for learning
- **Documentation**: Help with documentation and explanations of complex concepts

### Learning Focus Areas
- **Algorithm understanding**: Deep dive into HNSW theory and implementation details
- **Performance optimization**: Understanding memory layout, SIMD, cache optimization
- **Rust patterns**: Applying Rust's ownership system and type system to systems programming
- **Testing strategies**: Comprehensive testing approaches for approximate algorithms

### Test Suite Development
Priority should be given to porting USearch's test suite to Rust, including:
- Unit tests for core algorithms (search, insertion, graph construction)
- Property-based tests for HNSW invariants
- Performance benchmarks
- Correctness validation against known datasets
- Edge case testing (empty indices, single vectors, etc.)

## Development Commands

- **Build**: `cargo build`
- **Run tests**: `cargo test`
- **Run specific test**: `cargo test test_name`
- **Check code**: `cargo check`
- **Format code**: `cargo fmt`
- **Lint code**: `cargo clippy`
- **Benchmarks**: `cargo bench` (when implemented)

## Implementation Strategy

Focus areas for Rust reimplementation:
1. **Memory safety**: Leverage Rust's ownership system for safe memory management
2. **Zero-cost abstractions**: Use const generics and traits for template-like flexibility
3. **Performance**: Maintain careful memory layout optimizations from USearch
4. **Type system**: Support multiple vector types through trait system
5. **Concurrency**: Use Rust's async/threading primitives for safe parallelism

## Project Structure

- `src/lib.rs`: Main library entry point with basic placeholder implementation
- `Cargo.toml`: Project configuration using Rust 2024 edition

## Current State

The project is in its initial state with only a basic `add` function and test. The HNSW implementation has not yet been started. Use the USearch reference implementation to guide architectural decisions and optimization techniques.