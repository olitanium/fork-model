use fork_model;

#[derive(Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
struct Wi32(i32);

impl fork_model::HasHint for Wi32 {
    type Hint = Wi32;
}

impl std::fmt::Debug for Wi32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn main() {
    
    let code: Vec<fork_model::CodeLine<Wi32>> = vec![
        |content: &mut Wi32, _: &Wi32| -> Vec<Wi32> {
            content.0 += 1;
            return vec![Wi32(1),Wi32(2)];
        },
        |content: &mut Wi32, hint: &Wi32| -> Vec<Wi32> {
            content.0 += hint.0;
            return vec![];
        },
    ];
        
    let manager = fork_model::Manager::new();
    let process = fork_model::Process::new(Wi32(0), code, &manager);
    manager.add_process(process);

    loop {
        manager.run_fork();
        manager.prune(10, 2);
        println!("{}", manager);
    }
}