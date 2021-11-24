use expect_test::expect;

use crate::ControlFlowGraph;

#[test]
pub fn roundtrip() {
    let src = expect![[r#"
        {
        mut [_6];
        next_local _7;
        bb0:

            let _0 := f64.* [f64 3.141, f64 2.0];
            let _1 := f64.+ [p1, _0];
            let _2 := f64.<= [_1, p0];
            if _2 { bb1 } else { bb2 } 
        bb1:

            let _3 := move [_1];
            let _6 := move [str "bar"];
            goto bb3;
        bb2:

            let _4 := move [f64 2.0];
            let _6 := move [str "foo"];
            goto bb3;
        bb3:

            phi _5 := [(bb1, _3), (bb2, _4)];
            end
        }"#]];
    let (cfg, literals) = ControlFlowGraph::parse(src.data).unwrap();
    let actual = cfg.print(&literals);
    src.assert_eq(&actual)
}

// TODO test of real codegen here:
// idea in hir_lower a test case compiles a model to a CFG and stores the result in a file
// then read that file here, parse it, print again and compare
