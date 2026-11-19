//These functions were made public as they are needed by spelling bee
use std::time::Duration;

// Time left in 100 ms steps.
pub fn tenths_left(total: u32, elapsed: Duration) -> u32 {
    let gone = (elapsed.as_millis() / 100) as u32;
    (total * 10).saturating_sub(gone) // stops at 0, never negative
}

// 100 ms steps in the format "DD:HH:MM:SS.t".
pub fn format_time(tenths: u32) -> String {
    let secs = tenths / 10;
    let (days, hours, mins) = (secs / 86_400, secs / 3_600 % 24, secs / 60 % 60);
    format!("{days:02}:{hours:02}:{mins:02}:{:02}.{}", secs % 60, tenths % 10)
}