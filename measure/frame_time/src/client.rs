use ambient_api::{core::messages::Frame, prelude::*};
use packages::this::{
    components::{local_frame_time, report_frequency},
    messages::Report,
};

#[main]
pub fn main() {
    start_measuring_frame_time();

    let resources = entity::resources();
    entity::add_component(resources, report_frequency(), Duration::from_secs(1));
    run_async(async move {
        loop {
            let frame_time =
                entity::get_component(resources, local_frame_time()).unwrap_or_default();
            Report { frame_time }.send_server_unreliable();
            if let Some(duration) = entity::get_component(resources, report_frequency()) {
                sleep(duration.as_secs_f32()).await;
            } else {
                return;
            }
        }
    });
}

include!("shared.rs");
