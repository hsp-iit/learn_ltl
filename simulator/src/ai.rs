use petgraph::prelude::*;
use rand::prelude::*;

use crate::world::*;

#[derive(Clone, Debug)]
pub struct Ai {
    rng: StdRng,
}

impl Default for Ai {
    fn default() -> Self {
        Ai {
            rng: StdRng::seed_from_u64(rand::thread_rng().gen()),
        }
    }
}

impl Ai {
    pub fn decide(&mut self, world: &World) -> Action {
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
