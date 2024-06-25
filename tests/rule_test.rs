use::equaio::rule;

#[cfg(test)]
mod rule_test {
    use super::*;
    
    fn assert_rule_eq(rule: &rule::Rule, id: &str, label: &str, expr: &str) {
        assert_eq!(rule.id, id);
        assert_eq!(rule.label, label);
        assert_eq!(rule.expression.to_string(true), expr);
    }
    
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
