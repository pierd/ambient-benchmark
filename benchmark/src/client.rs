use ambient_api::{
    core::{player::components::user_id, rendering::components::color},
    element::{use_entity_component, use_query},
    prelude::*,
};
use packages::frame_time::components::{frame_time, local_frame_time};
use packages::input_lag::components::{input_lag, local_lag};

#[main]
pub fn main() {
    Status.el().spawn_interactive();
}

#[element_component]
fn Status(hooks: &mut Hooks) -> Element {
    let mut elements = Vec::new();

    let input_latency =
        use_entity_component(hooks, entity::resources(), local_lag()).unwrap_or_default();
    elements.push(Text::el(format!("Input latency: {:?}", input_latency)));

    let mut player_latencies = use_query(hooks, (user_id(), input_lag()));
    player_latencies.sort_unstable();
    let local_player_id = player::get_local();
    elements.extend(
        player_latencies
            .into_iter()
            .map(|(id, (user_id, latency))| {
                let el = Text::el(format!("{} latency: {:?}", user_id, latency));
                if id == local_player_id {
                    el.with(color(), vec4(1., 1., 1., 1.))
                } else {
                    el.with(color(), vec4(0.8, 0.8, 0.8, 1.))
                }
            }),
    );

    let time =
        use_entity_component(hooks, entity::resources(), local_frame_time()).unwrap_or_default();
    elements.push(Text::el(format!("Local frame time: {:?}", time)));
    let time = use_entity_component(hooks, entity::synchronized_resources(), frame_time())
        .unwrap_or_default();
    elements.push(Text::el(format!("Server frame time: {:?}", time)));
    let mut player_frame_times = use_query(hooks, (user_id(), frame_time()));
    player_frame_times.sort_unstable();
    elements.extend(player_frame_times.into_iter().map(|(id, (user_id, time))| {
        let el = Text::el(format!("{} frame time: {:?}", user_id, time));
        if id == local_player_id {
            el.with(color(), vec4(1., 1., 1., 1.))
        } else {
            el.with(color(), vec4(0.8, 0.8, 0.8, 1.))
        }
    }));

    FlowColumn::el(elements)
}
