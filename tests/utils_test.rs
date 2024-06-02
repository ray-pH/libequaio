use equaio::utils;

#[cfg(test)]
mod function {
    use super::*;
    
    #[test]
    fn gcd () {
        assert_eq!(utils::gcd(12, 15), 3);
        assert_eq!(utils::gcd(15, 12), 3);
        assert_eq!(utils::gcd(12, 0), 12);
        assert_eq!(utils::gcd(0, 12), 12);
    }
    
    #[test]
    fn lcm () {
        assert_eq!(utils::lcm(12, 15), 60);
        assert_eq!(utils::lcm(15, 12), 60);
        assert_eq!(utils::lcm(12, 0), 0);
        assert_eq!(utils::lcm(0, 12), 0);
    }
}
