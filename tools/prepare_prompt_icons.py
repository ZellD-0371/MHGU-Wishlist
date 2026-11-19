"""Install Kenney "Input Prompts" icons (CC0, www.kenney.nl) into the HD atlas.

Replaces the twelve controller prompt sprites of each layout (107..142) in place,
and places the extra controller prompts (D-pad up/down, plus/minus) and the
keyboard/mouse prompts into free atlas cells. Their rectangles are written to
src/prompt_icons_generated.rs; an existing file keeps every placement stable,
so rerunning reproduces the same atlas.

The Kenney pack is not part of the repository. Run this only to change icons:
  python tools/prepare_prompt_icons.py --source "<Kenney Input Prompts folder>"
then tools/prepare_hd_atlas.py and tools/prepare_low_memory.py (Build.ps1 does).
"""
from pathlib import Path
import argparse, re
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
ATLAS = ROOT / 'assets/wishlist-atlas.rgba'
GENERATED = ROOT / 'src/prompt_icons_generated.rs'
SIZE, PITCH, WIDTH, HEIGHT = 48, 50, 2048, 1024
DEFAULT_SOURCE = ROOT.parent / 'HD Icons/kenney Icons'

# Slot order of each layout: east, south, north, west, L3, R3, ZL, ZR,
# D-pad right, L, R, D-pad left (see button_prompts::icon_slot).
LAYOUTS = [
    ('Nintendo Switch', ['switch_button_a', 'switch_button_b', 'switch_button_x', 'switch_button_y',
        'switch_stick_l_press', 'switch_stick_r_press', 'switch_button_zl', 'switch_button_zr',
        'switch_right', 'switch_button_l', 'switch_button_r', 'switch_left'],
        ['switch_up', 'switch_down', 'switch_button_plus', 'switch_button_minus']),
    ('Xbox Series', ['xbox_button_color_b', 'xbox_button_color_a', 'xbox_button_color_y', 'xbox_button_color_x',
        'xbox_ls', 'xbox_rs', 'xbox_lt', 'xbox_rt',
        'xbox_dpad_right_outline', 'xbox_lb', 'xbox_rb', 'xbox_dpad_left_outline'],
        ['xbox_dpad_up_outline', 'xbox_dpad_down_outline', 'xbox_button_menu', 'xbox_button_view']),
    ('PlayStation Series', ['playstation_button_color_circle', 'playstation_button_color_cross',
        'playstation_button_color_triangle', 'playstation_button_color_square',
        'playstation_button_l3', 'playstation_button_r3',
        'playstation_trigger_l2_alternative', 'playstation_trigger_r2_alternative',
        'playstation_dpad_right_outline', 'playstation_trigger_l1_alternative',
        'playstation_trigger_r1_alternative', 'playstation_dpad_left_outline'],
        ['playstation_dpad_up_outline', 'playstation_dpad_down_outline',
         'playstation4_button_options', 'playstation4_button_share']),
]
# Keyboard keys and mouse buttons; whole-device pictures and highlight variants are left out.
SKIPPED_KBM = {'keyboard', 'keyboard_any', 'mouse', 'mouse_horizontal', 'mouse_move',
               'mouse_scroll_vertical', 'mouse_side', 'mouse_small', 'mouse_vertical'}

def rects(path):
    return [tuple(map(int, m)) for m in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}', path.read_text(encoding='utf-8'))]

def icon(path):
    # Premultiplied resampling keeps transparent edges from darkening.
    return Image.open(path).convert('RGBA').convert('RGBa').resize((SIZE, SIZE), Image.Resampling.LANCZOS).convert('RGBA')

def paste(atlas, image, x, y):
    atlas.paste((0, 0, 0, 0), (x, y, x + SIZE, y + SIZE))
    atlas.alpha_composite(image, (x, y))

