use std::collections::HashMap;

use crate::expression::Address;

use super::expression::{Context, Expression};
type NormalizationFunction = fn(&Expression, &Context) -> Expression;

pub enum Action {
    Introduce,
    ApplyRule(String),
    ApplyAction(String),
}

pub struct ExpressionSequence {
    pub history: Vec<(Action, Expression)>,
    pub context: Context,
    normalization_function: Option<NormalizationFunction>,
    rule_map: HashMap<String, Rule>,
}

pub struct Worksheet {
    expression_sequences: Vec<ExpressionSequence>,
    pub context: Context,
    normalization_function: Option<NormalizationFunction>,
    rule_map: HashMap<String, Rule>,
}

#[derive(Clone)]
pub struct Rule {
    pub id: String,
    pub expression: Expression,
    pub label: String,
}

impl Action {
    pub fn to_string(&self) -> String {
        return self.as_str().to_string();
    }
    pub fn as_str(&self) -> &str {
        match self {
            Action::Introduce => "Introduce",
            Action::ApplyRule(rule) => rule,
            Action::ApplyAction(action) => action,
        }
    }
    
}

impl ExpressionSequence {
    pub fn new(expr: Expression, ctx: Context) -> ExpressionSequence {
        return ExpressionSequence {
            context: ctx,
            history: vec![(Action::Introduce, expr)],
            normalization_function: None,
            rule_map: HashMap::new(),
        };
    }
    
    pub fn expression(&self, index: usize) -> Option<&Expression> {
        return self.history.get(index).map(|(_, expr)| expr);
    }
    
    pub fn set_normalization_function(&mut self, f: NormalizationFunction) {
        self.normalization_function = Some(f);
    }
    
    pub fn reset_rule_map(&mut self) { 
        self.rule_map.clear(); 
    }
    pub fn set_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.rule_map = rule_map; 
    }
    pub fn extend_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.rule_map.extend(rule_map); 
    }
    
    pub fn apply_rule_at(&mut self, rule_id: &str, addr: &Address) -> bool {
        if let Some(rule) = self.rule_map.get(rule_id) {
            let rule_expr = rule.expression.clone();
            let expr = self.last_expression();
            let rule_label = format!("{}", rule.label);
            if rule_expr.is_equation() {
                let new_expr = expr.apply_equation_at(rule_expr, addr);
                return self.try_push(Action::ApplyRule(rule_label), new_expr)
            } else if rule_expr.is_implication() {
                let new_expr = expr.apply_implication(rule_expr);
                return self.try_push(Action::ApplyRule(rule_label), new_expr)
            }
            return false;
        } else {
            return false;
        }
    }
    
    pub fn last_expression(&self) -> &Expression {
        return &self.history.last().unwrap().1;
    }
    
    pub fn push(&mut self, action: Action, expr: Expression) {
        self.history.push((action, expr));
    }
    
    pub fn try_push(&mut self, action: Action, expr: Option<Expression>) -> bool {
        if let Some(expr) = expr {
            let expr = self.normalize(&expr);
            self.push(action, expr);
            return true;
        } else {
            return false;
        }
    }
    
    fn normalize(&self, expr: &Expression) -> Expression {
        if let Some(f) = self.normalization_function {
            return f(expr, &self.context);
        } else {
            return expr.clone();
        }
    }
    
    
}

impl Worksheet {
    pub fn new(ctx: Context) -> Worksheet {
        return Worksheet {
            context: ctx,
            expression_sequences: vec![],
            normalization_function: None,
            rule_map: HashMap::new()
        };
    }
    
    pub fn set_normalization_function(&mut self, f: NormalizationFunction) {
        self.normalization_function = Some(f);
        for seq in self.expression_sequences.iter_mut() {
            seq.set_normalization_function(f);
        }
    }
    
    pub fn reset_rule_map(&mut self) { 
        self.rule_map.clear(); 
    }
    pub fn set_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.rule_map = rule_map; 
        for seq in self.expression_sequences.iter_mut() {
            seq.set_rule_map(self.rule_map.clone())
        }
    }
    pub fn extend_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.rule_map.extend(rule_map); 
        for seq in self.expression_sequences.iter_mut() {
            seq.set_rule_map(self.rule_map.clone())
        }
    }
    
    pub fn introduce_expression(&mut self, expr: Expression) {
        let mut sequence = ExpressionSequence::new(expr, self.context.clone());
        if let Some(f) = self.normalization_function {
            sequence.set_normalization_function(f);
        }
        sequence.set_rule_map(self.rule_map.clone());
        self.expression_sequences.push(sequence);
    }
    
    pub fn get_expression_sequence(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.expression_sequences.get_mut(index);
    }
    
    pub fn get(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.get_expression_sequence(index);
    }
}
