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
    fn addition_train() {
        let ctx = arithmetic::get_arithmetic_ctx();
        let expr = parser_prefix::to_expression("+(1,2,3,4)", ctx).unwrap();
        let value = expr.calculate_numeric();
        assert_eq!(value.unwrap(), 10.0);
    }

}
