use std::env;
use equaio::rule;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} rulename", args[0]);
        return;
    }
    
    let rulename = &args[1];
    println!("Rulename: {}", rulename);
    
    let filepath = format!("rules/{}.json", rulename);
    let rulestr = std::fs::read_to_string(&filepath);
    if rulestr.is_err() {
        println!("Error reading file: {}", &filepath);
        return;
    }
    let rulestr = rulestr.unwrap();
    
    let rule_vec = rule::parse_ruleset_from_json(&rulestr);
    let ctx = rule::parse_context_from_json(&rulestr);
    if rule_vec.is_err() || ctx.is_err() {
        println!("Error parsing rule: {}", &filepath);
        return;
    }
    
    let rule_vec = rule_vec.unwrap();
    let ctx = ctx.unwrap();
    
    println!("Context:");
    println!("{:?}", ctx);
    for rule in rule_vec.iter() {
        println!();
        println!("{} ({})", rule.id, rule.label);
        println!("    {}", rule.expression.to_string(true));
    }
}
