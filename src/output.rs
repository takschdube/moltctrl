use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};

use indicatif::{ProgressBar, ProgressStyle};
use owo_colors::OwoColorize;

static NO_COLOR: AtomicBool = AtomicBool::new(false);
static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_no_color(v: bool) {
    NO_COLOR.store(v, Ordering::Relaxed);
}

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

fn color_enabled() -> bool {
    !NO_COLOR.load(Ordering::Relaxed) && io::stdout().is_terminal()
}

pub fn info(msg: &str) {
    if color_enabled() {
        println!("{} {}", "::".bold().blue(), msg);
    } else {
        println!(":: {}", msg);
    }
}

pub fn warn(msg: &str) {
    if color_enabled() {
        eprintln!("{} {}", "warning:".bold().yellow(), msg);
    } else {
        eprintln!("warning: {}", msg);
    }
}

pub fn error(msg: &str) {
    if color_enabled() {
        eprintln!("{} {}", "error:".bold().red(), msg);
    } else {
        eprintln!("error: {}", msg);
    }
}

pub fn success(msg: &str) {
    if color_enabled() {
        println!("{} {}", "✓".bold().green(), msg);
    } else {
        println!("✓ {}", msg);
    }
}

pub fn debug(msg: &str) {
    if VERBOSE.load(Ordering::Relaxed) {
        if color_enabled() {
            eprintln!("{} {}", "debug:".dimmed(), msg);
        } else {
            eprintln!("debug: {}", msg);
        }
    }
}

#[allow(dead_code)]
pub fn print_flush(msg: &str) {
    print!("{}", msg);
    let _ = io::stdout().flush();
}

pub fn spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.magenta} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}
