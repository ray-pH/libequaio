#include "block_display.h"
#include <assert.h>
#include <iostream>

using namespace BlockDisplay;

void BlockDisplay::Block::append(Block b){ this->children.push_back(b); }
void BlockDisplay::Block::append(vector<Block> b){ this->children.insert(this->children.end(), b.begin(), b.end()); }
void BlockDisplay::Block::prepend(Block b){ this->children.insert(this->children.begin(), b); }
void BlockDisplay::Block::prepend(vector<Block> b){ this->children.insert(this->children.begin(), b.begin(), b.end()); }

string BlockDisplay::Block::to_string() const{
    switch (this->type){
        case BASIC:{
            string st = "";
            for (const auto& c : this->children){
                st += c.to_string() + " ";
            }
            return st;
        }
        case VALUE:
            return this->value;
        case FRAC:
            return "{" + this->children[0].to_string() + "}" 
                    + "/" + "{" + this->children[1].to_string() + "}";
    }
    return "";
}
std::ostream& BlockDisplay::operator<<(std::ostream& os, const Block& b){
    os << b.to_string();
    return os;
}



Block __from_expression(Expression &rootexpr, address addr, Context ctx){
    (void)ctx;

    auto expr = rootexpr.at(addr);
    Block container;
    switch (expr.type){
        case EXPRESSION_OPERATOR_BINARY: {
            Block self = { VALUE, expr.symbol, {}, {addr, &rootexpr} };
            container = { BASIC, "", {self} };

            address addrleft(addr);
            addrleft.push_back(0);
            Block left  = __from_expression(rootexpr, addrleft, ctx);
            address addrright(addr);
            addrright.push_back(1);
            Block right = __from_expression(rootexpr, addrright, ctx);

            container.prepend(left.children);
            container.append(right.children);
            break;
        }
        case EXPRESSION_OPERATOR_UNARY: {
            Block self = { VALUE, expr.symbol, {}, {addr, &rootexpr} };
            container = { BASIC, "", {self} };

            address addrinner(addr);
            addrinner.push_back(0);
            Block inner = __from_expression(rootexpr, addrinner, ctx);
            container.append(inner.children);
            break;
        }
        case EXPRESSION_VALUE: {
            Block inner = { VALUE, expr.symbol, {}, {addr, &rootexpr}};
            container = { BASIC, "", {inner} };
            break;
        }
    }
    if (expr.bracketed){
        // container.prepend(openparen);
        // container.append(closeparen);
        container.prepend({ VALUE, "(", {}, {addr, &rootexpr} });
        container.append({ VALUE, ")", {}, {addr, &rootexpr} });
    }
    return container;
    // return {};
}

void __setup_leftright_metadata(Block* block){
    for (size_t i = 0; i < block->children.size(); i++){
        if (block->children[i].type == BlockDisplay::VALUE){
            address leftaddr = i > 0 ? block->children[i-1].metadata.addr : address();
            address rightaddr = i < block->children.size()-1 ? block->children[i+1].metadata.addr : address();
            block->children[i].metadata.leftaddr = leftaddr;
            block->children[i].metadata.rightaddr = rightaddr;
        }else{
            __setup_leftright_metadata(&block->children[i]);
        }
    }
}

Block BlockDisplay::from_expression(Expression expr, Context ctx){
    (void)ctx;
    auto block = __from_expression(expr, {}, ctx);
    __setup_leftright_metadata(&block);
    return block;
}
