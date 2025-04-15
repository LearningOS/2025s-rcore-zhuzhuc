//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;
/// yield syscall
const SYSCALL_YIELD: usize = 124;
/// gettime syscall
const SYSCALL_GET_TIME: usize = 169;
/// trace syscall
const SYSCALL_TRACE: usize = 410;
/// sleep syscall
const SYSCALL_SLEEP: usize = 411;

mod fs;
mod process;

use fs::*;
use process::*;
use crate::task::current_task;

/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    // 增加系统调用计数
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access().exclusive_access();
    inner.syscall_times[syscall_id] += 1;
    drop(inner);
    
    let ret = match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => sys_get_time(args[0] as *mut TimeVal, args[1]),
        SYSCALL_TRACE => sys_trace(args[0], args[1], args[2]),
        SYSCALL_SLEEP => sys_sleep(args[0]),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    };
    ret
}

/// trace syscall implementation
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    match trace_request {
        0 => {
            // 读取内存
            let ptr = id as *const u8;
            unsafe { *ptr as isize }
        }
        1 => {
            // 写入内存
            let ptr = id as *mut u8;
            unsafe {
                *ptr = data as u8;
            }
            0
        }
        2 => {
            // 查询系统调用次数
            let task = current_task().unwrap();
            let inner = task.inner_exclusive_access().exclusive_access();
            let count = inner.syscall_times[id];
            drop(inner);
            count as isize
        }
        _ => -1
    }
}
