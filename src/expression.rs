use std::collections::HashMap;
use super::utils;

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionType {
    OperatorUnary,
    OperatorBinary,
    OperatorNary,
    ValueConst,
    ValueVar,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub exp_type: ExpressionType,
    pub symbol: String,
    pub children: Option<Vec<Expression>>,
}

#[derive(Clone)]
pub struct Context {
    pub parameters: Vec<String>,
    pub unary_ops: Vec<String>,
    pub binary_ops: Vec<String>,
    // the rest of the symbols will be considered as n-ary operators (functions)
    pub handle_numerics: bool,
}

impl Context {
    pub fn add_param(&mut self, param : String) {
        if !self.parameters.contains(&param) { self.parameters.push(param); }
    }
    pub fn add_params(&mut self, params : Vec<String>) {
        for p in params { self.add_param(p); }
    }
    pub fn remove_param(&mut self, param : &String) {
        self.parameters.retain(|p| p != param);
    }
    pub fn remove_params(&mut self, params : &Vec<String>) {
        for p in params { self.remove_param(p); }
    }
}

pub type Address  = Vec<usize>;
pub type MatchMap = HashMap<String,Expression>;

impl Expression {
    pub fn is_operator(&self) -> bool {
        match self.exp_type {
            ExpressionType::OperatorUnary | 
            ExpressionType::OperatorBinary | 
            ExpressionType::OperatorNary => true,
            _ => false,
        }
    }
    pub fn is_value(&self) -> bool {
        match self.exp_type {
            ExpressionType::ValueConst | 
            ExpressionType::ValueVar => true,
            _ => false,
        }
    }

    pub fn to_string(&self, parentheses : bool) -> String {
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
                result.push_str("(");
                for (i, c) in self.children.as_ref().unwrap().iter().enumerate() {
                    if i > 0 { result.push_str(", "); }
                    result.push_str(&c.to_string(parentheses));
                }
                result.push_str(")");
                result
            },
        }
    }

    /// Get the expression from the address
    pub fn traverse_address(&self, address : Address) -> Option<&Expression> {
        if address.len() == 0 { return Some(self); }
        if self.children.is_none() { return None; }
        let index = address[0];
        if index >= self.children.as_ref().unwrap().len() { return None; }
        self.children.as_ref().unwrap()[index].traverse_address(address[1..].to_vec())
    }

    pub fn get_pattern_matches(&self, pattern : &Expression) -> Vec<(Address,MatchMap)> {
        self.f_get_patten_matches(pattern, Vec::new())
    }

    /// Try to match the pattern expression with this expression, and all its children.
    /// Returns a list of maps, where each map represents a match.
    fn f_get_patten_matches(&self, pattern : &Expression, current_address : Address) -> Vec<(Address,MatchMap)> {
        let mut result = Vec::new();

        // try to match the root node
        let root_map = self.patten_match_this_node(pattern);
        if let Some(map) = root_map { result.push((current_address.clone(), map)); }

        // try to match the children
        if let Some(children) = &self.children {
            for (i,c) in children.iter().enumerate() {
                let mut child_address = current_address.clone();
                child_address.push(i);
                let child_matches = c.f_get_patten_matches(pattern, child_address);
                // push the child matches to the result
                child_matches.iter().for_each(|m| {
                    result.push(m.clone());
                });
            }
        }

        result
    }

    /// Try to match the pattern expression with this expression. (match the root node)
    /// Returns a map of the symbols, or None if there is the pattern does not match.
    pub fn patten_match_this_node(&self, pattern : &Expression) -> Option<MatchMap> {
        match pattern.exp_type {
            // if the pattern is a constant parameter, then it must match exactly with this expression
            ExpressionType::ValueConst => {
                if pattern.symbol == self.symbol && pattern.exp_type == self.exp_type {
                    let empty_map = HashMap::new();
                    return Some(empty_map);
                }
                None
            },
            // if the pattern is a variable, then it must mapped to this expression
            ExpressionType::ValueVar => {
                let mut map = HashMap::new();
                map.insert(pattern.symbol.clone(), self.clone());
                Some(map)
            },
            // if the pattern is an operator, then it must match
            // then, pattern match each child
            ExpressionType::OperatorUnary | ExpressionType::OperatorBinary | ExpressionType::OperatorNary => {
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
                    let child_map = self_children[i].patten_match_this_node(&pattern_children[i]);
                    if child_map.is_none() { return None; }
                    // invalid if the child maps clash
                    if !utils::is_hashmap_no_clash(&map, &child_map.clone().unwrap()) { return None; }
                    // merge the child map with the current map
                    for (k,v) in child_map.unwrap() { map.insert(k,v); }
                }
                Some(map)
            },
        }
    }
}
