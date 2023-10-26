use std::time::Instant;

use ambient_api::{
    core::{
        app::components::window_logical_size,
        camera::{
            components::*,
            concepts::{OrthographicCamera, OrthographicCameraOptional},
        },
        messages::Frame,
        player::components::user_id,
        primitives::components::cube,
        rendering::components::color,
        transform::components::{scale, translation},
    },
    element::{use_entity_component, use_query},
    prelude::*,
};
use packages::input_lag::components::{input_lag, local_lag};
use packages::{
    frame_time::components::{frame_time, local_frame_time},
    this::{
        components::{delay, x_translation},
        messages::Input,
    },
};

const X_BOUNDARY: f32 = 20.;
const Y_BOUNDARY: f32 = X_BOUNDARY;
const DELAY_MS_STEP: u32 = 5;

#[main]
pub fn main() {
    Status.el().spawn_interactive();

    let camera_id = OrthographicCamera {
        optional: OrthographicCameraOptional {
            main_scene: Some(()),
            ..default()
        },
        ..OrthographicCamera::suggested()
    }
    .spawn();

    // Update camera so we have correct aspect ratio
    change_query(window_logical_size())
        .track_change(window_logical_size())
        .bind(move |windows| {
            for (_, window) in windows {
                let window = window.as_vec2();
                if window.x <= 0. || window.y <= 0. {
                    continue;
                }

                let x_boundary = X_BOUNDARY;
                let y_boundary = Y_BOUNDARY;
                let (left, right, top, bottom) = if window.x < window.y {
                    (
                        -x_boundary,
                        x_boundary,
                        y_boundary * window.y / window.x,
                        -y_boundary * window.y / window.x,
                    )
                } else {
                    (
                        -x_boundary * window.x / window.y,
                        x_boundary * window.x / window.y,
                        y_boundary,
                        -y_boundary,
                    )
                };
                entity::set_component(camera_id, orthographic_left(), left);
                entity::set_component(camera_id, orthographic_right(), right);
                entity::set_component(camera_id, orthographic_top(), top);
                entity::set_component(camera_id, orthographic_bottom(), bottom);
            }
        });

    entity::add_component(entity::resources(), delay(), 0);

    let local = Entity::new()
        .with(cube(), ())
        .with(scale(), Vec3::ONE)
        .with(translation(), Vec3::ZERO)
        .with(color(), vec4(255., 0., 0., 1.))
        .spawn();
    let remote = Entity::new()
        .with(cube(), ())
        .with(scale(), Vec3::ONE)
        .with(translation(), vec3(0., 1., 0.))
        .with(color(), vec4(0., 0., 255., 1.))
        .spawn();

    Frame::subscribe(move |_| {
        let now = Instant::now();

        // handle movement
        let (delta, input) = input::get_delta();
        let direction = if input.keys.contains(&KeyCode::Left) || input.keys.contains(&KeyCode::A) {
            -1.
        } else if input.keys.contains(&KeyCode::Right) || input.keys.contains(&KeyCode::D) {
            1.
        } else {
            0.
        };
        Input::new(direction).send_server_unreliable();
        entity::mutate_component(local, translation(), |t| t.x += direction).unwrap();
        entity::mutate_component(remote, translation(), |t| {
            t.x = entity::get_component(player::get_local(), x_translation()).unwrap_or_default()
        })
        .unwrap();

        if delta.keys.contains(&KeyCode::Up) || delta.keys.contains(&KeyCode::W) {
            entity::mutate_component(entity::resources(), delay(), |d| *d += DELAY_MS_STEP);
        } else if delta.keys.contains(&KeyCode::Down) || delta.keys.contains(&KeyCode::S) {
            entity::mutate_component(entity::resources(), delay(), |d| {
                *d = d.saturating_sub(DELAY_MS_STEP)
            });
        }

        let delay_ms = entity::get_component(entity::resources(), delay()).unwrap_or_default();
        if delay_ms > 0 {
            let deadline = now + Duration::from_millis(delay_ms as u64);
            while Instant::now() <= deadline {
                // busy wait
            }
        }
    });
}

#[element_component]
fn Status(hooks: &mut Hooks) -> Element {
    let mut elements = Vec::new();

    let artificial_delay =
        use_entity_component(hooks, entity::resources(), delay()).unwrap_or_default();
    elements.push(Text::el(format!(
        "Aritficial frame delay: {:?}ms",
        artificial_delay
    )));

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
    elements.push(Text::el(format!("Server rame time: {:?}", time)));
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
