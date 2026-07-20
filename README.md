# Intergen

Intergen is an interactive 3D shape-generation playground built with Rust and
Bevy. It grows recursive structures from cubes, tetrahedra, octahedra, and
dodecahedra, attaching each new shape to a parent vertex, edge, or face.

The project combines fast keyboard-driven modeling with live scene, material,
lighting, camera, and post-processing controls. Scenes can be saved as presets,
captured as images, or exported to Blender for further work.

## Highlights

- recursive, level-by-level shape generation with configurable placement rules
- inertial three-axis camera movement and keyboard zoom
- live parameter editing and LFO modulation through the `F2` control pages
- procedural material families, stage controls, and a camera-output shader stack
- 100 scene-preset slots with save, load, free, and collision-resolution flows
- Blender `.blend` export with scene data and effect metadata

## Gallery

![Recent Intergen screenshot](screenshots/intergen-1773391201-479-0000.png)

![Recent Intergen screenshot 2](screenshots/intergen-1773391741-567-0001.png)

|  |   |
| --- | --- |
| ![Intergen screenshot 0000](screenshots/intergen-0000.png) | ![Intergen screenshot 0001](screenshots/intergen-0001.png) |
| ![Intergen screenshot 0002](screenshots/intergen-0002.png) | ![Intergen screenshot 0003](screenshots/intergen-0003.png) |
| ![Intergen screenshot 0004](screenshots/intergen-0004.png) | ![Intergen screenshot 0005](screenshots/intergen-0005.png) |
| ![Intergen screenshot 0006](screenshots/intergen-0006.png) | ![Intergen screenshot 0007](screenshots/intergen-0007.png) |
| ![Recent Intergen screenshot 3](screenshots/intergen-2026-03-18_01-02-51-810-0001.png) | ![Recent Intergen screenshot 4](screenshots/intergen-2026-03-18_04-18-45-666-0000.png) |
| ![Screenshot 11](screenshots/intergen-2026-03-30_04-22-47-862-0002.png) | ![Screenshot 12](screenshots/intergen-2026-03-30_04-23-16-643-0003.png) |
| ![Screenshot 13](screenshots/intergen-2026-03-30_04-23-18-389-0004.png) | ![Screenshot 14](screenshots/intergen-2026-03-30_04-23-31-178-0006.png) |
| ![Screenshot 15](screenshots/intergen-2026-03-30_04-28-09-734-0012.png) | ![Screenshot 16](screenshots/intergen-2026-03-30_04-28-12-091-0013.png) |

## Documentation

Start with the [documentation index](docs/README.md), or go directly to:

- the [user guide](docs/USER_GUIDE.md) for setup, workflows, configuration,
  presets, capture, export, and development
- the [complete keybinding reference](docs/KEYBINDINGS.md), including every
  F-mode and its context-specific controls
- the [illustrated spawn-geometry guide](docs/SPAWN_GEOMETRY_GUIDE.md) for the
  geometry and placement model
- the [data-model reference](docs/DATA_MODEL.md) for scene, configuration, and
  preset internals

## License

Intergen is licensed under `GPL-3.0-or-later`.

Copyright (C) 2026 Francesco Stablum. See [LICENSE](LICENSE) for the project
notice and [COPYING](COPYING) for the full GNU General Public License text.
