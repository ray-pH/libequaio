use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};
use crate::expression::{Address, Rule};
use super::expression::{Context, Expression};

type NormalizationFunction = fn(&Expression, &Context) -> Expression;
type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, &Vec<Address>) -> Vec<(Action,Expression)>;

#[derive(Debug, PartialEq, Clone)]
pub enum Action {
    Introduce,
    ApplyRule(String),
    ApplyAction(String),
}

#[derive(Default)]
pub struct WorksheetContext {
    expression_context : Context,
    normalization_function: Option<NormalizationFunction>,
    pub rule_map: HashMap<String, Rule>,
    get_possible_actions_function: Option<GetPossibleActionsFunction>,
}

#[derive(Default)]
pub struct ExpressionSequence {
    pub history: Vec<(Action, Expression)>,
    context: Rc<RefCell<WorksheetContext>>,
}

#[derive(Default)]
pub struct Worksheet {
    expression_sequences: Vec<ExpressionSequence>,
    context: Rc<RefCell<WorksheetContext>>,
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
    pub fn new(ctx: Rc<RefCell<WorksheetContext>>) -> ExpressionSequence {
        return ExpressionSequence {
            context: ctx,
            history: vec![],
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
            let expr = self.last_expression();
            let rule_label = format!("{}", rule.label);
            let result_expr = expr.apply_rule_at(&rule, addr);
            return self.try_push(Action::ApplyRule(rule_label), result_expr);
        } else {
            return false;
        }
    }
    
    pub fn last_expression(&self) -> &Expression {
        return &self.history.last().unwrap().1;
    }
    
    pub fn push(&mut self, action: Action, expr: Expression) {
        let expr = self.normalize(&expr);
        self.history.push((action, expr));
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
        let ctx = self.context.borrow();
        if let Some(f) = ctx.normalization_function {
            return f(expr, &ctx.expression_context);
        } else {
            return expr.clone();
        }
    }
    
    pub fn get_possible_actions(&self, addr_vec: &Vec<Address>) -> Vec<(Action,Expression)> {
        let ctx = self.context.borrow();
        if let Some(f) = ctx.get_possible_actions_function {
            return f(self.last_expression(), &ctx, addr_vec).into_iter()
                .map(|(action, expr)| {(action, self.normalize(&expr))}).collect();
        } else {
            return vec![];
        }
    }
    
    pub fn try_apply_action_by_index(&mut self, addr_vec: &Vec<Address>, index: usize) -> bool {
        if let Some((action, expr)) = self.get_possible_actions(addr_vec).get(index) {
            self.push(action.clone(), expr.clone());
            return true;
        } else {
            return false;
        }
    }
    
}

impl Worksheet {
    pub fn new() -> Worksheet {
        let ctx = WorksheetContext::default();
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
    
    pub fn set_get_possible_actions_function(&mut self, f: GetPossibleActionsFunction) {
        let mut ctx = self.context.borrow_mut();
        ctx.get_possible_actions_function = Some(f);
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
        let mut sequence = ExpressionSequence::new(self.context.clone());
        sequence.push(Action::Introduce, expr);
        self.expression_sequences.push(sequence);
    }
    
    pub fn get_expression_sequence(&self, index: usize) -> Option<&ExpressionSequence> {
        return self.expression_sequences.get(index);
    }
    
    pub fn get_mut_expression_sequence(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.expression_sequences.get_mut(index);
    }
    
    pub fn get(&mut self, index: usize) -> Option<&mut ExpressionSequence> {
        return self.get_mut_expression_sequence(index);
    }
}
