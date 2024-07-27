use core::panic;
use std::marker::PhantomData;

use crate::{systems_base::SystemMut, Player, Position};

pub struct UpdateBlackboard;

impl SystemMut for UpdateBlackboard
{
    type In = crate::ai::Blackboard;
    type Out = ();

    fn run_mut(
        &self,
        world: &mut hecs::World,
        blackboard: &mut Self::In
    ) -> Self::Out {
        match world.query::<(&Player, &Position)>().iter().next() {
            Some((_, (_, position))) => blackboard.player_position = *position,
            None => panic!("player not found!"),
        }
    }
}
