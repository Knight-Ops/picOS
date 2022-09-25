use core::marker::PhantomData;

use crate::constants::TASK_STACK_SIZE;
use alloc::boxed::Box;
use alloc::string::String;

enum TaskState {
    Ready,
}

// #[derive(Clone)]
pub struct Task {
    name: String,
    stack_pointer: usize,
    args: Box<dyn TaskArgument>,
    stack: alloc::vec::Vec<u32>,
    _priority: u8,
    _state: TaskState,
    // phantom: PhantomData<&'a u8>,
}

impl<'a> Task {
    pub fn new<T>(name: String, function_pointer: fn(Box<T>) -> !, args: T) -> Self
    where
        T: TaskArgument + 'static,
    {
        let mut stack = alloc::vec![0; TASK_STACK_SIZE];

        let stack_pointer = unsafe { stack.as_ptr().add(TASK_STACK_SIZE - 16).addr() };
        let boxed_args = Box::new(args);

        stack[TASK_STACK_SIZE - 1] = 1 << 24;
        stack[TASK_STACK_SIZE - 2] = (function_pointer as *const u8).addr() as u32;
        stack[TASK_STACK_SIZE - 8] = (&*boxed_args as *const T).addr() as u32;

        Self {
            name,
            stack_pointer,
            args: boxed_args,
            stack: stack,
            _priority: 0,
            _state: TaskState::Ready,
            // phantom: PhantomData,
        }
    }

    pub(crate) fn set_task_sp(&mut self, sp: usize) {
        self.stack_pointer = sp;
    }

    pub(crate) fn get_task_sp(&self) -> usize {
        self.stack_pointer
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

pub trait TaskArgument: Send {}
