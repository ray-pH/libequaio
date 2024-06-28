use std::{cmp::Ordering, collections::{HashMap, HashSet}, fmt, str::FromStr};
use super::utils;

#[derive(Debug, PartialEq, Clone, Default)]
pub enum ExpressionType {
    OperatorUnary,
    OperatorBinary,
    OperatorNary,
    #[default]
    ValueConst,
    ValueVar,
    StatementOperatorBinary,
    // `AssocTrain` is a special type for *associative* binary operators
    // this is introduced because when representing expression as a tree,
    // the associative binary operator train can be represented in multiple ways
    // ex: `a + b + c + d` can be represented as `((a + b) + c) + d` or `a + ((b + c) + d)`
    // the value of the expression is the same, but the structure is different
    // it's hard to go from the first representation to the second one, or vice versa
    // it requires a lot of application of the associative property
    AssocTrain, 
    // `Variadic` is a special type for desribing rules that can have any number of arguments
    // ex `k*(a+b+...+n) = k*a + k*b + ... + k*n` works for any number of arguments
    // technically you can work with multiple application of binary addition, but it's not practical
    // `Variadic` can only be inside AssocTrain
    Variadic, 
}
impl ExpressionType {
    pub fn variadic_string() -> String { "...".into() }
    pub fn is_variadic_str(str: &str) -> bool {
        return str == "...";
    }
}

pub enum StatementSymbols {
    Equal,
    Implies,
}
impl FromStr for StatementSymbols {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "=" => Ok(StatementSymbols::Equal),
            "=>" => Ok(StatementSymbols::Implies),
            _ => Err(()),
        }
    }
}
impl fmt::Display for StatementSymbols {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
impl StatementSymbols {
    pub fn as_str(&self) -> &str {
        match self {
            StatementSymbols::Equal => "=",
            StatementSymbols::Implies => "=>"
        }
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Expression {
    pub exp_type: ExpressionType,
    pub symbol: String,
    pub children: Option<Vec<Expression>>,
}

#[derive(Clone, Default)]
pub struct Context {
    pub parameters: Vec<String>,
    pub unary_ops: Vec<String>,
    pub binary_ops: Vec<String>,
    pub assoc_ops: Vec<String>,
    // the rest of the symbols will be considered as n-ary operators (functions)
    pub handle_numerics: bool,
    pub flags: HashSet<String>
}


impl Context {
    pub fn add_param(&mut self, param: String) -> Context {
        if !self.parameters.contains(&param) { self.parameters.push(param); }
        return self.clone();
    }
    pub fn add_params(&mut self, params: Vec<String>) -> Context {
        for p in params { self.add_param(p); }
        return self.clone();
    }
    pub fn remove_param(&mut self, param: &String) -> Context {
        self.parameters.retain(|p| p != param);
        return self.clone();
    }
    pub fn remove_params(&mut self, params: &Vec<String>) -> Context {
        for p in params { self.remove_param(p); }
        return self.clone();
    }
    
    pub fn add_flag(&mut self, flag: impl ToString) -> Context {
        self.flags.insert(flag.to_string());
        return self.clone();
    }
    pub fn add_flags(&mut self, flags: Vec<impl ToString>) -> Context {
        for f in flags { self.add_flag(f.to_string()); }
        return self.clone();
    }
    
    pub fn contains_flag(&self, flag: impl ToString) -> bool {
        return self.flags.contains(&flag.to_string());
    }
    pub fn contains_all_flags(&self, flags: &Vec<impl ToString>) -> bool {
        for f in flags { if !self.flags.contains(&f.to_string()) { return false; } }
        return true;
    }
}


#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Address {
    pub path: Vec<usize>,
    pub sub: Option<usize>, // sub if for addressing subexpression in AssocTrain
}

impl Address {
    pub fn new(path: Vec<usize>, sub: Option<usize>) -> Self {
        return Address { path, sub };
    }
    pub fn append(&self, val: usize) -> Self {
        let mut new_path = self.path.clone();
        new_path.push(val);
        return Address { path: new_path, sub: self.sub };
    }
    pub fn sub(&self, val: usize) -> Self {
        return Address { path: self.path.clone(), sub: Some(val) };
    }
    pub fn no_sub(&self) -> Self {
        return Address { path: self.path.clone(), sub: None };
    }
    pub fn tail(&self) -> Self {
        return Address { path: self.path[1..].to_vec(), sub: self.sub };
    }
    pub fn head(&self) -> usize {
        return self.path[0];
    }
    pub fn last(&self) -> usize {
        return self.path[self.path.len()-1];
    }
    pub fn parent(&self) -> Self {
        return Address { path: self.path[..self.path.len()-1].to_vec(), sub: self.sub };
    }
    pub fn is_empty(&self) -> bool {
        return self.path.is_empty() && self.sub.is_none();
    }
    pub fn take(&self, n: usize) -> Self {
        return Address { path: self.path[..n].to_vec(), sub: self.sub };
    }
    pub fn to_vec(&self) -> Vec<usize> {
        let mut v = self.path.clone();
        if let Some(sub) = self.sub { v.push(sub); }
        return v;
    }
    
