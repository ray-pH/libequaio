use equaio::parser_prefix;
use equaio::arithmetic;
use equaio::expression::Address;

#[cfg(test)]
mod calculation {
    use super::*;

    #[test]
    fn simple_addition() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(+(1,0),+(2,3))", ctx).unwrap();
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 6.0);
    }
    
    #[test]
    fn simple_addition_with_negative_number() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(+(1,0),+(-2,3))", ctx).unwrap();
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 2.0);
    }
    
    #[test]
    fn simple_addition_with_negative_unary() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(+(1,0),+(-(2),3))", ctx).unwrap();
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 2.0);
    }

    #[test]
    fn addition_train() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(1,2,3,4)", ctx).unwrap();
        assert!(match expr.identify_arithmetic_operator() {
          Some(arithmetic::ArithmeticOperator::AddTrain) => true,
          _ => false,
        });
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 10.0);
    }
    
}

#[cfg(test)]
mod generate_equation {
    use super::*;
    #[test]
    fn simple_equation() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(1,2)", ctx).unwrap();
        let eq = expr.generate_simple_arithmetic_equation().unwrap();
        assert_eq!(eq.clone().children.unwrap()[1].symbol, "3");
        assert_eq!(eq.to_string(true), "((1 + 2) = 3)");
    }
    
    #[test]
    fn generate_from_simple_address() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("*(+(1,2),3)", ctx).unwrap();
        let eq = expr.generate_simple_artithmetic_equation_at(Address::new(vec![0], None)).unwrap();
        assert_eq!(eq.to_string(true), "((1 + 2) = 3)");
    }
    
    #[test]
    fn generate_from_train() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("*(+(1,2,3,4,5),3)", ctx).unwrap();
        let eq = expr.generate_simple_artithmetic_equation_at(Address::new(vec![0], Some(2))).unwrap();
        assert_eq!(eq.to_string(true), "((3 + 4) = 7)");
    }
}

#[cfg(test)]
mod normalization {
    use super::*;
    
    #[test]
    fn negative() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(-(-(1),0),+(-(2),3))", ctx).unwrap();
        let normalized_expr = expr.handle_negative_unary_on_numerics();
        assert_eq!(normalized_expr.at(Address::new(vec![0,0], None)).unwrap().symbol, "-1");
        assert_eq!(normalized_expr.at(Address::new(vec![1,0], None)).unwrap().symbol, "-2");
        assert_eq!(expr.to_string(true), "(((-1) - 0) + ((-2) + 3))");
        assert_eq!(normalized_expr.to_string(true), "((-1 - 0) + (-2 + 3))");
    }
}
