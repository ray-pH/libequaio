use equaio::parser::{parser, parser_prefix};
use equaio::expression as exp;
use equaio::vec_strings;

#[cfg(test)]
mod parsing {
    use super::*;
    
    #[test]
    fn simple() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "((a + b) = c)";
        let expr0 = parser_prefix::to_expression("=(+(a,b),c)", &ctx).unwrap();
        let expr1 = parser::to_expression_raw("a + b = c", &ctx).unwrap();
        let expr2 = parser::to_expression_raw("(a + b) = (c)", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0, expr2);
        assert_eq!(expr0.to_string(true), expected);
        assert!(expr0.is_statement());
    }
    
    #[test]
    fn preprocess_equation() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "(a = (b + c))";
        let expr0 = parser_prefix::to_expression("=(a,+(b,c))", &ctx).unwrap();
        let expr1 = parser::to_expression("a = b + c", &ctx).unwrap();
        let expr_raw = parser::to_expression_raw("a = b + c", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
        assert_eq!(expr_raw.to_string(true), "((a = b) + c)");
        assert!(expr0.is_statement());
        assert!(expr1.is_statement());
        assert!(!expr_raw.is_statement());
    }
    
    #[test]
    fn preprocess_statement() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "((a = b) => (b = a))";
        let expr0 = parser_prefix::to_expression("=>(=(a,b),=(b,a))", &ctx).unwrap();
        let expr1 = parser::to_expression("a = b => b = a", &ctx).unwrap();
        let expr_raw = parser::to_expression_raw("a = b => b = a", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
        assert_eq!(expr_raw.to_string(true), "(((a = b) => b) = a)");
        assert!(expr0.is_statement());
        assert!(expr1.is_statement());
    }
    
    #[test]
    fn assoc_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "(a + b + c + d)";
        let expr0 = parser_prefix::to_expression("+(a,b,c,d)", &ctx).unwrap();
        let expr1 = parser::to_expression("a + b + c + d", &ctx).unwrap();
        let expr_raw = parser::to_expression_raw("a + b + c + d", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
        assert_eq!(expr_raw.to_string(true), "(((a + b) + c) + d)");
    }
    
    #[test]
    fn with_function() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "(a + f(b))";
        let expr0 = parser_prefix::to_expression("+(a,f(b))", &ctx).unwrap();
        let expr1 = parser::to_expression("a + f(b)", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
    }
    
    #[test]
    fn with_function_in_front() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expected = "(f(a) + g(b))";
        let expr0 = parser_prefix::to_expression("+(f(a),g(b))", &ctx).unwrap();
        let expr1 = parser::to_expression("f(a) + g(b)", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
    }
    
    #[test]
    fn simple_unary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a"],
            unary_ops: vec_strings!["-"],
            ..Default::default()
        };
        
        let expected = "(-a)";
        let expr0 = parser_prefix::to_expression("-(a)", &ctx).unwrap();
        let expr1 = parser::to_expression("- a", &ctx).unwrap();
        // assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
    }
    
    #[test]
    fn unary() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d"],
            binary_ops: vec_strings!["+"],
            unary_ops: vec_strings!["-"],
            ..Default::default()
        };
        
        let expected = "((-a) + (-b))";
        let expr0 = parser_prefix::to_expression("+(-(a),-(b))", &ctx).unwrap();
        let expr1 = parser::to_expression("-a + -(b)", &ctx).unwrap();
        // assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
    }
    
    #[test]
    fn multiple_operator() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        
        let expected = "(a * (b + c))";
        let expr0 = parser_prefix::to_expression("*(a,+(b,c))", &ctx).unwrap();
        let expr1 = parser::to_expression("a * (b + c)", &ctx).unwrap();
        let expr2 = parser::to_expression("(a * (b + c))", &ctx).unwrap();
        assert_eq!(expr0, expr1);
        assert_eq!(expr0, expr2);
        assert_eq!(expr0.to_string(true), expected);
        // assert_eq!(expr1.to_string(true), expected);
        // assert_eq!(expr2.to_string(true), expected);
    }
    
    #[test]
    fn multiple_operator2() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        
        let expected = "((a * (b + c)) = ((a * b) + (a * c)))";
        let expr0 = parser_prefix::to_expression("=(*(a,+(b,c)),+(*(a,b),*(a,c)))", &ctx).unwrap();
        let expr1 = parser::to_expression("a * (b + c) = (a * b) + (a * c)", &ctx).unwrap();
        // assert_eq!(expr0, expr1);
        assert_eq!(expr0.to_string(true), expected);
        assert_eq!(expr1.to_string(true), expected);
    }
}