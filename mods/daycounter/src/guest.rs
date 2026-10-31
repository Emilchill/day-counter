//! Publishing the counter (the layer when its look or place changes, otherwise only the text),
//! the menu, dragging the label, and Mods Core.

use crate::config::Settings;
use crate::panel::{Effect, PanelState};
use mod_api::bindings::{
    Guest,
    cinnabar::extension::{hud, input, panel, player_state, settings},
};
use std::cell::RefCell;

struct Mod;

#[derive(Default)]
struct State {
    settings: Settings,
    panel: PanelState,
    /// The text on screen, once the layer is up.
    shown: Option<String>,
    /// The layer must be published again (look or place changed).
    relayout: bool,
    /// The screen size a moved label was laid out for.
    viewport: Option<[f32; 2]>,
    /// While dragging: where in the label the mouse took hold.
    grab: Option<[f32; 2]>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
    /// This mod's place in Mods Core's list.
    static CORE: RefCell<crate::mods_core::Link> = RefCell::new(crate::mods_core::Link::new(
        "day-counter",
        "Day Counter",
        env!("CARGO_PKG_VERSION"),
        "Shows the world day, and the time if you want. Change its colour and size, and move it.",
    ));
}

impl Guest for Mod {
    fn init() {
        STATE.with_borrow_mut(|state| {
            state.settings = settings::load().map(|text| Settings::from_json(&text)).unwrap_or_default();
            state.relayout = true;
        });
    }

    fn frame() {
        STATE.with_borrow_mut(State::frame);
    }
}

impl State {
    fn apply(&mut self, effect: Effect) {
        if effect.save {
            let _ = settings::save(&self.settings.to_json());
        }
        self.relayout |= effect.relayout;
        // No panel while the menu is closed (Mods Core): it is set whole when it opens.
        if effect.republish {
            let _ = panel::update_content(&crate::panel::patch(&self.settings, &self.panel));
        }
    }

    fn frame(&mut self) {
        self.controls();
        // A moved label follows the screen size.
        if self.settings.position.is_some() {
            let view = hud::read_viewport().map(|v| [v.width, v.height]);
            if view != self.viewport {
                self.viewport = view;
                self.relayout = true;
            }
        }
        let time = player_state::read_world_time().ok().flatten().map(|t| t.time);
        let text = if !self.settings.show {
            String::new()
        } else if self.settings.show_time {
            crate::card::day_and_time(time)
        } else {
            crate::card::day_text(time)
        };
        if self.relayout || self.shown.is_none() {
            if hud::set_surface(&crate::card::surface(&self.settings, self.viewport, &text)).is_ok() {
                self.shown = Some(text);
                self.relayout = false;
            }
            return;
        }
        if self.shown.as_deref() != Some(text.as_str()) {
            if hud::update_surface(&crate::card::patch(&text)).is_ok() {
                self.shown = Some(text);
            } else {
                self.shown = None;
            }
        }
    }

    fn controls(&mut self) {
        let capturing = self.panel.capturing;
        let own = crate::menu::has_own_key(&self.settings.menu_key);
        let mut keys = vec!["MouseLeft".to_owned()];
        if own {
            keys.push(self.settings.menu_key.clone());
        }
        let Ok(controls) = input::read_selected_controls(&input::Selection {
            keys_pressed: if capturing { None } else { Some(keys) },
            keys_held: Some(vec!["MouseLeft".to_owned()]),
            events: true,
            keys_admitted: Some(Vec::new()),
        }) else {
            return;
        };
        if !capturing && own && controls.focused && controls.keys_pressed.iter().any(|k| *k == self.settings.menu_key) {
            CORE.with_borrow_mut(|link| link.toggle(controls.panel_open));
        }
        // Mods Core: a click on this mod in its list sets and opens the menu.
        let whole = || crate::panel::build(&self.settings, &self.panel);
        let event = CORE.with_borrow_mut(|link| link.frame(controls.panel_open, whole));
        if event == crate::mods_core::Event::Closed {
            // Closing the menu ends a move.
            self.panel.moving = false;
            self.grab = None;
        }
        let mut effect = Effect::default();
        effect.republish |= crate::menu::kofi_tick(controls.seconds);
        for event in &controls.events {
            if crate::menu::is_kofi(&event.id) {
                let _ = hud::local_message(crate::menu::KOFI_MESSAGE);
                crate::menu::show_kofi_note();
                effect.republish = true;
                continue;
            }
            effect = effect.merge(self.panel.event(&mut self.settings, &event.id, event.value));
        }
        effect = effect.merge(self.panel.capture(&mut self.settings, &controls.keys_pressed));
        self.apply(effect);
        if self.panel.moving && controls.panel_open {
            self.drag(controls.pointer, &controls.keys_pressed, &controls.keys_held);
        }
    }

    /// Drags the label with the left mouse button while the menu is in its move mode.
    fn drag(&mut self, pointer: Option<(f32, f32)>, pressed: &[String], held: &[String]) {
        let (Some((px, py)), Some(view)) = (pointer, hud::read_viewport()) else {
            return;
        };
        let view = [view.width, view.height];
        let corner = crate::card::top_left(&self.settings, view);
        let [w, h] = crate::card::size(&self.settings);
        if pressed.iter().any(|k| k == "MouseLeft") {
            let inside = px >= corner[0] && px <= corner[0] + w && py >= corner[1] && py <= corner[1] + h;
            self.grab = inside.then_some([px - corner[0], py - corner[1]]);
        }
        let Some(grab) = self.grab else { return };
        if !held.iter().any(|k| k == "MouseLeft") {
            // Let go: keep the new place.
            self.grab = None;
            let _ = settings::save(&self.settings.to_json());
            return;
        }
        let target = [(px - grab[0]).round(), (py - grab[1]).round()];
        if target != corner {
            self.settings.position = Some(crate::card::place_at(&self.settings, view, target));
            self.viewport = Some(view);
            self.relayout = true;
        }
    }
}

mod_api::bindings::export!(Mod with_types_in mod_api::bindings);
