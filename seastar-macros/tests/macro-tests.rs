use seastar_macros;

#[seastar_macros::test]
fn macro_test() {
    assert_eq!(2+2, 4);
}
