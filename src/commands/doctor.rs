use crate::docker;
use crate::output;

pub fn run() -> i32 {
    println!("moltctrl doctor - Checking system requirements");
    println!("================================================");
    println!();

    let mut issues = 0;

    // Docker
    print!("  {:<20}", "Docker:");
    if docker::is_docker_installed() {
        if let Some(version) = docker::docker_version() {
            if docker::is_docker_running() {
                output::success(&version);
            } else {
                output::error("installed but daemon not running");
                issues += 1;
            }
        } else {
            output::error("installed but version unknown");
            issues += 1;
        }
    } else {
        output::error("not installed");
        issues += 1;
    }

    // Docker Compose v2
    print!("  {:<20}", "Docker Compose:");
    if docker::is_compose_available() {
        if let Some(version) = docker::compose_version() {
            output::success(&format!("v{}", version));
        } else {
            output::success("available");
        }
    } else {
        output::error("not available (need Docker Compose v2)");
        issues += 1;
    }

    // Runtime dependencies (Rust version eliminates jq, envsubst, openssl, websocat)
    print!("  {:<20}", "jq:");
    output::success("not needed (built-in JSON)");

    print!("  {:<20}", "envsubst:");
    output::success("not needed (built-in templates)");

    print!("  {:<20}", "openssl:");
    output::success("not needed (built-in token gen)");

    print!("  {:<20}", "websocat:");
    output::success("not needed (built-in WebSocket)");

    println!();
    if issues == 0 {
        output::success("All required dependencies are installed");
    } else {
        output::error(&format!("{} issue(s) found", issues));
    }

    issues
}
