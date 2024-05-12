use crate::expression::{Expression, ExpressionType};

use super::expression as exp;

#[derive(PartialEq, Clone)]
pub enum ArithmeticOperator {
    Add, Sub, Mul, Div,   // standard binary ops
    Negative, Reciprocal, // standard unary ops
    AddTrain, MulTrain,   // operator train
}

impl ArithmeticOperator {
    pub fn to_string(&self) -> String { self.as_str().into() }
    pub fn as_str(&self) -> &str {
        match self {
            ArithmeticOperator::Add => "+",
            ArithmeticOperator::Sub => "-",
            ArithmeticOperator::Mul => "*",
            ArithmeticOperator::Div => "/",
            ArithmeticOperator::Negative => "-",
            ArithmeticOperator::Reciprocal => "/",
            ArithmeticOperator::AddTrain => "+",
            ArithmeticOperator::MulTrain => "*",
        }
    }
}

pub fn get_arithmetic_ctx() -> exp::Context {
    use ArithmeticOperator::*;
    exp::Context {
        parameters: vec![],
        unary_ops:  vec![Negative.to_string(), Reciprocal.to_string()],
        binary_ops: vec![Add.to_string(), Sub.to_string(), Mul.to_string(), Div.to_string()],
        handle_numerics: true,
    }
}

impl exp::Expression {
    pub fn identify_arithmetic_operator(&self) -> Option<ArithmeticOperator> {
        if !self.is_operator() { return None; }
        let symbol = self.symbol.as_str();
        let children_len = self.children.as_ref().map(|c| c.len()).unwrap_or(0);
        match children_len {
            0 => None,
            1 => match symbol {
                "-" => Some(ArithmeticOperator::Negative),
                "/" => Some(ArithmeticOperator::Reciprocal),
                _ => None,
            },
            2 => match symbol {
                "+" => Some(ArithmeticOperator::Add),
                "-" => Some(ArithmeticOperator::Sub),
                "*" => Some(ArithmeticOperator::Mul),
                "/" => Some(ArithmeticOperator::Div),
                _ => None,
            },
            _ => match symbol {
                "+" => Some(ArithmeticOperator::AddTrain),
                "*" => Some(ArithmeticOperator::MulTrain),
                _ => None,
            },
        }
    }
    
    // if the expression is a negative unary operator on a numeric value, 
    // change it to the negative of the numeric value (as a numeric value)
    pub fn handle_negative_unary_on_numerics(&self) -> Expression {
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
            .map(|c| c.handle_negative_unary_on_numerics()).collect();
        return Expression {
            symbol: self.symbol.clone(),
            children: Some(new_children),
            exp_type: self.exp_type.clone(),
        }
    }
    
    pub fn is_numeric(&self) -> bool {
        return self.exp_type == ExpressionType::ValueConst 
            && self.symbol.parse::<f64>().is_ok()
    }
    
    pub fn is_arithmetic_train_operator(&self) -> bool {
        match self.identify_arithmetic_operator() {
            Some(ArithmeticOperator::AddTrain) => true,
            Some(ArithmeticOperator::MulTrain) => true,
            _ => false,
        }
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
            Reciprocal => {
                let child = children[0].calculate_numeric()?;
                Some(1.0 / child)
            },
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
