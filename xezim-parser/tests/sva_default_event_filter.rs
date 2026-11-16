//! §16.8/§9.4.2.3: iff is only retained for sequence/property formals.
use sv_parser::{
    ast::{Description, decl::ModuleItem},
    parse,
};
#[test]
fn event_filter_defaults_keep_the_existing_binding() {
    for (ty, kept) in [
        ("", false),
        ("untyped ", false),
        ("event ", false),
        ("property ", true),
    ] {
        for expr in ["clk iff en", "(clk iff en)"] {
            let body = if ty == "property " { "c" } else { "@(c) 1" };
            let source = format!(
                "module m; bit clk, en; property p({ty}c={expr}); {body}; endproperty endmodule"
            );
            let result = parse(&source);
            assert!(result.errors.is_empty(), "{:?}", result.errors);
            let Description::Module(m) = &result.source.descriptions[0] else {
                panic!("module")
            };
            let defaults = m
                .items
                .iter()
                .find_map(|i| match i {
                    ModuleItem::PropertyDeclaration(p) => Some(&p.defaults),
                    _ => None,
                })
                .unwrap();
            assert_eq!(!defaults.is_empty(), kept, "{ty}{expr}");
        }
    }
}
