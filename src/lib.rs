pub type CodeLine<C, H> = fn(&mut C, &H) -> Vec<H>;

#[derive(Clone)]
pub struct Process<C, H> {
    code: std::rc::Rc<Vec<CodeLine<C, H>>>,
    instruction_ptr: usize,
    
    content: C,
    curr_hint: H,
}

impl<C: core::fmt::Debug, H> core::fmt::Debug for Process<C, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<C: core::fmt::Display, H> core::fmt::Display for Process<C, H> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<C, H> core::ops::Deref for Process<C, H> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C, H> core::ops::DerefMut for Process<C, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<C, H: Default> Process<C, H> {
    pub fn new(content: C, code: Vec<CodeLine<C, H>>) -> Self {
        Self {
            code: std::rc::Rc::new(code),
            content,
            
            instruction_ptr: 0,
            curr_hint: H::default(),
        }
    }
}

impl<C: Clone, H: Clone> Process<C, H> {
    fn execute_fork(&mut self, manager: &mut Manager<C, H>) -> () {
        for codeline in &self.code.clone()[self.instruction_ptr..] {           
            
            let mut hints = codeline(&mut self.content, &self.curr_hint); // Mutates the process and returns the list of hints for branching
            self.instruction_ptr += 1; // Increment the instruction ptr so if the process needs resuming, it knows where to go
            
            if hints.len() > 0 { // If any hints were created, they need to be destributed
                if hints.len() > 1 { // If more than one hint was generated, then forks need to occur
                    for hint in hints.drain(1..) { // Drain the hints pool up from index 1, leaving 0 for the original
                        manager.fork(self, hint); // clone the process and assign the hint (nothing else. The manager is not responsible for the execution of code)
                    }
                }
                self.curr_hint = hints.pop().unwrap(); // There will be only one entry in the vector now, time to pop this
            }
        }

        self.instruction_ptr = 0; // Reset instruction pointer after full execution
    }
}

impl<C, H> Process<C, H> {
    pub fn execute(&mut self) -> () {
        for codeline in self.code.iter() {
            let mut hints = codeline(&mut self.content, &self.curr_hint); // Mutates the process and returns the list of hints for branching
            match hints.pop() {
                Some(val) => self.curr_hint = val,
                None => {}
            }
        }
    }
}

impl<C, H> Process<C, H> {
    fn set_hint(&mut self, hint: H) {
        self.curr_hint = hint
    }    
}

pub struct Manager<C, H> {
    vec: Vec<Process<C, H>>,
    clock: usize,
}


impl<C: std::fmt::Debug, H> std::fmt::Debug for Manager<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clock: {}, Processes: {:?}", self.clock(), self.vec)
    }
} 

impl<C, H: Default> Manager<C, H> {
    pub fn new(content: C, code: Vec<CodeLine<C, H>>) -> Self {
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
        new_process.set_hint(hint);
        
        self.add_process(new_process);
    }
}

impl<C: Clone, H: Clone> Manager<C, H> {
    pub fn execute(&mut self) {
        let mut start_index = 0;
        let mut temp_manager = Manager::empty();
        
        loop {
            for process in &mut self.vec[start_index..] {
                process.execute_fork(&mut temp_manager);
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

/* impl<C: HasHint> Manager<C> {    
    pub fn run(&mut self) {
        for process in &mut self.vec {
            process.execute();
        }
        
        self.clock += 1
    }
} */

impl<C: Ord, H> Manager<C, H> {   
    pub fn prune(&mut self, range: usize, step: usize) {
        if self.vec.len() > range {
            self.vec.sort_unstable_by(
                |a, b| std::cmp::Ord::cmp(&b.content, &a.content) // backwards because want greatest at front
            );

            let mut temp = 
                self.vec
                .drain(range + step..)
                .step_by(step)
                .collect();
            self.vec.append(&mut temp);
        }
    }
}