//! Day Counter for Cinnabar: the current world day, above the hotbar.
//!
//! A port of justwilly's Day Counter datapack: "Day 12" above the hotbar. The layer is plain Rust, so it can be
//! unit-tested on the host with `cargo test`.
#![cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]

mod card;
mod config;
mod menu;
mod menu_controls;
mod mods_core;
mod panel;

#[cfg(target_arch = "wasm32")]
mod guest;

/// The menu with default settings, embedded in the component as its panel bundle.
pub fn menu_panel() -> String {
    panel::build(&config::Settings::default(), &panel::PanelState::default())
}

/// The bundle route of the menu's surface.
pub const MENU_ROUTE: &str = menu::ROUTE;