def free_cells(atlas, used, count):
    """Cells for new sprites: first inside the already occupied height, then new rows."""
    alpha = atlas.getchannel('A')
    def clear(x, y):
        if any(x < rx + rw + 1 and rx < x + SIZE + 1 and y < ry + rh + 1 and ry < y + SIZE + 1 for rx, ry, rw, rh in used):
            return False
        return alpha.crop((max(x - 1, 0), max(y - 1, 0), x + SIZE + 1, y + SIZE + 1)).getbbox() is None
    bottom = max(y + h for x, y, w, h in used)
    cells = []
    for y in range(1, bottom - SIZE, 2):
        for x in range(1, WIDTH - SIZE, 2):
            if len(cells) == count:
                return cells
            if clear(x, y):
                cells.append((x, y)); used.append((x, y, SIZE, SIZE))
    y = (bottom + 3) // 2 * 2
    while len(cells) < count:
        for x in range(1, WIDTH - SIZE, PITCH):
            if len(cells) == count:
                break
            cells.append((x, y)); used.append((x, y, SIZE, SIZE))
        y += PITCH
    assert y + SIZE <= HEIGHT, 'atlas full'
    return cells

def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--source', type=Path, default=DEFAULT_SOURCE, help='Kenney Input Prompts folder')
    source = parser.parse_args().source
    base = rects(ROOT / 'src/icons_generated.rs')
    assert len(base) >= 143, 'unexpected base sprite table'
    atlas = Image.frombytes('RGBA', (WIDTH, HEIGHT), ATLAS.read_bytes())
    pad_extra = [(folder, name) for folder, _, extra in LAYOUTS for name in extra]
    kbm_dir = source / 'Keyboard & Mouse/Default'
    kbm = sorted(p.stem for p in kbm_dir.glob('*.png')
                 if p.stem not in SKIPPED_KBM and not p.stem.startswith('keyboard_arrows') and '_icon' not in p.stem)
    names = [n for _, n in pad_extra] + kbm
    # Reuse earlier placements; clear them before searching so reruns are stable.
    previous = {}
    if GENERATED.exists():
        text = GENERATED.read_text(encoding='utf-8')
        for name, x, y in re.findall(r'// ([\w]+)\nSprite\{x:(\d+),y:(\d+)', text):
            previous[name] = (int(x), int(y))
    for x, y in previous.values():
        atlas.paste((0, 0, 0, 0), (x, y, x + SIZE, y + SIZE))
    used = list(base) + [(x, y, SIZE, SIZE) for x, y in previous.values()]
    missing = [n for n in names if n not in previous]
    for name, cell in zip(missing, free_cells(atlas, used, len(missing))):
        previous[name] = cell
    # Controller slots 107..142, replaced in place.
    for layout, (folder, slots, _) in enumerate(LAYOUTS):
        for i, name in enumerate(slots):
            x, y, w, h = base[107 + layout * 12 + i]
            assert (w, h) == (SIZE, SIZE)
            paste(atlas, icon(source / folder / 'Default' / f'{name}.png'), x, y)
    for folder, name in pad_extra:
        paste(atlas, icon(source / folder / 'Default' / f'{name}.png'), *previous[name])
    for name in kbm:
        paste(atlas, icon(kbm_dir / f'{name}.png'), *previous[name])
    ATLAS.write_bytes(atlas.tobytes())
    start = len(base)
    code = ['// Generated by tools/prepare_prompt_icons.py from Kenney Input Prompts (CC0, www.kenney.nl).',
            '// Sprites after ART; tile index = EXTRA_START + position in EXTRA.',
            f'pub const EXTRA_START:usize={start};',
            '// Per layout (Nintendo, Xbox, PlayStation): D-pad up, D-pad down, plus/start, minus/back.',
            'pub const PAD_EXTRA:usize=EXTRA_START;',
            f'pub const KEY_START:usize=EXTRA_START+{len(pad_extra)};',
            'pub static EXTRA:&[Sprite]=&[']
    for name in names:
        x, y = previous[name]
        code += [f'// {name}', f'Sprite{{x:{x},y:{y},w:{SIZE},h:{SIZE}}},']
    code += ['];', '// Keyboard and mouse prompt names (Kenney file names), in EXTRA order from KEY_START.',
             'pub static KEYS:&[&str]=&[' + ','.join(f'"{n}"' for n in kbm) + '];', '']
    GENERATED.write_text('\n'.join(code), encoding='utf-8')
    print(f'{len(LAYOUTS) * 12} controller sprites replaced; {len(pad_extra)} controller and {len(kbm)} keyboard/mouse sprites placed')

if __name__ == '__main__':
    main()