    pub fn common_ancestor(a: &Address, b: &Address) -> Address {
        let mut i = 0;
        while i < a.path.len() && i < b.path.len() && a.path[i] == b.path[i] { i += 1; }
        return Address::new(a.path[..i].to_vec(), None);
    }
    
    /// 0-th order cousin is sibling
    pub fn is_nth_cousin(&self, other: &Address, n: usize) -> bool {
        if self.path.len() != other.path.len() { return false; }
        if self.path.len() < n+1 { return false; }
        return self.path[..self.path.len()-n-1] == other.path[..other.path.len()-n-1];
    }
    pub fn is_sibling(&self, other: &Address) -> bool {
        return self.is_nth_cousin(other, 0);
    }
    pub fn is_nth_order_child_of(&self, other: &Address, n: usize) -> bool {
        if self.path.len() != other.path.len() + n { return false; }
        if self.path.len() < n { return false; }
        return self.path[..self.path.len()-n] == other.path[..];
    }
    pub fn is_child_of(&self, other: &Address) -> bool {
        return self.is_nth_order_child_of(other, 1);
    }
    pub fn is_grandchild_of(&self, other: &Address) -> bool {
        return self.is_nth_order_child_of(other, 2);
    }
    
}

impl Ord for Address {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_vec().cmp(&other.to_vec())
    }
}
impl PartialOrd for Address {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[macro_export]
macro_rules! address {
    ($($elem:expr),*) => {
        Address::new(vec![$($elem),*], None)
    };
}

#[macro_export]
macro_rules! ctxflag {
    ($($elem:expr),*) => {
        HashSet::from(vec![$($elem.to_string()),*], None)
    };
}

pub type MatchMap = HashMap<String,Expression>;

#[derive(Debug)]
pub enum ExpressionError {
    InvalidAddress,
    ImplicationLHSMismatch(String, String),
    EquationLHSMismatch(String, String),
    ExpressionContainsVariable,
    PatternDoesNotMatch,
    NotAnEquation,
    NotAnImplication,
    NotAnAssocTrain,
    NotAParentOfVariadic,
    InvalidRule,
}

impl Expression {
    pub fn is_operator(&self) -> bool {
        matches!(
            self.exp_type,
            ExpressionType::OperatorUnary |
            ExpressionType::OperatorBinary |
            ExpressionType::OperatorNary |
            ExpressionType::StatementOperatorBinary |
            ExpressionType::AssocTrain
        )
    }
    pub fn is_assoc_train(&self) -> bool {
        return self.exp_type == ExpressionType::AssocTrain;
    }
    pub fn is_parent_of_variadic(&self) -> bool {
        if !self.is_assoc_train() { return false; }
        if let Some(children) = &self.children {
            if let Some(first) = children.first() {
                return first.exp_type == ExpressionType::Variadic;
            }
        }
        return false;
    }
    pub fn is_value(&self) -> bool {
        matches!(self.exp_type,
            ExpressionType::ValueConst | ExpressionType::ValueVar
        )
    }
    pub fn is_variable(&self) -> bool {
        return self.exp_type == ExpressionType::ValueVar;
    }
    pub fn is_constant(&self) -> bool {
        return self.exp_type == ExpressionType::ValueConst;
    }
    pub fn is_statement(&self) -> bool {
        return self.exp_type == ExpressionType::StatementOperatorBinary;
    }
    pub fn is_equation(&self) -> bool {
        match self.identify_statement_operator() { 
            Some(StatementSymbols::Equal) => {
                if self.children.is_none() { return false; }
                return self.children.as_ref().unwrap().len() == 2;
            }, 
            _ => false 
        }
    }
    pub fn is_implication(&self) -> bool {
        match self.identify_statement_operator() { 
            Some(StatementSymbols::Implies) => {
                if self.children.is_none() { return false; }
                return self.children.as_ref().unwrap().len() == 2;
            },
            _ => false 
        }
    }
    pub fn identify_statement_operator(&self) -> Option<StatementSymbols> {
        if !self.is_statement() { return None; }
        let symbol = self.symbol.as_str();
        return StatementSymbols::from_str(symbol).ok();
    }
    
