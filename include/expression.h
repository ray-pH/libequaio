#pragma once

#include <optional>
#include <string>
#include <vector>
#include <algorithm>
#include <map>
#include <tuple>

using std::optional; 
using std::string; 
using std::vector; 
using std::map;
using std::tuple;

typedef vector<int> address;

typedef enum{
    EXPRESSION_OPERATOR_BINARY,
    EXPRESSION_OPERATOR_UNARY,
    EXPRESSION_VALUE,
} ExpType;

// vector<string> variables;
// vector<string> binary\_operators; 
// vector<string> unary\_operators; 
// bool handle\_numerics;             
struct Context{
    vector<string> variables;
    vector<string> binary_operators; // also include ','
    vector<string> unary_operators;  // also include functions
    bool handle_numerics;            // true if numbers are allowed
};

// ExpType type;
// string  symbol;
// bool    bracketed;
// vector<Expression> child;
struct Expression {
    ExpType type;
    string  symbol;
    bool    bracketed;
    vector<Expression> children; // can only have 1 or 2 member

    Expression copy() const;
    string to_string() const;

    // check structural equality
    // check if this expression can be matched by the pattern
    // only based on operators
    bool can_pattern_match(Expression pattern, Context ctx) const;
    Expression at(address addr) const;
    Expression* ptr_at(address addr);
    vector<address> get_all_address() const;

    vector<string> extract_variables() const;
    vector<address> get_operator_chains_from(address addr) const;
    bool is_in_same_operator_chain(address addr1, address addr2) const;
    void strip_parentheses_for_associative_op(string op);
    void swap_two_element(address addr1, address addr2);
    // vector<vector<address>> get_all_operator_chains() const;

    // return map that maps variables from pattern to expressions in this expression
    // NOTE : this->can_pattern_match(pattern) must be true
    optional<map<string, Expression>> try_match_pattern(Expression pattern) const;
    Expression apply_variable_map(map<string, Expression> variable_map) const;

    // rule of equality
    vector<Expression> apply_rule_equal(Expression rule, Context ctx) const;
    friend bool operator==(const Expression& lhs, const Expression& rhs);
    friend bool operator!=(const Expression& lhs, const Expression& rhs);
    
    static address parent_address_of(address addr);
    static address child_address_of(address addr, int child_index);
    static Expression create_equality(Expression lhs, Expression rhs);
    static Expression create_symbol(string symbol);
};

std::ostream& operator<<(std::ostream& os, const Expression& exp);
