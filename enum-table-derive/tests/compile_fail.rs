#[test]
#[cfg_attr(
    miri,
    ignore = "trybuild spawns rustc and walks the filesystem, which Miri cannot support"
)]
fn compile_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