    pub fn is_contain_variable(&self) -> bool {
        if self.is_variable() { return true; }
        if self.children.is_none() { return false; }
        let children = self.children.as_ref().unwrap();
        // return true if any of the children is a variable
        return children.iter().any(|c| c.is_contain_variable());
    }
    
    // pub fn is_contain_variadic(&self) -> bool {
    //     if self.exp_type == ExpressionType::Variadic { return true; }
    //     if self.children.is_none() { return false; }
    //     let children = self.children.as_ref().unwrap();
    //     // return true if any of the children is a variadic
    //     return children.iter().any(|c| c.is_contain_variadic());
    // }
    
    // pub fn count_contained_variadic(&self) -> usize {
    //     if self.exp_type == ExpressionType::Variadic { return 1; }
    //     if self.children.is_none() { return 0; }
    //     let children = self.children.as_ref().unwrap();
    //     return children.iter().map(|c| c.count_contained_variadic()).sum();
    // }
    
    // pub fn is_contain_only_one_variadic(&self) -> bool {
    //     return self.count_contained_variadic() == 1;
    // }
    
    pub fn is_equivalent_to(&self, other: &Self) -> bool {
        let match_map = self.pattern_match_this_node(other);
        if match_map.is_none() { return false; }
        let new_other = other.apply_match_map(&match_map.unwrap());
        return *self == new_other;
    }

    pub fn to_string(&self, parentheses: bool) -> String {
        match &self.exp_type {
            ExpressionType::ValueConst => self.symbol.clone(),
            ExpressionType::ValueVar => self.symbol.clone(),
            ExpressionType::OperatorUnary => 
                if parentheses {
                    format!("({}{})", self.symbol, 
                        self.children.as_ref().unwrap()[0].to_string(parentheses))
                } else {
                    format!("{}{}", self.symbol, 
                        self.children.as_ref().unwrap()[0].to_string(parentheses))
                },
            ExpressionType::StatementOperatorBinary |
            ExpressionType::OperatorBinary => 
                if parentheses {
                    format!("({} {} {})", 
                        self.children.as_ref().unwrap()[0].to_string(parentheses), 
                        self.symbol, 
                        self.children.as_ref().unwrap()[1].to_string(parentheses))
                } else {
                    format!("{} {} {}", 
                        self.children.as_ref().unwrap()[0].to_string(parentheses), 
                        self.symbol, 
                        self.children.as_ref().unwrap()[1].to_string(parentheses))
                },
            ExpressionType::OperatorNary => {
                let mut result = String::new();
                result.push_str(&self.symbol);
                result.push('(');
                for (i, c) in self.children.as_ref().unwrap().iter().enumerate() {
                    if i > 0 { result.push_str(", "); }
                    result.push_str(&c.to_string(parentheses));
                }
                result.push(')');
                result
            },
            ExpressionType::AssocTrain if self.is_parent_of_variadic() => {
                if let Ok(grandchild) = self.at(&address![0,0]){
                    let variables = grandchild.collect_var(None);
                    let variable_symbols = variables.iter().map(|v| v.symbol.clone()).collect::<Vec<String>>();
                    
                    let mut granchild1 = grandchild.clone();
                    let mut granchild2 = grandchild.clone();
                    for symbol in &variable_symbols {
                        let new_symbol1 = symbol.clone() + "_1";
                        let new_symbol2 = symbol.clone() + "_2";
                        granchild1 = granchild1.substitute_symbol(symbol.clone(), new_symbol1);
                        granchild2 = granchild2.substitute_symbol(symbol.clone(), new_symbol2);
                    }
                    
                    let string = format!("{} {} {} {} ...", 
                        granchild1.to_string(parentheses),  self.symbol, 
                        granchild2.to_string(parentheses), self.symbol
                    );
                    if parentheses { return format!("({})", string); }
                    else { return string; }
                } else {
                    return "UNREACHABLE, VARIADIC MARKER SHOULD HAVE A CHILDREN".to_string();
                }
            },
            ExpressionType::AssocTrain => {
                let mut result = String::new();
                if parentheses { result.push('(') };
                for (i, c) in self.children.as_ref().unwrap().iter().enumerate() {
                    if i > 0 { 
                        result.push(' ');
                        result.push_str(&self.symbol);
                        result.push(' ');
                    }
                    result.push_str(&c.to_string(parentheses));
                }
                if parentheses { result.push(')') };
                result
            },
            ExpressionType::Variadic => {
                return "UNREACHABLE, VARIADIC MARKER SHOULD NEVER BE PRINTED".to_string();
            }
        }
    }
    
