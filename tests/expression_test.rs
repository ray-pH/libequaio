use equaio::expression as exp;
use equaio::expression::Address;
use equaio::vec_strings;
use equaio::parser_prefix;
use equaio::address;

fn print_matches(matches: Vec<(exp::Address,exp::MatchMap)>) {
    for (address, map) in matches {
        println!("Match at address {:?}:{:?}", address.path, address.sub);
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
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("=(a,+(b,c))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(a = (b + c))");
        assert!(expr.is_statement());
    }
    
    #[test]
    fn generate_subexpr_from_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("+(a,b,c)", &ctx).unwrap();
        let subexpr0 = expr.generate_subexpr_from_train(0).unwrap();
        let target0 = parser_prefix::to_expression("+(a,b)", &ctx).unwrap();
        assert_eq!(subexpr0, target0);
        let subexpr1 = expr.generate_subexpr_from_train(1).unwrap();
        let target1 = parser_prefix::to_expression("+(b,c)", &ctx).unwrap();
        assert_eq!(subexpr1, target1);
    }
}

#[cfg(test)]
mod pattern_matching {
    use equaio::address;

    use super::*;
    
    #[test]
    fn pattern_match_at_node() {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: true,
        };
        let expr = parser_prefix::to_expression("+(x,y)", &ctx).unwrap();
        let pattern = parser_prefix::to_expression("+(A,B)", &ctx).unwrap();
        let map = expr.pattern_match_this_node(&pattern).unwrap();
        assert_eq!(map.get("A").unwrap().to_string(true), "x");
        assert_eq!(map.get("B").unwrap().to_string(true), "y");
    }
    
    #[test]
    fn pattern_match_at_address() {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y", "z"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: true,
        };
        let expr = parser_prefix::to_expression("+(x,+(y,z))", &ctx).unwrap();
        let pattern = parser_prefix::to_expression("+(A,B)", &ctx).unwrap();
        
        let map0 = expr.pattern_match_at(&pattern, address![]).unwrap();
        assert_eq!(map0.get("A").unwrap().to_string(true), "x");
        assert_eq!(map0.get("B").unwrap().to_string(true), "(y + z)");
        let map1 = expr.pattern_match_at(&pattern, address![0]);
        assert!(map1.is_none());
        let map2 = expr.pattern_match_at(&pattern, address![1]).unwrap();
        assert_eq!(map2.get("A").unwrap().to_string(true), "y");
        assert_eq!(map2.get("B").unwrap().to_string(true), "z");
    }
    

    fn expr_pattern_match(expr: &str, pattern: &str) -> Vec<(exp::Address,exp::MatchMap)> {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y", "z", "a"],
            unary_ops: vec_strings!["+", "-"],
            binary_ops: vec_strings!["+", "-", "*", "/"],
            assoc_ops: vec_strings!["+", "*"],
            handle_numerics: true,
        };
        let expr = parser_prefix::to_expression(expr, &ctx).unwrap();
        let pattern = parser_prefix::to_expression(pattern, &ctx).unwrap();
        println!("matching {} with {}", expr.to_string(true), pattern.to_string(true));
        return expr.get_pattern_matches(&pattern);
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
        assert_eq!(address, &address![]);
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
        assert_eq!(address0, &address![]);
        assert_eq!(map0.len(), 2);
        assert_eq!(map0.get("A").unwrap().to_string(true), "0");
        assert_eq!(map0.get("B").unwrap().to_string(true), "(x + f(2, 4))");

        let (address1, map1) = &matches[1];
        assert_eq!(address1, &address![1]);
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
        assert_eq!(address0, &address![1]);
        assert_eq!(map0.len(), 1);
        assert_eq!(map0.get("B").unwrap().to_string(true), "f(2, 4)");
    }
    
    #[test]
    fn on_assoc_train() {
        let matches = expr_pattern_match("+(x,y,z)", "+(A,B)");
        print_matches(matches.clone());
        // make sure we have two matches
        assert_eq!(matches.len(), 2);
        
        let (address0, map0) = &matches[0];
        assert_eq!(address0, &address![].sub(0));
        assert_eq!(map0.len(), 2);
        assert_eq!(map0.get("A").unwrap().to_string(true), "x");
        assert_eq!(map0.get("B").unwrap().to_string(true), "y");
        
        let (address1, map1) = &matches[1];
        assert_eq!(address1, &address![].sub(1));
        assert_eq!(map1.len(), 2);
        assert_eq!(map1.get("A").unwrap().to_string(true), "y");
        assert_eq!(map1.get("B").unwrap().to_string(true), "z");
    }
    
    #[test]
    fn on_assoc_train_deep() {
        let matches = expr_pattern_match("+(x,y,+(z,a))", "+(A,B)");
        print_matches(matches.clone());
        // make sure we have two matches
        assert_eq!(matches.len(), 3);
        
        let (address0, map0) = &matches[0];
        assert_eq!(address0, &address![].sub(0));
        assert_eq!(map0.len(), 2);
        assert_eq!(map0.get("A").unwrap().to_string(true), "x");
        assert_eq!(map0.get("B").unwrap().to_string(true), "y");
        
        let (address1, map1) = &matches[1];
        assert_eq!(address1, &address![].sub(1));
        assert_eq!(map1.len(), 2);
        assert_eq!(map1.get("A").unwrap().to_string(true), "y");
        assert_eq!(map1.get("B").unwrap().to_string(true), "(z + a)");
        
        let (address2, map2) = &matches[2];
        assert_eq!(address2, &address![2]);
        assert_eq!(map2.len(), 2);
        assert_eq!(map2.get("A").unwrap().to_string(true), "z");
        assert_eq!(map2.get("B").unwrap().to_string(true), "a");
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
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("+(a,+(b,c))", &ctx).unwrap();
        let expr_as_replacement = parser_prefix::to_expression("*(b,c)", &ctx).unwrap();
        let new_expr = expr.replace_expression_at(expr_as_replacement.clone(), address![1]);
        assert_eq!(new_expr.unwrap().to_string(true),"(a + (b * c))");
        
        let new_expr2 = expr.replace_expression_at(expr_as_replacement, address![1,1]);
        assert_eq!(new_expr2.unwrap().to_string(true),"(a + (b + (b * c)))");
    }
    
    #[test]
    fn replace_expression_on_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: false,
        };
        let expr = parser_prefix::to_expression("+(a,b,c)", &ctx).unwrap();
        let expr_as_replacement = parser_prefix::to_expression("*(d,e)", &ctx).unwrap();
        let new_expr0 = expr.replace_expression_at(expr_as_replacement.clone(), address![].sub(0)).unwrap();
        assert_eq!(new_expr0.to_string(true),"((d * e) + c)");
        
        let new_expr1 = expr.replace_expression_at(expr_as_replacement.clone(), address![].sub(1)).unwrap();
        assert_eq!(new_expr1.to_string(true),"(a + (d * e))");
        
        let new_expr2 = expr.replace_expression_at(expr_as_replacement.clone(), address![].sub(2));
        assert!(new_expr2.is_none());
    }
}

