use bevy::{prelude::*, input::{mouse::MouseButtonInput, ElementState}, log};

use crate::{Board, events::TileTriggerEvent, resources::paused::{self, Paused}};

pub fn input_handling(
    windows: Res<Windows>,
    board: Res<Board>,
    mut button_evr: EventReader<MouseButtonInput>,
    mut tile_trigger_ewr: EventWriter<TileTriggerEvent>,
    paused: Res<Paused>,
) {
    if paused.0 == true { return; }
    let window = windows.get_primary().unwrap();

    for event in button_evr.iter() {
        if let ElementState::Pressed = event.state {
            if let Some(pos) = window.cursor_position() {
                log::trace!("Mouse button pressed: {:?} at {}", event.button, pos);
                if let Some(coordinates) = board.mouse_position(window, pos) {
                    match event.button {
                        MouseButton::Left => {
                            log::info!("Trying to uncover tile on {}", coordinates);
                            tile_trigger_ewr.send(TileTriggerEvent(coordinates));
                        },
                        MouseButton::Right => {
                            log::info!("Trying to mark tile on {}", coordinates);
                            // TODO: generate an event
                        },
                        _ => ()
                    }
                }
            }
        }
    }
}