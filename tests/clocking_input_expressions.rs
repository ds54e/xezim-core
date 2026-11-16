//! IEEE 1800-2023 §14.3, §14.5, §14.13: expression inputs use typed sample nets.
//! Width/sign follow §11.6, §11.8; drive bindings keep their §14.16 lvalues.
use xezim_core::{
    ast::{Description, expr::ExprKind},
    elaborate::{Definition, elaborate_module, inline_instantiations},
    hasher::HashMap,
};

#[test]
fn expression_inputs_lower_after_flattening_without_changing_lvalues() {
    let parsed = xezim_core::parse_str(
        r#"
module top;
  bit clk;
  int a;
  logic [3:0] b, q;
  clocking cb @(posedge clk);
    input sum = a + 1;
    input cat = {b, 1'b0};
    input nonzero = b != 0;
    input slice = b[2:1];
    output out = q[2:1];
    inout both = {q[1:0], q[3:2]};
  endclocking
endmodule
"#,
    )
    .expect("parse");
    let Description::Module(module) = &parsed.source.descriptions[0] else {
        panic!("module")
    };
    let mut elab =
        elaborate_module(Definition::Module(module), &HashMap::default()).expect("elaborate");
    let mut definitions = HashMap::default();
    definitions.insert("top".to_string(), Definition::Module(module));
    inline_instantiations(&mut elab, &definitions).expect("flatten");
    let cd = &elab.clocking_blocks["cb"];
    for (i, width, signed) in [(0, 32, true), (1, 5, false), (2, 1, false)] {
        let ExprKind::Ident(h) = &cd.signals[i].bound_to.as_ref().unwrap().kind else {
            panic!("sample net")
        };
        let sig = &elab.signals[&h.path[0].name.name];
        assert_eq!((sig.width, sig.is_signed), (width, signed));
        assert!(elab.continuous_assigns.iter().any(|ca| {
            matches!(&ca.lhs.kind, ExprKind::Ident(h) if h.path[0].name.name == sig.name.as_str())
        }));
    }
    assert!(matches!(
        cd.signals[3].bound_to.as_ref().unwrap().kind,
        ExprKind::RangeSelect { .. }
    ));
    assert!(matches!(
        cd.signals[4].bound_to.as_ref().unwrap().kind,
        ExprKind::RangeSelect { .. }
    ));
    assert!(matches!(
        cd.signals[5].bound_to.as_ref().unwrap().kind,
        ExprKind::Concatenation(_)
    ));
}
