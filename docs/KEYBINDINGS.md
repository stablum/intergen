# Intergen keybinding reference

This is the canonical reference for Intergen's keyboard controls. For a guided
introduction to the application, see the [user guide](USER_GUIDE.md). For the
geometry behind the generation controls, see the
[spawn-geometry guide](SPAWN_GEOMETRY_GUIDE.md).

## F-modes at a glance

| Key | Page or action | Repeated press | Controls inside the page |
| --- | --- | --- | --- |
| `F1` | Help overlay | text help → keyboard map → closed | Hover a key in the keyboard map for its neutral-mode action. |
| `F2` | Live controls | parameter groups → compact controls → full list → closed | Select and edit scene, generation, stage, material, lighting, camera, and shader-effect parameters. |
| `F3` | Scene presets | open ↔ closed | Load, save, free, and resolve collisions for slots `00`–`99`. |
| `F4` | Blender export | Each press starts an export. | No page; writes a timestamped `.blend` under `blend-exports/`. |
| `F5` | Recent changes | open ↔ closed | Read-only; neutral controls remain available while it is open. |
| `F12` | Screenshot | Each press captures an image. | No page; writes a screenshot under `screenshots/`. |

Only one of the `F2`, `F3`, and `F5` pages can be open at a time. Opening one
closes the other active page and hides `F1` help. Press `Esc` to close the
current page or help overlay. `F4` and `F12` perform immediate actions rather
than opening modes.

## F1: help

| Key | Action |
| --- | --- |
| `F1` | Cycle from hidden to the text reference, then the keyboard map, then hidden. |
| Hover a key | On the keyboard-map view, explain that key's neutral-mode action. |
| `Esc` | Close help. |

## F2: live controls

The first `F2` press opens the parameter-group page. A second press opens the
compact page, a third opens the complete scrolling list, and a fourth closes
the controls. Selecting a group with `Enter` opens that group's list; pressing
`F2` from a group-specific list continues to the compact page.

| Context | Key | Action |
| --- | --- | --- |
| Group page | `Up` / `Down` or mouse wheel | Select a parameter group. |
| Group page | `Enter` | Open the selected group's parameter list. |
| Compact or list page | `Up` / `Down` or mouse wheel | Select a parameter; keyboard selection supports hold-to-repeat. |
| Compact or list page | `Left` / `Right` | Move between value, LFO amplitude, LFO frequency, and LFO shape when available. |
| Compact or list page | `Tab` / `Shift + Tab` | Move forward or backward between the same fields. |
| Compact or list page | `Ctrl + Up` / `Ctrl + Down` | Increase or decrease the active field. |
| Compact or list page | `Shift` while adjusting | Use a coarser step. |
| Compact or list page | `Alt` while adjusting | Use a finer step. |
| Compact or list page | `L` | Toggle the selected parameter's LFO when supported. |
| Compact or list page | digits, `.`, `,`, `-`, `+` | Type an exact value for the active numeric field. |
| Compact or list page | `Backspace` | Erase typed numeric input. |
| Compact or list page | `Enter` | Confirm typed input; without pending input, toggle a selected shader effect. |
| Any F2 page | `Shift + Enter` | Open the reset-source chooser. |
| Reset chooser | `Up` / `Down`, then `Enter` | Choose `Cancel`, `config.toml`, or `Last loaded preset`. |
| Reset chooser | `Esc` | Cancel the reset. |
| Any F2 page | `F2` | Advance to the next page or close the full list. |
| Any F2 page | `Esc` | Close F2. |

F2 edits affect the running application but do not rewrite `config.toml`.
Resetting from a source restores all F2-controlled values and LFOs while
preserving the generated tree and recomputing its existing geometry. The last
loaded preset option is unavailable until a preset has been loaded.

`Space` and `Ctrl + Space` remain available in F2, so geometry can be spawned
while tuning. Other neutral-mode keys remain available unless F2 uses that key
for selection or numeric entry.

## F3: scene presets

The preset page has 10 banks with 10 slots each.

