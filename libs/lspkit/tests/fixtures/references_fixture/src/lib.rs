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

pub enum Shape {
    Circle,
    Square,
}

pub fn describe(shape: Shape) -> &'static str {
    match shape {
        Shape::Circle => "circle",
        Shape::Square => "square",
    }
}

pub trait Greeter {
    fn greet(&self) -> String;
}

pub struct EnglishGreeter;

impl Greeter for EnglishGreeter {
    fn greet(&self) -> String {
        "hello".to_string()
    }
}
