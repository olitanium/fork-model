fn main() {
    let code: Vec<fork_model::CodeLine<i32, i32>> = vec![
        |model: &mut i32, _: Option<i32>| {
            *model += 1;
            return vec![0, 1];
        },
        |model: &mut i32, hint: Option<i32>| {
            *model += hint.unwrap_or_default();
            return vec![];
        },
    ];
    let mut manager = fork_model::Manager::new(0, &code);
    println!("{:?}", manager);
    
    loop {
        manager.execute();
        manager.prune(10, 2);
        println!("{:?}", manager);
    }
}
