#![no_std]

extern crate alloc;

use core::{
    fmt,
    fmt::{
        Debug,
        Display,
    },
    ops::{
        Deref,
        DerefMut,
    }
};

use alloc::{
    rc::Rc,
    vec,
    vec::Vec,
    borrow::ToOwned,
};

pub type CodeLine<C, H> = fn(&mut C, Option<H>) -> Vec<H>;

#[derive(Clone)]
pub struct Process<C, H> {
    code: alloc::rc::Rc<Vec<CodeLine<C, H>>>,
    instruction_ptr: usize,

    content: C,
    curr_hint: Option<H>,
}

impl<C: Debug, H> Debug for Process<C, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.fmt(f)
    }
}

impl<C: Display, H> Display for Process<C, H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.content.fmt(f)
    }
}

impl<C, H> Deref for Process<C, H> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C, H> DerefMut for Process<C, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<C, H> Process<C, H> {
    pub fn new(content: C, code: &[CodeLine<C, H>]) -> Self {
        Self {
            code: Rc::new(code.to_owned()),
            content,

            instruction_ptr: 0,
            curr_hint: None,
        }
    }
}

impl<C: Clone, H: Clone> Process<C, H> {
    fn execute_fork(this: &mut Self, manager: &mut Manager<C, H>) -> () {
        for codeline in &this.code.clone()[this.instruction_ptr..] {
            let mut hints = codeline(&mut this.content, this.curr_hint.take()); // Mutates the process and returns the list of hints for branching
            this.instruction_ptr += 1; // Increment the instruction ptr so if the process needs resuming, it knows where to go

            if hints.len() > 0 {
                // If any hints were created, they need to be destributed
                if hints.len() > 1 {
                    // If more than one hint was generated, then forks need to occur
                    for hint in hints.drain(1..) {
                        // Drain the hints pool up from index 1, leaving 0 for the original
                        manager.fork(this, hint); // clone the process and assign the hint (nothing else. The manager is not responsible for the execution of code)
                    }
                }
                this.curr_hint = hints.pop(); // There will be only one entry in the vector now, time to pop this
            }
        }

        this.instruction_ptr = 0; // Reset instruction pointer after full execution
    }
}

impl<C, H> Process<C, H> {
    pub fn execute(this: &mut Self) -> () {
        for codeline in this.code.iter() {
            let mut hints = codeline(&mut this.content, this.curr_hint.take()); // Mutates the process and returns the list of hints for branching
            this.curr_hint = hints.pop();
        }
    }
}

impl<C, H> Process<C, H> {
    fn set_hint(this: &mut Self, hint: H) -> () {
        this.curr_hint = Some(hint)
    }
}

pub struct Manager<C, H> {
    vec: Vec<Process<C, H>>,
    clock: usize,
}

impl<C, H> Manager<C, H> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.vec.iter().map(|process| &process.content)
    }
}

impl<C: core::fmt::Debug, H> core::fmt::Debug for Manager<C, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Clock: {}, Processes: {:?}", self.clock(), self.vec)
    }
}

impl<C, H> Manager<C, H> {
    pub fn new(content: C, code: &[CodeLine<C, H>]) -> Self {
        Self {
            vec: vec![Process::new(content, code)],
            clock: 0,
        }
    }
}

impl<C, H> Manager<C, H> {
    pub fn empty() -> Self {
        Self {
            vec: vec![],
            clock: 0,
        }
    }
}

impl<C, H> Manager<C, H> {
    pub fn add_process(&mut self, new_process: Process<C, H>) {
        self.vec.push(new_process);
    }
}

impl<C, H> Manager<C, H> {
    pub fn clock(&self) -> usize {
        self.clock
    }
}

impl<C: Clone, H: Clone> Manager<C, H> {
    fn fork(&mut self, process: &mut Process<C, H>, hint: H) {
        let mut new_process = process.clone();
        Process::set_hint(&mut new_process, hint);

        self.add_process(new_process);
    }
}

impl<C: Clone, H: Clone> Manager<C, H> {
    pub fn execute(&mut self) {
        let mut start_index = 0;
        let mut temp_manager = Manager::empty();

        loop {
            for process in &mut self.vec[start_index..] {
                Process::execute_fork(process, &mut temp_manager);
            }

            if temp_manager.vec.len() == 0 {
                break;
            }

            start_index = self.vec.len();

            self.vec.extend(temp_manager.vec.drain(..));
        }

        self.clock += 1
    }
}

impl<C: Ord, H> Manager<C, H> {
    pub fn prune(&mut self, range: usize, step: usize) {
        if self.vec.len() > range {
            self.vec.sort_unstable_by(
                |a, b| b.content.cmp(&a.content), // backwards because want greatest at front
            );

            let mut temp = self.vec.drain(range + step..).step_by(step).collect();
            self.vec.append(&mut temp);
        }
    }
}

impl<C: Ord, H> Manager<C, H> {
    pub fn best(&self) -> Option<&C> {
        self.iter().max()
    }
}