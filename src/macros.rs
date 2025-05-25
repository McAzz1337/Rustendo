use crate::function_name;

#[macro_export]
macro_rules! trace {
    ($e: expr) => {{
        println!("{} {}: {}", line!(), file!(), stringify!($e));
        $e
    }};
}

#[macro_export]
macro_rules! log {
    () => {{
        println!("{} {} => : {}", line!(), file!(), function_name!())
    }};
    ($e: expr) => {{
        println!(
            "{} {} => : {} -> {}",
            line!(),
            file!(),
            function_name!(),
            format!($e)
        )
    }};
}

#[macro_export]
macro_rules! and {
    ($e: expr, $bits: expr) => {
        $e & $bits
    };
    ($e: expr, $bits: expr, $ty: ty) => {
        ($e & $bits) as $ty
    };
}

#[macro_export]
macro_rules! shift_left {
    ($value: expr, $bits: expr) => {
        $value << $bits
    };
    ($value: expr, $bits: expr, $ty: ty) => {
        ($value as $ty) << $bits
    };
}

#[macro_export]
macro_rules! shift_right {
    ($value: expr, $bits: expr) => {
        $value >> $bits
    };
    ($value: expr, $bits: expr, $ty: ty) => {
        ($value >> $bits) as $ty
    };
}