    pub fn collect_var(&self, const_symbols: Option<HashSet<String>>) -> Vec<Expression> {
        if self.is_variable() { 
            if let Some(const_symbols) = const_symbols {
                if const_symbols.contains(&self.symbol) { return Vec::default(); }
            }
            return vec![self.clone()]; 
        }
        if let Some(children) = &self.children {
            return children.iter().flat_map(|c| c.collect_var(const_symbols.clone())).collect();
        }
        return Vec::default();
    }
    
    pub fn substitute_symbol(&self, from: String, to: String) -> Expression {
        let mut new_exp = self.clone();
        if new_exp.symbol == from { new_exp.symbol.clone_from(&to); }
        if new_exp.children.is_some() {
            for c in new_exp.children.as_mut().unwrap() {
                *c = c.substitute_symbol(from.clone(), to.clone());
            }
        }
        return new_exp;
    }
    
    #[allow(clippy::needless_range_loop)]
    pub fn expand_variadic(&self, n: usize, const_symbols: Option<HashSet<String>>) -> Result<Expression, ExpressionError> {
        if !self.is_parent_of_variadic() { return Err(ExpressionError::NotAParentOfVariadic); }
        
        let inner_expr = self.at(&address![0,0])?;
        let variables = inner_expr.collect_var(const_symbols);
        let variable_symbols = variables.iter().map(|v| v.symbol.clone()).collect::<Vec<String>>();
        
        let mut children: Vec<_> = (0..n).map(|_| inner_expr.clone()).collect();
        for symbol in &variable_symbols {
            for i in 0..n {
                let new_symbol = symbol.clone() + "_" + &(i+1).to_string();
                children[i] = children[i].substitute_symbol(symbol.clone(), new_symbol);
            }
        }
        return Ok(Expression {
            exp_type: ExpressionType::AssocTrain,
            symbol: self.symbol.clone(),
            children: Some(children)
        })
    }
    
    pub fn lhs(&self) -> Option<&Expression> {
        if !self.is_equation() { return None; }
        return self.at(&address![0]).ok();
    }
    pub fn rhs(&self) -> Option<&Expression> {
        if !self.is_equation() { return None; }
        return self.at(&address![1]).ok();
    }

    /// Get the expression from the address
    pub fn at(&self, address: &Address) -> Result<&Expression, ExpressionError> {
        if address.path.is_empty() { 
            return Ok(self)
        }
        if self.children.is_none() { return Err(ExpressionError::InvalidAddress); }
        let index = address.path[0];
        if index >= self.children.as_ref().unwrap().len() { 
            return Err(ExpressionError::InvalidAddress);
        }
        return self.children.as_ref().unwrap()[index].at(&address.tail());
    }
    
    /// Normalize the expression, grouping the expression with the same operator together as a train
    /// ex: ((1 + 2) + 3) -> +(1, 2, 3)
    pub fn normalize_to_assoc_train(&self, assoc_ops: &Vec<String>) -> Expression {
        if self.children.is_none() { return self.clone() }
        
        let normalized_children = self.children.as_ref().unwrap().iter().map(|c| c.normalize_to_assoc_train(assoc_ops));
        
        if !assoc_ops.contains(&self.symbol) { 
            return Expression {
                exp_type : self.exp_type.clone(),
                symbol   : self.symbol.clone(),
                children : Some(normalized_children.collect()),
            }
        }
        
        let mut chidren : Vec<Expression> = Vec::new();
        for normalized_child in normalized_children {
            if normalized_child.is_assoc_train() 
                && normalized_child.symbol == self.symbol 
                && normalized_child.children.is_some() {
                chidren.extend(normalized_child.children.unwrap());
            } else {
                chidren.push(normalized_child) 
            }
        }
        return Expression {
            exp_type : ExpressionType::AssocTrain,
            symbol   : self.symbol.clone(),
            children : Some(chidren),
        }
    }
    
    /// turn assoc train that has two children into binary operator
    pub fn normalize_two_children_assoc_train_to_binary_op(&self, binary_ops: &Vec<String>) -> Expression {
        if self.children.is_none() { return self.clone() }
        let children = self.children.as_ref().unwrap();
        let normalized_children = children.iter().map(|c| c.normalize_two_children_assoc_train_to_binary_op(binary_ops));
        if self.is_assoc_train() && children.len() == 2 && binary_ops.contains(&self.symbol) {
            return Expression {
                exp_type : ExpressionType::OperatorBinary,
                symbol   : self.symbol.clone(),
                children : Some(normalized_children.collect()),
            }
        } else {
            return Expression {
                exp_type : self.exp_type.clone(),
                symbol   : self.symbol.clone(),
                children : Some(normalized_children.collect()),
            }
        }
    }
    
