use equaio::rule::{self, RuleMap};
use equaio::worksheet::{Worksheet,Action,WorkableExpressionSequence};
use equaio::parser::{parser_prefix,parser};
use equaio::arithmetic::{self, get_arithmetic_ctx};
use equaio::address;
use equaio::algebra;
use equaio::expression::{Address, expression_builder as eb};
use equaio::vec_strings;

fn get_algebra_rules() -> RuleMap {
    let filepath = "rules/algebra.json";
    let rulestr = std::fs::read_to_string(filepath).unwrap();
    return rule::parse_rulemap_from_json(&rulestr).unwrap();
}
fn init_algebra_worksheet(variables: Vec<String>) -> Worksheet {
    let mut ws = Worksheet::new();
    let ctx = get_arithmetic_ctx().add_params(variables);
    ws.set_expression_context(ctx);
    ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
    ws.set_rule_map(get_algebra_rules());
    ws.set_get_possible_actions_function(|expr,ctx,addr_vec| 
        algebra::get_possible_actions::algebra(expr,ctx,addr_vec));
    return ws;
}

#[cfg(test)]
mod algebra_test {
    use super::*;
    
    #[test]
    fn simple() {
        // solve 2*x - 1 = 3
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        ws.introduce_expression(parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap());
        
        let mut seq0 = ws.get_workable_expression_sequence(0).unwrap();
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Add, &eb::constant("1"));
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![1]);
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![0].sub(1));
        assert!(status);
        let status = seq0.apply_rule_at("algebra/add_zero/0", &address![0]);
        assert!(status);
        let status = seq0.apply_simple_arithmetic_to_both_side(arithmetic::ArithmeticOperator::Div, &eb::constant("2"));
        assert!(status);
        let status = seq0.do_arithmetic_calculation_at(&address![1]);
        assert!(status);
        let status = seq0.apply_fraction_arithmetic_at(0,0, &address![0]);
        assert!(status);
        let status = seq0.apply_rule_at("algebra/div_one", &address![0]);
        assert!(status);
        let status = seq0.apply_rule_at("algebra/mul_one/1", &address![0]);
        assert!(status);
        
        let target = [
            ("Introduce", "(((2 * x) - 1) = 3)"),
            ("Apply +1 to both side", "(((2 * x) + (-1) + 1) = (3 + 1))"),
            ("Calculate 3 + 1 = 4", "(((2 * x) + (-1) + 1) = 4)"),
            ("Calculate -1 + 1 = 0", "(((2 * x) + 0) = 4)"),
            ("Addition with 0", "((2 * x) = 4)"),
            ("Apply /2 to both side", "(((2 * x) / 2) = (4 / 2))"),
            ("Calculate 4 / 2 = 2", "(((2 * x) / 2) = 2)"),
            ("Simplify fraction", "(((1 * x) / 1) = 2)"),
            ("Division by 1", "((1 * x) = 2)"),
            ("Multiplication with 1", "(x = 2)"),
        ];
        assert_eq!(seq0.history.len(), target.len());
        for (i, (target_action_str, target_expr_str)) in target.iter().enumerate() {
            let (action, expr, _) = seq0.history.get(i).unwrap();
            assert_eq!(action.to_string(), target_action_str.to_string());
            assert_eq!(expr.to_string(true), target_expr_str.to_string());
        }
    }
}

#[cfg(test)]
mod get_possible_actions {
    use super::*;
    
