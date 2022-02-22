use crate::world::*;
use petgraph::prelude::*;
use std::fmt::Debug;

pub trait Task: Debug {
    fn success(&mut self, _world: &World) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct ReachRoom {
    room: Room,
    reached: bool,
}

impl ReachRoom {
    pub fn new(room: Room) -> ReachRoom {
        ReachRoom {
            room,
            reached: false,
        }
    }
}

impl Task for ReachRoom {
    fn success(&mut self, world: &World) -> bool {
        let room = world
            .rooms
            .node_weight(world.icub_location)
            .expect("icub location room");
        if *room == self.room {
            self.reached = true;
        }
        self.reached
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ReachNode {
    goal: NodeIndex,
    reached: bool,
}

impl ReachNode {
    pub fn new(goal: NodeIndex) -> ReachNode {
        ReachNode {
            goal,
            reached: false,
        }
    }
}

impl Task for ReachNode {
    fn success(&mut self, world: &World) -> bool {
        if world.icub_location == self.goal {
            self.reached = true;
        }
        self.reached
    }
}
