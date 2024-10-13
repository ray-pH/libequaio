use std::collections::HashMap;
use crate::arithmetic::get_arithmetic_ctx;
use crate::expression::{Address, Context, Expression, ExpressionError};
use crate::parser::{parser_prefix, parser};
use serde::{Serialize, Deserialize};

#[derive(Clone, Default)]
pub struct Rule {
    pub id: String,
    pub expression: Expression,
    pub label: String,
}

#[derive(Clone, Default)]
pub struct RuleSet {
    pub name: String,
    pub context: Context,
    pub rule_vec: Vec<Rule>, 
    pub rule_ids: Vec<String>, // to preserve the order of the rules
    pub auto_rule_ids: Vec<String>, // list of rules that needs to be automatically applied
}

pub type RuleMap = HashMap<String, Rule>;

impl Expression {
    pub fn apply_rule_expr_at(&self, rule_expr: &Expression, addr: &Address) -> Result<Expression, ExpressionError> {
        if rule_expr.is_equation() {
            return self.apply_equation_at(rule_expr, addr);
        } else if rule_expr.is_implication() {
            return self.apply_implication(rule_expr);
        }
        return Err(ExpressionError::InvalidRule);
    }
    pub fn apply_rule_at(&self, rule: &Rule, addr: &Address) -> Result<Expression, ExpressionError> {
        return self.apply_rule_expr_at(&rule.expression, addr)
    }
}

impl RuleSet {
    pub fn get_rule_map(&self) -> RuleMap {
        return HashMap::from_iter(self.rule_vec.iter().map(|rule| (rule.id.clone(), rule.clone())));
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
    expr: Option<String>
}
#[derive(Serialize, Deserialize, Debug)]
struct RulesetNormalizationJSON {
    expr_prefix: Option<String>,
    expr: Option<String>
}
#[derive(Serialize, Deserialize, Debug)]
struct ContextJSON {
    base: Option<String>,
    parameters: Option<Vec<String>>,
    unary_ops: Option<Vec<String>>,
    binary_ops: Option<Vec<String>>,
    assoc_ops: Option<Vec<String>>,
    handle_numerics: Option<bool>,
    flags: Option<Vec<String>>
}
#[derive(Serialize, Deserialize, Debug)]
struct RuleJSON {
    id: String,
    label: Option<String>,
    expr_prefix: Option<String>,
    expr: Option<String>,
    variations: Option<Vec<RulesetVariationJSON>>,
    auto: Option<bool>,
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
    if context_json.is_none() { return Ok(Context::default()); }
    let context_json = context_json.unwrap();
    
