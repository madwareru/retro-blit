use crate::{App, Player, Position};

pub struct Blackboard {
    /// shared data on a placement of player updated each frame which can then be observed by AI agents
    pub player_position: Position
}

pub enum FightPhase {
    /// Each hit does not take a place immediately, first a cool down phase should last.
    /// When **time_left** reaches a zero, fight phase switches to a hip.
    CoolDown { time_left: f32 },
    /// A phase of jump towards a player.
    /// When **end_position** reached, a distance toward player compared to **hit_distance**.
    /// If lower than or equal, the hit takes a place and player gets a damage.
    /// Whenever the player is hit or not, fight phase switches to a hop.
    Hip { start_position: Position, end_position: Position, t: f32 },
    /// A phase of jump back after the hip.
    /// When **end_position** reached, fight phase switches to a cool down.
    Hop { start_position: Position, end_position: Position, t: f32 }
}

pub enum MobState {
    /// Just wander to a random point on a map
    /// * **player spotted near** -> go to Angry state
    /// * **player spotted far** -> go to Anxious state
    /// * **point reached** -> update point and start over
    Wandering { destination: Position },
    /// Recently player spotted at far
    /// * **uncertainty reached 0** -> go to Angry state
    /// * **player out of sight** -> go to Wandering state
    Anxious { uncertainty: f32 },
    /// Player is near, a scent of blood is sweet
    /// * **player out of sight** -> go to Wandering state
    /// * **distance to a player is lower than fighting_range** -> go to Fight state
    Angry,
    /// Player is near enough to be hit
    /// * **player out of lost_fight_range** -> go to Angry state
    /// * **else** -> handle FightPhase
    Fight(FightPhase)
}

impl App {
    pub(crate) fn update_blackboard(&mut self) {
        if let Some((_, (_, position))) = self.world.query::<(&Player, &Position)>().iter().next() {
            self.blackboard.player_position = *position;
        }
    }
    pub(crate) fn update_ai(&self, dt: f32) {

    }
}