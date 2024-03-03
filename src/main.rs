use equaio::expression as exp;
use equaio::vec_strings;
use equaio::parser_prefix;

fn main() {
    let str : String = "+(0,+(1,f(2,4)))".into();
    let ctx = exp::Context {
        parameters: vec_strings!["x", "y"],
        unary_ops: vec_strings!["+", "-"],
        binary_ops: vec_strings!["+", "-", "*", "/"],
    };
    let expr = parser_prefix::to_expression(str, ctx).unwrap();
    println!("{}", expr.to_string(true));
}
