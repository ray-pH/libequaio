use crate::expression::{Address, Context, Expression, ExpressionType, ExpressionError, expression_builder as eb};
use crate::worksheet::{Action, ExpressionSequence, Rule};
use crate::arithmetic::{ArithmeticOperator, ArithmeticError};
use crate::utils::gcd;
use crate::{address, parser_prefix};
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    // because this pattern is extremely common, i think it's a good idea to hardcode it
    // A = B => _(A) = _(B)
    static ref FUNCTION_APPLICATION_TO_BOTH_SIDE_EXPR: Expression = {
        return parser_prefix::to_expression("=>(=(A,B),=(_(A),_(B)))", &Context::default()).unwrap();
    };
}

#[derive(Debug)]
pub enum AlgebraError {
    ArithmeticErr(ArithmeticError),
    ExpressionErr(ExpressionError),
    FunctionApplicationError,
    NotAFunction,
    NoAFraction,
}

impl From<ExpressionError> for AlgebraError {
    fn from(err: ExpressionError) -> Self {
        return AlgebraError::ExpressionErr(err);
    }
}
impl From<ArithmeticError> for AlgebraError {
    fn from(err: ArithmeticError) -> Self {
        return match err {
            ArithmeticError::ExpressionErr(err) => AlgebraError::ExpressionErr(err),
            _ => AlgebraError::ArithmeticErr(err),
        };
    }
}

impl Expression {
    pub fn is_function(&self) -> bool {
        // _(x) = ...
        if !self.is_equation() { return false; }
        let lhs = &self.children.as_ref().unwrap()[0];
        return lhs.exp_type == ExpressionType::OperatorNary;
    }
    
    pub fn normalize_algebra(&self, ctx: &Context) -> Expression {
        return self
            .normalize_sub_to_negative()
            .normalize_to_assoc_train(&ctx.assoc_ops)
            .normalize_two_children_assoc_train_to_binary_op(&ctx.binary_ops)
            .normalize_single_children_assoc_train()
    }
    
    /// turn the numerator and denominator into an AssocTrain of Mul
    pub fn normalize_fraction(&self) -> Expression {
        let op = self.identify_arithmetic_operator();
        if op.is_none() { return self.clone(); }
        if op.unwrap() != ArithmeticOperator::Div { return self.clone(); }
        let children = self.children.as_ref().unwrap();
        let numerator = children[0].turn_into_assoc_train(ArithmeticOperator::MulTrain.to_string());
        let denominator = children[1].turn_into_assoc_train(ArithmeticOperator::MulTrain.to_string());
        return Expression {
            exp_type: ExpressionType::OperatorBinary,
            symbol: ArithmeticOperator::Div.to_string(),
            children: Some(vec![numerator, denominator]),
        }
    }
    
    pub fn apply_function_to_both_side(&self, fn_expr: Expression) -> Result<Expression, AlgebraError> {
        if !fn_expr.is_function() { return Err(AlgebraError::NotAFunction); }
        if !self.is_equation()    { return Err(ExpressionError::NotAnEquation.into()); }
        let fn_lhs = &fn_expr.children.as_ref().unwrap()[0];
        let fn_symbol = fn_lhs.symbol.clone();
        // A = B => _(A) = _(B)
        let rule_expr = FUNCTION_APPLICATION_TO_BOTH_SIDE_EXPR.substitute_symbol("_".to_string(), fn_symbol);
        // wrapped_expr : _(A) = _(B)
        let wrapped_expr = self.apply_implication(rule_expr)?;
        let result_expr = wrapped_expr
            .apply_equation_at(fn_expr.clone(), &address![0])?
            .apply_equation_at(fn_expr.clone(), &address![1])?;
        return Ok(result_expr);
    }
    
    pub fn apply_fraction_arithmetic_at(&self, numerator_id: usize, denominator_id: usize, addr: &Address) 
        -> Result<Expression, AlgebraError> 
    {
        let expr = self.at(addr)?;
        let new_expr = expr.apply_fraction_arithmetic(numerator_id, denominator_id)?;
        let new_expr = self.replace_expression_at(new_expr, addr)?;
        return Ok(new_expr);
    }
    
