use equaio::expression as exp;
use equaio::expression::Address;
use equaio::vec_strings;
use equaio::parser_prefix;
use equaio::address;
use equaio::block::{Block, block_builder};

#[cfg(test)]
mod simple_block {
    use super::*;
    use block_builder as bb;
    
    #[test]
    fn unary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a"],
            unary_ops: vec_strings!["-"],
            binary_ops: vec_strings![],
            assoc_ops: vec_strings![],
            handle_numerics: false,
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
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
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
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: false,
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
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["*"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
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
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: false,
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
    
}
