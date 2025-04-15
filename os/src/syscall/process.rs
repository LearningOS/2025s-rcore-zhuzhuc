//! Process management syscalls
use crate::{
    task::{exit_current_and_run_next, suspend_current_and_run_next, current_task, TaskStatus},
    timer::{get_time_us, get_time_ms},
};
use log::trace;

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// get time with second and microsecond
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    unsafe {
        *ts = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
    }
    0
}

pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {

    match trace_request {
        0 => {
            // 读取内存，将id视为*const u8地址，读取一个字节
            unsafe {
                let value = *(id as *const u8);
                value as isize
            }
        }
        1 => {
            // 写入内存，将id视为*const u8地址，写入data的最低字节
            unsafe {
                *(id as *mut u8) = data as u8;
                0
            }
        }
        2 => {
            // 查询系统调用次数，返回编号为id的系统调用的调用次数
            let task = current_task().unwrap();
            let inner = task.inner_exclusive_access().exclusive_access();
            let count = inner.syscall_times[id];
            drop(inner);
            count as isize
        }
        _ => -1,
    }
}

pub fn sys_sleep(ms: usize) -> isize {
    trace!("kernel: sys_sleep for {} ms", ms);
    let current = current_task().unwrap();
    let current_time = get_time_ms();
    {
        let mut inner = current.inner_exclusive_access().exclusive_access();
        inner.sleep_until = current_time + ms;
        inner.task_status = TaskStatus::Blocked;
    }
    suspend_current_and_run_next();
    0
}
