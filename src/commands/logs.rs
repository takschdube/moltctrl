use std::fs;
use std::io::{self, BufRead, Seek, SeekFrom, Write};
use std::thread;
use std::time::Duration;

use anyhow::{bail, Result};

use crate::config;
use crate::docker::{self, DockerCompose};
use crate::output;
use crate::state::InstanceState;

pub fn run(name: &str, follow: bool, tail: &str) -> Result<()> {
    let state = InstanceState::require(name)?;

    if state.isolation == "process" {
        let log_path = config::instance_dir(name).join("output.log");

        if !log_path.exists() {
            output::info("No logs yet.");
            return Ok(());
        }

        let tail_n: usize = tail.parse().unwrap_or(100);

        if follow {
            // Print last N lines, then follow
            print_tail_lines(&log_path, tail_n)?;

            // Follow mode: poll for new content
            let mut file = fs::File::open(&log_path)?;
            file.seek(SeekFrom::End(0))?;
            let mut reader = io::BufReader::new(file);
            let stdout = io::stdout();
            let mut out = stdout.lock();

            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) => {
                        // No new data, sleep and retry
                        thread::sleep(Duration::from_millis(200));
                    }
                    Ok(_) => {
                        out.write_all(line.as_bytes())?;
                        out.flush()?;
                    }
                    Err(e) => {
                        bail!("Error reading log file: {}", e);
                    }
                }
            }
        } else {
            // Just print last N lines
            print_tail_lines(&log_path, tail_n)?;
        }

        return Ok(());
    }

    docker::require_docker()?;

    let compose_path = config::instance_dir(name).join("docker-compose.yml");
    if !compose_path.exists() {
        bail!("No docker-compose.yml found for instance '{}'", name);
    }

    let dc = DockerCompose::new(name, &compose_path.to_string_lossy());
    dc.logs(follow, tail)?;
    Ok(())
}

/// Print the last `n` lines of a file to stdout.
fn print_tail_lines(path: &std::path::Path, n: usize) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let lines: Vec<&str> = content.lines().collect();
    let start = if lines.len() > n { lines.len() - n } else { 0 };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in &lines[start..] {
        writeln!(out, "{}", line)?;
    }
    out.flush()?;
    Ok(())
}
