//! IEEE 1800-2023 §23.10.1 permits defparams targeting their containing
//! instance. §23.10 gives them precedence over instance parameter assignments;
//! §23.10.3 requires dependent parameters to be recomputed before sizing.

fn elaborate(src: &str) -> xezim_core::ElaboratedModule {
    xezim_core::parse_and_elaborate_multi(
        &[src.to_string()],
        Some("tb"),
        &[],
        &["own.sv".into()],
        &[],
    )
    .expect("legal own-instance defparam")
    .1
}

fn param(m: &xezim_core::ElaboratedModule, key: &str) -> i64 {
    m.parameters[key].to_i64().unwrap()
}

#[test]
fn own_defparams_precede_top_and_child_shapes() {
    let m = elaborate(
        r#"
module child;
  parameter int P = 0;
  parameter int K = 0;
  parameter int D = P + 2;
  defparam P = 0;
  defparam child.K = 5;
  defparam P = 1;
  typedef logic [P+2:0] word_t;
  word_t v;
  wire [D-1:0] r;
  if (P == 1) begin : good wire selected; end
endmodule
module tb;
  parameter int P = 0;
  parameter int D = P + 3;
  defparam tb.P = 2;
  typedef logic [P:0] word_t;
  word_t v;
  wire [D-1:0] r;
  child c1();
  child #(9) c2();
endmodule
"#,
    );
    assert_eq!((param(&m, "P"), param(&m, "D")), (2, 5));
    assert_eq!((m.signals["v"].width, m.signals["r"].width), (3, 5));
    for inst in ["c1", "c2"] {
        assert_eq!(param(&m, &format!("{inst}.P")), 1);
        assert_eq!(param(&m, &format!("{inst}.K")), 5);
        assert_eq!(param(&m, &format!("{inst}.D")), 3);
        assert_eq!(m.signals[&format!("{inst}.v")].width, 4);
        assert_eq!(m.signals[&format!("{inst}.r")].width, 3);
        assert!(m.signals.contains_key(&format!("{inst}.good.selected")));
    }
}

#[test]
fn own_and_external_defparams_follow_source_order() {
    let m = elaborate(
        r#"
module earlier;
  parameter int P = 0;
  defparam P = 3;
endmodule
module tb;
  earlier a();
  later b();
  defparam a.P = 7;
  defparam b.P = 7;
endmodule
module later;
  parameter int P = 0;
  defparam P = 4;
endmodule
"#,
    );
    assert_eq!(param(&m, "a.P"), 7);
    assert_eq!(param(&m, "b.P"), 4);
}

// §6.20.2 / §10.8 / §23.10: a typed formal supplies assignment context;
// an untyped formal takes the override's width and sign.
#[test]
fn own_defparams_keep_assignment_conversion() {
    let m = elaborate(
        r#"
module child;
  parameter logic [7:0] P=0;
  parameter int Q=0;
  parameter logic signed [3:0] S=0;
  parameter F=0;
  defparam P='1;
  defparam Q=(8'd128 + 8'd128) >> 8;
  defparam S=4'd15;
  defparam F=8'hf0;
endmodule
module tb;
  parameter logic [7:0] P=0;
  parameter int Q=0;
  parameter logic signed [3:0] S=0;
  parameter F=0;
  defparam P='1;
  defparam Q=(8'd128 + 8'd128) >> 8;
  defparam S=4'd15;
  defparam F=8'hf0;
  child c();
endmodule
"#,
    );
    for prefix in ["", "c."] {
        assert_eq!(param(&m, &format!("{prefix}P")), 255);
        assert_eq!(param(&m, &format!("{prefix}Q")), 1);
        assert_eq!(param(&m, &format!("{prefix}S")), -1);
        assert_eq!(param(&m, &format!("{prefix}F")), 240);
        assert_eq!(m.parameters[&format!("{prefix}F")].width, 8);
    }
}
