use anyhow::Result;

use crate::browser;
use crate::state::InstanceState;

pub fn run(name: &str) -> Result<()> {
    let state = InstanceState::require(name)?;
    let url = format!("http://127.0.0.1:{}", state.port);
    browser::open_url(&url);
    Ok(())
}