    let mut ctx = if let Some(base) = context_json.base {
        match base.as_str() {
            "arithmetic" => get_arithmetic_ctx(),
            _ => return Err(ParserError::InvalidRule(base)),
        }
    } else {
        Context::default()
    };
    if let Some(parameters) = context_json.parameters { ctx.parameters.extend(parameters); };
    if let Some(unary_ops) = context_json.unary_ops { ctx.unary_ops.extend(unary_ops); };
    if let Some(binary_ops) = context_json.binary_ops { ctx.binary_ops.extend(binary_ops); };
    if let Some(assoc_ops) = context_json.assoc_ops { ctx.assoc_ops.extend(assoc_ops); };
    if let Some(handle_numerics) = context_json.handle_numerics { ctx.handle_numerics = handle_numerics; };
    if let Some(flags) = context_json.flags { 
        for flag in flags {
            ctx.flags.insert(flag);
        }
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
        let expr = if let Some(expr) = variation_json.expr {
            parser::to_expression(&expr, context)
                .ok_or(ParserError::InvalidRule(expr))
        } else if let Some(expr_prefix) = variation_json.expr_prefix {
            parser_prefix::to_expression(&expr_prefix, context)
                .ok_or(ParserError::InvalidRule(expr_prefix))
        } else {
            Err(ParserError::InvalidRule("missing expr or expr_prefix".to_string()))
        }?;
        variations.push(expr);
    }
    return Ok(variations);
}

//TODO: support implication
fn f_generate_variation(
    expr: &Expression, variation_rule: &Expression,  last_modified_address: &Address
) -> Vec<Expression> {
    //NOTE: this assumes that the possible addresses are sorted
    let possible_addresses = expr.get_possible_equation_application_addresses(variation_rule);
    let mut to_modify_address = None;
    for address in possible_addresses {
        if &address > last_modified_address {
            to_modify_address = Some(address);
            break;
        }
    }
    
    
    if to_modify_address.is_none() { return vec![expr.clone()]; }
    let to_modify_address = to_modify_address.unwrap();
    
    //TODO: maybe check if we want to also do variations on the rhs
    if !to_modify_address.is_empty() && to_modify_address.head() > 0 {
        return vec![expr.clone()];
    }
    
    let new_expr = expr.apply_equation_at(variation_rule, &to_modify_address)
        .expect("Expression::get_possible_equation_application_addresses should return valid addresses");
    
    //NOTE: only check the equivalence of the lhs
    let is_equivalent = match (expr.lhs(), new_expr.lhs()) {
        (Some(expr_lhs), Some(new_lhs)) => expr_lhs.is_equivalent_to(new_lhs),
        _ => false,
    };
        
    if is_equivalent{
        let variation0 = f_generate_variation(expr, variation_rule, &to_modify_address);
        return variation0;
    } else {
        let variation0 = f_generate_variation(expr, variation_rule, &to_modify_address);
        let variation1 = f_generate_variation(&new_expr, variation_rule, &to_modify_address);
        return [variation0, variation1].concat();
    }
}
fn generate_variations_from_single_rule(base_expr: &Expression, variation_rule: &Expression) -> Vec<Expression> {
    let variations = f_generate_variation(base_expr, variation_rule, &Address::default());
    return variations;
}

fn filter_for_unique_variations(expr_variations: &[Expression]) -> Vec<Expression> {
    if expr_variations.len() <= 2 { return expr_variations.to_vec(); }
    if !expr_variations[0].is_equation() { return expr_variations.to_vec(); }
    
    let mut unique_variations = Vec::new();
    for i in 0..expr_variations.len() {
        let expr1 = &expr_variations[i];
        let mut is_unique = true;
        for expr2 in expr_variations.iter().skip(i+1) {
            //NOTE: only check the equivalence of the lhs
            if let (Some(expr1_lhs), Some(expr2_lhs)) = (expr1.lhs(), expr2.lhs()) {
                if expr1_lhs.is_equivalent_to(expr2_lhs) {
                    is_unique = false;
                    break;
                }
            }
        }
        if is_unique { unique_variations.push(expr1.clone()); }
    }
    return unique_variations;
}

// C*(A+B)
fn generate_variations(base: &Rule, variation_rules: Vec<Expression>) -> Vec<Rule> {
    let mut expr_variations: Vec<Expression> = vec![base.expression.clone()];
    for variation_rule in variation_rules {
        let mut new_expr_variations: Vec<Expression> = vec![];
        for expr in expr_variations {
            let new_expr_variations0 = generate_variations_from_single_rule(&expr, &variation_rule);
            new_expr_variations.extend(new_expr_variations0);
        }
        expr_variations = new_expr_variations;
    }
    let expr_variations = filter_for_unique_variations(&expr_variations);
    // the id of the rule is base.id + index
    let rules = expr_variations.iter().enumerate().map(|(i, expr)| {
        Rule {
            id: format!("{}/{}", base.id, i),
            label: base.label.clone(),
            expression: expr.clone(),
        }
    }).collect();
    return rules;
}

fn parse_rule_vector(
    rules_json: Vec<RuleJSON>,  name: String, ruleset_variations: Vec<Expression>, context: &Context
) -> Result<(Vec<Rule>, Vec<String>), ParserError> {
    let mut rules: Vec<Rule> = vec![];
    let mut auto_rule_ids: Vec<String> = vec![];
    for rule_json in rules_json {
        let id = format!("{}/{}", name, rule_json.id);
        let label = rule_json.label.unwrap_or_default();
        let rule_variations = rule_json.variations.map(|v| resolve_variations_json(Some(v), context));
        let auto = rule_json.auto.unwrap_or(false);
        
        let variations = match rule_variations {
            Some(Ok(variations)) => variations,
            Some(Err(err)) => return Err(err),
            None => ruleset_variations.clone(),
        };
        
        let expression = if let Some(expr) = rule_json.expr {
            parser::to_expression(&expr, context)
                .ok_or(ParserError::InvalidRule(expr))
        } else if let Some(expr_prefix) = rule_json.expr_prefix {
            parser_prefix::to_expression(&expr_prefix, context)
                .ok_or(ParserError::InvalidRule(expr_prefix))
        } else {
            Err(ParserError::InvalidRule("missing rule".to_string()))
        }?;
        
        let var_rules = generate_variations(&Rule {id: id.clone(), label, expression}, variations);
        if var_rules.len() == 1 {
            let mut rule = var_rules.first().unwrap().clone();
            rule.id.clone_from(&id);
            if auto { auto_rule_ids.push(rule.id.clone()); }
            rules.push(rule);
        } else {
            if auto { auto_rule_ids.extend(var_rules.iter().map(|rule| rule.id.clone())); }
            rules.extend(var_rules);
        }
    }
    return Ok((rules, auto_rule_ids));
}

pub fn parse_ruleset_from_json(json_string: &str) -> Result<RuleSet, ParserError> {
    let ruleset_json: RulesetJSON = serde_json::from_str(json_string)?;
    
    let name = ruleset_json.name;
    let rules_json = ruleset_json.rules;
    let context = resolve_context_json(ruleset_json.context)?;
    let ruleset_variations = resolve_variations_json(ruleset_json.variations, &context)?;
    let (rule_vec, auto_rule_ids) = parse_rule_vector(rules_json, name.clone(), ruleset_variations, &context)?;
    let rule_ids = rule_vec.iter().map(|rule| rule.id.clone()).collect();
    return Ok(RuleSet {name, context, rule_vec, rule_ids, auto_rule_ids});
}