    pub fn apply_fraction_arithmetic(&self, numerator_id: usize, denominator_id: usize) -> Result<Expression, AlgebraError> {
        if self.identify_arithmetic_operator() != Some(ArithmeticOperator::Div) { 
            return Err(AlgebraError::NoAFraction); 
        }
        let expr = self.normalize_fraction();
        let children = expr.children.as_ref().ok_or(ExpressionError::InvalidAddress)?;
        let numerator = children[0].clone();
        let denominator = children[1].clone();
        let numerator_selected_expr = match &numerator.children {
            None => return Err(ExpressionError::InvalidAddress.into()),
            Some(children) => children.get(numerator_id).ok_or(ExpressionError::InvalidAddress)?,
        };
        let denominator_selected_expr = match &denominator.children {
            None => return Err(ExpressionError::InvalidAddress.into()),
            Some(children) => children.get(denominator_id).ok_or(ExpressionError::InvalidAddress)?,
        };
        if !numerator_selected_expr.is_numeric() { return Err(ArithmeticError::NotNumeric.into()); }
        if !denominator_selected_expr.is_numeric() { return Err(ArithmeticError::NotNumeric.into()); }
        let both_integer = numerator_selected_expr.is_integer() && denominator_selected_expr.is_integer();
        let (new_num_sym, new_den_sym) = if both_integer {
            let num_val = numerator_selected_expr.symbol.parse::<i64>().map_err(|_| ArithmeticError::NotNumeric)?;
            let den_val = denominator_selected_expr.symbol.parse::<i64>().map_err(|_| ArithmeticError::NotNumeric)?;
            let gcd = gcd(num_val, den_val);
            let new_num_val = num_val / gcd;
            let new_den_val = den_val / gcd;
            (new_num_val.to_string(), new_den_val.to_string())
        } else {
            let num_val = numerator_selected_expr.symbol.parse::<f64>().map_err(|_| ArithmeticError::NotNumeric)?;
            let den_val = denominator_selected_expr.symbol.parse::<f64>().map_err(|_| ArithmeticError::NotNumeric)?;
            let new_num_val = num_val / den_val;
            let new_den_val = 1.0;
            (new_num_val.to_string(), new_den_val.to_string())
        };
        
        let numerator = children[0].clone();
        let denominator = children[1].clone();
        let new_num_selected_expr = eb::constant(new_num_sym.as_str());
        let new_den_selected_expr = eb::constant(new_den_sym.as_str());
        let new_numerator = numerator.replace_expression_at(new_num_selected_expr, &address![numerator_id])?;
        let new_denominator = denominator.replace_expression_at(new_den_selected_expr, &address![denominator_id])?;
        return Ok(Expression{
            exp_type: ExpressionType::OperatorBinary,
            symbol: ArithmeticOperator::Div.to_string(),
            children: Some(vec![new_numerator, new_denominator]),
        });
    }
}

impl ExpressionSequence {
    pub fn apply_simple_arithmetic_to_both_side(&mut self, op: ArithmeticOperator, val_str: &str) -> bool {
        let last_expr = self.last_expression();
        let (name, fn_expr) = generate_simple_apply_arithmetic_to_both_side_action(op, val_str);
        let expr = last_expr.apply_function_to_both_side(fn_expr);
        return self.try_push(Action::ApplyAction(name), expr);
    }
    
    pub fn apply_fraction_arithmetic_at(&mut self, numerator_id: usize, denominator_id: usize, addr: &Address) -> bool {
        let last_expr= self.last_expression();
        let expr = last_expr.apply_fraction_arithmetic_at(numerator_id, denominator_id, addr);
        return self.try_push(Action::ApplyAction("Simplify fraction".to_string()), expr);
    }
}

pub const ALGEBRA_RULE_STRING_TUPLE : [(&str, &str, &str); 7] = [
    ("add_zero", "=(+(X,0),X)", "Add by Zero"),
    ("zero_add", "=(+(0,X),X)", "Add by Zero"),
    ("mul_one", "=(*(X,1),X)", "Multiply by One"),
    ("one_mul", "=(*(1,X),X)", "Multiply by One"),
    ("mul_zero", "=(*(X,0),0)", "Multiply by Zero"),
    ("zero_mul", "=(*(0,X),0)", "Multiply by Zero"),
    ("div_one", "=(/(X,1),X)", "Divide by One"),
];
pub fn get_algebra_rules(ctx: &Context) -> HashMap<String, Rule> {
    let mut rules = HashMap::new();
    for (rule_id, rule_str, rule_label) in ALGEBRA_RULE_STRING_TUPLE.iter() {
        let rule_expr = parser_prefix::to_expression(rule_str, ctx).unwrap();
        rules.insert(rule_id.to_string(), Rule {
            id: rule_id.to_string(), 
            expression: rule_expr,
            label: rule_label.to_string(),
        });
    }
    return rules;
}


fn generate_simple_apply_arithmetic_to_both_side_action(op: ArithmeticOperator, val_str: &str) -> (String,Expression) {
    let name = format!("Apply {}{} to both side", op.to_string(), val_str);
    // =(_(X),{op}(X,{val}))
    let expression = eb::equation(
        eb::nary("_".to_string(), vec![eb::variable("X")]),
        eb::binary(op.to_string(), eb::variable("X"), eb::constant(val_str))
    );
    return (name, expression);
}
