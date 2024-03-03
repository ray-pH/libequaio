use equaio::expression as exp;
use equaio::vec_strings;
use equaio::parser_prefix;

fn main() {
    let str : String = "+(0,+(x,f(2,4)))".into();
    let ctx = exp::Context {
        parameters: vec_strings!["x", "y"],
        unary_ops: vec_strings!["+", "-"],
        binary_ops: vec_strings!["+", "-", "*", "/"],
        handle_numerics: true,
    };
    let expr = parser_prefix::to_expression(str, ctx.clone()).unwrap();
    println!("{}", expr.to_string(true));

    // let pattren = parser_prefix::to_expression("+(A,f(B,C))".into(), ctx.clone()).unwrap();
    let pattern = parser_prefix::to_expression("+(A,B)".into(), ctx.clone()).unwrap();
    let matches = expr.get_pattern_matches(&pattern);
    for (address, map) in matches {
        println!("Match at address {:?}", address);
        for (k,v) in map {
            println!("{} -> {}", k, v.to_string(true));
        }
        println!("");
    }
}