    pub fn normalize_single_children_assoc_train(&self) -> Expression {
        if self.children.is_none() { return self.clone() }
        let children = self.children.as_ref().unwrap();
        let mut normalized_children = children.iter().map(|c| c.normalize_single_children_assoc_train());
        if self.is_assoc_train() && children.len() == 1 {
            return normalized_children.next().unwrap();
        } else {
            return Expression {
                exp_type : self.exp_type.clone(),
                symbol   : self.symbol.clone(),
                children : Some(normalized_children.collect()),
            }
        }
    }
    
    /// turn values into a single child assoc train
    /// turn binary operator into a two children assoc train
    /// * `NOTE`: this function doesn't check if the operator is associative or not
    pub fn turn_into_assoc_train(&self, symbol: String) -> Expression {
        let children =  if self.children.is_none() {
            Some(vec![self.clone()])
        } else {
            self.children.clone()
        };
        return Expression {
            exp_type : ExpressionType::AssocTrain,
            symbol,
            children,
        }
    }
    
    pub fn generate_subexpr_from_train(&self, sub_address: usize) -> Result<Expression, ExpressionError> {
        if !self.is_assoc_train() { return Err(ExpressionError::NotAnAssocTrain); }
        let children = self.children.as_ref().unwrap();
        let children_len = children.len();
        if sub_address+1 >= children_len { return Err(ExpressionError::InvalidAddress); }
        let lhs = children[sub_address].clone();
        let rhs = children[sub_address+1].clone();
        return Ok(Expression{
            exp_type: ExpressionType::OperatorBinary,
            symbol: self.symbol.clone(),
            children: Some(vec![lhs, rhs]),
        });
    }

    pub fn get_pattern_matches(&self, pattern: &Expression) -> Vec<(Address,MatchMap)> {
        return self.f_get_patten_matches(pattern, &Address::default(), true);
    }
    
    /// Try to match the pattern expression with this expression, and all its children.
    /// Returns a list of maps, where each map represents a match.
    fn f_get_patten_matches(&self, pattern: &Expression, current_address: &Address, check_children: bool) -> Vec<(Address,MatchMap)> {
        let mut result = Vec::new();

        // try to match the root node
        let root_map = self.pattern_match_this_node(pattern);
        if let Some(map) = root_map { result.push((current_address.clone(), map)); }
        
        // try to match the subexpression (if is a train)
        if check_children && self.is_assoc_train() {
            for i in 0..self.children.as_ref().unwrap().len()-1 {
                let subexpr = self.generate_subexpr_from_train(i);
                if let Ok(sub) = subexpr {
                    let sub_matches = sub.f_get_patten_matches(pattern, &current_address.sub(i), false);
                    sub_matches.iter().for_each(|m| {
                        result.push(m.clone());
                    });
                }
            }
        }

        // try to match the children
        if check_children && self.children.is_some() {
            let children = self.children.as_ref().unwrap();
            for (i,c) in children.iter().enumerate() {
                let child_address = current_address.append(i);
                let child_matches = c.f_get_patten_matches(pattern, &child_address, true);
                // push the child matches to the result
                child_matches.iter().for_each(|m| {
                    result.push(m.clone());
                });
            }
        }

        return result;
    }
    
    /// Try to match the pattern expression with expression at the given address
    pub fn pattern_match_at(&self, pattern: &Expression, addr: &Address) -> Result<MatchMap, ExpressionError> {
        let curr_node = self.at(addr)?;
        if let Some(sub_addr) = addr.sub {
            let sub_expr = curr_node.generate_subexpr_from_train(sub_addr)?;
            return sub_expr.pattern_match_this_node(pattern)
                .ok_or(ExpressionError::PatternDoesNotMatch);
        } else {
            return curr_node.pattern_match_this_node(pattern)
                .ok_or(ExpressionError::PatternDoesNotMatch);
        }
    }


