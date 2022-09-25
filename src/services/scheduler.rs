use alloc;
use core::{arch::asm, cell::RefCell};
use cortex_m::peripheral::{syst::SystClkSource, SYST};
use cortex_m_rt::exception;

use crate::constants::MAX_TASKS;
use crate::debug;
use crate::services::task::Task;
use crate::sync::NakedMutex;

use super::post_office::PostOffice;

static SCHEDULER: NakedMutex<RefCell<Option<Scheduler>>> = NakedMutex::new(RefCell::new(None));

#[exception]
unsafe fn SysTick() {
    // Push the full context onto the stack
    cortex_m::interrupt::disable();
    asm!(
        // This is the hackiest of hacks, I do not like it at all, but without abandoning Rust, or going to a naked function that can only have ASM, I don't know how else to approaach Rust emitting sporadic code.
        // Rust is pushing 4 registers to the stack, then subtracting 8 more bytes for stack variables (that aren't used anywhere?)
        "add sp, 0x18",
        "push {{r4-r7}}",
        "mov r4, r8",
        "mov r5, r9",
        "mov r6, r10",
        "mov r7, r11",
        "push {{r4-r7}}",
    );

    {
        let mut borrow = SCHEDULER.borrow().borrow_mut();
        let sched = borrow.as_mut().unwrap();

        // // Finish saving this task's context by saving the stack pointer so we can find it again
        let mut curr_sp: usize;
        asm!("mov {0}, sp", out(reg) curr_sp);
        sched.set_current_task_stack_pointer(curr_sp);

        // Now we move the current task and load the context by getting the stack pointer it saved
        sched.next_task();

        let new_sp = sched.get_current_task_stack_pointer();
        asm!("mov sp, {0}", in(reg) new_sp);
    }

    // // // Pop the full context off of the stack
    asm!(
        "pop {{r4-r7}}",
        "mov r8, r4",
        "mov r9, r5",
        "mov r10, r6",
        "mov r11, r7",
        "pop {{r4-r7}}",
    );
    cortex_m::interrupt::enable();
    asm!("bx lr");
}

pub enum ScheduleType {
    RoundRobin(u32),
}

#[derive(Debug)]
pub enum SchedulerError {
    NoPopulatedTasks,
    TaskListFull,
}

pub struct Scheduler {
    schedule_type: ScheduleType,
    current_task_idx: Option<usize>,
    populated_tasks: usize,
    tasks: [Option<Task>; MAX_TASKS],
}

impl Scheduler {
    pub fn new(schedule_type: ScheduleType) -> Self {
        Self {
            schedule_type,
            current_task_idx: None,
            populated_tasks: 0,
            // tasks: [None; MAX_TASKS],
            tasks: [None, None, None, None],
        }
    }

    fn next_task(&mut self) {
        match self.schedule_type {
            ScheduleType::RoundRobin(_) => match self.current_task_idx {
                Some(idx) => {
                    if idx + 1 == self.populated_tasks {
                        self.current_task_idx = Some(0);
                    } else {
                        self.current_task_idx = Some(idx + 1);
                    }
                }
                None => self.current_task_idx = Some(0),
            },
        }
    }

    // If `current_task_idx` is None (only on first trigger of SysTick) we just ignore the store
    fn set_current_task_stack_pointer(&mut self, sp: usize) {
        if let Some(idx) = self.current_task_idx {
            unsafe {
                self.tasks
                    .get_unchecked_mut(idx)
                    .as_mut()
                    .unwrap()
                    .set_task_sp(sp);
            }
        }
    }

    // This function will panic if you call it without a current_task_idx being Some()
    fn get_current_task_stack_pointer(&mut self) -> usize {
        self.tasks[self.current_task_idx.unwrap()]
            .as_ref()
            .unwrap()
            .get_task_sp()
    }

    pub fn add_task(&mut self, task: Task) -> Result<usize, SchedulerError> {
        if let Some((idx, task_entry)) = self
            .tasks
            .iter_mut()
            .enumerate()
            .skip_while(|(_, val)| (**val).is_some())
            .take(1)
            .next()
        {
            PostOffice::register_mailbox(idx, task.get_name());
            *task_entry = Some(task);
            self.populated_tasks += 1;
            Ok(idx)
        } else {
            Err(SchedulerError::TaskListFull)
        }
    }

    pub fn get_task_count(&self) -> usize {
        self.populated_tasks
    }

    pub fn start(self, mut systick: SYST) -> Result<(), SchedulerError> {
        systick.set_clock_source(SystClkSource::Core);
        match self.schedule_type {
            ScheduleType::RoundRobin(interval) => systick.set_reload(interval),
        }
        systick.clear_current();

        if self.populated_tasks == 0 {
            return Err(SchedulerError::NoPopulatedTasks);
        }

        unsafe {
            cortex_m::interrupt::disable();
            SCHEDULER.borrow().borrow_mut().replace(self);
            cortex_m::interrupt::enable();
        }

        systick.enable_counter();
        systick.enable_interrupt();

        Ok(())
    }
}
