//! The Day Counter menu (`tools/gen_menu.py`): show the day and the time, the colour, size and
//! shadow, moving the label with the mouse, and the menu key.

use crate::config::{COLORS, MAX_SCALE, MIN_SCALE, Settings, valid_key};
use crate::menu::{self, Value};

#[derive(Default)]
pub struct PanelState {
    pub capturing: bool,
    /// "Move with the mouse": the window hides and the label can be dragged.
    pub moving: bool,
    tab: usize,
}

/// What an event changed: save the settings, send the menu again, lay the label out again.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Effect {
    pub save: bool,
    pub republish: bool,
    pub relayout: bool,
}

impl Effect {
    pub fn merge(self, o: Effect) -> Effect {
        Effect { save: self.save || o.save, republish: self.republish || o.republish, relayout: self.relayout || o.relayout }
    }
}

const COLOR_NAMES: [&str; COLORS.len()] = {
    let mut names = [""; COLORS.len()];
    let mut i = 0;
    while i < COLORS.len() {
        names[i] = COLORS[i].0;
        i += 1;
    }
    names
};

impl PanelState {
    pub fn event(&mut self, settings: &mut Settings, id: &str, value: f32) -> Effect {
        if let Some(tab) = menu::tab_event(id) {
            self.tab = tab;
            return Effect { republish: true, ..Effect::default() };
        }
        let on = value >= 0.5;
        match id {
            "show" => settings.show = on,
            "show_time" => settings.show_time = on,
            "shadow" => settings.shadow = on,
            "color" if value.is_finite() => settings.color = (value.max(0.0) as usize).min(COLORS.len() - 1),
            "scale" if value.is_finite() => settings.scale = f64::from(value).clamp(MIN_SCALE, MAX_SCALE),
            "reset_position" => settings.position = None,
            "move" | "move_done" => {
                self.moving = id == "move";
                return Effect { republish: true, ..Effect::default() };
            }
            "menu_key" => {
                self.capturing = !self.capturing;
                return Effect { republish: true, ..Effect::default() };
            }
            _ => return Effect::default(),
        }
        Effect { save: true, republish: true, relayout: true }
    }

    /// While the menu key waits, the next key becomes it; Escape clears it.
    pub fn capture(&mut self, settings: &mut Settings, pressed: &[String]) -> Effect {
        if !self.capturing {
            return Effect::default();
        }
        for key in pressed {
            if key == "Escape" {
                self.capturing = false;
                settings.menu_key.clear();
                return Effect { save: true, republish: true, relayout: false };
            }
            if key.starts_with("Mouse") {
                continue;
            }
            if valid_key(key) {
                settings.menu_key = if key == "F10" { String::new() } else { key.clone() };
                self.capturing = false;
                return Effect { save: true, republish: true, relayout: false };
            }
        }
        Effect::default()
    }
}

fn value<'a>(s: &'a Settings, id: &str) -> Value<'a> {
    match id {
        "show" => Value::Toggle(s.show),
        "show_time" => Value::Toggle(s.show_time),
        "shadow" => Value::Toggle(s.shadow),
        "color" => Value::Choice(s.color, &COLOR_NAMES),
        "scale" => Value::Slider(s.scale, MIN_SCALE, MAX_SCALE, 0.25),
        "menu_key" => Value::Key(&s.menu_key),
        _ => Value::Button(None),
    }
}

pub fn build(settings: &Settings, state: &PanelState) -> String {
    let value = |id: &str| value(settings, id);
    menu::build(&settings.menu_key, state.capturing.then_some("menu_key"), state.tab, state.moving, &value)
}

pub fn patch(settings: &Settings, state: &PanelState) -> String {
    let value = |id: &str| value(settings, id);
    menu::patch(state.capturing.then_some("menu_key"), state.tab, state.moving, &value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_and_events() {
        let json: serde_json::Value = serde_json::from_str(&build(&Settings::default(), &PanelState::default())).unwrap();
        assert_eq!(json["toggle_key"], "F24");
        let mut s = Settings::default();
        let mut p = PanelState::default();
        assert!(p.event(&mut s, "show_time", 1.0).save && s.show_time);
        assert!(p.event(&mut s, "color", 2.0).relayout && s.color == 2);
        p.event(&mut s, "scale", 2.0);
        assert_eq!(s.scale, 2.0);
        p.event(&mut s, "move", 1.0);
        assert!(p.moving);
        let open: serde_json::Value = serde_json::from_str(&patch(&s, &p)).unwrap();
        assert_eq!(open["show_gameplay_hud"], true);
        s.position = Some([0.5, 0.5]);
        p.event(&mut s, "reset_position", 1.0);
        assert_eq!(s.position, None);
        p.event(&mut s, "menu_key", 1.0);
        p.capture(&mut s, &["KeyJ".into()]);
        assert_eq!(s.menu_key, "KeyJ");
        p.event(&mut s, "menu_key", 1.0);
        p.capture(&mut s, &["Escape".into()]);
        assert_eq!(s.menu_key, "");
    }
}
