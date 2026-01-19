pub enum CacheLevel {
    L1,
    L2,
    #[allow(unused)]
    L3,
}

pub enum PrefetchKind {
    Temporal(CacheLevel),
    NonTemporal(CacheLevel),
}

/// Cache line size (in bytes) for architectures we target.
const CACHE_LINE_BYTES: usize = 64;

#[inline(always)]
pub fn prefetch<T>(ptr: *const T, kind: &PrefetchKind) {
    match kind {
        PrefetchKind::Temporal(CacheLevel::L1) => unsafe { prefetch_temporal_l1(ptr.cast::<u8>()) },
        PrefetchKind::Temporal(CacheLevel::L2) => unsafe { prefetch_temporal_l2(ptr.cast::<u8>()) },
        PrefetchKind::Temporal(CacheLevel::L3) => unsafe { prefetch_temporal_l3(ptr.cast::<u8>()) },
        PrefetchKind::NonTemporal(CacheLevel::L1) => unsafe {
            prefetch_non_temporal_l1(ptr.cast::<u8>())
        },
        PrefetchKind::NonTemporal(CacheLevel::L2) => unsafe {
            prefetch_non_temporal_l2(ptr.cast::<u8>())
        },
        PrefetchKind::NonTemporal(CacheLevel::L3) => unsafe {
            prefetch_non_temporal_l3(ptr.cast::<u8>())
        },
    }
}

#[inline(always)]
pub fn prefetch_region<T: Sized>(ptr: *const T, len: usize, kind: &PrefetchKind) {
    let elem_size = core::mem::size_of::<T>();
    if len == 0 || elem_size == 0 || ptr.is_null() {
        return;
    }

    let base = ptr.cast::<u8>();
    let Some(total_bytes) = elem_size.checked_mul(len) else {
        return;
    };
    let mut offset = 0usize;

    while offset < total_bytes {
        unsafe {
            prefetch(base.add(offset), kind);
        }
        offset += CACHE_LINE_BYTES;
    }
}

#[inline(always)]
#[allow(unused)]
pub fn prefetch_slice<T: Sized>(slice: &[T], kind: &PrefetchKind) {
    prefetch_region(slice.as_ptr(), slice.len(), kind);
}

#[inline(always)]
#[allow(unused)]
pub fn prefetch_array<T: Sized, const N: usize>(array: &[T; N], kind: &PrefetchKind) {
    prefetch_region(array.as_ptr(), N, kind);
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_temporal_l1(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
        "prfm pldl1keep, [{addr}]", 
        addr = in(reg) ptr,
        options(readonly, preserves_flags, nostack));
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_temporal_l2(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
            "prfm pldl2keep, [{addr}]", 
            addr = in(reg) ptr,
            options(readonly, preserves_flags, nostack));
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_temporal_l3(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
            "prfm pldl3keep, [{addr}]", 
            addr = in(reg) ptr,
            options(readonly, preserves_flags, nostack));
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_non_temporal_l1(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
            "prfm pldl1strm, [{addr}]", 
            addr = in(reg) ptr,
            options(readonly, preserves_flags, nostack));
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_non_temporal_l2(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
            "prfm pldl2strm, [{addr}]", 
            addr = in(reg) ptr,
            options(readonly, preserves_flags, nostack));
    }
}

#[cfg(target_arch = "aarch64")]
#[inline(always)]
unsafe fn prefetch_non_temporal_l3(ptr: *const u8) {
    unsafe {
        core::arch::asm!(
            "prfm pldl3strm, [{addr}]", 
            addr = in(reg) ptr,
            options(readonly, preserves_flags, nostack));
    }
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_temporal_l1(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_T0, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_T0 }>(ptr as *const i8);
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_temporal_l2(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_T1, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_T1 }>(ptr as *const i8);
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_temporal_l3(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_T2, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_T2 }>(ptr as *const i8);
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l1(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_NTA }>(ptr as *const i8);
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l2(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_NTA }>(ptr as *const i8);
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l3(ptr: *const u8) {
    use core::arch::x86_64::{_MM_HINT_NTA, _mm_prefetch};
    _mm_prefetch::<{ _MM_HINT_NTA }>(ptr as *const i8);
}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_temporal_l1(_: *const u8) {}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_temporal_l2(_: *const u8) {}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_temporal_l3(_: *const u8) {}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l1(_: *const u8) {}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l2(_: *const u8) {}

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
#[inline(always)]
unsafe fn prefetch_non_temporal_l3(_: *const u8) {}