    /// Try to match the pattern expression with this expression. (match the root node)
    /// Returns a map of the symbols, or None if there is the pattern does not match.
    pub fn pattern_match_this_node(&self, pattern: &Expression) -> Option<MatchMap> {
        use ExpressionType::*;
        match pattern.exp_type {
            // if the pattern is a constant parameter, then it must match exactly with this expression
            ValueConst => {
                if pattern.symbol == self.symbol && pattern.exp_type == self.exp_type {
                    let empty_map = HashMap::new();
                    return Some(empty_map);
                }
                None
            },
            // if the pattern is a variable, then it must mapped to this expression
            ValueVar => {
                let mut map = HashMap::new();
                map.insert(pattern.symbol.clone(), self.clone());
                Some(map)
            },
            // if the pattern is a variadic assoc train,
            // then expand the pattern first to have the same size as the train
            AssocTrain if pattern.is_parent_of_variadic() => {
                // invalid if the expr is not a train
                if !self.is_assoc_train() { return None; }
                let children = self.children.as_ref()?;
                let children_len = children.len();
                let new_pattern = pattern.expand_variadic(children_len, None).ok()?;
                let mut match_map = self.pattern_match_this_node(&new_pattern)?;
                match_map.insert(
                    ExpressionType::variadic_string(), 
                    expression_builder::constant(&children_len.to_string()));
                return Some(match_map);
            },
            // if the pattern is an operator, then it must match
            // then, pattern match each child
            StatementOperatorBinary | OperatorUnary | OperatorBinary | OperatorNary | AssocTrain => {
                // invalid if the symbol or type is different
                if pattern.symbol != self.symbol { return None; }
                if pattern.exp_type != self.exp_type { return None; }
                // invalid if one of them does not have children (operator must have children)
                if self.children.is_none() || pattern.children.is_none() { return None; }
                // invalid if the number of children is different
                let self_children = self.children.as_ref().unwrap();
                let pattern_children = pattern.children.as_ref().unwrap();
                if self_children.len() != pattern_children.len() { return None; }
                // pattern match each child
                let mut map = HashMap::new();
                for i in 0..self_children.len() {
                    let child_map = self_children[i].pattern_match_this_node(&pattern_children[i])?;
                    // invalid if the child maps clash
                    if !utils::is_hashmap_no_clash(&map, &child_map) { return None; }
                    // merge the child map with the current map
                    for (k,v) in child_map { map.insert(k,v); }
                }
                Some(map)
            },
            Variadic => {
                todo!()
            }
        }
    }
    
    // apply match map to the expression
    // use case: self is a "rule expression" e.g. X + 0 = X
    pub fn apply_match_map(&self, match_map: &MatchMap) -> Expression {
        match self.exp_type {
            ExpressionType::ValueVar => {
                if let Some(expr) = match_map.get(&self.symbol) { return expr.clone(); }
                return self.clone();
            },
            _ if self.is_parent_of_variadic() => {
                if let Some(n_expr) = match_map.get(&ExpressionType::variadic_string()) {
                    let n = n_expr.symbol.parse::<usize>();
                    if n.is_err() { return self.clone(); }
                    let n = n.unwrap();
                    
                    let const_symbols : HashSet<String> = match_map.iter()
                        .filter(|(_,v)| v.is_constant())
                        .map(|(k,_)| k.clone())
                        .collect();
                    let resolved_self = self.expand_variadic(n, Some(const_symbols));
                    if resolved_self.is_err() { return self.clone(); }
                    return resolved_self.unwrap().apply_match_map(match_map);
                } else {
                    return self.clone();
                }
            },
            _ if self.children.is_some() => {
                let children = self.children.as_ref().unwrap();
                let new_children = children.iter().map(|c| c.apply_match_map(match_map)).collect();
                return Expression {
                    exp_type: self.exp_type.clone(),
                    symbol: self.symbol.clone(),
                    children: Some(new_children)
                }
            },
            _ => return self.clone()
        }
    }
    
    /// Create a new expression by replacing the expression at the address with the new expression
    pub fn replace_expression_at(&self, new_expr: Expression, addr: &Address) -> Result<Expression, ExpressionError> {
        if addr.path.is_empty() { 
            if addr.sub.is_none() { return Ok(new_expr);  }
            if !self.is_assoc_train() { return Err(ExpressionError::NotAnAssocTrain); }
            return self.replace_expression_at_train(new_expr, addr.sub.unwrap());
        }
        if self.children.is_none() { return Err(ExpressionError::InvalidAddress); }
        if addr.head() >= self.children.as_ref().unwrap().len() { 
            return Err(ExpressionError::InvalidAddress);
        }
        else {
            let new_child = self.children.as_ref().unwrap()[addr.head()]
                .replace_expression_at(new_expr, &addr.tail())?;
            let mut new_children = self.children.as_ref().unwrap().clone();
            new_children[addr.head()] = new_child;
            return Ok(Expression {
                exp_type : self.exp_type.clone(),
                symbol : self.symbol.clone(),
                children : Some(new_children),
            });
        }
    }
    
