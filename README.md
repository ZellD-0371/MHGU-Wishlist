<p align="center">
  <strong>English</strong> | <a href="README_ko.md">한국어</a>
</p>

<h3 align="center">Screenshots</h3>

<p align="center">
  <a href="docs/screenshots.md">View screenshots</a>
</p>

# MHGU Wishlist

A Forge mod that adds a wishlist to the Smithy in Monster Hunter Generations Ultimate. Register equipment you want to forge or upgrade, then track the required materials and how many you own. The mod notifies you when all required materials are available and automatically removes the corresponding entry once forging or upgrading is complete.

## Requirements

- MHGU global version 1.4.0
- MHGU Recomp with Forge plugin support, or an emulator with Forge 1.5.1 installed

## Features

- Supports forging weapons, armor, decorations, and Palico weapons and armor, as well as upgrading weapons and armor
- Adds a gold wishlist marker to registered equipment icons at the Smithy and lets you remove entries directly
- Displays required materials, costs, and category upgrade materials for each piece of equipment
- Displays total material requirements across all registered equipment and shows which equipment needs each material
- Automatically removes the corresponding entry after successful forging or upgrading
- Selectable Nintendo, Xbox, and PlayStation button icons
- UI scaling from 50% to 100% while preserving its aspect ratio

## Installation

1. Close the game.
2. Extract `Wishlist_v1.0.0.zip`.
3. Place the extracted `Wishlist` folder in the `mods` folder next to the Recomp executable.
4. Launch the game. If the mod has been disabled in the mod manager, enable it.

```text
mods/
└─ Wishlist/
   └─ romfs/
      └─ nativeNX/
         └─ plugins/
            ├─ wishlist.nro
            └─ wishlist/
               ├─ ui-text.bin
               ├─ wishlist-atlas.rgba
               └─ NotoSansCJKkr-Regular.otf
```

## Usage

### Opening the Wishlist

At the Smithy, hold L3 and press R3 to open the wishlist. In villages and quest areas, the same shortcut is available while the in-game menu opened with the + button is displayed. On a keyboard, press the backtick key (to the left of the number 1 key) to open the wishlist after loading your character.

| Input — Nintendo layout | Action |
| --- | --- |
| R3 / F5 at the Smithy | Register the selected equipment / Confirm removal of an existing entry |
| L3 + R3 at the Smithy or in the in-game menu | Open the wishlist |
| Backtick key ( ` ) | Open the wishlist |
| Up / Down | Select an entry on the current page |
| D-pad Left / Right or PageUp / PageDown | Change pages |
| L / R or Tab | Switch between the Registered Equipment and Total Materials tabs |
| ZL / ZR or Q / E | Switch equipment detail pages |
| A / Enter | Confirm / Enter material details |
| B / Esc | Cancel / Go back / Close |
| X / Delete in Registered Equipment | Confirm removal of the selected entry |

Changing the button layout changes the displayed icons. Actual controller bindings follow your Recomp settings. The wishlist cannot be opened on the title or character selection screen.

## Settings

Press Insert to open the Forge settings, then go to `Plugin UI > Wishlist`.

| Setting | Options |
| --- | --- |
| Button layout | Nintendo / Xbox / PlayStation |
| Language | Choose from nine supported languages |
| UI scale | 50 / 60 / 70 / 80 / 90 / 100% |

Supported languages: `en`, `zh-Hans`, `zh-Hant`, `ja`, `ko`, `de`, `fr`, `it`, `es`.

## Save Data

The wishlist and settings are stored in separate INI files within the game's current save directory.

| File | Contents |
| --- | --- |
| `wishlist.ini` | Shared settings and compatibility data for wishlists from earlier versions |
| `wishlist_character_1.ini` | Wishlist for the first character slot |
| `wishlist_character_2.ini` | Wishlist for the second character slot |
| `wishlist_character_3.ini` | Wishlist for the third character slot |

The default Windows Recomp path is `%APPDATA%\MHGURecomp\save\save_0\`. If your environment specifies a custom save path, that path is used instead. These files are not replaced when updating the mod.

## Building from Source

Required tools: Rust nightly with `rust-src`, the Forge Cargo build tool (`cargo forge`), devkitARM, and Python 3.11 or later. The Forge SDK uses the commit specified in `Cargo.toml`.

```powershell
./tools/Check.ps1
$env:DEVKITARM = 'D:/Utils/devkitPro/devkitARM'
./Build.ps1
```

If the dependencies are already available locally, you can build with `./Build.ps1 -Offline`. The build produces `wishlist.nro`, `dist/Wishlist`, and `Wishlist_v1.0.0.zip`.

Sound capture, UI panel inspection, and notification tests are hidden in the standard release. To enable them during development, build with:

```powershell
cargo +nightly forge build --release --features debug-ui
```

Use `tools/render_ui_capture.py` for offline UI layout checks. It requires the additional Python packages Pillow and fontTools and writes its output to `target/ui-steel-proof`. These checks do not replace in-game testing.

## Repository Structure

`src`, `.cargo`, `link`, `conf`, `assets`, and `tools` contain the files needed to build and validate the source. Keep `Cargo.lock` and the generated Rust data files, as they are also required build inputs.

Build outputs, release ZIP archives, local test game files and saves, logs, browser profiles, and preview caches are excluded by `.gitignore`. `Run-Test.cmd` is specific to the separate test environment in this workspace and is not required to install the release.
