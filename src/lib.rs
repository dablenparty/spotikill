pub mod aumid;
pub mod constants;

/// Like the [`dbg!`] macro, but only prints in debug mode.
/// This is very simple: it just wraps [`dbg!`] in an `#[cfg(debug_assertions)]` block.
#[macro_export]
macro_rules! debug_dbg {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        {
            dbg!($($arg)*)
        }
    };
}