    #[test]
    fn arithmetic_both_side_given_inner() {
        // x = 3 / (1-x)
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        let expr = parser_prefix::to_expression("=(x,/(3,-(1,x)))", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr.clone());
        
        let seq0 = ws.get_workable_expression_sequence(0).unwrap();
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
    fn swap_comutative_binary_ops() {
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        let expr = parser::to_expression("1 + x = 3", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr);
        
        let seq0 = ws.get_workable_expression_sequence(0).unwrap();
        let actions = seq0.get_possible_actions(&vec![address![0,0], address![0,1]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Reorder".to_string()));
        assert_eq!(actions[0].1.to_string(true), "((x + 1) = 3)");
        
        // nested
        let expr = parser::to_expression("(1 * x) + (x * 2) = 3", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr);
        
        let seq1 = ws.get_workable_expression_sequence(1).unwrap();
        let actions = seq1.get_possible_actions(&vec![address![0,0,1], address![0,1,0]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Reorder".to_string()));
        assert_eq!(actions[0].1.to_string(true), "(((x * 2) + (1 * x)) = 3)");
    }
    
    #[test]
    fn swap_comutative_assoc_train() {
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        let expr = parser::to_expression("1 + x + 2 + 4 = 3", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr);
        
        let seq0 = ws.get_workable_expression_sequence(0).unwrap();
        let actions = seq0.get_possible_actions(&vec![address![0,0], address![0,2]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Reorder".to_string()));
        assert_eq!(actions[0].1.to_string(true), "((2 + x + 1 + 4) = 3)");
        
        // nested
        let expr = parser::to_expression("(1 * x) + (x * 2) + (5 * 6) = 3", &ws.get_expression_context()).unwrap();
        ws.introduce_expression(expr);
        
        let seq1 = ws.get_workable_expression_sequence(1).unwrap();
        let actions = seq1.get_possible_actions(&vec![address![0,1,1], address![0,2,0]]);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].0, Action::ApplyAction("Reorder".to_string()));
        assert_eq!(actions[0].1.to_string(true), "(((1 * x) + (5 * 6) + (x * 2)) = 3)");
    }
    
    #[test]
    fn simple() {
        // solve 2*x - 1 = 3
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        ws.introduce_expression(parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap());
        
        let mut seq0 = ws.get_workable_expression_sequence(0).unwrap();
        assert!(seq0.try_apply_action_by_index(&vec![address![], address![0,1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0].sub(1)], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![], address![0,0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0,0,0], address![0,1]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq0.try_apply_action_by_index(&vec![address![0]], 0));
        
        let target = [
            ("Introduce", "(((2 * x) - 1) = 3)"),
            ("Apply +1 to both side", "(((2 * x) + (-1) + 1) = (3 + 1))"),
            ("Calculate 3 + 1 = 4", "(((2 * x) + (-1) + 1) = 4)"),
            ("Calculate -1 + 1 = 0", "(((2 * x) + 0) = 4)"),
            ("Addition with 0", "((2 * x) = 4)"),
            ("Apply /2 to both side", "(((2 * x) / 2) = (4 / 2))"),
            ("Calculate 4 / 2 = 2", "(((2 * x) / 2) = 2)"),
            ("Simplify fraction", "(((1 * x) / 1) = 2)"),
            ("Division by 1", "((1 * x) = 2)"),
            ("Multiplication with 1", "(x = 2)"),
        ];
        assert_eq!(seq0.history.len(), target.len());
        for (i, (target_action_str, target_expr_str)) in target.iter().enumerate() {
            let (action, expr, _) = seq0.history.get(i).unwrap();
            assert_eq!(action.to_string(), target_action_str.to_string());
            assert_eq!(expr.to_string(true), target_expr_str.to_string());
        }
    }
}

#[cfg(test)]
mod multiple_equation {
    use super::*;
    
    fn seq_eq(seq: &WorkableExpressionSequence, target: &[(&str, &str, &str)]) {
        assert_eq!(seq.history.len(), target.len());
        for (i, (target_action_str, target_expr_str, target_label_str)) in target.iter().enumerate() {
            let (action, expr, label) = seq.history.get(i).unwrap();
            if !target_action_str.is_empty() {
                assert_eq!(action.to_string(), target_action_str.to_string());
            }
            if !target_label_str.is_empty() {
                assert_eq!(label.clone().unwrap(), target_label_str.to_string());
            } else {
                assert!(label.is_none());
            }
            assert_eq!(expr.to_string(true), target_expr_str.to_string());
        }
    }
    
    #[test]
    fn sletv_substitution() {
        // x = 2 // y = 1
        // x + y = 3
        // x - y = 1
        
        let mut ws = init_algebra_worksheet(vec_strings!["x","y"]);
        ws.introduce_expression(parser::to_expression("x + y = 3", &ws.get_expression_context()).unwrap());
        ws.introduce_expression(parser::to_expression("x - y = 1", &ws.get_expression_context()).unwrap());
        
        let mut seq1 = ws.get_workable_expression_sequence(0).unwrap();
        assert!(seq1.try_apply_action_by_index(&vec![address![], address![0,1]], 0));
        assert!(seq1.try_apply_action_by_index(&vec![address![0].sub(1)], 0));
        assert!(seq1.try_apply_action_by_index(&vec![address![0]], 0));
        seq1.label_expression("Eq. 1".to_string(), 3);
        let target1 = [
            ("Introduce", "((x + y) = 3)", ""),
            ("Apply -y to both side", "((x + y + (-y)) = (3 - y))", ""),
            ("Self subtraction", "((x + 0) = (3 - y))", ""),
            ("Addition with 0", "(x = (3 - y))", "Eq. 1"),
        ];
        seq_eq(&seq1, &target1);
        ws.store(0, seq1);
        
        let mut seq2 = ws.get_workable_expression_sequence(1).unwrap();
        assert!(seq2.try_apply_action_by_index(&vec![address![0,0]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0].sub(1)], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0,1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![], address![0,1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0].sub(1)], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![1,0], address![1,1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![], address![1,1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![1].sub(1)], 1));
        assert!(seq2.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![], address![0,0]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0,0,0],address![0,1]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0]], 0));
        assert!(seq2.try_apply_action_by_index(&vec![address![0]], 0));
        seq2.label_expression("Eq. 2".to_string(), 17);
        let target2 = [
            ("Introduce", "((x - y) = 1)", ""),
            ("Substitute from Eq. 1", "((3 + (-y) + (-y)) = 1)", ""),
            ("Self addition", "((3 + (2 * (-y))) = 1)", ""),
            ("Factor out the minus sign", "((3 - (2 * y)) = 1)", ""),
            ("Apply +(2 * y) to both side", "((3 + (-(2 * y)) + (2 * y)) = (1 + (2 * y)))", ""),
            ("Self subtraction", "((3 + 0) = (1 + (2 * y)))", ""),
            ("Calculate 3 + 0 = 3", "(3 = (1 + (2 * y)))", ""),
            ("Reorder", "(3 = ((2 * y) + 1))", ""),
            ("Apply -1 to both side", "((3 - 1) = ((2 * y) + 1 + (-1)))", ""),
            ("Self subtraction", "((3 - 1) = ((2 * y) + 0))", ""),
            ("Addition with 0", "((3 - 1) = (2 * y))", ""),
            ("Calculate 3 - 1 = 2", "(2 = (2 * y))", ""),
            ("Flip the equation", "((2 * y) = 2)", ""),
            ("Apply /2 to both side", "(((2 * y) / 2) = (2 / 2))", ""),
            ("Calculate 2 / 2 = 1", "(((2 * y) / 2) = 1)", ""),
            ("Simplify fraction", "(((1 * y) / 1) = 1)", ""),
            ("Division by 1", "((1 * y) = 1)", ""),
            ("Multiplication with 1", "(y = 1)", "Eq. 2"),
        ];
        seq_eq(&seq2, &target2);
        ws.store(1, seq2);
        
        ws.introduce_from_label("Eq. 1");
        let mut seq3 = ws.get_workable_expression_sequence(2).unwrap();
        assert!(seq3.try_apply_action_by_index(&vec![address![1,1]], 0));
        assert!(seq3.try_apply_action_by_index(&vec![address![1]], 0));
        let target3 = [
            ("Introduce from Eq. 1", "(x = (3 - y))", ""),
            ("Substitute from Eq. 2", "(x = (3 - 1))", ""),
            ("Calculate 3 - 1 = 2", "(x = 2)", ""),
        ];
        seq_eq(&seq3, &target3);
    }
    
}