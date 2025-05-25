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
