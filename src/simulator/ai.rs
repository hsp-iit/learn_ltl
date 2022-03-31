use petgraph::algo::astar;
use petgraph::prelude::*;
use rand::prelude::*;
use std::fmt::Debug;

use crate::world::*;

pub trait Ai: Debug {
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
        // && self.rng.gen_bool(0.5)
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

#[derive(Clone, Debug)]
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

    fn pathing(&mut self, world: &World, to: NodeIndex) -> Option<(NodeIndex, EdgeIndex)> {
        if let Some((_, path)) = astar(
            &world.rooms,
            world.icub_location,
            |goal| goal == to,
            |e| e.weight().running_cost,
            |_| 0,
        ) {
            if let Some(node) = path.get(1) {
                world
                    .rooms
                    .edges_connecting(world.icub_location, *node)
                    .choose(&mut self.rng)
                    // .expect("edge connecting icub_location and first node of astar path")
                    .map(|edge| (*node, edge.id()))
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Ai for AStarAi {
    fn decide(&mut self, world: &World) -> Action {
        if matches!(
            world.rooms.node_weight(world.icub_location),
            Some(Room::ChargingStation)
        ) && world.icub_charge < World::MAX_CHARGE
        {
            Action::Recharge
        } else if world.icub_charge * 10 < World::MAX_CHARGE * 7 {
            if let Some(charging_station) = world.rooms.node_indices().find(|idx| world.rooms[*idx] == Room::ChargingStation) {
                if let Some((node, edge)) = self.pathing(world, charging_station) {
                    Action::Move(node, edge)
                } else {
                    Action::Wait
                }
            } else  if let Some((node, edge)) = self.pathing(world, self.goal) {
                Action::Move(node, edge)
            } else {
                Action::Wait
            }
        } else if let Some((node, edge)) = self.pathing(world, self.goal) {
            Action::Move(node, edge)
        } else {
            Action::Wait
        }
    }
}
