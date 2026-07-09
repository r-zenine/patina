pub fn target_function() -> i32 {
    42
}

pub fn caller_one() -> i32 {
    target_function() + 1
}

pub fn caller_two() -> i32 {
    target_function() * 2
}

pub fn chain_a() -> i32 {
    chain_b()
}

pub fn chain_b() -> i32 {
    chain_c() + 1
}

pub fn chain_c() -> i32 {
    7
}
