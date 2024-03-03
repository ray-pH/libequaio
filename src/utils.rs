#[macro_export]
macro_rules! vec_strings {
    ($($x:expr),*) => {
        vec![$($x.to_string(),)*]
    };
}