    pub fn swap_assoc_train_children_at(&self, index0: usize, index1: usize, addr: &Address) 
    -> Result<Expression, ExpressionError> {
        let expr = self.at(addr)?;
        let new_expr = expr.swap_assoc_train_children(index0, index1)?;
        return self.replace_expression_at(new_expr, addr);
    }
    
    pub fn swap_assoc_train_children(&self, index0: usize, index1: usize) -> Result<Expression, ExpressionError> {
        if !self.is_assoc_train() { return Err(ExpressionError::NotAnAssocTrain); }
        let children = self.children.as_ref().unwrap();
        let children_count = children.len();
        
        if index0 >= children_count || index1 >= children_count {
            return Err(ExpressionError::InvalidAddress);
        }
        
        let mut new_children = children.clone();
        new_children.swap(index0, index1);
        return Ok(Expression {
            exp_type : self.exp_type.clone(),
            symbol : self.symbol.clone(),
            children : Some(new_children),
        });
    }
    
    fn replace_expression_at_train(&self, new_expr: Expression, sub_address: usize) -> Result<Expression, ExpressionError> {
        if !self.is_assoc_train() { return Err(ExpressionError::NotAnAssocTrain); }
        let children = self.children.as_ref().unwrap();
        if sub_address >= children.len()-1 { return Err(ExpressionError::InvalidAddress); }
        let left_children = children[0..sub_address].to_vec();
        let right_children = children[sub_address+2..children.len()].to_vec();
        return Ok(Expression {
            exp_type : self.exp_type.clone(),
            symbol : self.symbol.clone(),
            children : Some([left_children, vec![new_expr], right_children].concat()),
        });
    }
    
    pub fn flip_equation(&self) -> Expression {
        if !self.is_equation() { return self.clone(); }
        let children = self.children.as_ref().unwrap();
        let left = children[0].clone();
        let right = children[1].clone();
        return Expression {
            exp_type : self.exp_type.clone(),
            symbol : self.symbol.clone(),
            children : Some(vec![right, left]),
        };
    }
    pub fn apply_equation_this_node(&self, equation: &Expression) -> Result<Expression, ExpressionError> {
        return self.apply_equation_ltr_this_node(equation);
    }
    pub fn apply_equation_rtl_this_node(&self, equation: &Expression) -> Result<Expression, ExpressionError> {
        return self.apply_equation_ltr_this_node(&equation.flip_equation());
    }
    pub fn apply_equation_ltr_this_node(&self, equation: &Expression) -> Result<Expression, ExpressionError> {
        if !equation.is_equation() { return Err(ExpressionError::NotAnEquation); }
        
        let eq_children = equation.children.as_ref().ok_or(ExpressionError::InvalidAddress)?;
        let lhs = &eq_children[0];
        
        if lhs == self {
            let rhs = eq_children[1].clone();
            return Ok(rhs);
        } else {
            let match_map = self.pattern_match_this_node(lhs)
                .ok_or(ExpressionError::PatternDoesNotMatch)?;
            let equation = equation.apply_match_map(&match_map);
            return self.apply_equation_ltr_this_node(&equation);
        }
    }
    pub fn apply_equation_at(&self, equation: &Expression, addr: &Address) -> Result<Expression, ExpressionError> {
        return self.apply_equation_ltr_at(equation, addr);
    }
    pub fn apply_equation_rtl_at(&self, equation: &Expression, addr: &Address) -> Result<Expression, ExpressionError> {
        return self.apply_equation_ltr_at(&equation.clone().flip_equation(), addr);
    }
    pub fn apply_equation_ltr_at(&self, equation: &Expression, addr: &Address) -> Result<Expression, ExpressionError> {
        let expr = self.at(addr)?;
        if addr.sub.is_none() {
            let new_expr = expr.apply_equation_this_node(equation)?;
            return self.replace_expression_at(new_expr, addr);
        } else {
            // AssocTrain
            let subexpr = expr.generate_subexpr_from_train(addr.sub.unwrap())?;
            let new_expr = subexpr.apply_equation_this_node(equation)?;
            return self.replace_expression_at(new_expr, addr);
        }
    }
    
    pub fn get_possible_equation_application_addresses(&self, equation: &Expression) -> Vec<Address> {
        if !equation.is_equation() { return vec![]; }
        let eq_children = equation.children.as_ref();
        if eq_children.is_none() { return vec![]; }
        let lhs = &eq_children.unwrap()[0];
        let pattern_matches = self.get_pattern_matches(lhs);
        return pattern_matches.iter().map(|(addr,_)| addr.clone()).collect()
    }
    
