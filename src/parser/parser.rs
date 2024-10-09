use std::str::FromStr;
use super::super::expression::{Expression, ExpressionType, Context, StatementSymbols, expression_builder as eb};
use super::parser_prefix::{Token, tokenize, get_value_expression};

#[derive(Debug, Clone)]
pub enum TokenItem {
    Token(Token),
    Group(Vec<TokenItem>),
}
impl TokenItem {
    pub fn get_symbol(&self) -> String {
        if let TokenItem::Token(Token::Symbol(s)) = self { return s.clone(); }
        return "".to_string();
    }
    pub fn is_group(&self) -> bool {
        if let TokenItem::Group(_) = self { return true; }
        return false;
    }
    pub fn is_comma(&self) -> bool {
        if let TokenItem::Token(t) = self { return t == &Token::Comma; }
        return false;
    }
    
    pub fn is_function_parameter(&self) -> bool {
        if let TokenItem::Group(group) = self {
            return group.iter().any(|t| t.is_comma());
        }
        return false;
    }
    
    pub fn is_simple_value(&self, ctx: &Context) -> bool {
        return !(self.is_group() || self.is_unary_op(ctx) || self.is_effectively_binary_op(ctx));
    }
    
    pub fn is_unary_op(&self, ctx: &Context) -> bool {
        if let TokenItem::Token(Token::Symbol(s)) = self {
            if ctx.unary_ops.contains(s) { return true; }
        }
        return false;
    }
    pub fn is_effectively_binary_op(&self, ctx: &Context) -> bool {
        if let TokenItem::Token(Token::Symbol(s)) = self {
            if ctx.binary_ops.contains(s) { return true; }
            if ctx.assoc_ops.contains(s) { return true; }
            if StatementSymbols::from_str(s.as_str()).is_ok() { return true; }
        }
        return false;
    }
}

// 2 + f(x + 3) = -(x + 70)
// S(2),S(+),S(f),Open,S(x),S(+),S(3),Close,S(=),S(-),Open,S(x),S(+),S(70),Close

// f(x,y,z) = x + y + z
// S(f),Open,S(x),S(,),S(y),S(,),S(z),Close,S(=),S(x),S(+),S(y),S(+),S(z)


/// group tokens based on parentheses
///     2  + f(x + g(x,y))   = -(x + 70)
/// -> [2] [+] [f] [(x+g(x,y))] [=] [-] [(x+70)]
/// ->              [x] [+] [g] [(x,y)]
fn group_tokens_by_parentheses(tokens: &[Token]) -> TokenItem {
    let mut result: Vec<TokenItem> = Vec::new();
    let mut current: Vec<Token> = Vec::new();
    let mut paren_count = 0;
    for t in tokens {
        match t {
            Token::OpenParen => { 
                if paren_count != 0 { current.push(t.clone()); }
                paren_count += 1; 
            },
            Token::CloseParen => { 
                paren_count -= 1; 
                if paren_count != 0 {  current.push(t.clone()); }
                if paren_count == 0 {
                    result.push(group_tokens_by_parentheses(&current));
                    current.clear();
                }
            },
            _ => {
                if paren_count != 0 {
                    current.push(t.clone());
                } else {
                    result.push(TokenItem::Token(t.clone()));
                    current.clear();
                }
            },
        }
    }
    return TokenItem::Group(result);
}

#[derive(Debug)]
enum SemanticSymbol {
    Value(String),
    ValueGroup(Vec<SemanticSymbol>), // expression inside parentheses (practically is a Value)
    UnaryOp(String, Vec<SemanticSymbol>),
    BinaryOp(String),
    Nary(String, Vec<Vec<SemanticSymbol>>),
    Variadic,
}

#[allow(clippy::enum_variant_names)]
enum SemanticParsingState {
    LookingForLeftElement,
    LookingForBinaryOp,
    LookingForNaryParameter,
    LookingForUnaryParameter,
}


