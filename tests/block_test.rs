use equaio::expression as exp;
use equaio::expression::Address;
use equaio::{vec_strings, vec_index_map};
use equaio::parser::{parser_prefix, parser};
use equaio::address;
use equaio::algebra;
use equaio::arithmetic::get_arithmetic_ctx;
use equaio::block::{Block, BlockContext, block_builder};
use equaio::worksheet::Worksheet;
use std::collections::HashMap;

#[cfg(test)]
mod simple_block {
    use super::*;
    use block_builder as bb;
    use equaio::{pair_map, rule::{self, RuleMap}};
    
    #[test]
    fn unary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a"],
            unary_ops: vec_strings!["-"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("-(a)", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("-".to_string(), address![]),
            bb::symbol("a".to_string(), address![0]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn binary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,b)", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("a".to_string(), address![0]),
            bb::symbol("+".to_string(), address![]),
            bb::symbol("b".to_string(), address![1]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    
    #[test]
    fn assoc_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,b,c,d,e)", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("a".to_string(), address![0]),
            bb::symbol("+".to_string(), address![].sub(0)),
            bb::symbol("b".to_string(), address![1]),
            bb::symbol("+".to_string(), address![].sub(1)),
            bb::symbol("c".to_string(), address![2]),
            bb::symbol("+".to_string(), address![].sub(2)),
            bb::symbol("d".to_string(), address![3]),
            bb::symbol("+".to_string(), address![].sub(3)),
            bb::symbol("e".to_string(), address![4]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn assoc_train_with_inverse() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e"],
            unary_ops: vec_strings!["-"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let block_ctx = BlockContext {
            inverse_ops: pair_map![("+","-")],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(-(a),b,c,-(d),-(e))", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("-".to_string(), address![0]),
            bb::symbol("a".to_string(), address![0,0]),
            bb::symbol("+".to_string(), address![].sub(0)),
            bb::symbol("b".to_string(), address![1]),
            bb::symbol("+".to_string(), address![].sub(1)),
            bb::symbol("c".to_string(), address![2]),
            bb::symbol("-".to_string(), address![].sub(2)),
            bb::symbol("d".to_string(), address![3,0]),
            bb::symbol("-".to_string(), address![].sub(3)),
            bb::symbol("e".to_string(), address![4,0]),
        ], address![]);
        
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn assoc_train_with_inverse_complex() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e"],
            unary_ops: vec_strings!["-"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let block_ctx = BlockContext {
            inverse_ops: pair_map![("+","-")],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(-(a),b,-(*(c,d)),e)", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("-".to_string(), address![0]),
            bb::symbol("a".to_string(), address![0,0]),
            bb::symbol("+".to_string(), address![].sub(0)),
            bb::symbol("b".to_string(), address![1]),
            bb::symbol("-".to_string(), address![].sub(1)),
            bb::horizontal_container(vec![
                bb::symbol("c".to_string(), address![2,0,0]),
                bb::symbol("*".to_string(), address![2,0]),
                bb::symbol("d".to_string(), address![2,0,1]),
            ], address![2,0]).parenthesis(),
            bb::symbol("+".to_string(), address![].sub(2)),
            bb::symbol("e".to_string(), address![3]),
        ], address![]);
        
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn nested_binary(){
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["*"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("*(*(a,b),c)", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::symbol("a".to_string(), address![0,0]),
                bb::symbol("*".to_string(), address![0]),
                bb::symbol("b".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("*".to_string(), address![]),
            bb::symbol("c".to_string(), address![1]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn nested_binary_assoc(){
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("*(*(a,b),+(c,*(e,f),d))", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::symbol("a".to_string(), address![0,0]),
                bb::symbol("*".to_string(), address![0]),
                bb::symbol("b".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("*".to_string(), address![]),
            bb::horizontal_container(vec![
                bb::symbol("c".to_string(), address![1,0]),
                bb::symbol("+".to_string(), address![1].sub(0)),
                bb::horizontal_container(vec![
                    bb::symbol("e".to_string(), address![1,1,0]),
                    bb::symbol("*".to_string(), address![1,1]),
                    bb::symbol("f".to_string(), address![1,1,1]),
                ], address![1,1]),
                bb::symbol("+".to_string(), address![1].sub(1)),
                bb::symbol("d".to_string(), address![1,2]),
            ], address![1]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn fraction() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+", "/"],
            ..Default::default()
        };
        let block_ctx = BlockContext {
            fraction_ops: vec_strings!["/"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,/(+(a,b),c))", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::symbol("a".to_string(), address![0]),
            bb::symbol("+".to_string(), address![]),
            bb::fraction_container(vec![
                bb::horizontal_container(vec![
                    bb::symbol("a".to_string(), address![1,0,0]),
                    bb::symbol("+".to_string(), address![1,0]),
                    bb::symbol("b".to_string(), address![1,0,1]),
                ], address![1,0]),
                bb::symbol("c".to_string(), address![1,1]),
            ], address![1])
        ], address![]);
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn algebra_with_number(){
        let ctx = get_arithmetic_ctx().add_param("x".to_string());
        let expr = parser_prefix::to_expression("=(-(*(2,x),1),3)", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(((2 * x) - 1) = 3)");
        let block = Block::from_root_expression(&expr, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::horizontal_container(vec![
                    bb::symbol("2".to_string(), address![0,0,0]),
                    bb::symbol("*".to_string(), address![0,0]),
                    bb::symbol("x".to_string(), address![0,0,1]),
                ], address![0,0]),
                bb::symbol("-".to_string(), address![0]),
                bb::symbol("1".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("=".to_string(), address![]),
            bb::symbol("3".to_string(), address![1]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn conceal_multiplication() {
        let ctx = exp::Context {
            parameters: vec_strings!["1", "2", "x"],
            binary_ops: vec_strings!["*", "+"],
            ..Default::default()
        };
        let block_ctx = BlockContext {
            conceal_ops: vec_strings!["*"],
            ..Default::default()
        };
        let expr = parser::to_expression("(2 * x) + 1", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::symbol("2".to_string(), address![0,0]),
                bb::symbol("x".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("+".to_string(), address![]),
            bb::symbol("1".to_string(), address![1]),
        ], address![]);
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn operator_presedence() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        let block_ctx = BlockContext {
            op_precedence: vec_index_map!["+", "*"],
            ..Default::default()
        };
        let expr = parser::to_expression("(a * b) + c", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::symbol("a".to_string(), address![0,0]),
                bb::symbol("*".to_string(), address![0]),
                bb::symbol("b".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("+".to_string(), address![]),
            bb::symbol("c".to_string(), address![1]),
        ], address![]);
        
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
        
        let expr = parser::to_expression("(a + b) * c", &ctx).unwrap();
        let block = Block::from_root_expression(&expr, &block_ctx);
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::symbol("a".to_string(), address![0,0]),
                bb::symbol("+".to_string(), address![0]),
                bb::symbol("b".to_string(), address![0,1]),
            ], address![0]).parenthesis(),
            bb::symbol("*".to_string(), address![]),
            bb::symbol("c".to_string(), address![1]),
        ], address![]);
        
        print_block_tree(&block);
        print_block_tree(&expected_block);
        assert_eq!(block, expected_block);
    }
    
    #[test]
    fn form_worksheet(){
        fn get_algebra_rules() -> RuleMap {
            let filepath = "rules/algebra.json";
            let rulestr = std::fs::read_to_string(filepath).unwrap();
            let ruleset = rule::parse_ruleset_from_json(&rulestr);
            return ruleset.unwrap().get_rule_map();
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
        
        let mut ws = init_algebra_worksheet(vec_strings!["x"]);
        let expr = parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap();
        assert_eq!(expr.clone().to_string(true), "(((2 * x) - 1) = 3)");
        ws.introduce_expression(expr.clone());
        let seq0 = ws.get_workable_expression_sequence(0).unwrap();
        let expr0 = seq0.last_expression();
        assert_eq!(expr0.to_string(true), "(((2 * x) - 1) = 3)");
        let block = Block::from_root_expression(&expr0, &BlockContext::default());
        let expected_block = bb::horizontal_container(vec![
            bb::horizontal_container(vec![
                bb::horizontal_container(vec![
                    bb::symbol("2".to_string(), address![0,0,0]),
                    bb::symbol("*".to_string(), address![0,0]),
                    bb::symbol("x".to_string(), address![0,0,1]),
                ], address![0,0]),
                bb::symbol("-".to_string(), address![0]),
                bb::symbol("1".to_string(), address![0,1]),
            ], address![0]),
            bb::symbol("=".to_string(), address![]),
            bb::symbol("3".to_string(), address![1]),
        ], address![]);
        assert_eq!(block, expected_block);
    }
    
}

fn print_block_tree(block: &Block) {
    f_print_block_tree(block, 0);
}
fn f_print_block_tree(block: &Block, indent: usize) {
    let left_pad = " ".repeat(indent);
    let has_parenthesis_str = if block.has_parenthesis { "[parenthesis]" } else { "" };
    if let Some(children) = &block.children {
        println!("{}{:?} {} {} {{", left_pad, block.block_type, block.address, has_parenthesis_str);
        for child in children {
            f_print_block_tree(child, indent+4);
        }
        println!("{}}}", left_pad);
    } else {
        println!("{}{:?} {} {}", left_pad, block.symbol.clone().unwrap_or("EMPTY".to_string()), block.address, has_parenthesis_str);
    }
}