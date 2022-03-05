#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_print {
    ($( $str:expr ),* ) => {
        print!(
            $(
                $str,
            )*
        )
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_print {
    ($( $str:expr ),* ) => {
    };
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug_println {
    ($( $str:expr ),* ) => {
        println!(
            $(
                $str,
            )*
        )
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug_println {
    ($( $str:expr ),* ) => {
    };
}

pub use debug_print;
pub use debug_println;