#[cfg(test)]
mod apply_equation {
    use super::*;
    
    #[test]
    fn generate_eq_from_match_map() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let lhs = rule_eq.clone().children.unwrap()[0].clone();
        let match_map = expr.pattern_match_this_node(&lhs).unwrap();
        assert_eq!(match_map.get("X").unwrap().to_string(true), "a");
        let applied_eq = rule_eq.apply_match_map(&match_map);
        assert_eq!(applied_eq.to_string(true), "((a + 0) = a)");
        let new_expr = expr.apply_equation_ltr_this_node(applied_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "a");
    }
    
    #[test]
    fn generate_eq_from_match_map_deep() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("+(+(a,0),b)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let lhs = rule_eq.clone().children.unwrap()[0].clone();
        let match_map = expr.pattern_match_at(&lhs, address![0]).unwrap();
        assert_eq!(match_map.get("X").unwrap().to_string(true), "a");
        let applied_eq = rule_eq.apply_match_map(&match_map);
        assert_eq!(applied_eq.to_string(true), "((a + 0) = a)");
        let new_expr = expr.apply_equation_ltr_at(applied_eq, address![0]).unwrap();
        assert_eq!(new_expr.to_string(true), "(a + b)");
    }
    
    #[test]
    fn simple_equation() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_ltr_this_node(rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "a");
    }
    
    #[test]
    fn simple_equation_rtl() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_rtl_this_node(rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "((a + 0) + 0)");
    }
    
    #[test]
    fn on_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("+(a,b,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_at(rule_eq, address![].sub(1)).unwrap();
        assert_eq!(new_expr.to_string(true), "(a + b)");
    }
}

#[cfg(test)]
mod apply_implication {
    use super::*;
    
    #[test]
    fn simple_implication() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            unary_ops: vec_strings![],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings![],
            handle_numerics: false,
        };
        
        let expr = parser_prefix::to_expression("=(+(a,b),a)", &ctx).unwrap();
        let rule = parser_prefix::to_expression("=>( =(+(X,Y),X), =(Y,0))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "((a + b) = a)");
        assert_eq!(rule.to_string(true), "(((X + Y) = X) => (Y = 0))");
        let new_expr = expr.apply_implication(rule).unwrap();
        assert_eq!(new_expr.to_string(true), "(b = 0)");
    }
}
