use equaio::address;
use equaio::expression::{Address, expression_builder as eb};
use equaio::arithmetic;
use equaio::parser_prefix;
use equaio::vec_strings;
use equaio::algebra;


#[cfg(test)]
mod rule {
    use super::*;
    
    #[test]
    fn rule() {
        let ctx = arithmetic::get_arithmetic_ctx();
        for (name, rule_str, _) in algebra::ALGEBRA_RULE_STRING_TUPLE.iter() {
            let rule_expr = parser_prefix::to_expression(rule_str, &ctx);
            println!("{}", name);
            assert!(rule_expr.is_some());
        }
    }
    
}

#[cfg(test)]
mod normalization {
    use algebra::AlgebraCtxFlags;

    use super::*;
    
    #[test]
    fn simple_expression() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x"]);
        let expr = parser_prefix::to_expression("+(-(x,1),2)", &ctx).unwrap();
        let normalized_expr = expr.normalize_algebra(&ctx);
        let target_expr = parser_prefix::to_expression("+(x,-(1),2)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn simplification_one_and_zero() {
        let ctx = arithmetic::get_arithmetic_ctx()
            .add_params(vec_strings!["x"])
            .add_flag(AlgebraCtxFlags::SimplifyOneAndZero);
        let expr = parser_prefix::to_expression("*(-(x,0),1)", &ctx).unwrap();
        let normalized_expr = expr.normalize_simplify_one_and_zero(&ctx);
        let target_expr = parser_prefix::to_expression("x", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
        
        let expr = parser_prefix::to_expression("-(-(x,0),0)", &ctx).unwrap();
        let normalized_expr = expr.normalize_simplify_one_and_zero(&ctx);
        let target_expr = parser_prefix::to_expression("x", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
        
        // (1/1 + 0)*(x - 0)
        let expr = parser_prefix::to_expression("*(+(/(1,1),0),-(x,0))", &ctx).unwrap();
        let normalized_expr = expr.normalize_simplify_one_and_zero(&ctx);
        let target_expr = parser_prefix::to_expression("x", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
}

#[cfg(test)]
mod function {
    use arithmetic::ArithmeticOperator;

    use super::*;
    
    #[test]
    fn simple_function() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("f(2)", &ctx).unwrap();
        let function_expr = parser_prefix::to_expression("=(f(X), +(X,3))", &ctx).unwrap();
        assert_eq!(function_expr.to_string(true), "(f(X) = (X + 3))");
        let expr = expr.apply_equation_at(&function_expr, &address![]).unwrap();
        assert_eq!(expr.to_string(true), "(2 + 3)");
        let equation = expr.generate_simple_artithmetic_equation_at(&address![]).unwrap();
        let expr = expr.apply_equation_at(&equation, &address![]).unwrap();
        assert_eq!(expr.to_string(true), "5");
    }
    
    #[test]
    fn apply_function_to_both_side() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x","y"]);
        let expr = parser_prefix::to_expression("=(x,y)", &ctx).unwrap();
        let function_expr = parser_prefix::to_expression("=(_(X),+(X,1))", &ctx).unwrap();
        assert!(function_expr.is_function());
        assert_eq!(function_expr.to_string(true), "(_(X) = (X + 1))");
        
        let new_expr = expr.apply_function_to_both_side(function_expr).unwrap();
        assert_eq!(new_expr.to_string(true), "((x + 1) = (y + 1))");
    }
    
    #[test]
    fn apply_algebra_to_both_side() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x","y"]);
        let expr = parser_prefix::to_expression("=(x,y)", &ctx).unwrap();
        let new_expr = expr.apply_simple_arithmetic_to_both_side(&ArithmeticOperator::Add, &eb::constant("1")).unwrap();
        let target_expr = parser_prefix::to_expression("=(+(x,1),+(y,1))", &ctx).unwrap();
        assert_eq!(new_expr.to_string(true), target_expr.to_string(true));
    }
}

#[cfg(test)]
mod fraction {
    use super::*;
    
    #[test]
    fn simple_division() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x"]);
        let expr = parser_prefix::to_expression("/(*(4,5,6),*(1,2,3))", &ctx).unwrap();
        let new_expr = expr.apply_fraction_arithmetic(0, 1).unwrap();
        let target_expr = parser_prefix::to_expression("/(*(2,5,6),*(1,1,3))", &ctx).unwrap();
        assert_eq!(new_expr.to_string(true), target_expr.to_string(true));
        // assert_eq!(new_expr, target_expr);
    }
    
}

