use learn_pltl_fast::Sample;
use scenario::Scenario;
use std::fs::File;
use std::io::BufWriter;

mod ai;
mod monitor;
mod scenario;
mod task;
mod world;

use monitor::*;
use world::*;

fn main() {
    let sample = collect_sample();
    // let sample = run_scenario(Box::new(Scenario::proc_gen_scenario));

    let name = format!("sample_simulator.ron");
    let file = File::create(name).expect("open sample file");
    let buf_writer = BufWriter::new(file);
    ron::ser::to_writer(buf_writer, &sample).expect("serialize sample");
}

fn run_scenario(init: Box<dyn Fn() -> Scenario>) -> Sample<8> {
    let mut sample = Sample::default();
    let monitors: [Box<dyn Monitor>; 8] = [
        Box::new(BatteryLevel(0)),
        Box::new(BatteryLevel(World::MAX_CHARGE / 3)),
        Box::new(BatteryLevel((World::MAX_CHARGE * 2) / 3)),
        Box::new(BatteryLevel(World::MAX_CHARGE)),
        Box::new(DoorClosed),
        Box::new(InRoom(Room::Lab)),
        Box::new(InRoom(Room::Office)),
        Box::new(InRoom(Room::ChargingStation)),
    ];
    for _ in 0..500 {
        let mut scenario = init();
        let (trace, success) = scenario.run::<8>(&monitors);
        if success {
            sample.add_positive_trace(trace).expect("add new trace");
        } else {
            sample.add_negative_trace(trace).expect("add new trace");
        }
    }
    sample
}

fn collect_sample() -> Sample<8> {
    let mut sample = Sample::default();
    let monitors: [Box<dyn Monitor>; 8] = [
        Box::new(BatteryLevel(0)),
        Box::new(BatteryLevel(World::MAX_CHARGE / 3)),
        Box::new(BatteryLevel((World::MAX_CHARGE * 2) / 3)),
        Box::new(BatteryLevel(World::MAX_CHARGE)),
        Box::new(DoorClosed),
        Box::new(InRoom(Room::Lab)),
        Box::new(InRoom(Room::Office)),
        Box::new(InRoom(Room::ChargingStation)),
    ];
    for _ in 0..1000 {
        // let mut world = World::lab_scenario();
        // let mut world = World::recharging_scenario();
        // let mut world = World::proc_gen_recharging_scenario();
        // let (mut world, mut task) = World::door_scenario();
        let (mut world, mut task, mut ai) = World::proc_gen_scenario();
        let (trace, success) = world.run::<8>(ai.as_mut(), task.as_mut(), &monitors);
        if success {
            sample.add_positive_trace(trace).expect("add new trace");
        } else {
            sample.add_negative_trace(trace).expect("add new trace");
        }
    }
    sample
}