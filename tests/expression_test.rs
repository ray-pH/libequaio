use equaio::expression as exp;
use equaio::vec_strings;
use equaio::parser_prefix;

fn print_matches(matches : Vec<(exp::Address,exp::MatchMap)>) {
    for (address, map) in matches {
        println!("Match at address {:?}", address);
        for (k,v) in map {
            println!("{} -> {}", k, v.to_string(true));
        }
        println!("");
    }
}

#[cfg(test)]
mod basic {
    use super::*;
    #[test]
    fn is_statement() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+", "*"],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("=(a,+(b,c))", ctx.clone()).unwrap();
        assert_eq!(expr.to_string(true), "(a = (b + c))");
        assert!(expr.is_statement());
    }
}

#[cfg(test)]
mod pattern_matching {
    use super::*;

    fn expr_pattern_match(expr : &str, pattern : &str) -> Vec<(exp::Address,exp::MatchMap)> {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y"],
            unary_ops: vec_strings!["+", "-"],
            binary_ops: vec_strings!["+", "-", "*", "/"],
            handle_numerics: true,
        };
        let expr = parser_prefix::to_expression(expr, ctx.clone()).unwrap();
        let pattern = parser_prefix::to_expression(pattern, ctx.clone()).unwrap();
        println!("matching {} with {}", expr.to_string(true), pattern.to_string(true));
        expr.get_pattern_matches(&pattern)
    }

    #[test]
    fn simple_matching() {
        let matches = expr_pattern_match("+(x,y)", "+(A,B)");
        print_matches(matches.clone());
        // make sure we have only one match
        assert_eq!(matches.len(), 1);
        // make sure in the match we have the correct values
        // A -> x, B -> y
        let (address, map) = &matches[0];
        assert_eq!(address.len(), 0);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get("A").unwrap().to_string(true), "x");
        assert_eq!(map.get("B").unwrap().to_string(true), "y");
    }

    #[test]
    fn two_matches() {
        let matches = expr_pattern_match("+(0,+(x,f(2,4)))", "+(A,B)");
        print_matches(matches.clone());
        assert_eq!(matches.len(), 2);
        let (address0, map0) = &matches[0];
        assert_eq!(address0.len(), 0);
        assert_eq!(map0.len(), 2);
        assert_eq!(map0.get("A").unwrap().to_string(true), "0");
        assert_eq!(map0.get("B").unwrap().to_string(true), "(x + f(2, 4))");

        let (address1, map1) = &matches[1];
        assert_eq!(address1, &vec![1]);
        assert_eq!(map1.len(), 2);
        assert_eq!(map1.get("A").unwrap().to_string(true), "x");
        assert_eq!(map1.get("B").unwrap().to_string(true), "f(2, 4)");
    }

    #[test]
    fn with_const_param() {
        let matches = expr_pattern_match("+(0,+(x,f(2,4)))", "+(x,B)");
        print_matches(matches.clone());
        assert_eq!(matches.len(), 1);
        let (address0, map0) = &matches[0];
        assert_eq!(address0, &vec![1]);
        assert_eq!(map0.len(), 1);
        assert_eq!(map0.get("B").unwrap().to_string(true), "f(2, 4)");
    }

}

#[cfg(test)]
mod expression_replacement {
    use super::*;
    
    #[test]
    fn replace_expression() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+", "*"],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("+(a,+(b,c))", ctx.clone()).unwrap();
        let expr_as_replacement = parser_prefix::to_expression("*(b,c)", ctx.clone()).unwrap();
        let new_expr = expr.replace_expression_at(expr_as_replacement.clone(), vec![1]);
        assert_eq!(new_expr.unwrap().to_string(true),"(a + (b * c))");
        
        let new_expr2 = expr.replace_expression_at(expr_as_replacement, vec![1,1]);
        assert_eq!(new_expr2.unwrap().to_string(true),"(a + (b + (b * c)))");
    }
}
