use std::fmt;
use crate::expression::{Address, Expression, ExpressionError, ExpressionType, StatementSymbols};
use crate::worksheet::{WorkableExpressionSequence, Action, WorksheetContext};
use super::expression as exp;

#[derive(PartialEq, Clone)]
pub enum ArithmeticOperator {
    Add, Sub, Mul, Div,   // standard binary ops
    Negative,             // standard unary ops
    // Negative, Reciprocal, // standard unary ops
    AddTrain, MulTrain,   // operator train
}

impl fmt::Display for ArithmeticOperator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl ArithmeticOperator {
    pub fn as_str(&self) -> &str {
        match self {
            ArithmeticOperator::Add => "+",
            ArithmeticOperator::Sub => "-",
            ArithmeticOperator::Mul => "*",
            ArithmeticOperator::Div => "/",
            ArithmeticOperator::Negative => "-",
            // ArithmeticOperator::Reciprocal => "/",
            ArithmeticOperator::AddTrain => "+",
            ArithmeticOperator::MulTrain => "*",
        }
    }
    pub fn inverse(&self) -> Self {
        use ArithmeticOperator::*;
        match self {
            Add => Sub,
            Sub => Add,
            Mul => Div,
            Div => Mul,
            AddTrain => Sub,
            MulTrain => Div,
            Negative => Negative,
        }
    }
}

#[derive(Debug)]
pub enum ArithmeticError {
    ExpressionErr(ExpressionError),
    NotAnArithmeticTrainOperator,
    NotNumeric,
    CalculationError,
}

impl From<ExpressionError> for ArithmeticError {
    fn from(err: ExpressionError) -> Self {
        ArithmeticError::ExpressionErr(err)
    }
}

pub fn get_arithmetic_ctx() -> exp::Context {
    use ArithmeticOperator::*;
    exp::Context {
        // unary_ops:  vec![Negative.to_string(), Reciprocal.to_string()],
        unary_ops:  vec![Negative.to_string()],
        binary_ops: vec![Add.to_string(), Sub.to_string(), Mul.to_string(), Div.to_string()],
        assoc_ops: vec![Add.to_string(), Mul.to_string()],
        handle_numerics: true,
        ..Default::default()
    }
}

impl exp::Expression {
    pub fn identify_arithmetic_operator(&self) -> Option<ArithmeticOperator> {
        if !self.is_operator() { return None; }
        let symbol = self.symbol.as_str();
        let children_len = self.children.as_ref().map(|c| c.len()).unwrap_or(0);
        if self.is_assoc_train() {
            return match symbol {
                "+" => Some(ArithmeticOperator::AddTrain),
                "*" => Some(ArithmeticOperator::MulTrain),
                _ => None,
            }
        } else {
          return match children_len {
              0 => None,
              1 => match symbol {
                  "-" => Some(ArithmeticOperator::Negative),
                  // "/" => Some(ArithmeticOperator::Reciprocal),
                  _ => None,
              },
              2 => match symbol {
                  "+" => Some(ArithmeticOperator::Add),
                  "-" => Some(ArithmeticOperator::Sub),
                  "*" => Some(ArithmeticOperator::Mul),
                  "/" => Some(ArithmeticOperator::Div),
                  _ => None,
              },
              _ => None,
          }
        }
    }
    
    pub fn is_numeric(&self) -> bool {
        return self.exp_type == ExpressionType::ValueConst 
            && self.symbol.parse::<f64>().is_ok()
    }
    
    pub fn is_integer(&self) -> bool {
        return self.exp_type == ExpressionType::ValueConst 
            && self.symbol.parse::<i64>().is_ok()
    }
    
    pub fn is_arithmetic_train_operator(&self) -> bool {
        matches!(
            self.identify_arithmetic_operator(),
            Some(ArithmeticOperator::AddTrain) | Some(ArithmeticOperator::MulTrain)
        )
    }
    
    // directly calculatable means that the value can be calculated without needing to calculate the children first
    // (i.e. the children are all numeric values)
    pub fn is_directly_calculatable(&self) -> bool {
        if !self.is_operator() { return false; }
        // return true if all children are numeric values
        return self.children.as_ref().unwrap().iter().all(|c| c.is_numeric());
    }
    
