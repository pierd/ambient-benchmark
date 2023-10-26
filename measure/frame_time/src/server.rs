use ambient_api::{
    core::{messages::Frame, player::components::is_player},
    prelude::*,
};
use packages::this::{
    components::{frame_time, local_frame_time, report_frequency},
    messages::Report,
};

#[main]
pub fn main() {
    start_measuring_frame_time();

    let resources = entity::resources();
    entity::add_component(resources, report_frequency(), Duration::from_secs(1));
    entity::add_component(
        entity::synchronized_resources(),
        frame_time(),
        Duration::ZERO,
    );
    run_async(async move {
        loop {
            let time = entity::get_component(resources, local_frame_time()).unwrap_or_default();
            entity::set_component(entity::synchronized_resources(), frame_time(), time);
            if let Some(duration) = entity::get_component(resources, report_frequency()) {
                sleep(duration.as_secs_f32()).await;
            } else {
                return;
            }
        }
    });

    spawn_query(is_player()).bind(|players| {
        for (id, _) in players {
            entity::add_component(id, frame_time(), Duration::ZERO);
        }
    });
    Report::subscribe(|ctx, msg| {
        if let Some(id) = ctx.client_entity_id() {
            entity::set_component(id, frame_time(), msg.frame_time);
        }
    });
}

include!("shared.rs");
