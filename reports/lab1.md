# Lab3 实验报告

1. 实现了 `sys_trace` 系统调用，支持三种功能：
   - 读取指定内存地址的一个字节
   - 写入指定内存地址的一个字节
   - 查询特定系统调用的调用次数

2. 在任务控制块中添加系统调用计数器，记录每个系统调用的使用次数

- 在 TaskControlBlockInner 中添加 syscall_times 数组记录系统调用次数
- 实现内存读写操作时使用 unsafe 进行类型转换
- 在系统调用处理函数中统计调用次数

### 实现中遇到的问题和解决方案

1. **BorrowMutError 问题**
   - 问题：在任务切换时出现 `BorrowMutError`，这是因为同时获取了多个任务的 `exclusive_access`
   - 解决方案：修改 `run_next_task` 函数，确保在获取下一个任务的引用之前释放当前任务的引用

2. **时钟周期转换问题**
   - 问题：测试用例期望以时钟周期为单位进行睡眠，而不是毫秒
   - 解决方案：在 `sys_sleep` 中将时钟周期转换为毫秒（每个时钟周期 = 10ms）

### 设计要点

1. **任务状态管理**
   - 使用 `TaskStatus::Blocked` 表示任务处于睡眠状态
   - 在 `TaskControlBlockInner` 中添加 `sleep_until` 字段记录唤醒时间

2. **安全的引用管理**
   - 使用 `UPSafeCell` 确保内部可变性的安全性
   - 及时释放 `exclusive_access` 以避免死锁

3. **时间管理**
   - 使用 `get_time_ms()` 获取当前时间
   - 在调度器中检查睡眠任务是否应该被唤醒

U态程序特权指令错误行为分析：
 

运行 ch2b_bad_*.rs 测例，观察到以下行为：
- 当使用 S 态特权指令时，触发非法指令异常
- 当访问 S 态寄存器时，触发非法指令异常
- 使用的 SBI 版本：RustSBI version 0.3.0

1. trap.S 中 __alltraps 和 __restore 函数分析：
a) __restore 函数进入时的 sp 值：
- sp 指向内核栈上的 TrapContext 结构体
- 使用情景：
  1. 从 S 态 trap 处理返回 U 态
  2. 任务切换时恢复新任务上下文
b) L43-L48 寄存器处理分析：
```assembly
ld t0, 32*8(sp)  # 恢复 sstatus
ld t1, 33*8(sp)  # 恢复 sepc
ld t2, 2*8(sp)   # 恢复用户栈指针
csrw sstatus, t0
csrw sepc, t1
csrw sscratch, t2
```
这些指令处理了：
- sstatus：控制处理器特权级
- sepc：异常返回地址
- sscratch：保存用户栈指针

c) L50-L56 跳过寄存器原因：
- x2 (sp)：需要特殊处理栈指针切换
- x4 (tp)：用户程序未使用此寄存器

d) L60 指令效果分析：
```assembly
csrrw sp, sscratch, sp
```
执行后：
- sp：切换到用户栈
- sscratch：保存内核栈地址

e) 状态切换分析：
- sret 指令触发特权级切换
- 通过 sstatus.SPP 位确定目标特权级

f) L13 指令分析：
```assembly
csrrw sp, sscratch, sp
```
执行后：
- sp：切换到内核栈
- sscratch：保存用户栈地址

g) U态进入S态：
- 通过 ecall 指令触发环境调用异常

## 荣誉准则

本人承诺完成的所有实验内容均为个人独立完成，没有抄袭或与他人合作。