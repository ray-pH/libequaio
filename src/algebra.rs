use crate::expression::{Address, Context, Expression, ExpressionType, ExpressionError, expression_builder as eb};
use crate::worksheet::{Action, ExpressionSequence, Worksheet, WorksheetContext};
use crate::arithmetic::{ArithmeticOperator, ArithmeticError, get_arithmetic_ctx};
use crate::rule::{Rule, RuleMap};
use crate::utils::gcd;
use crate::{address, parser_prefix};
use std::collections::HashMap;
use lazy_static::lazy_static;

// this is a module for algebra (with arithmetic)
// for abstract algebra, we will create a new module (algebra_abstract.rs)

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
    NotAFraction,
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

pub enum AlgebraCtxFlags {
    SimplifyOneAndZero,
}
impl ToString for AlgebraCtxFlags {
    fn to_string(&self) -> String {
        return match self {
            AlgebraCtxFlags::SimplifyOneAndZero => "algebra:simplify_one_and_zero".to_string(),
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
            .normalize_add_negative_to_sub()
            .normalize_single_children_assoc_train()
            .normalize_simplify_one_and_zero(&ctx)
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
    
    pub fn normalize_simplify_one_and_zero(&self, ctx: &Context) -> Expression {
        if !ctx.contains_flag(AlgebraCtxFlags::SimplifyOneAndZero) { return self.clone(); }
        let add_zero = eb::equation(
            eb::binary("+", eb::variable("X"), eb::constant("0")),
            eb::variable("X"));
        let zero_add = eb::equation(
            eb::binary("+", eb::constant("0"), eb::variable("X")),
            eb::variable("X"));
        let mul_one = eb::equation(
            eb::binary("*", eb::variable("X"), eb::constant("1")),
            eb::variable("X"));
        let one_mul = eb::equation(
            eb::binary("*", eb::constant("1"), eb::variable("X")),
            eb::variable("X"));
        let mul_zero = eb::equation(
            eb::binary("*", eb::variable("X"), eb::constant("0")),
            eb::constant("0"));
        let zero_mul = eb::equation(
            eb::binary("*", eb::constant("0"), eb::variable("X")),
            eb::constant("0"));
        let sub_zero = eb::equation(
            eb::binary("-", eb::variable("X"), eb::constant("0")),
            eb::variable("X"));
        let div_one = eb::equation(
            eb::binary("/", eb::variable("X"), eb::constant("1")),
            eb::variable("X"));
        let rule_exprs = vec![add_zero, zero_add, mul_one, one_mul, mul_zero, zero_mul, div_one, sub_zero];
        
        let mut prev_expr = Expression::default();
        let mut expr = self.clone();
        while &expr != &prev_expr {
            prev_expr = expr.clone();
            for e in &rule_exprs {
                let possible_eq_addr = expr.get_possible_equation_application_addresses(e);
                if possible_eq_addr.len() < 1 { continue; }
                let addr = possible_eq_addr.get(0)
                    .expect("Expression::get_possible_equation_application_address is valid");
                if let Ok(new_expr) = expr.apply_equation_at(e, addr) {
                    expr = new_expr;
                }
            }
        }
        return expr;
    }
    
    pub fn apply_function_to_both_side(&self, fn_expr: Expression) -> Result<Expression, AlgebraError> {
        if !fn_expr.is_function() { return Err(AlgebraError::NotAFunction); }
        if !self.is_equation()    { return Err(ExpressionError::NotAnEquation.into()); }
        let fn_lhs = &fn_expr.children.as_ref().unwrap()[0];
        let fn_symbol = fn_lhs.symbol.clone();
        // A = B => _(A) = _(B)
        let rule_expr = FUNCTION_APPLICATION_TO_BOTH_SIDE_EXPR.substitute_symbol("_".to_string(), fn_symbol);
        // wrapped_expr : _(A) = _(B)
        let wrapped_expr = self.apply_implication(&rule_expr)?;
        let result_expr = wrapped_expr
            .apply_equation_at(&fn_expr, &address![0])?
            .apply_equation_at(&fn_expr, &address![1])?;
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
            return Err(AlgebraError::NotAFraction); 
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
    
    pub fn apply_simple_arithmetic_to_both_side(&self, op: &ArithmeticOperator, expr: &Expression) 
    -> Result<Expression, AlgebraError> 
    {
        let fn_expr = generate_simple_apply_arithmetic_to_both_side_expr(op, expr);
        let expr = self.apply_function_to_both_side(fn_expr)?;
        return Ok(expr);
    }
}

impl ExpressionSequence {
    pub fn apply_simple_arithmetic_to_both_side(&mut self, op: ArithmeticOperator, expr: &Expression) -> bool {
        let name = generate_simple_apply_arithmetic_to_both_side_name(&op, expr);
        let expr = self.last_expression().apply_simple_arithmetic_to_both_side(&op, expr);
        return self.try_push(Action::ApplyAction(name), expr);
    }
    
    pub fn apply_fraction_arithmetic_at(&mut self, numerator_id: usize, denominator_id: usize, addr: &Address) -> bool {
        let last_expr= self.last_expression();
        let expr = last_expr.apply_fraction_arithmetic_at(numerator_id, denominator_id, addr);
        return self.try_push(Action::ApplyAction("Simplify fraction".to_string()), expr);
    }
}

impl Worksheet {
    pub fn init_algebra_worksheet(variables: Vec<String>) -> Worksheet {
        let mut ws = Worksheet::new();
        let ctx = get_arithmetic_ctx().add_params(variables);
        ws.set_expression_context(ctx);
        ws.set_normalization_function(|expr,ctx| expr.normalize_algebra(ctx));
        ws.set_rule_map(get_algebra_rules(&ws.get_expression_context()));
        ws.set_get_possible_actions_function(|expr,ctx,addr_vec| 
            get_possible_actions::algebra(expr,ctx,addr_vec));
        return ws;
    }
}

pub const ALGEBRA_RULE_STRING_TUPLE : [(&str, &str, &str); 15] = [
    ("add_zero", "=(+(X,0),X)", "Add by Zero"),
    ("zero_add", "=(+(0,X),X)", "Add by Zero", ),
    ("mul_one", "=(*(X,1),X)", "Multiply by One"),
    ("one_mul", "=(*(1,X),X)", "Multiply by One"),
    ("mul_zero", "=(*(X,0),0)", "Multiply by Zero"),
    ("zero_mul", "=(*(0,X),0)", "Multiply by Zero"),
    ("sub_zero", "=(-(X,0),X)", "Subtract by Zero"),
    ("div_one", "=(/(X,1),X)", "Divide by One"),
    ("add_self", "=(+(X,X),*(2,X))", "Self Addition"),
    ("sub_self", "=(-(X,X),0)", "Self Subtraction"),
    ("sub_self2", "=(+(X,-(X)),0)", "Self Subtraction"),
    ("sub_self3", "=(+(-(X),X),0)", "Self Subtraction"),
    ("div_self", "=(/(X,X),1)", "Self Division"),
    ("distribute_property", "=(*(X,+(Y,Z)),+(*(X,Y),*(X,Z)))", "Distributive Property"), // X*(Y+Z) = X*Y + X*Z
    ("distribute_property2", "=(*(+(Y,Z),X),+(*(Y,X),*(Z,X)))", "Distributive Property"), // (Y+Z)*X = Y*X + Z*X
];
pub fn get_algebra_rules(ctx: &Context) -> RuleMap {
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


fn generate_simple_apply_arithmetic_to_both_side_expr(op: &ArithmeticOperator, expr: &Expression) -> Expression {
    // =(_(X),{op}(X,{expr}))
    return eb::equation(
        eb::nary("_", vec![eb::variable("X")]),
        eb::binary(op.as_str(), eb::variable("X"), expr.clone())
    );
}
fn generate_simple_apply_arithmetic_to_both_side_name(op: &ArithmeticOperator, expr: &Expression) -> String {
    return format!("Apply {}{} to both side", op.to_string(), expr.to_string(true));
}

// type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, Vec<Address>) -> Vec<(Action,Expression)>;
pub mod get_possible_actions {
    use crate::arithmetic;
    use crate::expression;

    use super::*;
    pub fn algebra(expr: &Expression, context: &WorksheetContext, addr_vec: &Vec<Address>) -> Vec<(Action, Expression)>  {
        return vec![
            apply_operation_both_side(expr, addr_vec),
            arithmetic::get_possible_actions::arithmetic(expr, context, addr_vec),
            expression::get_possible_actions::from_rule_map(expr, context, addr_vec),
            apply_fraction_arithmetic(expr, addr_vec),
        ].into_iter().flatten().collect();
    }
    
    pub fn apply_operation_both_side(expr: &Expression, addr_vec: &Vec<Address>) -> Vec<(Action, Expression)>   {
        match f_apply_operation_both_side(expr, addr_vec) {
            Some((action, new_expr)) => vec![(action, new_expr)],
            None => vec![],
        }
    }
    fn f_apply_operation_both_side(expr: &Expression, addr_vec: &Vec<Address>) -> Option<(Action,Expression)>   {
        if addr_vec.len() < 2 { return None; }
        let addr0 = &addr_vec[addr_vec.len()-2]; // =
        let addr1 = &addr_vec[addr_vec.len()-1];
        // addr0 should be equation and add1 should be its grandchild
        
        let should_be_equation_expr = expr.at(addr0).ok()?;
        if !addr0.is_empty() { return None; }
        if !should_be_equation_expr.is_equation() { return None; }
        if addr1.path.len() < 2 { return None; }
        
        let addr_target = addr1.take(2).no_sub();
        let addr_parent = addr_target.parent();
        let expr_parent = expr.at(&addr_parent).ok()?;
        
        let parent_arithmetic_op = expr_parent.identify_arithmetic_operator()?;
        let inverse_op = parent_arithmetic_op.inverse();
        let expr_target = expr.at(&addr_target).ok()?;
        
        let result_expr = expr.apply_simple_arithmetic_to_both_side(&inverse_op, &expr_target).ok()?;
        let action = Action::ApplyAction(generate_simple_apply_arithmetic_to_both_side_name(&inverse_op, &expr_target));
        return Some((action, result_expr));
    }
    
    pub fn apply_fraction_arithmetic(expr: &Expression, addr_vec: &Vec<Address>) -> Vec<(Action, Expression)>   {
        match f_apply_fraction_arithmetic(expr, addr_vec) {
            Some((action, new_expr)) => vec![(action, new_expr)],
            None => vec![],
        }
    }
    fn f_apply_fraction_arithmetic(expr: &Expression, addr_vec: &Vec<Address>) -> Option<(Action,Expression)>   {
        if addr_vec.len() < 2 { return None; }
        let addr0 = &addr_vec[addr_vec.len()-2];
        let addr1 = &addr_vec[addr_vec.len()-1];
        let addr_common_ancestor = Address::common_ancestor(addr0, addr1);
        
        let addr_num = std::cmp::min(addr0, addr1);
        let addr_den = std::cmp::max(addr0, addr1);
        
        let addr_to_index = |addr: &Address| -> Option<usize> {
            if addr.is_child_of(&addr_common_ancestor){ return Some(0); }
            return addr.path.get(addr_common_ancestor.path.len()+1).copied();
        };
        
        let numerator_id   = addr_to_index(addr_num)?;
        let denominator_id = addr_to_index(addr_den)?;
        let expr = expr.apply_fraction_arithmetic_at(numerator_id, denominator_id, &addr_common_ancestor).ok()?;
        return Some((Action::ApplyAction("Simplify fraction".to_string()), expr));
    }
}
