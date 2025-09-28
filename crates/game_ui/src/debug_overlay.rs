use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use game_core::config::GlobalConfig;
use game_core::debug::debug_info::DebugSnapshot;
use game_core::debug::{overlay_visible, DebugOverlayState};
use game_core::states::AppState;
use game_core::v_ram_detection::fmt_bytes;

pub struct DebugOverlayScreen;

impl Plugin for DebugOverlayScreen {
    #[coverage(off)]
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
                        (
                            toggle_overlay,
                            ensure_overlay_exists,
                            render_debug_text.run_if(overlay_visible),
                        ).run_if(in_state(AppState::Preload))
        );
    }
}

/// Toggles the debug overlay visibility when the configured key is pressed.
/// Reads the key from `GlobalConfig::input_config` and flips `DebugOverlayState::show`.
///
/// # Parameters
/// * `keys` - Keyboard input state used to detect edge-triggered presses.
/// * `state` - Overlay state resource storing current visibility.
/// * `global_config` - Provides the bound key for toggling.
#[coverage(off)]
fn toggle_overlay(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<DebugOverlayState>,
    global_config: Res<GlobalConfig>,
) {
    let key = global_config.input_config.get_system_info_key();

    if keys.just_pressed(key) {
        state.show = !state.show;
    }
}

/// Ensures the overlay UI exists and updates its visibility.
/// Spawns a root `Node` and a `Text` child on the first run, caches their entities
/// in `DebugOverlayState`, and shows/hides, according to `state.show`.
///
/// # Parameters
/// * `state` - Overlay state containing cached entity handles and visibility flag.
/// * `commands` - Used to spawn UI entities and set up hierarchy.
/// * `q_node` - Query to update the root node's `Visibility`.
#[coverage(off)]
fn ensure_overlay_exists(
    mut state: ResMut<DebugOverlayState>,
    mut commands: Commands,
    mut q_node: Query<&mut Visibility>,
) {
    if let (Some(root), Some(_)) = (state.root, state.text) {
        if let Ok(mut vis) = q_node.get_mut(root) {
            *vis = if state.show { Visibility::Visible } else { Visibility::Hidden };
        }
        return;
    }

    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.28)),
            BorderRadius::all(Val::Px(6.0)),
            ZIndex(1000),
            Visibility::Hidden,
            Name::new("DebugOverlayRoot"),
            RenderLayers::layer(1)
        ))
        .id();

    let text = commands
        .spawn((
            Text::new(""),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Name::new("DebugText"),
            RenderLayers::layer(1)
        ))
        .id();

    commands.entity(root).add_child(text);

    if state.show {
        commands.entity(root).insert(Visibility::Visible);
    }
    state.root = Some(root);
    state.text = Some(text);
}

/// Renders diagnostic text into the overlay from the current `DebugSnapshot`.
/// Writes FPS, backend, CPU/RAM/GPU info, player position, and the toggle hint
/// into the overlay `Text` entity when the overlay is visible.
///
/// # Parameters
/// * `state` - Overlay state; skipped when hidden or uninitialized.
/// * `q_text` - Query granting mutable access to the overlay `Text`.
/// * `snap` - Snapshot of runtime metrics and labels to display.
#[coverage(off)]
fn render_debug_text(
    state: Res<DebugOverlayState>,
    mut q_text: Query<&mut Text>,
    snap: Res<DebugSnapshot>,
) {
    if !state.show { return; }
    let Some(text_e) = state.text else { return; };

    let mem_str = fmt_bytes(snap.app_mem_bytes);
    let txt = format!(
        "{app} {app_ver}  (Bevy {bevy_ver})\n\
     FPS: {:>5.1}\n\
     \n\
     Graphic: {}\n\
     Used CPU: {}\n\
     V-RAM: {}\n\
     CPU: ({:>4.1}% / {:>4.1}%)  RAM: {}  Backend: {}\n\
     \n\
     Player Location (x: {:.2}, y: {:.2})\n\
     \n\
     {}: Toggle Debug Overlay | {}: Toggle Gizmos",
        snap.fps,
        snap.backend_name,
        snap.cpu_brand,
        snap.v_ram_label,
        snap.cpu_all_percent,
        snap.app_cpu_percent,
        mem_str,
        snap.backend_str,
        snap.player_pos.x, snap.player_pos.y,
        snap.key_debug_info,
        snap.key_gizmos,
        app = snap.app_name,
        app_ver = snap.app_ver,
        bevy_ver = snap.bevy_ver,
    );

    if let Ok(mut t) = q_text.get_mut(text_e) {
        *t = Text::new(txt);
    }
}

