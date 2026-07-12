mod helpers_common;

#[test]
fn second_uses_drive() {
    assert_eq!(helpers_common::drive(), 7);
}
