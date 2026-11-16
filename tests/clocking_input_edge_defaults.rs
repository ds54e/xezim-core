//! §14.3: retain default input edges without overriding an explicit skew.
use xezim_core::ast::{Description, decl::ModuleItem, stmt::Edge};

#[test]
fn default_input_edge_applies_only_without_an_explicit_skew() {
    let parsed = xezim_core::parse_str(
        r#"
module top;
  bit clk;
  int a, b, c, d, e, q;
  clocking cb @(posedge clk);
    input a;
    default input negedge output posedge;
    input b;
    input #1step c;
    input #0 d;
    input posedge e;
    output q;
  endclocking
endmodule
"#,
    )
    .expect("parse");
    let Description::Module(module) = &parsed.source.descriptions[0] else {
        panic!("module")
    };
    let cb = module
        .items
        .iter()
        .find_map(|item| match item {
            ModuleItem::ClockingDeclaration(cb) => Some(cb),
            _ => None,
        })
        .expect("clocking block");
    let signal = |name: &str| cb.signals.iter().find(|s| s.name.name == name).unwrap();
    assert_eq!(signal("a").skew_edge, Some(Edge::Negedge));
    assert_eq!(signal("b").skew_edge, Some(Edge::Negedge));
    assert!(signal("c").skew_edge.is_none() && signal("c").skew.is_some());
    assert!(signal("d").skew_edge.is_none() && signal("d").skew.is_some());
    assert_eq!(signal("e").skew_edge, Some(Edge::Posedge));
    assert!(signal("q").skew_edge.is_none());
    assert_eq!(cb.default_output_edge, Some(Edge::Posedge));
}
