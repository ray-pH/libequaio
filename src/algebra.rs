use crate::expression::{empty_context, Address, Context, Expression, ExpressionType};
use crate::arithmetic::ArithmeticOperator;
use crate::utils::gcd;
use crate::{address, parser_prefix};
use lazy_static::lazy_static;

lazy_static! {
    // because this pattern is extremely common, i think it's a good idea to hardcode it
    // A = B => _(A) = _(B)
    static ref FUNCTION_APPLICATION_TO_BOTH_SIDE_EXPR: Expression = {
        return parser_prefix::to_expression("=>(=(A,B),=(_(A),_(B)))", &empty_context()).unwrap();
    };
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
    
    pub fn apply_function_to_both_side(&self, fn_expr: Expression) -> Option<Expression> {
        if !fn_expr.is_function() { return None; }
        if !self.is_equation()    { return None; }
        let fn_lhs = &fn_expr.children.as_ref().unwrap()[0];
        let fn_symbol = fn_lhs.symbol.clone();
        // A = B => _(A) = _(B)
        let rule_expr = FUNCTION_APPLICATION_TO_BOTH_SIDE_EXPR.substitute_symbol("_".to_string(), fn_symbol);
        // wrapped_expr : _(A) = _(B)
        let wrapped_expr = self.apply_implication(rule_expr)?;
        return wrapped_expr
            .apply_equation_at(fn_expr.clone(), &address![0])?
            .apply_equation_at(fn_expr.clone(), &address![1]);
    }
    
    pub fn apply_fraction_aritmetic_at(&self, numerator_id: usize, denominator_id: usize, addr: &Address) -> Option<Expression> {
        let expr = self.at(addr)?;
        let new_expr = expr.apply_fraction_arithmetic(numerator_id, denominator_id)?;
        return self.replace_expression_at(new_expr, addr);
    }
    
    pub fn apply_fraction_arithmetic(&self, numerator_id: usize, denominator_id: usize) -> Option<Expression> {
        if self.identify_arithmetic_operator()? != ArithmeticOperator::Div { return None; }
        let expr = self.normalize_fraction();
        let children = expr.children.as_ref()?;
        let mut numerator = children[0].clone();
        let mut denominator = children[1].clone();
        let numerator_selected_expr = numerator.children.as_ref()?.get(numerator_id)?;
        let denominator_selected_expr = denominator.children.as_ref()?.get(denominator_id)?;
        if !numerator_selected_expr.is_numeric() { return None; }
        if !denominator_selected_expr.is_numeric() { return None; }
        let both_integer = numerator_selected_expr.is_integer() && denominator_selected_expr.is_integer();
        if both_integer {
            let num_val = numerator_selected_expr.symbol.parse::<i64>().ok()?;
            let den_val = denominator_selected_expr.symbol.parse::<i64>().ok()?;
            let gcd = gcd(num_val, den_val);
            let new_num_val = num_val / gcd;
            let new_den_val = den_val / gcd;
            numerator.children.as_mut()?.get_mut(numerator_id)?.symbol = new_num_val.to_string();
            denominator.children.as_mut()?.get_mut(denominator_id)?.symbol = new_den_val.to_string();
        } else {
            let num_val = numerator_selected_expr.symbol.parse::<f64>().ok()?;
            let den_val = denominator_selected_expr.symbol.parse::<f64>().ok()?;
            let new_num_val = num_val / den_val;
            let new_den_val = 1.0;
            numerator.children.as_mut()?.get_mut(numerator_id)?.symbol = new_num_val.to_string();
            denominator.children.as_mut()?.get_mut(denominator_id)?.symbol = new_den_val.to_string();
        }
        return Some(Expression{
            exp_type: ExpressionType::OperatorBinary,
            symbol: ArithmeticOperator::Div.to_string(),
            children: Some(vec![numerator, denominator]),
        });
    }
}
