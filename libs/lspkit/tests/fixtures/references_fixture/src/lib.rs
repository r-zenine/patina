pub fn target_function() -> i32 {
    42
}

pub fn caller_one() -> i32 {
    target_function() + 1
}

pub fn caller_two() -> i32 {
    target_function() * 2
}
