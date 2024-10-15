use std::collections::HashMap;
use crate::expression::{Address, Expression, ExpressionType};

#[derive(Debug, PartialEq, Default, Clone)]
pub enum BlockType {
    #[default]
    Symbol,
    HorizontalContainer,
    // VerticalContainer,
    FractionContainer,
}

#[derive(Default, Clone)]
pub struct BlockContext {
    pub inverse_op: HashMap<String, String>,
    pub fraction_op: Vec<String>
}

impl BlockContext {
    pub fn generate_inverse_op(op_pairs: Vec<(&str, &str)>) -> HashMap<String, String> {
        let mut inverse_op_map = HashMap::new();
        for (op, inverse_op) in op_pairs {
            inverse_op_map.insert(inverse_op.to_string(), op.to_string());
            inverse_op_map.insert(op.to_string(), inverse_op.to_string());
        }
        return inverse_op_map;
    }
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Block {
    pub block_type: BlockType,
    pub address: Address,
    pub symbol: Option<String>,
    pub children: Option<Vec<Block>>,
}

impl Block {
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
                let left_block = Block::from_expression(left_expr, left_addr, ctx);
                let right_addr = addr.append(1);
                let right_expr = expr_children.get(1).expect("BinaryOps have two children");
                let right_block = Block::from_expression(right_expr, right_addr, ctx);
                
                dbg!(&ctx.fraction_op);
                dbg!(&symbol);
                dbg!(&ctx.fraction_op.contains(&symbol));
                if ctx.fraction_op.contains(&symbol) {
                    block_builder::fraction_container(vec![left_block, right_block], addr)
                } else {
                    let operator_block = block_builder::symbol(symbol, addr.clone());
                    block_builder::horizontal_container(vec![left_block, operator_block, right_block], addr)
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
                let inverse_symbol = ctx.inverse_op.get(&symbol);
                for (i, child) in expr_children.iter().enumerate() {
                    let child_addr = addr.append(i);
                    
                    if child.exp_type == ExpressionType::OperatorUnary && Some(&child.symbol) == inverse_symbol {
                        let grandchildren = child.children.as_ref().expect("Unary has children");
                        let grandchild = grandchildren.first().expect("Unary has one child");
                        let grandchild_addr = child_addr.append(0);
                        let grandchild_block = Block::from_expression(grandchild, grandchild_addr, ctx);
                        
                        // TODO: add parenthesis if the grandchild is not just a simple value
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
}

pub mod block_builder {
    use super::*;
    pub fn symbol(symbol: String, addr: Address) -> Block {
        Block {
            block_type: BlockType::Symbol,
            symbol: Some(symbol),
            address: addr,
            children: None,
        }
    }
    
    pub fn container(block_type: BlockType, children: Vec<Block>, addr: Address) -> Block {
        Block {
            block_type,
            symbol: None,
            address: addr,
            children: Some(children),
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
