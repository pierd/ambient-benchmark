use ambient_api::{core::player::components::is_player, prelude::*};
use packages::this::{components::x_translation, messages::Input};

#[main]
pub fn main() {
    spawn_query(is_player()).bind(|players| {
        for (id, _) in players {
            entity::add_component(id, x_translation(), 0.);
        }
    });

    Input::subscribe(|ctx, msg| {
        if let Some(player_id) = ctx.client_entity_id() {
            entity::mutate_component(player_id, x_translation(), |t| *t += msg.direction);
        }
    });
}
