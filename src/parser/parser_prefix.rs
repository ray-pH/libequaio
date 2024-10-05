use std::str::FromStr;
use super::super::expression::{Expression, ExpressionType, Context, StatementSymbols};

#[derive(PartialEq, Clone)]
enum Token {
    OpenParen,
    CloseParen,
    Comma,
    Symbol(String),
}

fn tokenize(s: String) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_symbol = String::new();

    let push_symbol_if_any = |current_symbol: &mut String, tokens: &mut Vec<Token>| {
        if !current_symbol.is_empty() {
            tokens.push(Token::Symbol(current_symbol.clone()));
            current_symbol.clear();
        }
    };
    for c in s.chars() {
        match c {
            '(' => {
                push_symbol_if_any(&mut current_symbol, &mut tokens);
                tokens.push(Token::OpenParen);
            },
            ')' => {
                push_symbol_if_any(&mut current_symbol, &mut tokens);
                tokens.push(Token::CloseParen);
            },
            ',' => {
                push_symbol_if_any(&mut current_symbol, &mut tokens);
                tokens.push(Token::Comma);
            },
            _ if c.is_whitespace() => {
                // ignore whitespace
            },
            _ => {
                current_symbol.push(c);
            },
        }
    };
    push_symbol_if_any(&mut current_symbol, &mut tokens);
    tokens
}


/// split the tokens by commas
/// 1,2,f(3,4),g(f(1,2),3)
/// -> [[1], [2], [f(3,4)], [g(f(1,2),3)]]
fn split_tokens_by_comma(tokens: &Vec<Token>) -> Vec<Vec<Token>> {
    let mut result  : Vec<Vec<Token>> = Vec::new();
    let mut current : Vec<Token> = Vec::new();
    let mut paren_count = 0;
    for t in tokens {
        match t {
            Token::OpenParen => { paren_count += 1; current.push(t.clone()); },
            Token::CloseParen => { paren_count -= 1; current.push(t.clone()); },
            Token::Comma => {
                if paren_count == 0 {
                    result.push(current);
                    current = Vec::new();
                } else {
                    current.push(t.clone());
                }
            },
            _ => {
                current.push(t.clone());
            },
        }
    }
    if !current.is_empty() { result.push(current); }
    result
}

fn tokens_to_expression(tokens: &[Token], ctx: &Context) -> Option<Expression> {
    // first token must be a symbol
    if let Token::Symbol(ref s) = tokens[0] {
        if tokens.len() == 1 {
            let is_param   = ctx.parameters.contains(s);
            let is_numeric = ctx.handle_numerics && s.parse::<f64>().is_ok();
            let exp_type = if is_param || is_numeric {
                ExpressionType::ValueConst
            } else {
                ExpressionType::ValueVar
            };
            return Some(Expression {
                exp_type,
                symbol: s.clone(),
                children: None,
            });
        }
        // must be an operator, which mean
        // the second token must be a open paren
        // the last token must be a close paren
        if tokens[1] != Token::OpenParen { return None; }
        if tokens[tokens.len() - 1] != Token::CloseParen { return None; }
        // get a slice of the tokens between the open and close paren
        let inner_tokens = tokens[2..tokens.len() - 1].to_vec();
        let child_tokens = split_tokens_by_comma(&inner_tokens);
        let children = child_tokens.iter().map(
            |t| tokens_to_expression(t,ctx)).collect::<Option<Vec<Expression>>>()?;
        let exp_type = match children.len() {
            1 if ExpressionType::is_variadic_str(s) => ExpressionType::Variadic,
            1 if ctx.unary_ops.contains(s) => ExpressionType::OperatorUnary,
            2 if ctx.binary_ops.contains(s) => ExpressionType::OperatorBinary,
            2 if StatementSymbols::from_str(s.as_str()).is_ok() => ExpressionType::StatementOperatorBinary,
            _ if ctx.assoc_ops.contains(s) => ExpressionType::AssocTrain,
            _ => ExpressionType::OperatorNary,
        };
        return Some(Expression {
            exp_type,
            symbol: s.clone(),
            children: Some(children),
        });
    }
    None
}

pub fn to_expression<T: AsRef<str>>(text: T, ctx: &Context) -> Option<Expression> {
    let s : String = text.as_ref().to_string();
    tokens_to_expression(&tokenize(s), ctx)
}
