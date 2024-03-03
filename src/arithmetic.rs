use super::expression as exp;

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

    pub fn calculate_expr(&self) -> exp::Expression {
        let result = self.calculate_numeric();
        if result.is_none() { return self.clone(); }
        let result = result.unwrap();
        exp::Expression {
            exp_type: exp::ExpressionType::ValueConst,
            symbol: result.to_string(),
            children: None,
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