    pub fn apply_implication(&self, implication: &Expression) -> Result<Expression, ExpressionError>{
        if !implication.is_implication() { return Err(ExpressionError::NotAnImplication); }
        
        let impl_children = implication.children.as_ref().ok_or(ExpressionError::InvalidAddress)?;
        let lhs = impl_children[0].clone();
        
        if !implication.is_contain_variable() {
            // if the implication contains no variables (all of the values are parameters)
            // then the implication must match the current node
            if !(&lhs == self) { 
                return Err(ExpressionError::ImplicationLHSMismatch(lhs.to_string(true), self.to_string(true)));  
            }
            let rhs = impl_children[1].clone();
            return Ok(rhs);
        } else {
            // if the implication contains variables (not all of the value is parameters)
            // try to pattern match and transform the implication first
            let match_map = self.pattern_match_this_node(&lhs)
                .ok_or(ExpressionError::PatternDoesNotMatch)?;
            let implication = implication.apply_match_map(&match_map);
            // if theres still a variable in the implication, then the implication is invalid
            if implication.is_contain_variable() { return Err(ExpressionError::ExpressionContainsVariable); }
            return self.apply_implication(&implication);
        }
    }
    
}

pub mod expression_builder {
    use super::*;
    pub fn constant(symbol: &str) -> Expression {
        return Expression {
            exp_type : ExpressionType::ValueConst,
            symbol : symbol.to_string(),
            children : None,
        };
    }
    
    pub fn variable(symbol: &str) -> Expression {
        return Expression {
            exp_type : ExpressionType::ValueVar,
            symbol : symbol.to_string(),
            children : None,
        };
    }
    
    pub fn equation(lhs: Expression, rhs: Expression) -> Expression {
        return Expression {
            exp_type : ExpressionType::StatementOperatorBinary,
            symbol : StatementSymbols::Equal.to_string(),
            children : Some(vec![lhs, rhs]),
        };
    }
    
    pub fn unary(symbol: &str, child: Expression) -> Expression {
        return Expression {
            exp_type : ExpressionType::OperatorUnary,
            symbol   : symbol.to_string(),
            children : Some(vec![child]),
        };
    }
    
    pub fn binary(symbol: &str, left: Expression, right: Expression) -> Expression {
        return Expression {
            exp_type : ExpressionType::OperatorBinary,
            symbol   : symbol.to_string(),
            children : Some(vec![left, right]),
        };
    }
    
    pub fn nary(symbol: &str, children: Vec<Expression>) -> Expression {
        return Expression {
            exp_type : ExpressionType::OperatorNary,
            symbol   : symbol.to_string(),
            children : Some(children),
        };
    }
    
}

// type GetPossibleActionsFunction = fn(&Expression, &WorksheetContext, Vec<Address>) -> Vec<(Action,Expression)>;
pub mod get_possible_actions {
    use crate::worksheet::{Action, WorksheetContext};

    use super::*;
    pub fn from_rule_map(expr: &Expression, context: &WorksheetContext, addr_vec: &[Address]) -> Vec<(Action, Expression)>  {
        if addr_vec.is_empty() { return vec![]; }
        let addr = &addr_vec[addr_vec.len()-1];
        let rule_map = &context.rule_map;
        let mut possible_actions = Vec::new();
        for (_, rule) in rule_map.iter() {
            if let Ok(new_expr) = expr.apply_rule_at(rule, addr) {
                let action = Action::ApplyRule(rule.label.clone());
                possible_actions.push((action, new_expr));
            }
        }
        return possible_actions;
    }
    
    pub fn swap_position_in_assoc_train(expr: &Expression, addr_vec: &[Address]) 
    -> Vec<(Action, Expression)> 
    {
        if addr_vec.len() <= 2 { return vec![]; }
        let addr0 = &addr_vec[addr_vec.len()-1];
        let addr1 = &addr_vec[addr_vec.len()-2];
        if addr0.is_empty() || addr1.is_empty() { return vec![]; }
        
        // non-empty address must have parent
        let addr_parent0 = addr0.parent();
        let addr_parent1 = addr1.parent();
        if addr_parent0 != addr_parent1 { return vec![]; }
        
        let addr_parent = addr_parent0;
        let index0 = addr0.last();
        let index1 = addr1.last();
        
        let new_expr = expr.swap_assoc_train_children_at(index0, index1, &addr_parent);
        if new_expr.is_err() { return vec![]; }
        
        let new_expr = new_expr.unwrap();
        return vec![(Action::ApplyAction("Reorder".to_string()), new_expr)];
    }
    
}
