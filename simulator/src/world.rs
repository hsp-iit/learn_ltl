use learn_pltl_fast::{Time, Trace};
use petgraph::prelude::*;
use rand::prelude::*;

use crate::ai::*;
use crate::monitor::*;
use crate::task::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Room {
    Kitchen,
    Lab,
    ChargingStation,
}

pub struct Path {
    pub running_cost: Time,
    pub locked: bool,
}

pub enum Action {
    Move(NodeIndex, EdgeIndex),
    Recharge,
    Wait,
}

pub enum Failure {
    DoorClosed,
    InsufficientCharge,
}

pub struct World {
    pub rooms: UnGraph<Room, Path>,
    pub time: Time,
    pub icub_location: NodeIndex,
    pub icub_charge: Time,
    pub outcome: (Action, Result<(), Failure>),
}

impl World {
    pub const MAX_CHARGE: Time = 10;

    pub fn new(rooms: UnGraph<Room, Path>, icub_location: NodeIndex) -> World {
        World {
            rooms,
            time: 0,
            icub_location,
            icub_charge: World::MAX_CHARGE,
            outcome: (Action::Wait, Ok(())),
        }
    }

    pub fn running(&self) -> bool {
        self.time < 255
    }

    pub fn run<const N: usize>(
        &mut self,
        ai: &mut dyn Ai,
        task: &mut dyn Task,
        monitors: &[Box<dyn Monitor>; N],
    ) -> (Trace<N>, bool) {
        let mut records = [false; N];
        for (val, monitorable) in (records.iter_mut()).zip(monitors.iter()) {
            *val = monitorable.get(self);
        }
        let mut trace = vec![records];
        while self.running() {
            let action = ai.decide(self);
            let result = self.execute(&action);
            self.outcome = (action, result);
            let mut records = [false; N];
            for (val, monitorable) in (records.iter_mut()).zip(monitors.iter()) {
                *val = monitorable.get(self);
            }
            trace.push(records);
            if task.success(self) {
                return (trace, true);
            }
        }
        (trace, false)
    }

    pub fn execute(&mut self, action: &Action) -> Result<(), Failure> {
        self.time += 1;
        match *action {
            Action::Wait => self.icub_charge = self.icub_charge.saturating_sub(1),
            Action::Move(destination, path) => {
                if self.rooms.edge_endpoints(path) == Some((self.icub_location, destination))
                    || self.rooms.edge_endpoints(path) == Some((destination, self.icub_location))
                {
                    let path = self.rooms.edge_weight(path).expect("edge weight");
                    if path.locked {
                        self.icub_charge = self.icub_charge.saturating_sub(1);
                        return Err(Failure::DoorClosed);
                    } else if self.icub_charge >= path.running_cost {
                        self.icub_charge = self.icub_charge.saturating_sub(path.running_cost);
                        self.icub_location = destination;
                    } else {
                        self.icub_charge = self.icub_charge.saturating_sub(path.running_cost);
                        return Err(Failure::InsufficientCharge);
                    }
                }
            }
            Action::Recharge => {
                if matches!(
                    self.rooms.node_weight(self.icub_location),
                    Some(Room::ChargingStation)
                ) {
                    self.icub_charge = Self::MAX_CHARGE;
                }
            }
        }
        Ok(())
    }

    pub fn lab_scenario() -> Self {
        let mut rooms = Graph::new_undirected();
        let lab_1 = rooms.add_node(Room::Lab);
        let lab_2 = rooms.add_node(Room::Lab);
        let kitchen = rooms.add_node(Room::Kitchen);
        rooms.add_edge(lab_1, lab_2, Path { running_cost: 3, locked: false });
        rooms.add_edge(lab_2, kitchen, Path { running_cost: 3, locked: false });
        rooms.add_edge(lab_1, kitchen, Path { running_cost: 3, locked: false });
        World {
            rooms,
            time: 0,
            icub_location: lab_1,
            icub_charge: 4,
            outcome: (Action::Wait, Ok(())),
        }
    }

    pub fn recharging_scenario() -> Self {
        let mut rooms = Graph::new_undirected();
        let lab = rooms.add_node(Room::Lab);
        let charging = rooms.add_node(Room::ChargingStation);
        let kitchen = rooms.add_node(Room::Kitchen);
        rooms.add_edge(lab, charging, Path { running_cost: 2, locked: false });
        rooms.add_edge(charging, kitchen, Path { running_cost: 5, locked: false });
        rooms.add_edge(lab, kitchen, Path { running_cost: 5, locked: false });
        World {
            rooms,
            time: 0,
            icub_location: lab,
            icub_charge: 3,
            outcome: (Action::Wait, Ok(())),
        }
    }

    pub fn proc_gen_recharging_scenario() -> Self {
        let mut rng = StdRng::seed_from_u64(rand::thread_rng().gen());
        let mut rooms = Graph::new_undirected();

        let start = rooms.add_node(Room::Lab);

        for _ in 0..10 {
            let room_type = if rng.gen_bool(0.333) {
                Room::ChargingStation
            } else {
                Room::Lab
            };
            let room = rooms.add_node(room_type);
            // let charging = rooms.add_node(Room::ChargingStation);
            // let kitchen = rooms.add_node(Room::Kitchen);
            let other_node = rooms
                .node_indices()
                .choose(&mut rng)
                .expect("choose a random node");
            rooms.add_edge(room, other_node, Path { running_cost: 2, locked: false });
        }

        let kitchen = rooms.add_node(Room::Kitchen);
        let other_node = rooms
            .node_indices()
            .choose(&mut rng)
            .expect("choose a random node");
        rooms.add_edge(kitchen, other_node, Path { running_cost: 3, locked: false });

        World {
            rooms,
            time: 0,
            icub_location: start,
            icub_charge: World::MAX_CHARGE,
            outcome: (Action::Wait, Ok(())),
        }
    }

    pub fn door_scenario() -> (Self, Box<dyn Task>) {
        const ROOM_TYPES: [Room; 3] = [Room::Kitchen, Room::ChargingStation, Room::Lab];
        let mut rng = StdRng::seed_from_u64(rand::thread_rng().gen());

        let mut rooms = Graph::new_undirected();
        let room_1 = rooms.add_node(*ROOM_TYPES.choose(&mut rng).expect("choose room type"));
        let room_2 = rooms.add_node(*ROOM_TYPES.choose(&mut rng).expect("choose room type"));
        rooms.add_edge(room_1, room_2, Path { running_cost: 1, locked: rng.gen_bool(0.5) });
        let world = World {
            rooms,
            time: 0,
            icub_location: room_1,
            icub_charge: 5,
            outcome: (Action::Wait, Ok(())),
        };
        let task = ReachNode::new(room_2);
        (world, Box::new(task))
    }

    pub fn proc_gen_scenario() -> (Self, Box<dyn Task>, Box<dyn Ai>) {
        const ROOM_TYPES: [Room; 3] = [Room::Kitchen, Room::ChargingStation, Room::Lab];
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

        let task = ReachNode::new(goal_room);

        let world = World::new(rooms, start);

        // let ai = RandomAi::default();
        let ai = AStarAi::new(goal_room);

        (world, Box::new(task), Box::new(ai))
    }

}
