#[macro_export]
macro_rules! wait_for {
    ($cond:expr) => {
        loop {
            if $cond {
                break;
            }
        }
    }
}
