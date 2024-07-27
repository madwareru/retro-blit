use hecs::{Entity, Query, World};

pub trait System {
    type In;
    type Out;
    fn run(&self, world: &mut World, input: &Self::In) -> Self::Out;
}

pub trait SystemMut  {
    type In;
    type Out;
    fn run_mut(&self, world: &mut World, input: &mut Self::In) -> Self::Out;
}