#[cfg(test)]
mod simple_algebra {
    use algebra::get_algebra_rules;
    use super::*;
    
    #[test]
    fn question1() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x"]);
        
        let x_plus_zero = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let x_div_one   = parser_prefix::to_expression("=(/(X,1),X)", &ctx).unwrap();
        let one_times_x = parser_prefix::to_expression("=(*(1,X),X)", &ctx).unwrap();
        let algebra_rules = get_algebra_rules(&ctx);
        assert_eq!(algebra_rules.get("add_zero").unwrap().expression, x_plus_zero);
        assert_eq!(algebra_rules.get("div_one").unwrap().expression, x_div_one);
        assert_eq!(algebra_rules.get("one_mul").unwrap().expression, one_times_x);
        
        // 2*x - 1 = 3
        let expr = parser_prefix::to_expression("=(-(*(2,x),1),3)", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(((2 * x) - 1) = 3)");
        // add 1 to both side
        let add1_fn = parser_prefix::to_expression("=(_(X),+(X,1))", &ctx).unwrap();
        let expr = expr.apply_function_to_both_side(add1_fn)
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) + (-1) + 1) = (3 + 1))");
        let expr = expr.apply_simple_arithmetic_equation_at(&address![1])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) + (-1) + 1) = 4)");
        let expr = expr.apply_simple_arithmetic_equation_at(&address![0].sub(1))
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) + 0) = 4)");
        let expr = expr.apply_equation_at(&x_plus_zero, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((2 * x) = 4)");
        let div2_fn = parser_prefix::to_expression("=(_(X),/(X,2))", &ctx).unwrap();
        let expr = expr.apply_function_to_both_side(div2_fn)
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) / 2) = (4 / 2))");
        let expr = expr.apply_simple_arithmetic_equation_at(&address![1])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) / 2) = 2)");
        let expr = expr.apply_fraction_arithmetic_at(0, 0, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((1 * x) / 1) = 2)");
        let expr = expr.apply_equation_at(&x_div_one, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((1 * x) = 2)");
        let expr = expr.apply_equation_at(&one_times_x, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(x = 2)");
    }
    
    #[test]
    fn question2() {
        let ctx = arithmetic::get_arithmetic_ctx().add_params(vec_strings!["x"]);
        let algebra_rules = get_algebra_rules(&ctx);
        
        // x = 4 - x
        let expr = parser_prefix::to_expression("=(x,-(4,x))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(x = (4 - x))");
        let func = parser_prefix::to_expression("=(_(X),+(X,x))", &ctx).unwrap();
        let expr = expr.apply_function_to_both_side(func)
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((x + x) = (4 + (-x) + x))");
        let rule_eq = &algebra_rules.get("sub_self3").unwrap().expression;
        let expr = expr.apply_equation_at(rule_eq, &address![1].sub(1))
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((x + x) = (4 + 0))");
        let expr = expr.apply_simple_arithmetic_equation_at(&address![1])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((x + x) = 4)");
        let rule_eq = &algebra_rules.get("add_self").unwrap().expression;
        let expr = expr.apply_equation_at(rule_eq, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((2 * x) = 4)");
        let func = parser_prefix::to_expression("=(_(X),/(X,2))", &ctx).unwrap();
        let expr = expr.apply_function_to_both_side(func)
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) / 2) = (4 / 2))");
        let expr = expr.apply_simple_arithmetic_equation_at(&address![1])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((2 * x) / 2) = 2)");
        let expr = expr.apply_fraction_arithmetic_at(0, 0, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(((1 * x) / 1) = 2)");
        let rule_eq = &algebra_rules.get("div_one").unwrap().expression;
        let expr = expr.apply_equation_at(rule_eq, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "((1 * x) = 2)");
        let rule_eq = &algebra_rules.get("one_mul").unwrap().expression;
        let expr = expr.apply_equation_at(rule_eq, &address![0])
            .unwrap().normalize_algebra(&ctx);
        assert_eq!(expr.to_string(true), "(x = 2)");
    }
}
