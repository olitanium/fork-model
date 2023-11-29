use fork_model::{self};

type Cargo = f32;

enum VesselMethod {
    ROAD,
    SEA,
    AIR,
}

enum QueueMode {
    LOAD,
    UNLOAD,
    PASS,
}

struct VesselConfig {
    name: std::string::String,
    method: VesselMethod,
    capacity: Cargo,
    speed: f32,
    volume: i32,
}

struct Vessel {
    config: VesselConfig,
    destination: std::string::String,
    time_to_run: f32,
    contents: Cargo,
    queue_mode: QueueMode, 
    ready_to_leave: bool,
}


#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
struct Model {}

impl fork_model::HasHint for Model {
    type Hint = std::string::String;
}

fn main() {
    let code: Vec<fork_model::CodeLine<Model, i32>> = vec![];
    let mut manager = fork_model::Manager::new(Model {}, code.clone());

    loop {
        manager.execute();
        manager.prune(10, 2);
        println!("{:?}", manager);
    }
}