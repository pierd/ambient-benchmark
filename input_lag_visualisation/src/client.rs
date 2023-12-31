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
use packages::{
    frame_time::components::{frame_time, local_frame_time},
    this::{
        components::{delay, x_translation},
        messages::{Input, Reset},
    },
};
use packages::{
    input_lag::components::{input_lag, local_lag},
    this::components::visibility,
};

const X_BOUNDARY: f32 = 20.;
const Y_BOUNDARY: f32 = X_BOUNDARY;
const DELAY_MS_STEP: u32 = 5;
const LOCAL_COLOR: Vec4 = vec4(255., 0., 0., 1.);
const REMOTE_COLOR: Vec4 = vec4(0., 0., 255., 1.);
const BLACK_COLOR: Vec4 = vec4(0., 0., 0., 1.);

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
    entity::add_component(entity::resources(), visibility(), 0);

    let local = Entity::new()
        .with(cube(), ())
        .with(scale(), Vec3::ONE)
        .with(translation(), Vec3::ZERO)
        .with(color(), LOCAL_COLOR)
        .spawn();
    let remote = Entity::new()
        .with(cube(), ())
        .with(scale(), Vec3::ONE)
        .with(translation(), vec3(0., 1., 0.))
        .with(color(), REMOTE_COLOR)
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

        // handle reset
        if delta.keys.contains(&KeyCode::R) {
            Reset::new().send_server_reliable();
            entity::mutate_component(local, translation(), |t| t.x = 0.).unwrap();
        }

        // handle toggling
        if delta.keys.contains(&KeyCode::Space) {
            entity::mutate_component(entity::resources(), visibility(), |v| {
                *v = v.wrapping_add(1)
            })
            .unwrap();
            let visibility = entity::get_component(entity::resources(), visibility()).unwrap();
            entity::mutate_component(local, color(), |c| {
                *c = if visibility & 0b01 == 0b01 {
                    BLACK_COLOR
                } else {
                    LOCAL_COLOR
                }
            });
            entity::mutate_component(remote, color(), |c| {
                *c = if visibility & 0b10 == 0b10 {
                    BLACK_COLOR
                } else {
                    REMOTE_COLOR
                }
            });
        }

        // handle delay
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

    elements.push(Text::el(
        r#"The top square is synced with the server and as such it's affected by the input lag.
The bottom square is controlled directly on the client.
The delay is added locally to the client and simulates longer frame processing time.

If the squares stop being aligned then some messages were lost.

Controls:
AD/left-right = move
WS/up-down = add or remove delay
R = reset square positions
space = toggle square visibility

Stats:"#,
    ));

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
