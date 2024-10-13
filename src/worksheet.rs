use std::{collections::HashMap, fmt::{self, Debug}};
use crate::expression::Address;
use crate::rule::Rule;
use super::expression::{Context, Expression};

type NormalizationFunction = fn(&Expression, &Context) -> Expression;
type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, &Vec<Address>) -> Vec<(Action,Expression)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Introduce(String),
    ApplyRule(String),
    ApplyAction(String),
}

#[derive(Default,Clone)]
pub struct WorksheetContext {
    pub expression_context : Context,
    normalization_function: Option<NormalizationFunction>,
    pub rule_map: HashMap<String, Rule>,
    get_possible_actions_function: Option<GetPossibleActionsFunction>,
    pub labelled_expression: Vec<(String, Expression)>
}

#[derive(Clone)]
pub struct ExpressionLine {
    pub action: Action,
    pub expr: Expression,
    pub label: Option<String>,
}

#[derive(Default)]
pub struct WorkableExpressionSequence {
    pub history: Vec<ExpressionLine>,
    context: WorksheetContext,
}

#[derive(Default)]
pub struct ExpressionSequence {
    pub history: Vec<ExpressionLine>,
}

#[derive(Default)]
pub struct Worksheet {
    expression_sequences: Vec<ExpressionSequence>,
    context: WorksheetContext,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl Action {
    pub fn as_str(&self) -> &str {
        match self {
            Action::Introduce(str) => str,
            Action::ApplyRule(rule) => rule,
            Action::ApplyAction(action) => action,
        }
    }
    
}
impl From<WorkableExpressionSequence> for ExpressionSequence {
    fn from(seq: WorkableExpressionSequence) -> Self {
        return ExpressionSequence { history: seq.history, };
    }
}
impl ExpressionSequence {
    pub fn with_context(&self, ctx: WorksheetContext) -> WorkableExpressionSequence {
        return WorkableExpressionSequence {
            history: self.history.clone(),
            context: ctx
        };
    }
    pub fn last_expression(&self) -> &Expression {
        return &self.history.last().unwrap().expr;
    }
}
impl WorkableExpressionSequence {
    pub fn new(ctx: WorksheetContext) -> WorkableExpressionSequence {
        return WorkableExpressionSequence {
            context: ctx,
            history: vec![],
        };
    }
    
    pub fn set_context(&mut self, ctx: WorksheetContext) {
        self.context = ctx;
    }
    
    pub fn expression(&self, index: usize) -> Option<&Expression> {
        return self.history.get(index).map(|line| &line.expr);
    }
    
    pub fn apply_rule_at(&mut self, rule_id: &str, addr: &Address) -> bool {
        let rule = self.context.rule_map.get(rule_id).cloned();
        if let Some(rule) = rule {
            let expr = self.last_expression();
            let rule_label = rule.label.to_string();
            let result_expr = expr.apply_rule_at(&rule, addr);
            return self.try_push(Action::ApplyRule(rule_label), result_expr);
        } else {
            return false;
        }
    }
    
    pub fn last_expression(&self) -> &Expression {
        return &self.history.last().unwrap().expr;
    }
    
    pub fn push(&mut self, action: Action, expr: Expression) {
        let expr = self.normalize(&expr);
        self.history.push(ExpressionLine{action, expr, label: None});
    }
    
    pub fn try_push<T: Debug>(&mut self, action: Action, expr: Result<Expression,T>) -> bool {
        match expr {
            Ok(expr) => {
                self.push(action, expr);
                return true;
            },
            Err(err) => {
                dbg!(err);
                return false;
            }
        }
    }
    
    fn normalize(&self, expr: &Expression) -> Expression {
        let ctx = &self.context;
        if let Some(f) = ctx.normalization_function {
            return f(expr, &ctx.expression_context);
        } else {
            return expr.clone();
        }
    }
    
    pub fn get_possible_actions_from_labelled_equations(&self, addr_vec: &[Address]) -> Vec<(Action,Expression)> {
        if addr_vec.is_empty() { return vec![]; }
        let addr = &addr_vec[addr_vec.len()-1];
        let mut possible_actions = Vec::new();
        for (label, expr) in self.context.labelled_expression.iter() {
            if let Ok(new_expr) = self.last_expression().apply_rule_expr_at(expr, addr) {
                let rulestr = format!("Substitute from {}", label);
                let action = Action::ApplyRule(rulestr);
                let normalized_expr = self.normalize(&new_expr);
                possible_actions.push((action, normalized_expr));
            }
        }
        return possible_actions;
    }
    
