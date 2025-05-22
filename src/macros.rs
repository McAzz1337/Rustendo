#[macro_export]
macro_rules! trace {
    ($e: expr) => {{
        println!("{} {}: {}", line!(), file!(), stringify!($e));
        $e
    }};
}
