use std::{collections::HashMap, fmt::{self, Debug}};
use crate::expression::Address;
use crate::rule::{Rule, RuleSet};
use super::expression::{Context, Expression};

type NormalizationFunction = fn(&Expression, &Context) -> Expression;
type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, &Vec<Address>) -> Vec<(Action,Expression)>;

const LIMIT_OF_AUTO_GENERATED_STEPS: usize = 100;

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Introduce(String),
    ApplyRule(String),
    ApplyAction(String),
}

#[derive(Default, Clone, PartialEq)]
pub struct WorksheetContext {
    pub expression_context : Context,
    normalization_function: Option<NormalizationFunction>,
    pub rule_map: HashMap<String, Rule>,
    pub rule_ids: Vec<String>,
    get_possible_actions_function: Option<GetPossibleActionsFunction>,
    pub labelled_expression: Vec<(String, Expression)>,
    pub auto_rule_ids: Vec<String>, // list of rules that needs to be automatically applied
}

#[derive(Clone, PartialEq)]
pub struct ExpressionLine {
    pub action: Action,
    pub expr: Expression,
    pub label: Option<String>,
    pub is_auto_generated: bool,
}

#[derive(Default, Clone, PartialEq)]
pub struct WorkableExpressionSequence {
    pub history: Vec<ExpressionLine>,
    context: WorksheetContext,
}

#[derive(Default, Clone, PartialEq)]
pub struct ExpressionSequence {
    pub history: Vec<ExpressionLine>,
}

#[derive(Default, PartialEq)]
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
        self.history.push(ExpressionLine{action, expr, label: None, is_auto_generated: false});
        self.try_apply_auto_rules();
    }
    fn push_auto(&mut self, action: Action, expr: Expression) {
        let expr = self.normalize(&expr);
        self.history.push(ExpressionLine{action, expr, label: None, is_auto_generated: true});
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
            if label.is_empty() {
                line.label = None;
            } else {
                line.label = Some(label);
            }
        }
    }
    
    pub fn reset_to(&mut self, index: usize) {
        if index < self.history.len() {
            self.history = self.history.iter().take(index+1).cloned().collect();
        }
    }
    
    pub fn try_apply_auto_rules(&mut self) {
        if self.context.auto_rule_ids.is_empty() { return; }
        let rule_map = &self.context.rule_map;
        let auto_rules = self.context.auto_rule_ids.iter()
            .filter_map(|id| rule_map.get(id)).cloned().collect::<Vec<Rule>>();
        for _ in 0..LIMIT_OF_AUTO_GENERATED_STEPS {
            let changed = self.f_try_apply_auto_rules(&auto_rules);
            if !changed { break; }
        }
    }
    
    /// return `true` if the expression is changed
    fn f_try_apply_auto_rules(&mut self, rules: &[Rule]) -> bool {
        let expr = self.last_expression();
        for rule in rules {
            let rule_expr = &rule.expression;
            let possible_eq_addr = expr.get_possible_equation_application_addresses(rule_expr);
            if possible_eq_addr.is_empty() { continue; }
            let addr = possible_eq_addr.first()
                .expect("Expression::get_possible_equation_application_address is valid");
            if let Ok(new_expr) = expr.apply_equation_at(rule_expr, addr) {
                let action = Action::ApplyRule(rule.label.clone());
                self.push_auto(action, new_expr);
                return true;
            }
        }
        return false;
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
    
    pub fn set_ruleset(&mut self, ruleset: RuleSet) {
        let rule_map = ruleset.get_rule_map();
        self.set_expression_context(ruleset.context);
        self.set_rule_ids(ruleset.rule_ids);
        self.set_auto_rule_ids(ruleset.auto_rule_ids);
        self.set_rule_map(rule_map);
    }
    pub fn reset_rule_map(&mut self) { 
        self.context.rule_map.clear();
    }
    pub fn set_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.context.rule_map = rule_map;
    }
    pub fn set_rule_ids(&mut self, rule_ids: Vec<String>) { 
        self.context.rule_ids = rule_ids;
    }
    pub fn set_auto_rule_ids(&mut self, rule_ids: Vec<String>) { 
        self.context.auto_rule_ids = rule_ids;
    }
    pub fn extend_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        self.context.rule_map.extend(rule_map);
    }
    pub fn extend_rule_ids(&mut self, rule_ids: Vec<String>) { 
        self.context.rule_ids.extend(rule_ids);
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
    
    pub fn get(&self, index: usize) -> Option<WorkableExpressionSequence> {
        return self.get_workable_expression_sequence(index);
    }
    pub fn store(&mut self, index: usize, seq: WorkableExpressionSequence) {
        self.store_expression_sequence(index, seq);
    }
    pub fn len(&self) -> usize {
        return self.expression_sequences.len();
    }
    pub fn is_empty(&self) -> bool {
        return self.expression_sequences.is_empty();
    }
    
}
