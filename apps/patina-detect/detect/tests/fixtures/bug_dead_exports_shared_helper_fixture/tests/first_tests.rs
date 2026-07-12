mod helpers_common;

#[test]
fn first_uses_park() {
    assert_eq!(helpers_common::park(), 3);
}
