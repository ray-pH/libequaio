use equaio::expression as exp;
use equaio::expression::Address;
use equaio::vec_strings;
use equaio::parser_prefix;
use equaio::address;
use equaio::arithmetic::get_arithmetic_ctx;
use equaio::block::{Block, block_builder};
use equaio::worksheet::Worksheet;

#[cfg(test)]
mod simple_block {
    use super::*;
    use block_builder as bb;
    
    #[test]
    fn unary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a"],
            unary_ops: vec_strings!["-"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("-(a)", &ctx).unwrap();
        let block = Block::from(expr);
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
        let block = Block::from(expr);
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
        let block = Block::from(expr);
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
    fn nested_binary(){
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["*"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("*(*(a,b),c)", &ctx).unwrap();
        let block = Block::from(expr);
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
        let block = Block::from(expr);
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
    fn algebra_with_number(){
        let ctx = get_arithmetic_ctx().add_param("x".to_string());
        let expr = parser_prefix::to_expression("=(-(*(2,x),1),3)", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(((2 * x) - 1) = 3)");
        let block = Block::from(expr);
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
    fn form_worksheet(){
        let mut ws = Worksheet::init_algebra_worksheet(vec_strings!["x"]);
        let expr = parser_prefix::to_expression("=(-(*(2,x),1),3)", &ws.get_expression_context()).unwrap();
        assert_eq!(expr.clone().to_string(true), "(((2 * x) - 1) = 3)");
        ws.introduce_expression(expr.clone());
        let seq0 = ws.get_expression_sequence(0).unwrap();
        let expr0 = seq0.last_expression();
        assert_eq!(expr0.to_string(true), "(((2 * x) - 1) = 3)");
        let block = Block::from(expr0.clone());
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