    // if the expression is a negative unary operator on a numeric value, 
    // change it to the negative of the numeric value (as a numeric value)
    pub fn normalize_handle_negative_unary_on_numerics(&self) -> Expression {
        if self.children.is_none() { return self.clone(); }
        let op = self.identify_arithmetic_operator();
        if op == Some(ArithmeticOperator::Negative) && self.children.as_ref().unwrap()[0].is_numeric(){
            let child = self.children.as_ref().unwrap()[0].clone();
            let new_symbol = format!("{}", -child.symbol.parse::<f64>().unwrap());
            return Expression {
                symbol: new_symbol,
                children: None,
                exp_type: ExpressionType::ValueConst,
            }
        }
        let new_children = self.children.as_ref().unwrap().iter()
            .map(|c| c.normalize_handle_negative_unary_on_numerics()).collect();
        return Expression {
            symbol: self.symbol.clone(),
            children: Some(new_children),
            exp_type: self.exp_type.clone(),
        }
    }
    
    pub fn normalize_sub_to_negative(&self) -> Expression {
        match self.identify_arithmetic_operator() {
            Some(ArithmeticOperator::Sub) => {
                let children = self.children.as_ref().unwrap();
                let left = children[0].clone().normalize_sub_to_negative();
                let right = children[1].clone().normalize_sub_to_negative();
                let negative_right = Expression {
                    exp_type : ExpressionType::OperatorUnary,
                    symbol   : ArithmeticOperator::Negative.to_string(),
                    children : Some(vec![right])
                };
                return Expression {
                    exp_type : ExpressionType::OperatorBinary,
                    symbol   : ArithmeticOperator::Add.to_string(),
                    children : Some(vec![left, negative_right])
                };
            },
            _ if self.children.is_some() => {
                let children = self.children.as_ref().unwrap();
                let normalized_children = children.iter().map(|c| c.normalize_sub_to_negative()).collect();
                return Expression {
                    exp_type : self.exp_type.clone(),
                    symbol   : self.symbol.clone(),
                    children : Some(normalized_children)
                }
            }
            _ => {
                return self.clone();
            }
        }
    }
    
    // (a + (-b))
    fn is_add_negative(&self) -> bool {
        if self.identify_arithmetic_operator() != Some(ArithmeticOperator::Add) { return false; }
        if self.children.is_none() { return false; }
        let children = self.children.as_ref().unwrap();
        if children.len() != 2 { return false; }
        if children[1].identify_arithmetic_operator() != Some(ArithmeticOperator::Negative) { return false; }
        return true;
    }
    pub fn normalize_add_negative_to_sub(&self) -> Expression {
        match self.identify_arithmetic_operator() {
            Some(ArithmeticOperator::Add) if self.is_add_negative() => {
                let children = self.children.as_ref().unwrap();
                let left = children[0].clone();
                let right = children[1].clone();
                let right_children = right.children.as_ref().map(|v| v.first())
                    .expect("Negative operator should have children")
                    .expect("Negative operator should have one children")
                    .clone();
                return Expression {
                    exp_type : ExpressionType::OperatorBinary,
                    symbol   : ArithmeticOperator::Sub.to_string(),
                    children : Some(vec![left, right_children])
                };
            },
            _ if self.children.is_some() => {
                let children = self.children.as_ref().unwrap();
                let normalized_children = children.iter().map(|c| c.normalize_add_negative_to_sub()).collect();
                return Expression {
                    exp_type : self.exp_type.clone(),
                    symbol   : self.symbol.clone(),
                    children : Some(normalized_children)
                }
            }
            _ => {
                return self.clone();
            }
        }
    }
    
    pub fn generate_simple_arithmetic_equation(&self) -> Result<Expression,ArithmeticError> {
        let normalized_self = self.normalize_handle_negative_unary_on_numerics();
        if !normalized_self.is_directly_calculatable() { 
            return Err(ArithmeticError::NotNumeric);
        };
        let val = normalized_self.calculate_numeric().ok_or(ArithmeticError::CalculationError)?;
        let lhs = self.clone();
        let rhs = Expression {
            symbol: format!("{}", val),
            children: None,
            exp_type: ExpressionType::ValueConst,
        };
        return Ok(Expression {
            symbol: StatementSymbols::Equal.to_string(),
            children: Some(vec![lhs, rhs]),
            exp_type: ExpressionType::StatementOperatorBinary,
        })
    }
    
