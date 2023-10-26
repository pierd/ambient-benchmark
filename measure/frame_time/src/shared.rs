pub fn start_measuring_frame_time() {
    use std::time::SystemTime;
    use packages::this::components::*;

    let resources = entity::resources();
    entity::add_component(resources, local_frame_time(), Duration::ZERO);

    Frame::subscribe(move |_| {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
        let Some(last_start) = entity::get_component(resources, last_frame_start()) else {
            // first frame
            entity::add_component(resources, last_frame_start(), now);
            return;
        };
        entity::set_component(resources, last_frame_start(), now);

        let time = now.saturating_sub(last_start);
        let factor = entity::get_component(resources, smoothing_factor()).unwrap_or(16);
        entity::mutate_component(resources, local_frame_time(), |old_frame_time| {
            *old_frame_time = ((factor - 1) * *old_frame_time + time) / factor;
        });
    });
}
