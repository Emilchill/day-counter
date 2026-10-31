# Day Counter for Cinnabar

**⬇️ Download:** [`day-counter.wasm`](https://github.com/Emilchill/day-counter/raw/main/download/day-counter.wasm) (Cinnabar v0.1.31 or newer). Import it in **Settings → Mods → Import mod...**

Counts the days above your hotbar. A port of
[Day Counter](https://modrinth.com/datapack/daycounter) by justwilly, whose datapack reads
the world clock each tick and writes `DAY n` to the action bar.

Here the mod reads the world clock and draws `Day 12` on its own HUD layer, centred above
the hotbar at the game's normal text size, below the held item's name so it never covers it. The layer is published once; afterwards only the text changes, when the day does,
so an ordinary frame costs almost nothing. Nothing shows before the server sends the time.

Needs a Cinnabar build with the new mod API (October 25, 2026 or later).

## Building

```
python scripts/build-mod.py
```

That writes `dist/daycounter.wasm`. Import it in Cinnabar from Settings → Mods → Import.

## Permissions

HUD, to draw the text, and player status, to read the world clock. The clock is fenced to
the current session, so it never shows a stale number from a world you have left.

## License

CC0-1.0, as the original is.


## Menu

Open it from **Mods Core** (F10, then click Day Counter), or set your own key in it (empty by default). It can hide the counter and add the time of day ("Day 12 - 14:30").
