use std::collections::HashMap;
use crate::expression::{Address, Expression, ExpressionType};
use crate::utils;
use crate::address;

#[derive(Debug, PartialEq, Default, Clone)]
pub enum BlockType {
    #[default]
    Symbol,
    HorizontalContainer,
    // VerticalContainer,
    FractionContainer,
}

/// use `pair_map!` macro to generate `inverse_ops`
///
/// example: `pair_map![("+", "-"), ("*", "/")]`
///
/// use `vec_index_map!` macro to generate `op`
///
/// example: `vec_index_map!["-", "+", "/", "*"]`
#[derive(Default, PartialEq, Clone)]
pub struct BlockContext {
    pub inverse_ops: HashMap<String, String>,
    pub fraction_ops: Vec<String>,
    pub conceal_ops: Vec<String>, // ops that can be hidden, like multiplication
    //TODO: add rules for concealing (e.g. don't conceal * if it appled to numbers)
    pub op_precedence: HashMap<String, usize>,
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Block {
    pub block_type: BlockType,
    pub address: Address,
    pub symbol: Option<String>,
    pub children: Option<Vec<Block>>,
    pub tags: Vec<BlockTag>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum BlockTag {
    Parentheses,
    Concealed,
    LeftOfConcealed,
    RightOfConcealed,
}

impl BlockContext {
    pub fn has_precedence_over(&self, a: &str, b: &str) -> bool {
        let a_precedence = self.op_precedence.get(a);
        let b_precedence = self.op_precedence.get(b);
        if a_precedence.is_none() || b_precedence.is_none() { return false; }
        return a_precedence.unwrap() > b_precedence.unwrap();
    }
}

impl Block {
    pub fn add_tag(mut self, tag: BlockTag) -> Self {
        self.tags.push(tag);
        self
    }
    pub fn remove_tag(mut self, tag: &BlockTag) -> Self {
        self.tags.retain(|t| t != tag);
        self
    }
    pub fn contains_tag(&self, tag: &BlockTag) -> bool {
        return self.tags.contains(tag);
    }
    
    fn set_parenthesis_based_on_precedence(self, ctx: &BlockContext, src_expr: &Expression, parent_op: &str) -> Block {
        if !src_expr.is_operator() { return self; }
        let src_op = &src_expr.symbol;
        if ctx.has_precedence_over(parent_op, src_op) { 
            return self.add_tag(BlockTag::Parentheses)
        } else {
            return self;
        }
    }
    
    pub fn from_root_expression(expr: &Expression, ctx: &BlockContext) -> Block {
        return Block::from_expression(expr, Address::default(), ctx);
    }
    
    pub fn from_expression(expr: &Expression, addr: Address, ctx: &BlockContext) -> Block {
        let symbol = expr.symbol.clone();
        return match expr.exp_type {
            ExpressionType::ValueConst | ExpressionType::ValueVar => {
                block_builder::symbol(symbol, addr)
            },
            ExpressionType::Variadic | 
            ExpressionType::OperatorUnary => {
                let operator_block = block_builder::symbol(symbol, addr.clone());
                let expr_children = expr.children.as_ref().expect("UnaryOps have one child");
                let operand_addr = addr.append(0);
                let operand_expr = expr_children.first().expect("UnaryOps have one child");
                let operand_block = Block::from_expression(operand_expr, operand_addr, ctx);
                block_builder::horizontal_container(vec![operator_block, operand_block], addr)
            },
            ExpressionType::StatementOperatorBinary |
            ExpressionType::OperatorBinary => {
                let expr_children = expr.children.as_ref().expect("BinaryOps have two children");
                let left_addr = addr.append(0);
                let left_expr = expr_children.first().expect("BinaryOps have two children");
                let left_block = Block::from_expression(left_expr, left_addr, ctx)
                    .set_parenthesis_based_on_precedence(ctx, left_expr, &symbol);
                let right_addr = addr.append(1);
                let right_expr = expr_children.get(1).expect("BinaryOps have two children");
                let right_block = Block::from_expression(right_expr, right_addr, ctx)
                    .set_parenthesis_based_on_precedence(ctx, right_expr, &symbol);
                
                // dbg!(&ctx.fraction_ops);
                // dbg!(&symbol);
                // dbg!(&ctx.fraction_ops.contains(&symbol));
                
                if ctx.fraction_ops.contains(&symbol) {
                    block_builder::fraction_container(vec![left_block, right_block], addr)
                } else {
                    let is_conceal = ctx.conceal_ops.contains(&symbol) && !utils::is_number(right_expr.symbol.as_str());
                    let operator_block = block_builder::symbol(symbol, addr.clone());
                    if is_conceal {
                        let operator_block = operator_block.add_tag(BlockTag::Concealed);
                        let left_block = left_block.add_tag(BlockTag::LeftOfConcealed);
                        let right_block = right_block.add_tag(BlockTag::RightOfConcealed);
                        block_builder::horizontal_container(vec![left_block, operator_block, right_block], addr)
                    } else {
                        block_builder::horizontal_container(vec![left_block, operator_block, right_block], addr)
                    }
                }
            },
            ExpressionType::OperatorNary => {
                let operator_block = block_builder::symbol(symbol, addr.clone());
                let expr_children = expr.children.as_ref().expect("NaryOps have children");
                let mut children_blocks = Vec::new();
                for (i, child) in expr_children.iter().enumerate() {
                    let child_addr = addr.append(i);
                    let child_block = Block::from_expression(child, child_addr, ctx);
                    children_blocks.push(child_block);
                    if i < expr_children.len() - 1 {
                        children_blocks.push(block_builder::comma());
                    }
                }
                let children_block = block_builder::horizontal_container(children_blocks, addr.clone());
                block_builder::horizontal_container(vec![operator_block, children_block], addr)
            },
            ExpressionType::AssocTrain => {
                let mut children_blocks = Vec::new();
                let expr_children = expr.children.as_ref().expect("AssocTrain has children");
                let inverse_symbol = ctx.inverse_ops.get(&symbol);
                for (i, child) in expr_children.iter().enumerate() {
                    let child_addr = addr.append(i);
                    
                    if child.exp_type == ExpressionType::OperatorUnary && Some(&child.symbol) == inverse_symbol {
                        let grandchildren = child.children.as_ref().expect("Unary has children");
                        let grandchild = grandchildren.first().expect("Unary has one child");
                        let grandchild_addr = child_addr.append(0);
                        let grandchild_block = Block::from_expression(grandchild, grandchild_addr, ctx);
                        
                        // add parenthesis if the grandchild is not just a simple value
                        let grandchild_block = if grandchild_block.block_type != BlockType::Symbol {
                            grandchild_block.add_tag(BlockTag::Parentheses)
                        } else {
                            grandchild_block
                        };
                        
                        let op_addr = if i == 0 { addr.append(i) } else { addr.sub(i-1) };
                        let op_block = block_builder::symbol(inverse_symbol.unwrap().clone(), op_addr);
                        children_blocks.push(op_block);
                        children_blocks.push(grandchild_block);
                    } else {
                        if i > 0 {
                            let op_block = block_builder::symbol(symbol.clone(), addr.sub(i-1));
                            children_blocks.push(op_block);
                        }
                        let child_block = Block::from_expression(child, child_addr, ctx);
                        children_blocks.push(child_block);
                    }
                }
                block_builder::horizontal_container(children_blocks, addr)
            }
        }
    }
    
    /// returns (left, middle, right) blocks
    pub fn from_root_expression_to_alignable_blocks(expr: &Expression, ctx: &BlockContext) 
    -> (Option<Block>, Option<Block>, Option<Block>) 
    {
        // TODO: implement this for non equations
        if let (Some(lhs), Some(rhs)) = (expr.lhs(), expr.rhs()) {
          let lhs_block = Block::from_expression(lhs, address![0], ctx);
          let rhs_block = Block::from_expression(rhs, address![1], ctx);
          let eq_block  = block_builder::symbol(expr.symbol.clone(), address![]);
          return (Some(lhs_block), Some(eq_block), Some(rhs_block));
        } else {
          let block = Block::from_expression(expr, address![], ctx);
          return (None, None, Some(block));
        }
    }
}

pub mod block_builder {
    use super::*;
    pub fn symbol(symbol: String, addr: Address) -> Block {
        Block {
            block_type: BlockType::Symbol,
            symbol: Some(symbol),
            address: addr,
            children: None,
            ..Default::default()
        }
    }
    
    pub fn container(block_type: BlockType, children: Vec<Block>, addr: Address) -> Block {
        Block {
            block_type,
            symbol: None,
            address: addr,
            children: Some(children),
            ..Default::default()
        }
    }
    
    pub fn comma() -> Block {
        symbol(",".to_string(), Address::default())
    }
    
    pub fn horizontal_container(children: Vec<Block>, addr: Address) -> Block {
        container(BlockType::HorizontalContainer, children, addr)
    }
    pub fn fraction_container(children: Vec<Block>, addr: Address) -> Block {
        container(BlockType::FractionContainer, children, addr)
    }
}
