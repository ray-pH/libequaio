use equaio::expression as exp;
use equaio::expression::Address;
use equaio::vec_strings;
use equaio::parser::parser_prefix;
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
            binary_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("=(a,+(b,c))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(a = (b + c))");
        assert!(expr.is_statement());
    }
    
    #[test]
    fn generate_subexpr_from_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,b,c)", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(a + b + c)");
        let subexpr0 = expr.generate_subexpr_from_train(0).unwrap();
        let target0 = parser_prefix::to_expression("+(a,b)", &ctx).unwrap();
        assert_eq!(subexpr0, target0);
        let subexpr1 = expr.generate_subexpr_from_train(1).unwrap();
        let target1 = parser_prefix::to_expression("+(b,c)", &ctx).unwrap();
        assert_eq!(subexpr1, target1);
    }
    
    #[test]
    fn substitute_symbol() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "x"],
            binary_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,+(b,c))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(a + (b + c))");
        let new_expr = expr.substitute_symbol("b".to_string(), "x".to_string());
        assert_eq!(new_expr.to_string(true), "(a + (x + c))");
    }
    
    #[test]
    fn swap_assoc_train_children() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,b,c,d)", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "(a + b + c + d)");
        let new_expr = expr.swap_assoc_train_children(1, 2).unwrap();
        assert_eq!(new_expr.to_string(true), "(a + c + b + d)");
    }
}

#[cfg(test)]
mod assoc_train_normalization {
    use super::*;
    
