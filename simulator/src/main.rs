use learn_pltl_fast::Sample;
use std::fs::File;
use std::io::BufWriter;

mod ai;
mod monitor;
mod task;
mod world;

use ai::*;
use monitor::*;
use world::*;

fn main() {
    let sample = collect_sample();

    let name = format!("sample_simulator.ron");
    let file = File::create(name).expect("open sample file");
    let buf_writer = BufWriter::new(file);
    ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
}

fn collect_sample() -> Sample<8> {
    let mut sample = Sample {
        positive_traces: Vec::new(),
        negative_traces: Vec::new(),
    };
    for _ in 0..100 {
        // let mut world = World::lab_scenario();
        // let mut world = World::recharging_scenario();
        // let mut world = World::proc_gen_recharging_scenario();
        // let (mut world, mut task) = World::door_scenario();
        let (mut world, mut task) = World::proc_gen_scenario();
        let mut ai = Ai::default();
        let monitors: [Box<dyn Monitor>; 8] = [
            Box::new(BatteryLevel(0)),
            Box::new(BatteryLevel(World::MAX_CHARGE / 3)),
            Box::new(BatteryLevel((World::MAX_CHARGE * 2) / 3)),
            Box::new(BatteryLevel(World::MAX_CHARGE)),
            Box::new(DoorClosed),
            Box::new(InRoom(Room::Lab)),
            Box::new(InRoom(Room::Kitchen)),
            Box::new(InRoom(Room::ChargingStation)),
        ];
        let (trace, success) = world.run::<8>(&mut ai, task.as_mut(), &monitors);
        if success {
            sample.add_positive_trace(trace).expect("add new trace");
        } else {
            sample.add_negative_trace(trace).expect("add new trace");
        }
    }
    sample
}