use ctru::prelude::{Hid, KeyPad};
use egui::{Event, Modifiers, Pos2};

pub fn handle_input(hid: &Hid, last_pos: &mut Pos2) -> (Vec<Event>, bool) {
    let mut events = vec![];
    let down = hid.keys_down();
    let held = hid.keys_held();
    let up = hid.keys_up();
    let pos = u16pairtopos2(hid.touch_position());
    if held.contains(KeyPad::TOUCH) {
        events.push(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        });
    }
    if down.contains(KeyPad::TOUCH) {
        events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(0),
            phase: egui::TouchPhase::Start,
            pos,
            force: None,
        });
    } else if held.contains(KeyPad::TOUCH) {
        events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(0),
            phase: egui::TouchPhase::Move,
            pos,
            force: None,
        });
    }
    if up.contains(KeyPad::TOUCH) {
        events.push(egui::Event::PointerButton {
            pos: *last_pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        });
        events.push(egui::Event::Touch {
            device_id: egui::TouchDeviceId(0),
            id: egui::TouchId(0),
            phase: egui::TouchPhase::End,
            pos: *last_pos,
            force: None,
        });
    }

    let key_mapping = [
        (KeyPad::DOWN, egui::Key::ArrowDown),
        (KeyPad::UP, egui::Key::ArrowUp),
        (KeyPad::RIGHT, egui::Key::ArrowRight),
        (KeyPad::LEFT, egui::Key::ArrowLeft),
    ];

    for (ds_key, egui_key) in key_mapping {
        if down.contains(ds_key) {
            events.push(egui::Event::Key {
                key: egui_key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: Modifiers::default(),
            });
        }
        if up.contains(ds_key) {
            events.push(egui::Event::Key {
                key: egui_key,
                physical_key: None,
                pressed: false,
                repeat: false,
                modifiers: Modifiers::default(),
            });
        }
    }

    *last_pos = pos;
    (events, down.contains(KeyPad::START))
}

fn u16pairtopos2((x, y): (u16, u16)) -> egui::Pos2 {
    egui::Pos2::new(x as f32, y as f32)
}
