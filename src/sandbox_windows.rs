use anyhow::Result;

use crate::sandbox::{ProcessSandbox, SandboxConfig};

#[derive(Default)]
pub struct WindowsSandbox;

impl WindowsSandbox {
    pub fn new() -> Self {
        Self
    }
}

impl ProcessSandbox for WindowsSandbox {
    fn apply(&self, _config: &SandboxConfig) -> Result<()> {
        // Windows Job Object implementation
        // Uses windows-sys to create a job object with resource limits
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::JobObjects::*;
            use windows_sys::Win32::System::Threading::*;

            unsafe {
                let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
                if job.is_null() {
                    anyhow::bail!("Failed to create job object");
                }

                let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
                info.BasicLimitInformation.LimitFlags =
                    JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;

                if let Some(mem) = _config.mem_limit_bytes {
                    info.ProcessMemoryLimit = mem as usize;
                }
                if let Some(pids) = _config.pid_limit {
                    info.BasicLimitInformation.ActiveProcessLimit = pids;
                }

                SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &info as *const _ as *const _,
                    std::mem::size_of_val(&info) as u32,
                );

                let process = GetCurrentProcess();
                AssignProcessToJobObject(job, process);
            }
        }

        Ok(())
    }

    fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}
