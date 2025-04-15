//! Task management implementation
//!
//! Everything about task management, like starting and switching tasks is
//! implemented here.
//!
//! A single global instance of [`TaskManager`] called `TASK_MANAGER` controls
//! all the tasks in the operating system.
//!
//! Be careful when you see `__switch` ASM function in `switch.S`. Control flow around this function
//! might not be what you expect.

mod context;
mod switch;
#[allow(clippy::module_inception)]
mod task;

use crate::config::MAX_APP_NUM;
use crate::loader::{get_num_app, init_app_cx};
use crate::sync::UPSafeCell;
use crate::timer::get_time_ms;
use lazy_static::*;
use log::trace;
use switch::__switch;
pub use task::{TaskControlBlock, TaskStatus};


use alloc::sync::Arc;

pub use context::TaskContext;

/// The task manager, where all the tasks are managed.
///
/// Functions implemented on `TaskManager` deals with all task state transitions
/// and task context switching. For convenience, you can find wrappers around it
/// in the module level.
///
/// Most of `TaskManager` are hidden behind the field `inner`, to defer
/// borrowing checks to runtime. You can see examples on how to use `inner` in
/// existing functions on `TaskManager`.
pub struct TaskManager {
    /// total number of tasks
    num_app: usize,
    /// use inner value to get mutable access
    inner: UPSafeCell<TaskManagerInner>,
}

/// Inner of Task Manager
pub struct TaskManagerInner {
    /// task list
    tasks: [Arc<TaskControlBlock>; MAX_APP_NUM],
    /// id of current `Running` task
    current_task: usize,
}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let tasks = core::array::from_fn(|_| Arc::new(TaskControlBlock::new()));
        for (i, task) in tasks.iter().enumerate() {
            let mut inner = task.inner_exclusive_access().exclusive_access();
            inner.task_cx = TaskContext::goto_restore(init_app_cx(i));
            inner.task_status = TaskStatus::Ready;
            drop(inner);
        }
        TaskManager {
            num_app,
            inner: unsafe {
                UPSafeCell::new(TaskManagerInner {
                    tasks,
                    current_task: 0,
                })
            },
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) -> ! {
        let inner = self.inner.exclusive_access();
        let task0 = &inner.tasks[0];
        let mut task0_inner = task0.inner_exclusive_access().exclusive_access();
        task0_inner.task_status = TaskStatus::Running;
        let next_task_cx_ptr = &task0_inner.task_cx as *const TaskContext;
        drop(task0_inner);
        drop(inner);
        let mut _unused = TaskContext::zero_init();
        unsafe {
            __switch(&mut _unused as *mut TaskContext, next_task_cx_ptr);
        }
        panic!("unreachable in run_first_task!");
    }

    fn mark_current_suspended(&self) {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let task = &inner.tasks[current];
        let mut task_inner = task.inner_exclusive_access().exclusive_access();
        task_inner.task_status = TaskStatus::Ready;
        drop(task_inner);
    }

    fn mark_current_exited(&self) {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let task = &inner.tasks[current];
        let mut task_inner = task.inner_exclusive_access().exclusive_access();
        task_inner.task_status = TaskStatus::Exited;
        drop(task_inner);
    }

    fn find_next_task(&self) -> Option<usize> {
        let inner = self.inner.exclusive_access();
        let current = inner.current_task;
        let current_time = get_time_ms();
        trace!("kernel: find_next_task, current_time = {}", current_time);
        (current + 1..current + self.num_app + 1)
            .map(|id| id % self.num_app)
            .find(|id| {
                let task = &inner.tasks[*id];
                let mut task_inner = task.inner_exclusive_access().exclusive_access();
                let status = match task_inner.task_status {
                    TaskStatus::Ready => {
                        trace!("kernel: task {} is ready", id);
                        true
                    }
                    TaskStatus::Blocked => {
                        trace!("kernel: task {} is blocked, sleep_until = {}", id, task_inner.sleep_until);
                        if current_time >= task_inner.sleep_until {
                            trace!("kernel: task {} is ready to wake up", id);
                            task_inner.task_status = TaskStatus::Ready;
                            true
                        } else {
                            false
                        }
                    }
                    _ => false
                };
                drop(task_inner);
                status
            })
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            trace!("kernel: run_next_task, next = {}", next);
            let mut inner = self.inner.exclusive_access();
            let current = inner.current_task;
            inner.current_task = next;
            let current_task = &inner.tasks[current];
            let next_task = &inner.tasks[next];
            let mut next_task_inner = next_task.inner_exclusive_access().exclusive_access();
            next_task_inner.task_status = TaskStatus::Running;
            let mut current_task_inner = current_task.inner_exclusive_access().exclusive_access();
            let current_task_cx_ptr = &mut current_task_inner.task_cx as *mut TaskContext;
            let next_task_cx_ptr = &next_task_inner.task_cx as *const TaskContext;
            drop(next_task_inner);
            drop(current_task_inner);
            drop(inner);
            unsafe {
                __switch(current_task_cx_ptr, next_task_cx_ptr);
            }
        } else {
            panic!("All applications completed!");
        }
    }
}

/// Run the first task in task list.
pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

/// Switch current `Running` task to the task we have found,
/// or there is no `Ready` task and we can exit with all applications completed
fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

/// Change the status of current `Running` task into `Ready`.
fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

/// Change the status of current `Running` task into `Exited`.
fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

/// Suspend the current 'Running' task and run the next task in task list.
pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

/// Exit the current 'Running' task and run the next task in task list.
pub fn exit_current_and_run_next() {
    mark_current_exited();
    run_next_task();
}

/// Get the current task's TaskControlBlock
pub fn current_task() -> Option<Arc<TaskControlBlock>> {
    let inner = TASK_MANAGER.inner.exclusive_access();
    Some(Arc::clone(&inner.tasks[inner.current_task]))
}
