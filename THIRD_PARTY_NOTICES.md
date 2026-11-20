# Third-party notices

This project is licensed under the MIT License (see `LICENSE`). It also contains or is built from the following.
This file records sources and terms as far as they have been confirmed; it is not legal advice.

## forge-rs (mhgu-forge)

Git dependency in `Cargo.toml` (https://github.com/Lawrence-al/forge-rs, rev `f2af062`), compiled into `wishlist.nro`.

```
MIT License

Copyright (c) 2026 Fexty

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

## Noto Sans CJK KR

`assets/NotoSansCJKkr-Regular.otf` and its subset `assets/languages/font.shared.otf` (shipped in the release).
© 2014-2021 Adobe; licensed under the SIL Open Font License 1.1. The full text is in `assets/NOTO_LICENSE.txt` and is
shipped next to the font as `NOTO_LICENSE.txt`.

## Kenney Input Prompts

The controller, keyboard and mouse prompt pictures placed into the atlas by `tools/prepare_prompt_icons.py`
(rectangles in `src/prompt_icons_generated.rs`; shipped in `assets/hd/wishlist-atlas.rgba` and
`assets/low-memory/wishlist-atlas.rgba`) are from Kenney Input Prompts (https://kenney.nl), released under CC0.

## HD Layout UI pack art

The other pictures in the atlas (`assets/wishlist-atlas.rgba`, cropped to `assets/hd/wishlist-atlas.rgba` and reduced
to `assets/low-memory/wishlist-atlas.rgba`; rectangles in `src/icons_generated.rs`) — equipment, material, button,
chest and slot pictures — are the game's UI art as found in the "HD Layout" UI pack. The author and redistribution terms
of that pack have not been confirmed yet.

## Game data and game art

Monster Hunter Generations Ultimate is © CAPCOM. The equipment, item and material names and descriptions, the UI text
and the equipment and recipe tables (`assets/languages/names.*.bin`, `assets/languages/text.*.bin`,
`assets/ui-names.bin`, `assets/ui-text.bin`, `src/generated.rs`, `src/equipment_generated.rs`,
`src/item_text_generated.rs`, `src/categories_generated.rs`) come from the game and remain the property of their owner.

These are included only so the mod can show the game's own names, data and icons; no license is granted for them here,
and this project is not affiliated with or endorsed by CAPCOM.
