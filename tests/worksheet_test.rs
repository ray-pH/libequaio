use equaio::worksheet::Worksheet;
use equaio::parser_prefix;
use equaio::arithmetic;
use equaio::address;
use equaio::expression::Address;
use equaio::algebra;

#[cfg(test)]
mod algebra_test {
    use super::*;
    
    #[test]
    fn simple() {
        // solve 2*x - 1 = 3
        // BEGIN setup
        let mut ws = Worksheet::new(arithmetic::get_arithmetic_ctx().add_param("x".to_string()));
        ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
        ws.introduce_expression(parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.context).unwrap());
        ws.set_rule_map(algebra::get_algebra_rules(&ws.context));
        // END setup
        let seq0 = ws.get_expression_sequence(0).unwrap();
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Add, "1");
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![1]);
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![0].sub(1));
        assert!(status);
        let status = seq0.apply_rule_at("add_zero", &address![0]);
        assert!(status);
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Div, "2");
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![1]);
        assert!(status);
        let status = seq0.apply_fraction_arithmetic_at(0,0, &address![0]);
        assert!(status);
        let status = seq0.apply_rule_at("div_one", &address![0]);
        assert!(status);
        let status = seq0.apply_rule_at("one_mul", &address![0]);
        assert!(status);
        
        let target = [
            ("Introduce", "(((2 * x) - 1) = 3)"),
            ("Apply +1 to both side", "(((2 * x) + (-1) + 1) = (3 + 1))"),
            ("Calculate 3 + 1 = 4", "(((2 * x) + (-1) + 1) = 4)"),
            ("Calculate -1 + 1 = 0", "(((2 * x) + 0) = 4)"),
            ("Add by Zero", "((2 * x) = 4)"),
            ("Apply /2 to both side", "(((2 * x) / 2) = (4 / 2))"),
            ("Calculate 4 / 2 = 2", "(((2 * x) / 2) = 2)"),
            ("Simplify fraction", "(((1 * x) / 1) = 2)"),
            ("Divide by One", "((1 * x) = 2)"),
            ("Multiply by One", "(x = 2)"),
        ];
        assert_eq!(seq0.history.len(), target.len());
        for (i, (target_action_str, target_expr_str)) in target.iter().enumerate() {
            let (action, expr) = seq0.history.get(i).unwrap();
            assert_eq!(action.to_string(), target_action_str.to_string());
            assert_eq!(expr.to_string(true), target_expr_str.to_string());
        }
    }
}
