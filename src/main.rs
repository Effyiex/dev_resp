
#![windows_subsystem = "windows"]

use std::{
    time::Duration,
    thread::sleep,
    io::Cursor
};

use rand::random;

use device_query::{
    DeviceState,
    DeviceQuery,
    Keycode
};

use rodio::{
    OutputStream,
    OutputStreamHandle,
    Decoder,
    Source
};

const TICKRATE: u64 = 10000;
const VOLUME: f32 = 0.05;

const KEY_PRESS_AUDIO: Option<&[u8]> = Some(include_bytes!("press.wav"));
const KEY_RELEASE_AUDIO: Option<&[u8]> = Some(include_bytes!("release.wav"));

const TRIGGERLESS: [&Keycode; 6] = [
    &Keycode::LShift,
    &Keycode::LControl,
    &Keycode::LAlt,
    &Keycode::RShift,
    &Keycode::RControl,
    &Keycode::RAlt
];

const TOGGLE_SEQ: [&Keycode; 3] = [
    &Keycode::LControl,
    &Keycode::LAlt,
    &Keycode::Enter
];

fn invoke_cursor_audio(
    handle: &OutputStreamHandle, 
    cursor: &Cursor<Vec<u8>>
) {
    if let Ok(audio_src) = Decoder::new(cursor.clone()) {
        if let Err(audio_error) = handle.play_raw(
            audio_src
                .speed(0.75 + random::<f32>() * 0.5)
                .amplify(VOLUME * (random::<f32>() + 0.5))
                .convert_samples::<f32>()
        ) {
            println!("ERROR: {}", audio_error);
        }
    }
}

fn handle_key_states(
    audio_handle: &OutputStreamHandle,
    latest_hold_keys: &Vec<Keycode>,
    prev_hold_keys: &Vec<Keycode>,
) {

    if let Some(audio_data) = KEY_PRESS_AUDIO {
        let audio_cursor = Cursor::new(audio_data.to_vec());
        for latest_key in latest_hold_keys {
            if !prev_hold_keys.contains(&latest_key)
            && !TRIGGERLESS.contains(&latest_key) {
                invoke_cursor_audio(&audio_handle, &audio_cursor);
            }
        }
    }

    if let Some(audio_data) = KEY_RELEASE_AUDIO {
        let audio_cursor = Cursor::new(audio_data.to_vec());
        for prev_key in prev_hold_keys {
            if !latest_hold_keys.contains(&prev_key)
            && !TRIGGERLESS.contains(&prev_key) {
                invoke_cursor_audio(&audio_handle, &audio_cursor);
            }
        }
    }

}

fn main() {

    let (_stream, audio_handle) = OutputStream::try_default().unwrap();

    let dv_state = DeviceState::new();

    let mut prev_hold_keys: Vec<Keycode> = dv_state.get_keys();
    let mut prev_toggle_keys: usize = 0;

    let mut active: bool = true;

    loop {

        let latest_hold_keys: Vec<Keycode> = dv_state.get_keys();

        let mut active_toggle_keys: usize = 0;
        for toggle_key in &TOGGLE_SEQ {
            if latest_hold_keys.contains(&toggle_key) {
                active_toggle_keys += 1;
            }
        }
        if TOGGLE_SEQ.len() <= active_toggle_keys 
        && active_toggle_keys != prev_toggle_keys {
            active = !active;
            println!("Toggled: {}", active);
        }
        prev_toggle_keys = active_toggle_keys;

        if active {
            handle_key_states(&audio_handle, &latest_hold_keys, &prev_hold_keys);
        }

        prev_hold_keys = latest_hold_keys.clone();
        sleep(Duration::from_nanos(1000000000 / TICKRATE));

    }

}
