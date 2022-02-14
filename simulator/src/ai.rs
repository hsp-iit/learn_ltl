use petgraph::prelude::*;
use petgraph::algo::astar;
use rand::prelude::*;

use crate::world::*;

pub trait Ai {
    fn decide(&mut self, world: &World) -> Action;
}

#[derive(Clone, Debug)]
pub struct RandomAi {
    rng: StdRng,
}

impl Default for RandomAi {
    fn default() -> Self {
        RandomAi {
            rng: StdRng::from_entropy(),
        }
    }
}

impl Ai for RandomAi {
    fn decide(&mut self, world: &World) -> Action {
        if matches!(
            world.rooms.node_weight(world.icub_location),
            Some(Room::ChargingStation)
        ) && world.icub_charge < World::MAX_CHARGE
            && self.rng.gen_bool(0.5)
        {
            Action::Recharge
        } else if let Some(destination) = world
            .rooms
            .neighbors(world.icub_location)
            .choose(&mut self.rng)
        {
            if let Some(path) = world
                .rooms
                .edges_connecting(world.icub_location, destination)
                .choose(&mut self.rng)
            {
                Action::Move(destination, path.id())
            } else {
                Action::Wait
            }
        } else {
            Action::Wait
        }
    }
}

pub struct AStarAi {
    goal: NodeIndex,
    rng: StdRng,
}

impl AStarAi {

    pub fn new(goal: NodeIndex) -> AStarAi {
        AStarAi {
            goal,
            rng: StdRng::from_entropy(),
        }
    }
}

impl Ai for AStarAi {
    fn decide(&mut self, world: &World) -> Action {
        if matches!(
            world.rooms.node_weight(world.icub_location),
            Some(Room::ChargingStation)
        ) && world.icub_charge < World::MAX_CHARGE
            && self.rng.gen_bool(0.5)
        {
            Action::Recharge
        } else if let Some((_, path)) = astar(&world.rooms, world.icub_location, |goal| goal == self.goal, |e| e.weight().running_cost, |_| 0) {
            if let Some(node) = path.get(1) {
                let edge = world
                    .rooms
                    .edges_connecting(world.icub_location, *node)
                    .choose(&mut self.rng)
                    .expect("edge connecting icub_location and first node of astar path");
                Action::Move(*node, edge.id())
            } else {
                Action::Wait
            }
        } else {
            Action::Wait
        }
    }
}
