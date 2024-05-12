use equaio::parser_prefix;
use equaio::arithmetic;

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
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 10.0);
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
        assert_eq!(normalized_expr.at(vec![0,0]).unwrap().symbol, "-1");
        assert_eq!(normalized_expr.at(vec![1,0]).unwrap().symbol, "-2");
        assert_eq!(expr.to_string(true), "(((-1) - 0) + ((-2) + 3))");
        assert_eq!(normalized_expr.to_string(true), "((-1 - 0) + (-2 + 3))");
    }
}
