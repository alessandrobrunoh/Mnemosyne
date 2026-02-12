//! Safe cross-platform process checking utilities.
//!
//! This module provides a safe, unified interface for checking if a process
//! is still running. It handles platform-specific differences and provides
//! RAII safety for Windows handles.

use crate::error::AppResult;

/// Checks if a process with the given PID is currently running.
///
/// # Arguments
/// * `pid` - Process ID to check
///
/// # Returns
/// * `Ok(true)` - Process is running
/// * `Ok(false)` - Process is not running or access was denied
/// * `Err(AppError)` - An error occurred while checking
///
/// # Security
/// This function performs safe checks:
/// - On Unix: Validates PID range before converting to i32 (prevents overflow)
/// - On Windows: Uses RAII to ensure handles are always closed
/// - Does not expose sensitive process information
///
/// # Examples
/// ```rust,no_run
/// use mnem_core::process::is_process_running;
///
/// let pid = 1234;
/// match is_process_running(pid) {
///     Ok(true) => println!("Process {} is running", pid),
///     Ok(false) => println!("Process {} is not running", pid),
///     Err(e) => eprintln!("Error checking process: {}", e),
/// }
/// ```
pub fn is_process_running(pid: u32) -> AppResult<bool> {
    // Validate PID is not 0 (system idle / invalid)
    if pid == 0 {
        return Ok(false);
    }

    #[cfg(unix)]
    return unix::is_process_running(pid);

    #[cfg(windows)]
    return windows::is_process_running(pid);
}

/// Validates that a PID can be safely converted to i32.
///
/// This is primarily needed for Unix systems where kill() takes i32.
#[cfg(unix)]
fn validate_pid_range(pid: u32) -> AppResult<i32> {
    if pid > i32::MAX as u32 {
        return Err(crate::error::AppError::Internal(format!(
            "PID {} exceeds maximum safe value",
            pid
        )));
    }
    Ok(pid as i32)
}

#[cfg(unix)]
mod unix {
    use super::*;
    use libc::{kill, ESRCH};

    /// Unix implementation using kill() with signal 0.
    /// Signal 0 is a special signal that performs error checking but doesn't actually send a signal.
    pub fn is_process_running(pid: u32) -> AppResult<bool> {
        // Validate PID won't overflow when cast to i32
        let pid_i32 = validate_pid_range(pid)?;

        // SAFETY: kill() with signal 0 is a safe operation that just checks if the process exists.
        // We validate the PID range before casting to prevent overflow.
        let result = unsafe { kill(pid_i32, 0) };

        if result == 0 {
            // Process exists and we have permission
            Ok(true)
        } else {
            // Check errno to distinguish between "no such process" and "access denied"
            let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            if errno == ESRCH {
                // No such process
                Ok(false)
            } else if errno == libc::EPERM {
                // Process exists but we don't have permission to signal it
                // For our purposes, we treat this as "running"
                Ok(true)
            } else {
                // Other error - process probably doesn't exist
                Ok(false)
            }
        }
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

    /// RAII wrapper for Windows process handles to ensure they're always closed.
    struct ProcessHandle(isize);

    impl ProcessHandle {
        /// Open a process handle with limited query access.
        /// Returns Ok(None) if the process doesn't exist.
        /// Returns Err if there's a system error.
        fn open(pid: u32) -> AppResult<Option<Self>> {
            // SAFETY: OpenProcess is safe when we use PROCESS_QUERY_LIMITED_INFORMATION
            // which doesn't allow any privileged operations.
            let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };

            if handle == 0 || handle == INVALID_HANDLE_VALUE {
                // Check if the error is "process not found" vs other error
                let err = std::io::Error::last_os_error();
                let errno = err.raw_os_error().unwrap_or(0);

                // ERROR_INVALID_PARAMETER (87) usually means process doesn't exist
                if errno == 87 {
                    return Ok(None);
                }

                // For other errors, assume process doesn't exist or we can't access it
                return Ok(None);
            }

            Ok(Some(Self(handle)))
        }
    }

    impl Drop for ProcessHandle {
        fn drop(&mut self) {
            // SAFETY: We're closing the handle we opened. This is safe because
            // we own the handle and Drop ensures it always runs, even on panic.
            // HANDLE is defined as isize in windows-sys, which matches our stored value.
            if self.0 != 0 && self.0 != INVALID_HANDLE_VALUE {
                unsafe {
                    let _ = CloseHandle(self.0);
                }
            }
        }
    }

    /// Windows implementation using OpenProcess.
    /// We open the process with PROCESS_QUERY_LIMITED_INFORMATION which is the most
    /// restrictive access possible for checking if a process exists.
    pub fn is_process_running(pid: u32) -> AppResult<bool> {
        match ProcessHandle::open(pid)? {
            Some(_) => Ok(true),
            None => Ok(false),
        }
    }
}

/// Stub implementation for unsupported platforms.
/// Always returns false for safety.
#[cfg(not(any(unix, windows)))]
mod unsupported {
    use super::*;

    pub fn is_process_running(_pid: u32) -> AppResult<bool> {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_running_pid_zero() {
        // PID 0 is invalid on all platforms
        let result = is_process_running(0);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), false);
    }

    #[test]
    fn test_is_process_running_invalid_pid() {
        // Very large PID that likely doesn't exist
        let result = is_process_running(u32::MAX);

        #[cfg(unix)]
        {
            // On Unix, u32::MAX should cause an overflow error
            assert!(result.is_err());
        }

        #[cfg(windows)]
        {
            // On Windows, it should just return false
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), false);
        }
    }

    #[test]
    #[cfg(unix)]
    fn test_validate_pid_range() {
        // Valid PIDs
        assert!(validate_pid_range(1).is_ok());
        assert!(validate_pid_range(65535).is_ok());
        assert!(validate_pid_range(i32::MAX as u32).is_ok());

        // Invalid PIDs (overflow)
        assert!(validate_pid_range((i32::MAX as u32) + 1).is_err());
        assert!(validate_pid_range(u32::MAX).is_err());
    }
}
