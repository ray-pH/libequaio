use crate::expression::{Address, Expression, ExpressionType};

#[derive(Debug, PartialEq)]
pub enum BlockType {
    Symbol,
    HorizontalContainer,
    // VerticalContainer,
    FractionContainer,
}

#[derive(Debug, PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub address: Address,
    pub symbol: Option<String>,
    pub children: Option<Vec<Block>>,
}

impl Block {
    pub fn from_expression(expr: Expression, addr: Address) -> Block {
        return match expr.exp_type {
            ExpressionType::ValueConst | ExpressionType::ValueVar => {
                block_builder::symbol(expr.symbol, addr)
            },
            ExpressionType::OperatorUnary => {
                let operator_block = block_builder::symbol(expr.symbol, addr.clone());
                let expr_children = expr.children.expect("UnaryOps have one child");
                let operand_addr = addr.append(0);
                let operand_expr = expr_children.get(0).expect("UnaryOps have one child").clone();
                let operand_block = Block::from_expression(operand_expr, operand_addr);
                block_builder::horizontal_container(vec![operator_block, operand_block], addr)
            },
            ExpressionType::StatementOperatorBinary |
            ExpressionType::OperatorBinary => {
                let operator_block = block_builder::symbol(expr.symbol, addr.clone());
                let expr_children = expr.children.expect("BinaryOps have two children");
                let left_addr = addr.append(0);
                let left_expr = expr_children.get(0).expect("BinaryOps have two children").clone();
                let left_block = Block::from_expression(left_expr, left_addr);
                let right_addr = addr.append(1);
                let right_expr = expr_children.get(1).expect("BinaryOps have two children").clone();
                let right_block = Block::from_expression(right_expr, right_addr);
                block_builder::horizontal_container(vec![left_block, operator_block, right_block], addr)
            },
            ExpressionType::OperatorNary => {
                let operator_block = block_builder::symbol(expr.symbol, addr.clone());
                let expr_children = expr.children.expect("NaryOps have children");
                let mut children_blocks = Vec::new();
                for (i, child) in expr_children.iter().enumerate() {
                    let child_addr = addr.append(i);
                    let child_block = Block::from_expression(child.clone(), child_addr);
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
                let expr_children = expr.children.expect("AssocTrain has children");
                let expr_children_count = expr_children.len();
                for (i, child) in expr_children.iter().enumerate() {
                    let child_addr = addr.append(i);
                    let child_block = Block::from_expression(child.clone(), child_addr);
                    children_blocks.push(child_block);
                    if i < expr_children_count - 1 {
                        let operator_block = block_builder::symbol(expr.symbol.clone(), addr.sub(i));
                        children_blocks.push(operator_block);
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
        symbol(",".to_string(), Address::empty())
    }
    
    pub fn horizontal_container(children: Vec<Block>, addr: Address) -> Block {
        container(BlockType::HorizontalContainer, children, addr)
    }
    pub fn fraction_container(children: Vec<Block>, addr: Address) -> Block {
        container(BlockType::FractionContainer, children, addr)
    }
}