| Key | Action |
| --- | --- |
| `00`–`99` | Load the preset assigned to the bank and slot. |
| `O`, then `00`–`99` | Load only the complete 3D object structure; keep all current parameters and post effects. |
| `E`, then `00`–`99` | Load only the 2D post-scene effect values and effect LFO settings. |
| `P`, then `00`–`99` | Load scene parameters and scene-parameter LFO settings; keep the current object structure and 2D post effects. |
| `S`, then `00`–`99` | Save the current scene and assign it to that slot. |
| `S`, then `O`, then `00`–`99` | Save only the current 3D object structure into that preset; preserve its parameters and post effects. |
| `S`, then `E`, then `00`–`99` | Save only the current 2D post-scene effect values and effect LFO settings into that preset. |
| `S`, then `P`, then `00`–`99` | Save only current scene parameters and scene-parameter LFO settings; preserve that preset's object structure and 2D post effects. |
| `Delete`, then `00`–`99` | Free that slot in every preset file that claims it. |
| `Up` / `Down` | In a slot-collision chooser, select which file keeps the slot. |
| `Enter` | Confirm the selected collision resolution. |
| `F3` or `Esc` | Close the preset page. |

The preset page captures digits, `O`, `E`, `P`, `S`, and `Delete`; when the collision chooser
is visible it also captures `Up`, `Down`, and `Enter`. Other neutral controls
remain available.

A component-only save updates the preset already assigned to the slot. When the
slot is empty, it creates a complete backing preset from the current scene so
that every later full or component-only load remains valid.

## F4: Blender export

Press `F4` during a normal interactive run to export the current scene as a
timestamped `.blend` under `blend-exports/`. This is an action, not a persistent
page. See [Blender export](USER_GUIDE.md#blender-export) for contents and
limitations.

## F5: recent changes

Press `F5` to show or hide recent explicit parameter changes. The most recent
change stays visible and other changes from the last few seconds appear in
alphabetical order. Held controls are coalesced and continuously sampled LFO
values are excluded, so the list does not flood.

F5 is read-only and does not capture neutral bindings: camera, generation, and
other regular controls continue to work while it is open.

## F12: screenshots

Press `F12` during a normal interactive run to save a timestamped image under
`screenshots/`. This is an action, not a persistent page.

## Neutral-mode controls

These are the normal scene controls. Some are temporarily captured when an
F-page assigns them another meaning, as described above.

### Camera

| Key | Action |
| --- | --- |
| `Arrow Up` / `Arrow Down` | Pitch the camera. |
| `Arrow Left` / `Arrow Right` | Yaw the camera. |
| `Q` / `E` | Roll the camera. |
| `W` / `S` | Zoom in or out. |
| `Backspace` | Stop camera rotation momentum. |

### Generation

| Key | Action |
| --- | --- |
| `Space` | Spawn using the current add mode; hold to repeat. |
| `Ctrl + Space` | Cycle between single-object and fill-current-level add modes. |
| `G` | Cycle placement through vertex, edge, and face attachments. |
| `R` | Reset the scene with the selected shape as the new root. |
| `1` / `2` / `3` / `4` | Select cube, tetrahedron, octahedron, or dodecahedron. |
| `-` / `+` | Decrease or increase child scale; hold to repeat. |
| `[` / `]` | Decrease or increase child twist; hold to repeat. |
| `T` | Reset child twist to its configured default. |
| `Z` / `X` | Decrease or increase child outward offset; hold to repeat. |
| `C` | Reset child outward offset to its configured default. |
| `V` / `B` | Decrease or increase spawn-exclusion probability; hold to repeat. |
| `N` | Reset spawn-exclusion probability to its configured default. |
| `,` / `.` | Decrease or increase the single-spawn source repeat count; hold to repeat. |

### Materials

| Key | Action |
| --- | --- |
| `O` / `P` | Decrease or increase global object opacity in 1% steps; hold to repeat. |
| `I` | Reset global opacity to its configured default. |