impl SemanticSymbol {
    pub fn value_or_variadic(s: String) -> SemanticSymbol {
        if ExpressionType::is_variadic_str(&s) {
            return SemanticSymbol::Variadic;
        } else {
            return SemanticSymbol::Value(s);
        }
    }
}
fn parse_semantic_symbol(token_items: &[TokenItem], ctx: &Context) -> Option<Vec<SemanticSymbol>> {
    let mut result: Vec<SemanticSymbol> = Vec::new();
    let mut state = SemanticParsingState::LookingForLeftElement;
    let mut temp_nary_token: Option<&TokenItem> = None;
    let mut temp_unary_token: Option<&TokenItem> = None;
    
    let mut iter = token_items.iter().peekable();
    while let Some(t) = iter.next() {
        dbg!(&result);
        match state {
            SemanticParsingState::LookingForNaryParameter => {
                if let TokenItem::Group(group) = t {
                    let params = split_token_item_by_comma(group);
                    let params_semantic = params.iter().map(|p| parse_semantic_symbol(p, ctx))
                        .collect::<Option<Vec<Vec<SemanticSymbol>>>>();
                    if temp_nary_token.is_none() || params_semantic.is_none() { return None; }
                    let nary_symbol = temp_nary_token.unwrap().get_symbol();
                    result.push(SemanticSymbol::Nary(nary_symbol, params_semantic.unwrap()));
                    temp_nary_token = None;
                    state = SemanticParsingState::LookingForBinaryOp;
                    continue;
                } else {
                    return None;
                }
            },
            SemanticParsingState::LookingForUnaryParameter => {
                temp_unary_token.as_ref()?;
                let unary_symbol = temp_unary_token.unwrap().get_symbol();
                
                if t.is_simple_value(ctx) {
                    let value = SemanticSymbol::value_or_variadic(t.get_symbol());
                    result.push(SemanticSymbol::UnaryOp(unary_symbol, vec![value]));
                    temp_unary_token = None;
                    state = SemanticParsingState::LookingForBinaryOp;
                    continue;
                } else if let TokenItem::Group(group) = t {
                    if let Some(group_semantic) = parse_semantic_symbol(group, ctx) {
                        let value = SemanticSymbol::ValueGroup(group_semantic);
                        result.push(SemanticSymbol::UnaryOp(unary_symbol, vec![value]));
                        temp_unary_token = None;
                        state = SemanticParsingState::LookingForBinaryOp;
                        continue;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            },
            SemanticParsingState::LookingForLeftElement => {
                if t.is_unary_op(ctx) {
                    // -2 + x
                    temp_unary_token = Some(t);
                    state = SemanticParsingState::LookingForUnaryParameter;
                    continue;
                } else if iter.peek().is_some() && iter.peek().unwrap().is_group() {
                    // f(x,y) + 3
                    temp_nary_token = Some(t);
                    state = SemanticParsingState::LookingForNaryParameter;
                    continue;
                } else if let TokenItem::Group(group) = t {
                    // (2 * 3) + f(x)
                    if let Some(group_semantic) = parse_semantic_symbol(group, ctx) {
                        result.push(SemanticSymbol::ValueGroup(group_semantic));
                        state = SemanticParsingState::LookingForBinaryOp;
                        continue;
                    } else {
                        return None;
                    }
                } else {
                    // 2 + f(x)
                    result.push(SemanticSymbol::value_or_variadic(t.get_symbol()));
                    state = SemanticParsingState::LookingForBinaryOp;
                    continue;
                }
            },
            SemanticParsingState::LookingForBinaryOp => {
                if t.is_effectively_binary_op(ctx) {
                    result.push(SemanticSymbol::BinaryOp(t.get_symbol()));
                    state = SemanticParsingState::LookingForLeftElement;
                    continue;
                } else {
                    return None;
                }
                
            }
        }
    }
    return Some(result);
}

pub fn split_token_item_by_comma(tokens: &Vec<TokenItem>) -> Vec<Vec<TokenItem>> {
    let mut result  : Vec<Vec<TokenItem>> = Vec::new();
    let mut current : Vec<TokenItem> = Vec::new();
    let mut paren_count = 0;
    for t in tokens {
        match t {
            TokenItem::Token(Token::OpenParen) => { paren_count += 1; current.push(t.clone()); },
            TokenItem::Token(Token::CloseParen) => { paren_count -= 1; current.push(t.clone()); },
            TokenItem::Token(Token::Comma) => {
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

#[derive(PartialEq)]
enum ExpressionParsingState {
    Initial,
    AfterValue,
    AfterOp,
}

fn semantic_to_expression(semantic: &[SemanticSymbol], ctx: &Context) -> Option<Expression> {
    let mut left_expr: Option<Expression> = None;
    let mut op_symbol: Option<String> = None;
    let mut state: ExpressionParsingState = ExpressionParsingState::Initial;
    for s in semantic {
        match state {
            ExpressionParsingState::Initial => {
                match s {
                    SemanticSymbol::Value(v) => {
                        left_expr = Some(get_value_expression(v, ctx));
                        state = ExpressionParsingState::AfterValue;
                    },
                    SemanticSymbol::ValueGroup(group) => {
                        left_expr = semantic_to_expression(group, ctx);
                        state = ExpressionParsingState::AfterValue;
                    },
                    SemanticSymbol::UnaryOp(op,param) => {
                        let child = semantic_to_expression(param, ctx);
                        left_expr = Some(eb::unary(op, child.unwrap()));
                        state = ExpressionParsingState::AfterValue;
                    },
                    SemanticSymbol::Nary(op, params) => {
                        let children = params.iter().map(|p| semantic_to_expression(p, ctx))
                            .collect::<Option<Vec<Expression>>>();
                        children.as_ref()?;
                        left_expr = Some(eb::nary(op, children.unwrap()));
                        state = ExpressionParsingState::AfterValue;
                    },
                    SemanticSymbol::BinaryOp(_) | SemanticSymbol::Variadic => {
                        // invalid
                        return None
                    },
                }
            },
            ExpressionParsingState::AfterValue => {
                match s {
                    SemanticSymbol::BinaryOp(v) => {
                        op_symbol = Some(v.clone());
                    },
                    _ => {
                        return None;
                    }
                }
                state = ExpressionParsingState::AfterOp;
            },
            ExpressionParsingState::AfterOp => {
                let right_expr: Option<Expression> = match s {
                    SemanticSymbol::Value(v) => {
                        Some(get_value_expression(v, ctx))
                    },
                    SemanticSymbol::ValueGroup(group) => {
                        semantic_to_expression(group, ctx)
                    },
                    SemanticSymbol::UnaryOp(op,param) => {
                        let child = semantic_to_expression(param, ctx);
                        Some(eb::unary(op, child.unwrap()))
                    },
                    SemanticSymbol::BinaryOp(_) => {
                        return None;
                    },
                    SemanticSymbol::Nary(op, params) => {
                        let children = params.iter().map(|p| semantic_to_expression(p, ctx))
                            .collect::<Option<Vec<Expression>>>();
                        children.as_ref()?;
                        Some(eb::nary(op, children.unwrap()))
                    },
                    SemanticSymbol::Variadic => {
                        // A + ... => +(...(A))
                        let symbol = op_symbol.as_ref()?;
                        if !ctx.assoc_ops.contains(symbol) { return None; }
                        left_expr.as_ref()?;
                        
                        left_expr = Some(eb::unary(symbol, eb::variadic(left_expr.unwrap())));
                        state = ExpressionParsingState::AfterValue;
                        // early exit for variadic
                        continue;
                    }
                };
                
                right_expr.as_ref()?;
                op_symbol.as_ref()?;
                let symbol = op_symbol.clone().unwrap();
                if left_expr.is_none(){
                    // unary op
                    left_expr = Some(eb::unary(symbol.as_str(), right_expr.unwrap()))
                } else {
                    // binary op
                    if StatementSymbols::from_str(symbol.as_str()).is_ok() {
                        left_expr = Some(eb::binary_statement(symbol.as_str(), left_expr.unwrap(), right_expr.unwrap()))
                    } else {
                        left_expr = Some(eb::binary(symbol.as_str(), left_expr.unwrap(), right_expr.unwrap()))
                    }
                }
                state = ExpressionParsingState::AfterValue;
            }
        }
    }
    
    if state != ExpressionParsingState::AfterValue { return None; }
    return left_expr;
}

pub fn to_expression_raw<T: AsRef<str>>(text: T, ctx: &Context) -> Option<Expression> {
    let s : String = text.as_ref().to_string();
    let tokens = tokenize(s, false);
    dbg!(&tokens);
    let token_items = group_tokens_by_parentheses(&tokens);
    dbg!(&token_items);
    if let TokenItem::Group(group) = &token_items {
        let semantic = parse_semantic_symbol(group, ctx);
        dbg!(&semantic);
        if let Some(semantic) = semantic {
            return semantic_to_expression(&semantic, ctx);
        }
    }
    return None;
}

fn preprocess_statement<T: AsRef<str>>(text: T) -> String {
    if text.as_ref().contains("=>") {
        let split = text.as_ref().split("=>").collect::<Vec<&str>>();
        if split.len() != 2 { return text.as_ref().to_string() }
        let left = split[0].trim();
        let right = split[1].trim();
        let processed_left = preprocess_statement(left);
        let processed_right = preprocess_statement(right);
        return format!("({}) => ({})", processed_left, processed_right);
    } else if text.as_ref().contains("=") {
        let split = text.as_ref().split("=").collect::<Vec<&str>>();
        if split.len() != 2 { return text.as_ref().to_string() }
        let left = split[0].trim();
        let right = split[1].trim();
        return format!("({}) = ({})", left, right);
    } else {
        return text.as_ref().to_string();
    }
}
fn preprocess_unary<T: AsRef<str>>(text: T, ctx: &Context) -> String {
    // if there's a unary op that is stuck to a symbol, split it
    // -b => - b
    // -a + -b => - a + - b
    let mut result = text.as_ref().to_string();
    for op in &ctx.unary_ops {
        result = result.replace(op, &format!("{} ", op));
    }
    return result;
}

pub fn to_expression<T: AsRef<str>>(text: T, ctx: &Context) -> Option<Expression> {
    let preprocessed = preprocess_statement(text);
    let preprocessed = preprocess_unary(preprocessed, ctx);
    dbg!(&preprocessed);
    let expr = to_expression_raw(preprocessed, ctx)?;
    let result = expr.normalize_to_assoc_train(&ctx.assoc_ops)
        .normalize_two_children_assoc_train_to_binary_op(&ctx.assoc_ops);
    return Some(result);
}