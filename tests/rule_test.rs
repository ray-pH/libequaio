use::equaio::rule;

fn assert_rule_eq(rule: &rule::Rule, id: &str, label: &str, expr: &str) {
    assert_eq!(rule.id, id);
    assert_eq!(rule.label, label);
    assert_eq!(rule.expression.to_string(true), expr);
}

#[cfg(test)]
mod rule_test {
    use super::*;
    
    #[test]
    fn simple() {
        let str = r#"
        {
            "name": "simple",
            "context": { "base": "arithmetic" },
            "rules": [
                {
                    "id": "rule0",
                    "expr_prefix": "=(1,2)"
                },
                {
                    "id": "rule1",
                    "label": "Rule 1",
                    "expr_prefix": "=(1,3)"
                }
            ]
        }
        "#;
        let rules = rule::parse_ruleset_from_json(str).unwrap();
        
        assert_eq!(rules.len(), 2);
        assert_rule_eq(&rules[0], "simple/rule0", "", "(1 = 2)");
        assert_rule_eq(&rules[1], "simple/rule1", "Rule 1", "(1 = 3)");
    }
    
    #[test]
    fn with_variation() {
        let str = r#"
        {
            "name": "simple",
            "context": { "base": "arithmetic" },
            "variations": [
                {"expr_prefix":  "=(+(A,B),+(B,A))"}
            ],
            "rules": [
                {
                    "id": "rule0",
                    "expr_prefix": "=(+(X,0),X)"
                }
            ]
        }
        "#;
        let rules = rule::parse_ruleset_from_json(str).unwrap();
        assert_eq!(rules.len(), 2);
        assert_rule_eq(&rules[0], "simple/rule0/0", "", "((X + 0) = X)");
        assert_rule_eq(&rules[1], "simple/rule0/1", "", "((0 + X) = X)");
    }
    
    #[test]
    fn with_multiple_variations() {
        // A*(B+C) = A*B + A*C
        // (B+C)*A = A*B + A*C
        let str = r#"
        {
            "name": "simple",
            "context": { "base": "arithmetic" },
            "variations": [
                {"expr_prefix":  "=(+(A,B),+(B,A))"},
                {"expr_prefix":  "=(*(A,B),*(B,A))"}
            ],
            "rules": [
                {
                    "id": "rule0",
                    "expr_prefix": "=(*(A,+(B,C)),+(*(A,B),*(A,C)))"
                }
            ]
        }
        "#;
        let rules = rule::parse_ruleset_from_json(str).unwrap();
        assert_eq!(rules.len(), 2);
        assert_rule_eq(&rules[0], "simple/rule0/0", "", "((A * (B + C)) = ((A * B) + (A * C)))");
        assert_rule_eq(&rules[1], "simple/rule0/1", "", "(((B + C) * A) = ((A * B) + (A * C)))");
    }
}

#[cfg(test)]
mod from_file {
    use super::*;
    
    #[test]
    fn algebra() {
        let filepath = "rules/algebra.json";
        let rulestr = std::fs::read_to_string(filepath).unwrap();
        let rules = rule::parse_ruleset_from_json(&rulestr).unwrap();
        assert_eq!(rules.len(), 19);
        let rule_descriptions = vec![
            ("algebra/add_zero/0", "Addition with 0", "((X + 0) = X)"),
            ("algebra/add_zero/1", "Addition with 0", "((0 + X) = X)"),
            ("algebra/mul_one/0", "Multiplication with 1", "((X * 1) = X)"),
            ("algebra/mul_one/1", "Multiplication with 1", "((1 * X) = X)"),
            ("algebra/mul_zero/0", "Multiplication with 0", "((X * 0) = 0)"),
            ("algebra/mul_zero/1", "Multiplication with 0", "((0 * X) = 0)"),
            ("algebra/sub_zero", "Subtraction by 0", "((X - 0) = X)"),
            ("algebra/div_one", "Division by 1", "((X / 1) = X)"),
            ("algebra/sub_self", "Self subtraction", "((X - X) = 0)"),
            ("algebra/add_negative_self/0", "Self subtraction", "((X + (-X)) = 0)"),
            ( "algebra/add_negative_self/1", "Self subtraction", "(((-X) + X) = 0)"),
            ( "algebra/add_self", "Self addition", "((X + X) = (2 * X))"),
            ( "algebra/distribution/0", "Distribution", "((X * (A + B)) = ((X * A) + (X * B)))"),
            ( "algebra/distribution/1", "Distribution", "(((A + B) * X) = ((X * A) + (X * B)))"),
            ( "algebra/factor_out_left", "Factoring Out", "(((X * A) + (X * B)) = (X * (A + B)))"),
            ( "algebra/factor_out/0", "Factoring Out", "(((A * X) + (B * X)) = ((A + B) * X))"),
            ( "algebra/factor_out/1", "Factoring Out", "(((A * X) + (X * B)) = ((A + B) * X))"),
            ( "algebra/factor_out/2", "Factoring Out", "(((X * A) + (B * X)) = ((A + B) * X))"),
            ( "algebra/factor_out/3", "Factoring Out", "(((X * A) + (X * B)) = ((A + B) * X))"),
        ];
        for (i, (id, description, expression)) in rule_descriptions.iter().enumerate() {
            assert_rule_eq(&rules[i], id, description, expression);
        }
    }
}
