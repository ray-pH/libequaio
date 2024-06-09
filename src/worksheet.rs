use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::expression::{empty_context, Address};
use super::expression::{Context, Expression};

type NormalizationFunction = fn(&Expression, &Context) -> Expression;

pub enum Action {
    Introduce,
    ApplyRule(String),
    ApplyAction(String),
}

pub struct WorksheetContext {
    expression_context : Context,
    normalization_function: Option<NormalizationFunction>,
    rule_map: HashMap<String, Rule>,
}

pub struct ExpressionSequence {
    pub history: Vec<(Action, Expression)>,
    context: Rc<RefCell<WorksheetContext>>,
}

pub struct Worksheet {
    expression_sequences: Vec<ExpressionSequence>,
    context: Rc<RefCell<WorksheetContext>>,
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
    pub fn new(expr: Expression, ctx: Rc<RefCell<WorksheetContext>>) -> ExpressionSequence {
        return ExpressionSequence {
            context: ctx,
            history: vec![(Action::Introduce, expr)],
        };
    }
    
    pub fn expression(&self, index: usize) -> Option<&Expression> {
        return self.history.get(index).map(|(_, expr)| expr);
    }
    
    pub fn apply_rule_at(&mut self, rule_id: &str, addr: &Address) -> bool {
        let rule = {
            let ctx = self.context.borrow();
            ctx.rule_map.get(rule_id).cloned()
        };
        if let Some(rule) = rule {
            let rule_expr = rule.expression.clone();
            let expr = self.last_expression();
            let rule_label = format!("{}", rule.label);
            if rule_expr.is_equation() {
                let new_expr = expr.apply_equation_at(rule_expr, addr);
                return self.try_push(Action::ApplyRule(rule_label), new_expr);
            } else if rule_expr.is_implication() {
                let new_expr = expr.apply_implication(rule_expr);
                return self.try_push(Action::ApplyRule(rule_label), new_expr);
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
        let ctx = self.context.borrow();
        if let Some(f) = ctx.normalization_function {
            return f(expr, &ctx.expression_context);
        } else {
            return expr.clone();
        }
    }
    
    
}

impl Worksheet {
    pub fn new() -> Worksheet {
        let ctx = WorksheetContext {
            expression_context: empty_context(),
            normalization_function: None,
            rule_map: HashMap::new()
        };
        return Worksheet {
            context: Rc::new(RefCell::new(ctx)),
            expression_sequences: vec![],
        };
    }
    
    pub fn set_expression_context(&mut self, expression_ctx: Context) {
        let mut ctx = self.context.borrow_mut();
        ctx.expression_context = expression_ctx;
    }
    
    pub fn get_expression_context(&self) -> Context {
        let ctx = self.context.borrow();
        return ctx.expression_context.clone();
    }
    
    pub fn set_normalization_function(&mut self, f: NormalizationFunction) {
        let mut ctx = self.context.borrow_mut();
        ctx.normalization_function = Some(f);
    }
    
    pub fn reset_rule_map(&mut self) { 
        let mut ctx = self.context.borrow_mut();
        ctx.rule_map.clear();
    }
    pub fn set_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        let mut ctx = self.context.borrow_mut();
        ctx.rule_map = rule_map;
    }
    pub fn extend_rule_map(&mut self, rule_map: HashMap<String, Rule>) { 
        let mut ctx = self.context.borrow_mut();
        ctx.rule_map.extend(rule_map);
    }
    
    pub fn introduce_expression(&mut self, expr: Expression) {
        let sequence = ExpressionSequence::new(expr, self.context.clone());
        self.expression_sequences.push(sequence);
    }
    
    pub fn get_expression_sequence(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.expression_sequences.get_mut(index);
    }
    
    pub fn get(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.get_expression_sequence(index);
    }
}
