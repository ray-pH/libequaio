use std::collections::HashMap;
use std::hash::Hash;

#[macro_export]
macro_rules! vec_strings {
    ($($x:expr),*) => {
        vec![$($x.to_string(),)*]
    };
}

// Math Functions ===================================================

pub fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 { a } else { gcd(b, a % b) }
}
pub fn lcm(a: i64, b: i64) -> i64 {
    a / gcd(a,b) * b
}

// HashMap ===================================================

/// check if two hashmaps have no clash
/// if a key is present in both maps, the value must be the same
/// if a key is present in only one map, the value can be anything
pub fn is_hashmap_no_clash<T,V>(map1: &HashMap<T,V>, map2: &HashMap<T,V>) -> bool
where T: Eq + Hash, V: PartialEq {
    for (k,v) in map1 {
        if map2.contains_key(k) && map2[k] != *v { return false; }
    }
    true
}
