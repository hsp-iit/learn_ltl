use learn_pltl_fast::Trace;
use petgraph::prelude::*;
use rand::prelude::*;
use crate::*;
use crate::ai::{Ai, AStarAi};
use crate::task::{Task, ReachNode};

#[derive(Debug)]
pub struct Scenario {
    world: World,
    ai: Box<dyn Ai>,
    task: Box<dyn Task>,
}

impl Scenario {
    pub fn run<const N: usize>(
        &mut self,
        monitors: &[Box<dyn Monitor>; N],
    ) -> (Trace<N>, bool) {
        let mut trace = Vec::new();
        while self.world.running() {
            let mut records = [false; N];
            for (val, monitorable) in (records.iter_mut()).zip(monitors.iter()) {
                *val = monitorable.get(&self.world);
            }
            trace.push(records);
            if self.task.success(&self.world) {
                return (trace, true);
            }
            let action = self.ai.decide(&self.world);
            let result = self.world.execute(&action);
            self.world.outcome = (action, result);
        }
        (trace, false)
    }

    pub fn proc_gen_scenario() -> Self {
        const ROOM_TYPES: [Room; 3] = [Room::Office, Room::ChargingStation, Room::Lab];
        let mut rng = StdRng::from_entropy();
        let mut rooms = Graph::new_undirected();

        let start_room_type = *ROOM_TYPES.choose(&mut rng).expect("choose room type");
        let start = rooms.add_node(start_room_type);

        for _ in 0..10 {
            let room_type = *ROOM_TYPES.choose(&mut rng).expect("choose room type");
            let other_node = rooms
                .node_indices()
                .choose(&mut rng)
                .expect("choose a random node");
            let room = rooms.add_node(room_type);
            rooms.add_edge(room, other_node, Path { running_cost: rng.gen_range(2..=3), locked: rng.gen_bool(0.2) });
        }

        let goal_room = rooms.node_indices().choose(&mut rng).expect("goal room");

        let world = World::new(rooms, start);
        
        let task = Box::new(ReachNode::new(goal_room));

        let ai = Box::new(AStarAi::new(goal_room));

        Scenario {
            world,
            ai,
            task,
        }
    }

}