    fn get_ctx() -> exp::Context {
        return exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e", "f", "g", "h"],
            binary_ops: vec_strings!["+","*","@"],
            assoc_ops: vec_strings!["+","@"],
            ..Default::default()
        };
    }
    
    #[test]
    fn binary_ops_only() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("+(+(a,b),+(c,d))", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("+(a,b,c,d)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn nested_binary_opsy() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("+(+(a,+(b,+(c,d))),+(+(e,f),g))", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("+(a,b,c,d,e,f,g)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn from_binary_and_trains() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("+(+(a,b,c),+(d,+(e,f,g)))", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("+(a,b,c,d,e,f,g)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn mixed() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("+(+(a,b),c,d,*(e,f),+(g,h))", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("+(a,b,c,d,*(e,f),g,h)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn multiple_assoc_op() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("+(+(a,b),c,d,@(e,f,@(g,h)))", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("+(a,b,c,d,@(e,f,g,h))", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn deep1() {
        let ctx = get_ctx();
        let expr = parser_prefix::to_expression("=(+(a,+(b,c)),d)", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("=(+(a,b,c),d)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn deep2() {
        let ctx = get_ctx();
        // let expr = parser_prefix::to_expression("=(+(+(1,2),3),4)", &ctx).unwrap();
        let expr = parser_prefix::to_expression("=(+(+(a,b),c),d)", &ctx).unwrap();
        let normalized_expr = expr.normalize_to_assoc_train(&ctx.assoc_ops);
        let target_expr = parser_prefix::to_expression("=(+(a,b,c),d)", &ctx).unwrap();
        assert_eq!(normalized_expr, target_expr);
    }
    
    #[test]
    fn two_children_assoc_train_to_binary_op(){
        let ctx = get_ctx();
        let mut expr = parser_prefix::to_expression("+(a,b)", &ctx).unwrap();
        expr.exp_type = exp::ExpressionType::AssocTrain;
        assert_eq!(expr.exp_type, exp::ExpressionType::AssocTrain);
        let normalized_expr = expr.normalize_two_children_assoc_train_to_binary_op(&ctx.binary_ops);
        assert_eq!(normalized_expr.exp_type, exp::ExpressionType::OperatorBinary);
    }
    
    #[test]
    fn single_children_assoc_train(){
        let ctx = get_ctx();
        let mut expr = parser_prefix::to_expression("+(a)", &ctx).unwrap();
        expr.exp_type = exp::ExpressionType::AssocTrain;
        assert_eq!(expr.exp_type, exp::ExpressionType::AssocTrain);
        let normalized_expr = expr.normalize_single_children_assoc_train();
        assert!(normalized_expr.is_value());
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
            binary_ops: vec_strings!["+"],
            ..Default::default()
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
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(x,+(y,z))", &ctx).unwrap();
        let pattern = parser_prefix::to_expression("+(A,B)", &ctx).unwrap();
        
        let map0 = expr.pattern_match_at(&pattern, &address![]).unwrap();
        assert_eq!(map0.get("A").unwrap().to_string(true), "x");
        assert_eq!(map0.get("B").unwrap().to_string(true), "(y + z)");
        let map1 = expr.pattern_match_at(&pattern, &address![0]);
        assert!(map1.is_err());
        let map2 = expr.pattern_match_at(&pattern, &address![1]).unwrap();
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
            ..Default::default()
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
    fn assoc_train() {
        let matches = expr_pattern_match("+(x,y,z)", "+(A,B,C)");
        print_matches(matches.clone());
        assert_eq!(matches.len(), 1);
        
        let (address0, map0) = &matches[0];
        assert_eq!(address0, &address![]);
        assert_eq!(map0.len(), 3);
        assert_eq!(map0.get("A").unwrap().to_string(true), "x");
        assert_eq!(map0.get("B").unwrap().to_string(true), "y");
        assert_eq!(map0.get("C").unwrap().to_string(true), "z");
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
            binary_ops: vec_strings!["+", "*"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,+(b,c))", &ctx).unwrap();
        let expr_as_replacement = parser_prefix::to_expression("*(b,c)", &ctx).unwrap();
        let new_expr = expr.replace_expression_at(expr_as_replacement.clone(), &address![1]);
        assert_eq!(new_expr.unwrap().to_string(true),"(a + (b * c))");
        
        let new_expr2 = expr.replace_expression_at(expr_as_replacement, &address![1,1]);
        assert_eq!(new_expr2.unwrap().to_string(true),"(a + (b + (b * c)))");
    }
    
    #[test]
    fn replace_expression_on_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "e"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(a,b,c)", &ctx).unwrap();
        let expr_as_replacement = parser_prefix::to_expression("*(d,e)", &ctx).unwrap();
        let new_expr0 = expr.replace_expression_at(expr_as_replacement.clone(), &address![].sub(0)).unwrap();
        assert_eq!(new_expr0.to_string(true),"((d * e) + c)");
        
        let new_expr1 = expr.replace_expression_at(expr_as_replacement.clone(), &address![].sub(1)).unwrap();
        assert_eq!(new_expr1.to_string(true),"(a + (d * e))");
        
        let new_expr2 = expr.replace_expression_at(expr_as_replacement.clone(), &address![].sub(2));
        assert!(new_expr2.is_err());
    }
}

#[cfg(test)]
mod apply_equation {
    use super::*;
    
    #[test]
    fn generate_eq_from_match_map() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let lhs = rule_eq.clone().children.unwrap()[0].clone();
        let match_map = expr.pattern_match_this_node(&lhs).unwrap();
        assert_eq!(match_map.get("X").unwrap().to_string(true), "a");
        let applied_eq = rule_eq.apply_match_map(&match_map);
        assert_eq!(applied_eq.to_string(true), "((a + 0) = a)");
        let new_expr = expr.apply_equation_ltr_this_node(&applied_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "a");
    }
    
    #[test]
    fn generate_eq_from_match_map_deep() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(+(a,0),b)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let lhs = rule_eq.clone().children.unwrap()[0].clone();
        let match_map = expr.pattern_match_at(&lhs, &address![0]).unwrap();
        assert_eq!(match_map.get("X").unwrap().to_string(true), "a");
        let applied_eq = rule_eq.apply_match_map(&match_map);
        assert_eq!(applied_eq.to_string(true), "((a + 0) = a)");
        let new_expr = expr.apply_equation_ltr_at(&applied_eq, &address![0]).unwrap();
        assert_eq!(new_expr.to_string(true), "(a + b)");
    }
    
    #[test]
    fn simple_equation() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_ltr_this_node(&rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "a");
    }
    
    #[test]
    fn simple_equation_rtl() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "0"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(a,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_rtl_this_node(&rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "((a + 0) + 0)");
    }
    
    #[test]
    fn on_train() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(a,b,0)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let new_expr = expr.apply_equation_at(&rule_eq, &address![].sub(1)).unwrap();
        assert_eq!(new_expr.to_string(true), "(a + b)");
    }
    
    #[test]
    fn other() {
        let ctx = exp::Context {
            parameters: vec_strings![""],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            handle_numerics: true,
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("=(+(X,0),X)", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(+(A,B),+(B,A))", &ctx).unwrap();
        let new_expr = expr.apply_equation_at(&rule_eq, &address![0]).unwrap();
        assert_eq!(new_expr.to_string(true), "((0 + X) = X)");
    }
}

#[cfg(test)]
mod apply_implication {
    use super::*;
    
    #[test]
    fn simple_implication() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "0"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("=(+(a,b),a)", &ctx).unwrap();
        let rule = parser_prefix::to_expression("=>( =(+(X,Y),X), =(Y,0))", &ctx).unwrap();
        assert_eq!(expr.to_string(true), "((a + b) = a)");
        assert_eq!(rule.to_string(true), "(((X + Y) = X) => (Y = 0))");
        let new_expr = expr.apply_implication(&rule).unwrap();
        assert_eq!(new_expr.to_string(true), "(b = 0)");
    }
}

#[cfg(test)]
mod equivalence {
    use super::*;
    
    #[test]
    fn simple() {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y"],
            binary_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr1 = parser_prefix::to_expression("+(x,y)", &ctx).unwrap();
        let expr2 = parser_prefix::to_expression("+(y,x)", &ctx).unwrap();
        assert!(!expr1.is_equivalent_to(&expr2));
        
        let expr1 = parser_prefix::to_expression("+(A,B)", &ctx).unwrap();
        let expr2 = parser_prefix::to_expression("+(B,C)", &ctx).unwrap();
        assert!(expr1.is_equivalent_to(&expr2));
        
        let expr1 = parser_prefix::to_expression("*(+(A,B),x)", &ctx).unwrap();
        let expr2 = parser_prefix::to_expression("*(+(B,C),x)", &ctx).unwrap();
        assert!(expr1.is_equivalent_to(&expr2));
        
        let expr1 = parser_prefix::to_expression("*(+(A,x),B)", &ctx).unwrap();
        let expr2 = parser_prefix::to_expression("*(+(B,x),C)", &ctx).unwrap();
        assert!(expr1.is_equivalent_to(&expr2));
        
        let expr1 = parser_prefix::to_expression("*(+(x,A),B)", &ctx).unwrap();
        let expr2 = parser_prefix::to_expression("*(+(B,x),C)", &ctx).unwrap();
        assert!(!expr1.is_equivalent_to(&expr2));
    }
}

#[cfg(test)]
mod variadic {
    use exp::ExpressionType;

    use super::*;
    
    #[test]
    fn parse_and_string() {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("+(...(A))", &ctx).unwrap();
        assert_eq!(expr.at(&address![0]).unwrap().exp_type, ExpressionType::Variadic);
        assert_eq!(expr.to_string(true), "(A_1 + A_2 + ...)");
        
        let expr = parser_prefix::to_expression("+(...(*(A,x)))", &ctx).unwrap();
        assert_eq!(expr.at(&address![0]).unwrap().exp_type, ExpressionType::Variadic);
        assert_eq!(expr.to_string(true), "((A_1 * x) + (A_2 * x) + ...)");
        
        let expr = parser_prefix::to_expression("+(...(*(A,B)))", &ctx).unwrap();
        assert_eq!(expr.at(&address![0]).unwrap().exp_type, ExpressionType::Variadic);
        assert_eq!(expr.to_string(true), "((A_1 * B_1) + (A_2 * B_2) + ...)");
    }
    
    #[test]
    fn pattern_match() {
        let ctx = exp::Context {
            parameters: vec_strings!["x", "y"],
            binary_ops: vec_strings!["+"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        let expr = parser_prefix::to_expression("+(x,y,z)", &ctx).unwrap();
        let pattern = parser_prefix::to_expression("+(...(A))", &ctx).unwrap();
        let map = expr.pattern_match_this_node(&pattern).unwrap();
        assert_eq!(map.len(), 4);
        assert_eq!(map.get("...").unwrap().to_string(true), "3");
        assert_eq!(map.get("A_1").unwrap().to_string(true), "x");
        assert_eq!(map.get("A_2").unwrap().to_string(true), "y");
        assert_eq!(map.get("A_3").unwrap().to_string(true), "z");
    }
    
    #[test]
    fn apply_match_map() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "x"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("*(x,+(a,b,c))", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(*(X,+(...(A))),+(...(*(X,A))))", &ctx).unwrap();
        
        let eq_children = rule_eq.children.as_ref().unwrap();
        let lhs = &eq_children[0];
        let match_map = expr.pattern_match_this_node(lhs).unwrap();
        let equation = rule_eq.apply_match_map(&match_map);
        assert_eq!(equation.to_string(true), "((x * (a + b + c)) = ((x * a) + (x * b) + (x * c)))");
    }
    
    #[test]
    fn apply_equation() {
        let ctx = exp::Context {
            parameters: vec_strings!["a", "b", "c", "d", "x"],
            binary_ops: vec_strings!["+", "*"],
            assoc_ops: vec_strings!["+"],
            ..Default::default()
        };
        
        let expr = parser_prefix::to_expression("*(x,+(a,b,c))", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(*(X,+(...(A))),+(...(*(X,A))))", &ctx).unwrap();
        let new_expr = expr.apply_equation_ltr_this_node(&rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "((x * a) + (x * b) + (x * c))");
        
        let expr = parser_prefix::to_expression("*(x,+(a,b,c,d))", &ctx).unwrap();
        let rule_eq = parser_prefix::to_expression("=(*(X,+(...(A))),+(...(*(X,A))))", &ctx).unwrap();
        let new_expr = expr.apply_equation_ltr_this_node(&rule_eq).unwrap();
        assert_eq!(new_expr.to_string(true), "((x * a) + (x * b) + (x * c) + (x * d))");
    }
}