    fn generate_expr_from_train_sub_address(&self, sub_address: usize) -> Result<Expression, ArithmeticError> {
        if !self.is_arithmetic_train_operator() { 
            return Err(ArithmeticError::NotAnArithmeticTrainOperator); 
        }
        if sub_address+1 >= self.children.as_ref().unwrap().len() { 
            return Err(ArithmeticError::ExpressionErr(ExpressionError::InvalidAddress)); 
        }
        let lhs = self.children.as_ref().unwrap()[sub_address].clone();
        let rhs = self.children.as_ref().unwrap()[sub_address+1].clone();
        return Ok(Expression {
            symbol: self.symbol.clone(),
            children: Some(vec![lhs, rhs]),
            exp_type: ExpressionType::OperatorBinary,
        });
    }
    
    pub fn generate_simple_artithmetic_equation_at(&self, addr: &exp::Address) -> Result<Expression, ArithmeticError> {
        let target = self.at(addr)?;
        if let Some(sub_address) = addr.sub {
            let expr = target.generate_expr_from_train_sub_address(sub_address)?;
            return expr.generate_simple_arithmetic_equation();
        } else {
            return target.generate_simple_arithmetic_equation();
        };
    }
    
    pub fn apply_simple_arithmetic_equation_at(&self, addr: &exp::Address) -> Result<Expression, ArithmeticError> {
        let equation = self.generate_simple_artithmetic_equation_at(addr)?;
        let result = self.apply_equation_at(&equation, addr)?;
        return Ok(result);
    }

    /// Calculate the value of the expression if it is an arithmetic operation
    pub fn calculate_numeric(&self) -> Option<f64> {
        use ArithmeticOperator::*;
        // if a numeric value, return the value
        if self.is_value() { return self.symbol.parse::<f64>().ok(); }
        let op = self.identify_arithmetic_operator()?;
        let children = self.children.as_ref().unwrap();
        match op {
            Add => {
                let left = children[0].calculate_numeric()?;
                let right = children[1].calculate_numeric()?;
                Some(left + right)
            },
            Sub => {
                let left = children[0].calculate_numeric()?;
                let right = children[1].calculate_numeric()?;
                Some(left - right)
            },
            Mul => {
                let left = children[0].calculate_numeric()?;
                let right = children[1].calculate_numeric()?;
                Some(left * right)
            },
            Div => {
                let left = children[0].calculate_numeric()?;
                let right = children[1].calculate_numeric()?;
                Some(left / right)
            },
            Negative => {
                let child = children[0].calculate_numeric()?;
                Some(-child)
            },
            // Reciprocal => {
            //     let child = children[0].calculate_numeric()?;
            //     Some(1.0 / child)
            // },
            AddTrain => {
                let result = children.iter().map(|c| c.calculate_numeric()).sum::<Option<f64>>()?;
                Some(result)
            },
            MulTrain => {
                let result = children.iter().map(|c| c.calculate_numeric()).product::<Option<f64>>()?;
                Some(result)
            },
        }
    }
}

impl WorkableExpressionSequence {
    pub fn do_arithmetic_calculation_at(&mut self, addr: &Address) -> bool {
        let last_expr = self.last_expression();
        let name = format!(
            "Calculate {}", 
            last_expr.generate_simple_artithmetic_equation_at(addr)
                .map(|e| e.to_string(false)).unwrap_or("".to_string())
        );
        let expr = last_expr.apply_simple_arithmetic_equation_at(addr);
        return self.try_push(Action::ApplyAction(name), expr);
    }
}

// type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, Vec<Address>) -> Vec<(Action,Expression)>;
pub mod get_possible_actions {

    use super::*;
    pub fn arithmetic(expr: &Expression, _context: &WorksheetContext, addr_vec: &[Address]) -> Vec<(Action, Expression)>  {
        return arithmetic_calculation(expr, addr_vec);
    }
    
    pub fn arithmetic_calculation(root_expr: &Expression, addr_vec: &[Address]) -> Vec<(Action, Expression)>  {
        match f_arithmetic_calculation(root_expr, addr_vec) {
            Some((action, new_expr)) => vec![(action, new_expr)],
            None => vec![],
        }
    }
    fn f_arithmetic_calculation(root_expr: &Expression, addr_vec: &[Address]) -> Option<(Action,Expression)>  {
        if addr_vec.is_empty() { return None; }
        let addr = &Address::common_ancestor_from_vec(addr_vec);
        let result = root_expr.apply_simple_arithmetic_equation_at(addr).ok()?;
        let equation_expr = root_expr.generate_simple_artithmetic_equation_at(addr).ok()?;
        let name = format!("Calculate {}",  equation_expr.to_string(false));
        return Some((Action::ApplyAction(name), result));
    }
}
