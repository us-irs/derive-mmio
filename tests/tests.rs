#[test]
fn all_tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/cases/basic.rs");
    t.pass("tests/cases/no_ctors.rs");
    t.compile_fail("tests/cases/repr_c_mandatory.rs");

    t.compile_fail("tests/cases/padding_forbidden.rs");
    t.compile_fail("tests/cases/read_only.rs");
}
