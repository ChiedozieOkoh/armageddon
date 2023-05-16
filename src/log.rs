#[macro_export]
macro_rules! dbg_ln {
    ($($arg:tt)*) => {
       #[cfg(debug_assertions)]
       println!($($arg)*);

    }
}

#[macro_export]
macro_rules! dbg_print {
    ($($arg:tt)*) => {
       #[cfg(debug_assertions)]
       print!($($arg)*);

    }
}
