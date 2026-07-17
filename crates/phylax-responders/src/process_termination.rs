use phylax_common::{Error, Result};
use phylax_engine::{Decision, Action};
use crate::{Responder, ResponseResult};

pub struct ProcessTerminationResponder {
    require_approval: bool,
    allow_termination: bool,
}

impl ProcessTerminationResponder {
    pub fn new(require_approval: bool, allow_termination: bool) -> Self {
        ProcessTerminationResponder {
            require_approval,
            allow_termination,
        }
    }

    fn terminate_process(&self, pid: u32) -> Result<ResponseResult> {
        if !self.allow_termination {
            return Ok(ResponseResult::failure("Process termination is not allowed".to_string()));
        }

        #[cfg(unix)]
        {
            use nix::sys::signal::{self, Signal};
            signal::kill(nix::unistd::Pid::from_raw(pid as i32), Signal::SIGTERM)
                .map_err(|e| Error::ResponderError(format!("Failed to terminate process: {}", e)))?;
            Ok(ResponseResult::success(format!("Terminated process: {}", pid)))
        }

        #[cfg(windows)]
        {
            use windows::Win32::System::Threading::{OpenProcess, TerminateProcess, PROCESS_TERMINATE};
            
            let handle = unsafe {
                OpenProcess(PROCESS_TERMINATE, false, pid)
                    .map_err(|e| Error::ResponderError(format!("Failed to open process: {}", e)))?
            };

            let result = unsafe {
                TerminateProcess(handle, 1)
                    .map_err(|e| Error::ResponderError(format!("Failed to terminate process: {}", e)))?
            };

            if result.as_bool() {
                Ok(ResponseResult::success(format!("Terminated process: {}", pid)))
            } else {
                Ok(ResponseResult::failure("Failed to terminate process".to_string()))
            }
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(ResponseResult::failure("Process termination not supported on this platform".to_string()))
        }
    }
}

impl Responder for ProcessTerminationResponder {
    fn name(&self) -> &'static str {
        "process_termination"
    }

    fn execute(&self, _decision: &Decision, action: &Action) -> Result<ResponseResult> {
        if let Some(pid) = action.parameters.get("pid") {
            if let Some(pid_num) = pid.as_u64() {
                return self.terminate_process(pid_num as u32);
            }
        }
        Ok(ResponseResult::failure("No process ID provided".to_string()))
    }

    fn requires_approval(&self) -> bool {
        self.require_approval
    }
}
