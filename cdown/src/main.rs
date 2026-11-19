//same timer implementation as the timer.zip
use cdown::{format_time, tenths_left};
use std::io::{self, Write};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const MAX_SECS: u32 = 1_000_000;

// 3x5 font: '#' = filled. Index 0-9 = digits, 10 = ':', 11 = '.'.
const FONT: [[&str; 5]; 12] = [
    ["###", "# #", "# #", "# #", "###"],
    [" # ", "## ", " # ", " # ", "###"],
    ["###", "  #", "###", "#  ", "###"],
    ["###", "  #", "###", "  #", "###"],
    ["# #", "# #", "###", "  #", "  #"],
    ["###", "#  ", "###", "  #", "###"],
    ["###", "#  ", "###", "# #", "###"],
    ["###", "  #", "  #", "  #", "  #"],
    ["###", "# #", "###", "# #", "###"],
    ["###", "# #", "###", "  #", "###"],
    [" ", "#", " ", "#", " "],
    [" ", " ", " ", " ", "#"],
];

fn main() {
    let rx = spawn_input();
    loop {
        let total = read_duration(&rx);
        if total == 0 {
            break;
        }
        let passed = run_timer(total, 1, &rx);
        if passed == 0 {
            println!("Timer stopped."); // q was pressed
        } else {
            println!("Time's up! {passed} seconds passed");
        }
    }
}

// Starts a thread that sends every line typed down a channel.
fn spawn_input() -> Receiver<String> {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            let mut line = String::new();
            if io::stdin().read_line(&mut line).unwrap() == 0 {
                break; // end of input: the channel closes
            }
            tx.send(line.trim().to_string()).unwrap();
        }
    });
    rx
}

// Asks until it gets a valid duration; returns it in seconds (0 = quit).
fn read_duration(rx: &Receiver<String>) -> u32 {
    loop {
        print!("Duration, e.g. 1d 2h 30m 15s (q to quit): ");
        io::stdout().flush().unwrap();
        let line = match rx.recv() {
            Ok(line) => line,
            Err(_) => return 0, // input closed
        };
        if line == "q" {
            return 0;
        }
        match parse_duration(&line) {
            Some(total) => return total,
            None => println!("Use numbers with d, h (0-23), m (0-59), s (0-59); at most {MAX_SECS} s in total."),
        }
    }
}

// "1d 2h 30m 15s" -> Some(total seconds); None if any part is invalid.
fn parse_duration(text: &str) -> Option<u32> {
    let mut total: u32 = 0;
    for part in text.split_whitespace() {
        if !part.is_ascii() || part.len() < 2 {
            return None; // needs a number and a unit letter
        }
        let (number, unit) = part.split_at(part.len() - 1); // "30m" -> ("30", "m")
        let n: u8 = match number.parse() {
            Ok(n) => n,
            Err(_) => return None, // not a number, or above 255
        };
        let (limit, seconds) = match unit {
            "d" => (255, 86_400),
            "h" => (23, 3_600),
            "m" => (59, 60),
            "s" => (59, 1),
            _ => return None, // unknown unit
        };
        if n > limit {
            return None;
        }
        total += n as u32 * seconds;
    }
    if total == 0 || total > MAX_SECS { None } else { Some(total) }
}

// Runs one timer; returns the real seconds it took, pauses included
// (0 if stopped with q).
fn run_timer(total: u32, design: u8, rx: &Receiver<String>) -> u32 {
    let mut design = design;
    let mut running = true;
    let mut banked = Duration::ZERO; // time run before the last pause
    let mut resumed = Instant::now(); // when it last started running
    let started = Instant::now(); // for "x seconds passed"
    let mut shown = u32::MAX; // tenths on screen (MAX = nothing drawn yet)
    loop {
        if let Ok(command) = rx.try_recv() {
            match command.as_str() {
                "" => {
                    // Enter: pause saves the time run; start resets the clock
                    if running {
                        banked += resumed.elapsed();
                    } else {
                        resumed = Instant::now();
                    }
                    running = !running;
                }
                "1" => design = 1,
                "2" => design = 2,
                "3" => design = 3,
                "q" => return 0, // stop the timer
                _ => {}
            }
            shown = u32::MAX; // redraw after any command
        }
        let elapsed = if running { banked + resumed.elapsed() } else { banked };
        let left = tenths_left(total, elapsed);
        if left != shown {
            render(left, total, !running, design); // only when 100 ms has passed
            shown = left;
        }
        if left == 0 {
            return started.elapsed().as_secs() as u32;
        }
        thread::sleep(Duration::from_millis(10)); // don't use a whole CPU core
    }
}

