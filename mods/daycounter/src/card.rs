//! The counter itself: one label on the mod's HUD layer, "Day 12". By default it sits centred
//! above the hotbar at the game's text size, between the armor row and the held item's name;
//! the player can recolour, resize and move it.
//!
//! The document changes only when the look or the place does; otherwise only `#day` changes.

use crate::config::{COLORS, Settings};

/// Width of the label's box at size 1 (the text is centred in it).
const LABEL_WIDTH: f32 = 120.0;
const LABEL_HEIGHT: f32 = 10.0;
/// From the bottom edge to the label's bottom: above the hotbar, hearts and armor (which end
/// 49 GUI pixels up) and below the held item's name (from 59 up).
const FROM_BOTTOM: f32 = 50.0;
/// Ticks in a Minecraft day.
pub const DAY_TICKS: i64 = 24_000;
pub const SCREEN: &str = "daycounter.hud";

/// The label's box at the current size.
pub fn size(settings: &Settings) -> [f32; 2] {
    let s = settings.scale as f32;
    [LABEL_WIDTH * s, LABEL_HEIGHT * s]
}

/// The label's top-left corner on a `viewport`-sized screen.
pub fn top_left(settings: &Settings, viewport: [f32; 2]) -> [f32; 2] {
    let [w, h] = size(settings);
    match settings.position {
        Some([x, y]) => [((viewport[0] - w).max(0.0) * x).round(), ((viewport[1] - h).max(0.0) * y).round()],
        None => [((viewport[0] - w) / 2.0).round(), viewport[1] - FROM_BOTTOM - h],
    }
}

/// The placement that puts the label's top-left corner at `corner`, kept on screen.
pub fn place_at(settings: &Settings, viewport: [f32; 2], corner: [f32; 2]) -> [f32; 2] {
    let [w, h] = size(settings);
    let free = [(viewport[0] - w).max(1.0), (viewport[1] - h).max(1.0)];
    [(corner[0] / free[0]).clamp(0.0, 1.0), (corner[1] / free[1]).clamp(0.0, 1.0)]
}

fn num(o: &mut String, v: f32) {
    let v = if v.is_finite() { v } else { 0.0 };
    o.push_str(&format!("{}", (v * 1000.0).round() / 1000.0));
}

/// The HUD layer for the current look and place (`viewport` is needed for a moved label).
pub fn document(settings: &Settings, viewport: Option<[f32; 2]>) -> String {
    let [w, h] = size(settings);
    let [r, g, b] = COLORS[settings.color.min(COLORS.len() - 1)].1;
    let mut o = String::with_capacity(640);
    o.push_str("{\"namespace\":\"daycounter\",\"hud\":{\"type\":\"screen\",\"size\":[\"100%\",\"100%\"],");
    o.push_str("\"controls\":[{\"day\":{\"type\":\"label\",\"text\":\"#day\",\"bindings\":[{\"binding_name\":\"#day\"}],");
    o.push_str("\"localize\":false,\"shadow\":");
    o.push_str(if settings.shadow { "true" } else { "false" });
    o.push_str(",\"color\":[");
    num(&mut o, r);
    o.push(',');
    num(&mut o, g);
    o.push(',');
    num(&mut o, b);
    o.push_str(",1],\"font_scale_factor\":");
    num(&mut o, settings.scale as f32);
    o.push_str(",\"text_alignment\":\"center\",\"size\":[");
    num(&mut o, w);
    o.push(',');
    num(&mut o, h);
    match (settings.position, viewport) {
        (Some(_), Some(view)) => {
            let [x, y] = top_left(settings, view);
            o.push_str("],\"anchor_from\":\"top_left\",\"anchor_to\":\"top_left\",\"offset\":[");
            num(&mut o, x);
            o.push(',');
            num(&mut o, y);
        }
        _ => {
            o.push_str("],\"anchor_from\":\"bottom_middle\",\"anchor_to\":\"bottom_middle\",\"offset\":[0,-");
            num(&mut o, FROM_BOTTOM);
        }
    }
    o.push_str("]}}]}}");
    o
}

/// The full surface for `hud.set-surface`, with the current text.
pub fn surface(settings: &Settings, viewport: Option<[f32; 2]>, text: &str) -> String {
    let doc = document(settings, viewport);
    let mut o = String::with_capacity(doc.len() + 96);
    o.push_str("{\"screen\":\"");
    o.push_str(SCREEN);
    o.push_str("\",\"document\":\"");
    for ch in doc.chars() {
        if ch == '"' {
            o.push('\\');
        }
        o.push(ch);
    }
    o.push_str("\",\"bindings\":{\"#day\":\"");
    o.push_str(text);
    o.push_str("\"}}");
    o
}

/// A patch that changes only the text.
pub fn patch(text: &str) -> String {
    let mut o = String::with_capacity(48);
    o.push_str("{\"binding_changes\":{\"#day\":\"");
    o.push_str(text);
    o.push_str("\"}}");
    o
}

/// "Day 12" for a world clock in ticks, counting whole days as `/time query day` does; empty
/// before the server has sent a time.
pub fn day_text(time: Option<i64>) -> String {
    match time {
        Some(ticks) => format!("Day {}", ticks.max(0).div_euclid(DAY_TICKS)),
        None => String::new(),
    }
}

/// "Day 12 - 14:30": the day and the time of day (tick 0 is 6:00 in the morning).
pub fn day_and_time(time: Option<i64>) -> String {
    let mut o = day_text(time);
    if let Some(ticks) = time {
        let of_day = ticks.rem_euclid(DAY_TICKS);
        let hours = (of_day / 1000 + 6) % 24;
        let minutes = of_day % 1000 * 60 / 1000;
        o.push_str(&format!(" - {hours:02}:{minutes:02}"));
    }
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(settings: &Settings, view: Option<[f32; 2]>) -> serde_json::Value {
        let v: serde_json::Value = serde_json::from_str(&document(settings, view)).unwrap();
        v["hud"]["controls"][0]["day"].clone()
    }

    #[test]
    fn default_sits_above_the_hotbar() {
        let day = doc(&Settings::default(), None);
        assert_eq!(day["anchor_from"], "bottom_middle");
        assert_eq!(day["offset"], serde_json::json!([0, -50]));
        assert_eq!(day["font_scale_factor"], 1);
        assert_eq!(day["shadow"], true);
        let s: serde_json::Value = serde_json::from_str(&surface(&Settings::default(), None, "Day 3")).unwrap();
        assert_eq!(s["bindings"]["#day"], "Day 3");
    }

    #[test]
    fn colours_size_and_place() {
        let s = Settings { color: 2, scale: 2.0, position: Some([1.0, 0.0]), ..Settings::default() };
        let day = doc(&s, Some([400.0, 240.0]));
        assert_eq!(day["anchor_from"], "top_left");
        assert_eq!(day["offset"], serde_json::json!([160, 0]));
        assert_eq!(day["color"][1], 0.667);
        assert_eq!(day["size"], serde_json::json!([240, 20]));
        assert_eq!(place_at(&s, [400.0, 240.0], [80.0, 110.0]), [0.5, 0.5]);
    }

    #[test]
    fn texts() {
        assert_eq!(day_text(Some(24_000 * 12 + 5)), "Day 12");
        assert_eq!(day_and_time(Some(24_000 + 8_500)), "Day 1 - 14:30");
        assert_eq!(day_text(None), "");
    }
}
