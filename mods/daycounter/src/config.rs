//! Day Counter settings, saved as a small JSON object.

/// Text colours, Minecraft's own.
/// At most 8: Cinnabar menus allow 8 options in a dropdown.
pub const COLORS: [(&str, [f32; 3]); 8] = [
    ("White", [1.0, 1.0, 1.0]),
    ("Yellow", [1.0, 1.0, 0.333]),
    ("Gold", [1.0, 0.667, 0.0]),
    ("Aqua", [0.333, 1.0, 1.0]),
    ("Green", [0.333, 1.0, 0.333]),
    ("Light Purple", [1.0, 0.333, 1.0]),
    ("Red", [1.0, 0.333, 0.333]),
    ("Gray", [0.667, 0.667, 0.667]),
];
pub const MIN_SCALE: f64 = 0.5;
pub const MAX_SCALE: f64 = 3.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Settings {
    /// Show "Day N".
    pub show: bool,
    /// Add the time of day ("Day 12 - 14:30").
    pub show_time: bool,
    /// Index into `COLORS`.
    pub color: usize,
    /// Text size, 1 is the game's own.
    pub scale: f64,
    pub shadow: bool,
    /// Where the label's top-left corner sits, as fractions of the free screen; `None` is
    /// centred above the hotbar.
    pub position: Option<[f32; 2]>,
    /// The mod's own menu key; empty means none (Mods Core opens the menu).
    pub menu_key: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self { show: true, show_time: false, color: 0, scale: 1.0, shadow: true, position: None, menu_key: String::new() }
    }
}

/// Physical key names are short ASCII alphanumerics such as `KeyC` or `F6`.
pub fn valid_key(key: &str) -> bool {
    !key.is_empty() && key.len() <= 32 && key.bytes().all(|b| b.is_ascii_alphanumeric())
}

/// The value after `"name":` in a flat JSON object.
fn field<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("\"{name}\":");
    text.find(&key).map(|at| text[at + key.len()..].trim_start())
}

fn number(rest: &str) -> Option<f64> {
    let end = rest.find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == 'e' || c == 'E')).unwrap_or(rest.len());
    rest[..end].parse().ok().filter(|v: &f64| v.is_finite())
}

impl Settings {
    pub fn from_json(text: &str) -> Self {
        let d = Self::default();
        let flag = |name: &str, default: bool| match field(text, name) {
            Some(rest) if rest.starts_with("true") => true,
            Some(rest) if rest.starts_with("false") => false,
            _ => default,
        };
        let menu_key = field(text, "menu_key")
            .and_then(|rest| rest.strip_prefix('"'))
            .and_then(|rest| rest.split('"').next())
            .filter(|key| valid_key(key) && *key != "F10")
            .unwrap_or("")
            .to_owned();
        let position = field(text, "position").and_then(|rest| rest.strip_prefix('[')).and_then(|rest| {
            let mut parts = rest.split(|c| c == ',' || c == ']');
            let x = number(parts.next()?.trim())?;
            let y = number(parts.next()?.trim())?;
            Some([x.clamp(0.0, 1.0) as f32, y.clamp(0.0, 1.0) as f32])
        });
        Self {
            show: flag("show", d.show),
            show_time: flag("show_time", d.show_time),
            color: field(text, "color").and_then(number).map_or(d.color, |v| (v.max(0.0) as usize).min(COLORS.len() - 1)),
            scale: field(text, "scale").and_then(number).map_or(d.scale, |v| v.clamp(MIN_SCALE, MAX_SCALE)),
            shadow: flag("shadow", d.shadow),
            position,
            menu_key,
        }
    }

    pub fn to_json(&self) -> String {
        let position = match self.position {
            Some([x, y]) => format!("[{x},{y}]"),
            None => "null".into(),
        };
        format!(
            "{{\"show\":{},\"show_time\":{},\"color\":{},\"scale\":{},\"shadow\":{},\"position\":{},\"menu_key\":\"{}\"}}",
            self.show, self.show_time, self.color, self.scale, self.shadow, position, self.menu_key
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_repairs() {
        let s = Settings { show: false, show_time: true, color: 3, scale: 2.5, shadow: false, position: Some([0.25, 0.75]), menu_key: "KeyJ".into() };
        assert_eq!(Settings::from_json(&s.to_json()), s);
        assert_eq!(Settings::from_json("junk"), Settings::default());
        assert_eq!(Settings::from_json(&Settings::default().to_json()), Settings::default());
        let bad = Settings::from_json(r#"{"menu_key":"F 1!","color":99,"scale":40,"position":[3,-1]}"#);
        assert_eq!((bad.menu_key.as_str(), bad.color, bad.scale, bad.position), ("", COLORS.len() - 1, MAX_SCALE, Some([1.0, 0.0])));
    }
}
