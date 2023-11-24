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
pub struct Process<'a, C: HasHint> {
    code: std::rc::Rc<Vec<CodeLine<C>>>,
    instruction_ptr: usize,
    
    manager: &'a Manager<'a, C>,
    
    content: C,
    curr_hint: C::Hint,
}

impl<'a, C: HasHint + core::fmt::Debug> core::fmt::Debug for Process<'a, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<'a, C: HasHint + core::fmt::Display> core::fmt::Display for Process<'a, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.content.fmt(f)
    }
}

impl<'a, C: HasHint> Process<'a, C> {
    pub fn new(content: C, code: Vec<CodeLine<C>>, manager: &'a Manager<'a, C>) -> Self {
        Self {
            code: std::rc::Rc::new(code),
            manager,
            content,
            
            instruction_ptr: 0,
            curr_hint: C::Hint::default(),
        }
    }
}

impl<'a, C: Clone + HasHint> Process<'a, C> {
    pub fn execute_fork(&mut self) -> () {
        for codeline in self.code.clone()[self.instruction_ptr..].iter() {
            let mut hints = codeline(&mut self.content, &self.curr_hint); // Mutates the process and returns the list of hints for branching
            self.instruction_ptr += 1; // Increment the instruction ptr so if the process needs resuming, it knows where to go
            
            if hints.len() > 0 { // If any hints were created, they need to be destributed
                if hints.len() > 1 { // If more than one hint was generated, then forks need to occur
                    for hint in hints.drain(1..) { // Drain the hints pool up from index 1, leaving 0 for the original
                        self.manager.fork(self, hint); // clone the process and assign the hint (nothing else. The manager is not responsible for the execution of code)
                    }
                }
                self.curr_hint = hints.pop().unwrap(); // There will be only one entry in the vector now, time to pop this
            }
        }

        self.instruction_ptr = 0; // Reset instruction pointer after full execution
    }
}

impl<'a, C: HasHint> Process<'a, C> {
    pub fn execute(&mut self) -> () {
        for codeline in self.code.clone().iter() {
            let mut hints = codeline(&mut self.content, &self.curr_hint); // Mutates the process and returns the list of hints for branching
            if hints.len() > 0 {
                self.curr_hint = hints.pop().unwrap(); // There will be only one entry in the vector now, time to pop this
            }
        }
    }
    
    pub fn set_hint(&mut self, hint: C::Hint) {
        self.curr_hint = hint
    }    
}

pub struct Manager<'a, C: HasHint> {
    vec_process: core::cell::RefCell<Vec<Box< Process<'a, C> >>>,
    vec_new_process: core::cell::RefCell<Vec<Box< Process<'a, C> >>>,
    clock: core::cell::Cell<usize>,  
}

impl<'a, C: HasHint + std::fmt::Debug> std::fmt::Display for Manager<'a, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clock: {}, Processes: {:?}", self.clock(), self.vec_process.borrow())
    }
} 

impl<'a, C: HasHint> Manager<'a, C> {
    pub fn new() -> Self {
        Self {
            vec_process: core::cell::RefCell::new(vec![]),
            vec_new_process: core::cell::RefCell::new(vec![]),
            clock: core::cell::Cell::new(0),
        }
    }

    pub fn add_process(&self, new_process: Process<'a, C>) {
        let result_borrowed = self.vec_process.try_borrow_mut();
        let mut borrowed_vec = result_borrowed.unwrap_or_else(|_| self.vec_new_process.borrow_mut());

        borrowed_vec.push(Box::new(new_process))
    }

    pub fn clock(&self) -> usize {
        self.clock.get()
    }
}

impl<'a, C: HasHint + Clone> Manager<'a, C> {    
    fn fork(&'a self, process: &mut Process<'a, C>, hint: C::Hint) {
        let mut new_process = process.clone();
        new_process.set_hint(hint);
        
        self.add_process(new_process);
    }

    pub fn run_fork(&self) {
        let mut start_index = 0;
        let mut borrowed_vec = self.vec_process.borrow_mut();
        
        loop {
            for process in borrowed_vec[start_index..].iter_mut() {
                process.execute_fork();
            }
            
            let mut borrowed_vec_new = self.vec_new_process.borrow_mut();
            if borrowed_vec_new.len() == 0 {
                break;
            }
            
            start_index = borrowed_vec.len();

            borrowed_vec.extend(borrowed_vec_new.drain(..));
        }
        
        self.clock.set(self.clock() + 1)
    }
}

impl<'a, C: HasHint> Manager<'a, C> {    
    pub fn run(&mut self) {
        for process in self.vec_process.borrow_mut().iter_mut() {
            process.execute();
        }
        
        self.clock.set(self.clock() + 1)
    }
}

impl<'a, C: HasHint + Ord> Manager<'a, C> {   
    pub fn prune(&self, range: usize, step: usize) {
        let mut borrowed_vec = self.vec_process.borrow_mut();
        if borrowed_vec.len() <= range {
            return
        }

        
        borrowed_vec.sort_unstable_by(
            |a, b| std::cmp::Ord::cmp(&b.content, &a.content) // backwards because want greatest at front
        );
        let iter = borrowed_vec.drain(range + step..).step_by(step);
        
        let mut borrowed_vec_new = self.vec_new_process.borrow_mut();
        borrowed_vec_new.extend(iter);
        borrowed_vec.extend(borrowed_vec_new.drain(..));
    }
}