// Draws the text as 5 large rows in design 1, 2 or 3.
fn big_text(text: &str, design: u8) -> Vec<String> {
    let mut rows = vec![String::new(); 5];
    for c in text.chars() {
        let i = match c {
            ':' => 10,
            '.' => 11,
            _ => c.to_digit(10).unwrap() as usize,
        };
        for (r, row) in rows.iter_mut().enumerate() {
            for p in FONT[i][r].chars() {
                let cell = if p == ' ' {
                    ' '
                } else if design == 1 {
                    '█'
                } else if design == 2 {
                    '*'
                } else {
                    '#'
                };
                row.push(cell); // twice, so the digits look square
                row.push(cell);
            }
            row.push(' ');
        }
    }
    rows
}

// Clears the screen and draws one frame.
fn render(tenths: u32, total: u32, paused: bool, design: u8) {
    let mut screen = String::from("\x1B[2J\x1B[H"); // clear screen
    for row in big_text(&format_time(tenths), design) {
        screen.push_str(&row);
        screen.push('\n');
    }
    screen.push_str(&format!("\nSet to: {}\n", format_time(total * 10)));
    screen.push_str(&format!("100 ms ticks: {}\n", total * 10 - tenths));
    if design == 3 {
        let done = ((total * 10 - tenths) * 40 / (total * 10)) as usize;
        screen.push_str(&format!("[{}{}]\n", "#".repeat(done), "-".repeat(40 - done)));
    }
    let state = if paused { "PAUSED" } else { "RUNNING" };
    screen.push_str(&format!("{state}   Enter = start/pause   1/2/3 + Enter = design   q + Enter = stop\n"));
    print!("{screen}");
    io::stdout().flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid() {
        assert_eq!(parse_duration("1d 2h 30m 15s"), Some(95_415));
        assert_eq!(parse_duration("45s"), Some(45));
        assert_eq!(parse_duration("2h"), Some(7_200));
        assert_eq!(parse_duration("11d 13h 46m 40s"), Some(MAX_SECS));
    }

    #[test]
    fn parse_invalid() {
        assert_eq!(parse_duration("90m"), None); // minutes stop at 59
        assert_eq!(parse_duration("24h"), None); // hours stop at 23
        assert_eq!(parse_duration("300s"), None); // above u8
        assert_eq!(parse_duration("5x"), None); // unknown unit
        assert_eq!(parse_duration("abc"), None);
        assert_eq!(parse_duration("0s"), None); // nothing to count
        assert_eq!(parse_duration("12d"), None); // above 1,000,000 s
    }

    #[test]
    fn tenths() {
        assert_eq!(tenths_left(5, Duration::ZERO), 50);
        assert_eq!(tenths_left(5, Duration::from_millis(1550)), 35);
        assert_eq!(tenths_left(5, Duration::from_secs(9)), 0);
        assert_eq!(tenths_left(MAX_SECS, Duration::ZERO), 10_000_000);
    }

    #[test]
    fn format() {
        assert_eq!(format_time(0), "00:00:00:00.0");
        assert_eq!(format_time(36_615), "00:01:01:01.5");
        assert_eq!(format_time(MAX_SECS * 10), "11:13:46:40.0");
    }

    #[test]
    fn designs() {
        assert!(big_text("8", 1)[0].contains('█'));
        assert_eq!(big_text("8", 2)[0], "****** ");
        assert_eq!(big_text("1", 3)[1], "####   ");
        assert_eq!(big_text("12:34:56.7", 1).len(), 5);
    }
}