    pub fn get_possible_actions(&self, addr_vec: &Vec<Address>) -> Vec<(Action,Expression)> {
        let ctx = &self.context;
        let mut possible_actions = Vec::new();
        possible_actions.extend(self.get_possible_actions_from_labelled_equations(addr_vec));
        if let Some(f) = ctx.get_possible_actions_function {
            possible_actions.extend(
                f(self.last_expression(), ctx, addr_vec).into_iter()
                    .map(|(action, expr)| {(action, self.normalize(&expr))})
            );
        }
        return possible_actions;
    }
    
    pub fn try_apply_action_by_index(&mut self, addr_vec: &Vec<Address>, index: usize) -> bool {
        if let Some((action, expr)) = self.get_possible_actions(addr_vec).get(index) {
            self.push(action.clone(), expr.clone());
            return true;
        } else {
            return false;
        }
    }
    
    pub fn label_expression(&mut self, label: String, index: usize) {
        if let Some(line) = self.history.get_mut(index) {
            line.label = Some(label);
        }
    }
    
}

impl Worksheet {
    pub fn new() -> Worksheet {
        let ctx = WorksheetContext::default();
        return Worksheet {
            context: ctx,
            expression_sequences: vec![],
        };
    }
    
    pub fn set_expression_context(&mut self, expression_ctx: Context) {
        self.context.expression_context = expression_ctx;
    }
    
    pub fn get_expression_context(&self) -> Context {
        let ctx = &self.context;
        return ctx.expression_context.clone();
    }
    
    pub fn set_normalization_function(&mut self, f: NormalizationFunction) {
        self.context.normalization_function = Some(f);
    }
    
    pub fn set_get_possible_actions_function(&mut self, f: GetPossibleActionsFunction) {
        self.context.get_possible_actions_function = Some(f);
    }
    
    pub fn reset_rule_map(&mut self) { 
        self.context.rule_map.clear();
    }
    pub fn set_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.context.rule_map = rule_map;
    }
    pub fn extend_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.context.rule_map.extend(rule_map);
    }
    
    pub fn introduce_expression(&mut self, expr: Expression) {
        let mut sequence = WorkableExpressionSequence::new(self.context.clone());
        sequence.push(Action::Introduce("Introduce".to_string()), expr);
        self.expression_sequences.push(sequence.into());
    }
    
    pub fn introduce_from_label(&mut self, label: &str) -> bool {
        if let Some((_, expr)) = self.context.labelled_expression.iter().find(|(l, _)| l == label) {
            let mut sequence = WorkableExpressionSequence::new(self.context.clone());
            let action_str = format!("Introduce from {}", label);
            sequence.push(Action::Introduce(action_str), expr.clone());
            self.expression_sequences.push(sequence.into());
            return true
        } else {
            return false;
        }
    }
    
    fn check_and_update_labelled_expr(&mut self, seq: &WorkableExpressionSequence) {
        for line in &seq.history {
            if let Some(label) = &line.label {
                if self.context.labelled_expression.iter().any(|(l, e)| l == label && e == &line.expr) {
                    continue;
                }
                self.context.labelled_expression.push((label.clone(), line.expr.clone()));
            }
        }
    }
    
    pub fn get_workable_expression_sequence(&self, index: usize) -> Option<WorkableExpressionSequence> {
        return self.expression_sequences.get(index).map(|seq| seq.with_context(self.context.clone()));
    }
    
    pub fn store_expression_sequence(&mut self, index: usize, seq: WorkableExpressionSequence) {
        self.check_and_update_labelled_expr(&seq);
        if index < self.expression_sequences.len() {
            self.expression_sequences[index] = seq.into();
        } else {
            self.expression_sequences.push(seq.into());
        }
    }
    
    pub fn get(&mut self, index: usize) -> Option<WorkableExpressionSequence> {
        return self.get_workable_expression_sequence(index);
    }
    pub fn store(&mut self, index: usize, seq: WorkableExpressionSequence) {
        self.store_expression_sequence(index, seq);
    }
    
}
