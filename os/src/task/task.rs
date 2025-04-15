use crate::sync::UPSafeCell;
use crate::task::context::TaskContext;

pub struct TaskControlBlock {
    pub inner: UPSafeCell<TaskControlBlockInner>,
}

pub struct TaskControlBlockInner {
    pub task_status: TaskStatus,
    pub task_cx: TaskContext,
    pub syscall_times: [usize; 500],
    pub sleep_until: usize,
}

impl TaskControlBlock {
    pub fn new() -> Self {
        Self {
            inner: unsafe {
                UPSafeCell::new(TaskControlBlockInner {
                    task_status: TaskStatus::Ready,
                    task_cx: TaskContext::zero_init(),
                    syscall_times: [0; 500],
                    sleep_until: 0,
                })
            }
        }
    }

    pub fn inner_exclusive_access(&self) -> &UPSafeCell<TaskControlBlockInner> {
        &self.inner
    }
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
    /// blocked (e.g. sleeping)
    Blocked,
}
