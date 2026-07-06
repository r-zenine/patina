pub fn find_fn_names(source: &str) -> bool {
    let re = regex::Regex::new(r"fn [a-z]+").unwrap();
    re.is_match(source)
}

pub fn naive_split(source: &str) -> Vec<&str> {
    source.split("fn ").collect()
}

pub fn find_index(source: &str) -> Option<usize> {
    source.find("fn ")
}

pub fn allowed_ast_like_usage(source: &str) -> bool {
    source.starts_with("fn ") && source.ends_with(';') && source.contains('{')
}
