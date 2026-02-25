use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    },
};

use std::os::windows::io::AsRawHandle;

pub struct ProcessJob {
    handle: HANDLE,
}

impl ProcessJob {
    /// Create a process job that handles
    /// - Limiting process memory (JOB_OBJECT_LIMIT_PROCESS_MEMORY)
    /// - Process termination when job closes (JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE)
    pub fn create_job_object(memory_limit: usize, kill_on_close: bool) -> Result<Option<Self>, windows::core::Error> {
        if memory_limit != 0 || kill_on_close {
            unsafe {
                let job = CreateJobObjectW(None, None)?;
                let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
                if kill_on_close {
                    limits.BasicLimitInformation.LimitFlags |= JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
                }
                if memory_limit > 0 {
                    limits.ProcessMemoryLimit = memory_limit;
                    limits.BasicLimitInformation.LimitFlags |=
                        windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_PROCESS_MEMORY;
                }

                SetInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &limits as *const _ as *const _,
                    std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                )?;

                Ok(Some(Self { handle: job }))
            }
        } else {
            Ok(None)
        }
    }

    /// Assign a child process to this job
    pub fn assign(&self, child: &std::process::Child) -> Result<(), windows::core::Error> {
        unsafe {
            let process_handle = HANDLE(child.as_raw_handle());
            AssignProcessToJobObject(self.handle, process_handle)?;
        }
        Ok(())
    }
}

impl Drop for ProcessJob {
    fn drop(&mut self) {
        unsafe {
            // Closing the job handle terminates all child processes
            let _ = CloseHandle(self.handle);
        }
    }
}
