use simsimd::SpatialSimilarity;

/// Type alias for distance function pointers
pub type DistanceFn = fn(&[f32], &[f32]) -> f32;

/// SimSIMD-optimized Euclidean distance
/// Uses hardware-accelerated SIMD (AVX-512, ARM NEON, etc.)
pub fn euclidean_f32(a: &[f32], b: &[f32]) -> f32 {
    match f32::sqeuclidean(a, b) {
        Some(squared_distance) => (squared_distance as f32).sqrt(),
        None => f32::MAX, // Return max distance on error (mismatched lengths, etc.)
    }
}

/// SimSIMD-optimized Cosine distance
/// SimSIMD cosine function returns cosine distance directly (1 - cosine_similarity)
pub fn cosine_f32(a: &[f32], b: &[f32]) -> f32 {
    match f32::cosine(a, b) {
        Some(distance) => distance as f32, // SimSIMD returns distance directly
        None => f32::MAX,
    }
}

/// SimSIMD-optimized Dot product distance
/// Returns negative dot product for distance metric (larger dot product = smaller distance)
pub fn dot_product_f32(a: &[f32], b: &[f32]) -> f32 {
    match f32::dot(a, b) {
        Some(dot_product) => -(dot_product as f32), // Negative for distance metric
        None => f32::MAX,
    }
}

/// SimSIMD-optimized Manhattan (L1) distance
pub fn manhattan_f32(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return f32::MAX;
    }

    // SimSIMD doesn't have Manhattan, so we implement it with SIMD-friendly operations
    a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
}

/// Runtime selection of optimal distance function based on available hardware
/// SimSIMD automatically uses the best SIMD implementation available
pub fn select_best_euclidean() -> DistanceFn {
    euclidean_f32 // SimSIMD handles hardware detection internally
}

pub fn select_best_cosine() -> DistanceFn {
    cosine_f32
}

pub fn select_best_dot_product() -> DistanceFn {
    dot_product_f32
}

pub fn select_best_manhattan() -> DistanceFn {
    manhattan_f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        let distance = euclidean_f32(&a, &b);
        let expected =
            ((1.0f32 - 4.0f32).powi(2) + (2.0f32 - 5.0f32).powi(2) + (3.0f32 - 6.0f32).powi(2))
                .sqrt();

        assert!((distance - expected).abs() < 1e-6);
    }

    #[test]
    fn test_cosine_distance() {
        let a = [1.0, 0.0, 0.0];
        let b = [0.0, 1.0, 0.0];

        let distance = cosine_f32(&a, &b);
        // Perpendicular vectors should have cosine distance of 1.0
        // SimSIMD returns cosine distance directly (1 - cosine_similarity)
        assert!((distance - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_dot_product_distance() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        let distance = dot_product_f32(&a, &b);
        let expected_dot = 1.0 * 4.0 + 2.0 * 5.0 + 3.0 * 6.0; // = 32.0
        let expected_distance = -expected_dot; // = -32.0

        assert!((distance - expected_distance).abs() < 1e-6);
    }

    #[test]
    fn test_manhattan_distance() {
        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        let distance = manhattan_f32(&a, &b);
        let expected = (1.0f32 - 4.0f32).abs() + (2.0f32 - 5.0f32).abs() + (3.0f32 - 6.0f32).abs(); // = 9.0

        assert!((distance - expected).abs() < 1e-6);
    }

    #[test]
    fn test_mismatched_vector_lengths() {
        let a = [1.0, 2.0];
        let b = [1.0, 2.0, 3.0];

        assert_eq!(euclidean_f32(&a, &b), f32::MAX);
        assert_eq!(cosine_f32(&a, &b), f32::MAX);
        assert_eq!(dot_product_f32(&a, &b), f32::MAX);
        assert_eq!(manhattan_f32(&a, &b), f32::MAX);
    }

    #[test]
    fn test_function_pointer_selection() {
        let euclidean_fn = select_best_euclidean();
        let cosine_fn = select_best_cosine();
        let dot_fn = select_best_dot_product();
        let manhattan_fn = select_best_manhattan();

        let a = [1.0, 2.0, 3.0];
        let b = [4.0, 5.0, 6.0];

        // Test that function pointers work correctly
        assert!(euclidean_fn(&a, &b) > 0.0);
        assert!(cosine_fn(&a, &b) >= 0.0);
        assert!(dot_fn(&a, &b) < 0.0); // Should be negative
        assert!(manhattan_fn(&a, &b) > 0.0);
    }
}
