use equaio::worksheet::Worksheet;
use equaio::parser_prefix;
use equaio::arithmetic;
use equaio::address;
use equaio::expression::{Address, expression_builder as eb};
use equaio::algebra;

#[cfg(test)]
mod algebra_test {
    use super::*;
    
    #[test]
    fn simple() {
        // solve 2*x - 1 = 3
        // BEGIN setup
        let mut ws = Worksheet::new();
        ws.set_expression_context(arithmetic::get_arithmetic_ctx().add_param("x".to_string()));
        ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
        ws.set_rule_map(algebra::get_algebra_rules(&ws.get_expression_context()));
        ws.introduce_expression(parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap());
        // END setup
        let seq0 = ws.get_expression_sequence(0).unwrap();
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Add, &eb::constant("1"));
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![1]);
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![0].sub(1));
        assert!(status);
        let status = seq0.apply_rule_at("add_zero", &address![0]);
        assert!(status);
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Div, &eb::constant("2"));
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

#[cfg(test)]
mod get_possible_actions {
    use equaio::worksheet::Action;

    use super::*;
    
    #[test]
    fn arithmetic_both_side_given_inner() {
        // x = 3 / (1-x)
        let mut ws = Worksheet::new();
        ws.set_expression_context(arithmetic::get_arithmetic_ctx().add_param("x".to_string()));
        ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
        ws.set_rule_map(algebra::get_algebra_rules(&ws.get_expression_context()));
        ws.set_get_possible_actions_function(|expr,_ctx,addr_vec| 
            algebra::get_possible_actions::apply_operation_both_side(expr,addr_vec));
        let expr = parser_prefix::to_expression("=(x,/(3,-(1,x)))", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr.clone());
        
        let seq0 = ws.get_expression_sequence(0).unwrap();
        let actions = seq0.get_possible_actions(&vec![address![], address![1,1]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Apply *(1 - x) to both side".to_string()));
        assert_eq!(actions[0].1.to_string(true), "((x * (1 - x)) = ((3 / (1 - x)) * (1 - x)))");
        
        let actions = seq0.get_possible_actions(&vec![address![], address![1,1,0]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Apply *(1 - x) to both side".to_string()));
        assert_eq!(actions[0].1.to_string(true), "((x * (1 - x)) = ((3 / (1 - x)) * (1 - x)))");
        
        let actions = seq0.get_possible_actions(&vec![address![], address![1,1,1]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Apply *(1 - x) to both side".to_string()));
        assert_eq!(actions[0].1.to_string(true), "((x * (1 - x)) = ((3 / (1 - x)) * (1 - x)))");
    }
    
    #[test]
    fn simple() {
        // solve x - 1 = 3
        let mut ws = Worksheet::new();
        ws.set_expression_context(arithmetic::get_arithmetic_ctx().add_param("x".to_string()));
        ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
        ws.set_rule_map(algebra::get_algebra_rules(&ws.get_expression_context()));
        ws.set_get_possible_actions_function(|expr,ctx,addr_vec| 
            algebra::get_possible_actions::algebra(expr,ctx,addr_vec));
        ws.introduce_expression(parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap());
        
        let seq0 = ws.get_expression_sequence(0).unwrap();
        assert!(seq0.try_apply_action_by_index(&vec![address![], address![0,1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0].sub(1)], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![], address![0,0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0,0], address![0,1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        
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
