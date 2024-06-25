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
        assert_eq!(rules.len(), 9);
        assert_rule_eq(&rules[0], "algebra/add_zero/0", "Addition with 0", "((X + 0) = X)");
        assert_rule_eq(&rules[1], "algebra/add_zero/1", "Addition with 0", "((0 + X) = X)");
        assert_rule_eq(&rules[2], "algebra/mul_one/0", "Multiplication with 1", "((X * 1) = X)");
        assert_rule_eq(&rules[3], "algebra/mul_one/1", "Multiplication with 1", "((1 * X) = X)");
        assert_rule_eq(&rules[4], "algebra/mul_zero/0", "Multiplication with 0", "((X * 0) = 0)");
        assert_rule_eq(&rules[5], "algebra/mul_zero/1", "Multiplication with 0", "((0 * X) = 0)");
        assert_rule_eq(&rules[6], "algebra/sub_zero", "Subtraction by Zero", "((X - 0) = X)");
        assert_rule_eq(&rules[7], "algebra/div_one", "Division by One", "((X / 1) = X)");
        assert_rule_eq(&rules[8], "algebra/sub_self", "Self subtraction", "((X - X) = 0)");
    }
}
