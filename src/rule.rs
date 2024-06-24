use crate::arithmetic::get_arithmetic_ctx;
use crate::expression::{Address, Context, Expression, ExpressionError};
use crate::{address, parser_prefix};
use serde::{Serialize, Deserialize};

#[derive(Clone, Default)]
pub struct Rule {
    pub id: String,
    pub expression: Expression,
    pub label: String,
}

impl Expression {
    pub fn apply_rule_at(&self, rule: &Rule, addr: &Address) -> Result<Expression, ExpressionError> {
        let rule_expr = &rule.expression;
        if rule_expr.is_equation() {
            return self.apply_equation_at(rule_expr, addr);
        } else if rule_expr.is_implication() {
            return self.apply_implication(rule_expr);
        }
        return Err(ExpressionError::InvalidRule);
    }
}

#[derive(Debug)]
pub enum ParserError {
    InvalidJSON(String),
    InvalidRule(String),
}
impl From<serde_json::Error> for ParserError {
    fn from(err: serde_json::Error) -> Self {
        ParserError::InvalidJSON(err.to_string())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct RulesetVariationJSON {
    expr_prefix: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct RulesetNormalizationJSON {
    expr_prefix: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct ContextJSON {
    base: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct RuleJSON {
    id: String,
    label: Option<String>,
    expr_prefix: Option<String>,
}
#[derive(Serialize, Deserialize, Debug)]
struct RulesetJSON {
    name: String,
    context: Option<ContextJSON>,
    variations: Option<Vec<RulesetVariationJSON>>,
    normalization: Option<Vec<RulesetNormalizationJSON>>,
    rules: Vec<RuleJSON>
}

fn resolve_context_json(context_json: Option<ContextJSON>) -> Result<Context, ParserError> {
    let ctx = match context_json {
        Some(context_json) => {
            let base = context_json.base.unwrap_or_default();
            match base.as_str() {
                "arithmetic" => get_arithmetic_ctx(),
                _ => return Err(ParserError::InvalidRule(base)),
            }
        },
        None => Context::default()
    };
    return Ok(ctx);
}
fn resolve_variations_json(
    variations_json: Option<Vec<RulesetVariationJSON>>,
    context: &Context
) -> Result<Vec<Expression>, ParserError> {
    if variations_json.is_none() { return Ok(vec![]); }
    let variations_json = variations_json.unwrap();
    let mut variations: Vec<Expression> = vec![];
    for variation_json in variations_json {
        let expr_prefix = variation_json.expr_prefix.unwrap_or_default();
        let expr = parser_prefix::to_expression(&expr_prefix, context)
            .ok_or(ParserError::InvalidRule(expr_prefix))?;
        variations.push(expr);
    }
    return Ok(variations);
}

pub fn parse_ruleset_from_json(json_string: &str) -> Result<Vec<Rule>, ParserError> {
    let ruleset_json: RulesetJSON = serde_json::from_str(json_string)?;
    let mut rules: Vec<Rule> = vec![];
    let name = ruleset_json.name;
    let rules_json = ruleset_json.rules;
    let context = resolve_context_json(ruleset_json.context)?;
    let variations = resolve_variations_json(ruleset_json.variations, &context)?;
    
    for rule_json in rules_json {
        let id = format!("{}/{}", name, rule_json.id);
        let label = rule_json.label.unwrap_or_default();
        if let Some(expr_prefix) = rule_json.expr_prefix {
            let expression = parser_prefix::to_expression(&expr_prefix, &context)
                .ok_or(ParserError::InvalidRule(expr_prefix))?;
            rules.push(Rule {id, label, expression});
        }
    }
    
    return Ok(rules);
}
