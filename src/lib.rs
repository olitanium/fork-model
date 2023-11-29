/*
pub struct IterMutVec<'a, T> {
    vec: &'a mut MutVec<T>,
    iter: std::slice::IterMut<'a, T>,
}

impl<'a, T> Iterator for IterMutVec<'a, T> {
    type Item = &'a mut T;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

pub struct MutVec<T> {
    vec_top_level: Vec<T>,
    vec_adnl: Vec<T>,
}

impl<T> MutVec<T> {
    fn new(input: Vec<T>) -> Self {
        Self {
            vec_top_level: input,
            vec_adnl: vec![],
        }
    }

    fn merge(&mut self) -> usize {
        let length = self.vec_top_level.len();
        self.vec_top_level.extend(self.vec_adnl.drain(..));

        length
    }

}
*/

pub trait HasHint {
    type Hint: Default + Clone;
}

pub type CodeLine<C> = fn(&mut C, &<C as HasHint>::Hint) -> Vec<<C as HasHint>::Hint>;

#[derive(Clone)]
pub struct Process<C: HasHint> {
    code: std::rc::Rc<Vec<CodeLine<C>>>,
    instruction_ptr: usize,
    
    content: C,
    curr_hint: C::Hint,
}

impl<C: HasHint + core::fmt::Debug> core::fmt::Debug for Process<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<C: HasHint + core::fmt::Display> core::fmt::Display for Process<C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<C: HasHint> core::ops::Deref for Process<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.content
    }
}

impl<C: HasHint> core::ops::DerefMut for Process<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.content
    }
}

impl<C: HasHint> Process<C> {
    pub fn new(content: C, code: Vec<CodeLine<C>>) -> Self {
        Self {
            code: std::rc::Rc::new(code),
            content,
            
            instruction_ptr: 0,
            curr_hint: C::Hint::default(),
        }
    }
}

impl<C: Clone + HasHint> Process<C> {
    fn execute_fork(&mut self, manager: &mut Manager<C>) -> () {
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

impl<C: HasHint> Process<C> {
    pub fn execute(&mut self) -> () {
        for codeline in self.code.clone().iter() {
            let mut hints = codeline(&mut self.content, &self.curr_hint); // Mutates the process and returns the list of hints for branching
            match hints.pop() {
                Some(val) => self.curr_hint = val,
                None => {}
            }
        }
    }
}

impl<C: HasHint> Process<C> {
    fn set_hint(&mut self, hint: C::Hint) {
        self.curr_hint = hint
    }    
}

pub struct Manager<C: HasHint> {
    vec: Vec<Process<C>>,
    clock: usize,
}


impl<C: HasHint + std::fmt::Debug> std::fmt::Display for Manager<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clock: {}, Processes: {:?}", self.clock(), self.vec)
    }
} 

impl<C: HasHint> Manager<C> {
    pub fn new(content: C, code: Vec<CodeLine<C>>) -> Self {
        Self {
            vec: vec![Process::new(content, code)],
            clock: 0,
        }
    }
}

impl<C: HasHint> Manager<C> {
    pub fn empty() -> Self {
        Self {
            vec: vec![],
            clock: 0,
        }
    }
}

impl<C: HasHint> Manager<C> {
    pub fn add_process(&mut self, new_process: Process<C>) {
        self.vec.push(new_process);
    }
}

impl<C: HasHint> Manager<C> {
    pub fn clock(&self) -> usize {
        self.clock
    }
}

impl<C: HasHint + Clone> Manager<C> {
    fn fork(&mut self, process: &mut Process<C>, hint: C::Hint) {
        let mut new_process = process.clone();
        new_process.set_hint(hint);
        
        self.add_process(new_process);
    }
}

impl<C: HasHint + Clone> Manager<C> {
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

impl<C: HasHint + Ord> Manager<C> {   
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