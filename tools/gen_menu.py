#!/usr/bin/env python3
"""Generates the Day Counter menu (src/menu.json, src/menu.escaped, src/menu_controls.rs)."""
import pathlib, sys
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from menukit import Menu

m = Menu("daycounter", "Day Counter", height=240, move_hint="Drag the day with the mouse.")
m.tab("Settings",
      [("head", "Counter"), ("toggle", "show", "Show the day"), ("toggle", "show_time", "Show the time"),
       ("choice", "color", "Color"), ("slider", "scale", "Size"), ("toggle", "shadow", "Shadow")],
      [("head", "Position"), ("button", "move", "Move with the mouse", "accent"),
       ("button", "reset_position", "Back above the hotbar"),
       ("head", "Menu"), ("key", "menu_key", "Menu key"),
       ("note", "Empty: open it from Mods Core (F10)."), ("note", "Escape clears the key.")])
m.generate(pathlib.Path(__file__).resolve().parents[1] / "mods/daycounter/src")
