use crate::world::*;
use learn_ltl::Time;

pub trait Monitor {
    fn get(&self, _world: &World) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InRoom(pub Room);

impl Monitor for InRoom {
    fn get(&self, world: &World) -> bool {
        let room = world.rooms.node_weight(world.icub_location);
        room == Some(&self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct BatteryLevel(pub Time);

impl Monitor for BatteryLevel {
    fn get(&self, world: &World) -> bool {
        world.icub_charge <= self.0
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DoorClosed;

impl Monitor for DoorClosed {
    fn get(&self, world: &World) -> bool {
        matches!(world.outcome, (_, Err(Failure::DoorClosed)))
    }
}
