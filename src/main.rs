#[derive(Debug, Clone, Ord, PartialEq, PartialOrd, Eq)]
struct Model(i32);

impl Model {
    fn new(input: i32) -> Self {
        Model {0:input}
    }

    fn square(&self) -> i32 {
        self.0 * self.0
    }
}

fn main() {
    let code: Vec<fork_model::Code<Model, i32>> = vec![
        |model: &mut Model, _: Option<i32>| {
            model.0 += 1;
            return vec![0, 1];
        },
        |model: &mut Model, hint: Option<i32>| {
            model.0 += hint.unwrap_or_default();
            return vec![];
        },
        ];
        
    let model = fork_model::Process::new(Model::new(10), &code);
    
    println!("{}", model.square());

    let mut manager = fork_model::Manager::from(model);

    println!("{:?}", manager);
    
    loop {
        manager.execute();
        manager.prune(10, 2);
        println!("{:?}", manager);
    }

}