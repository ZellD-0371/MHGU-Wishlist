"""Check the shared Recomp Forge region using installed NRO segment headers."""
from pathlib import Path
import argparse, json, struct

def check(mods, replacement=None):
    folders = {p.name: p for p in mods.iterdir() if p.is_dir()}
    order, disabled = [], set()
    order_file = mods / 'load_order.txt'
    for line in order_file.read_text(encoding='utf-8-sig').splitlines() if order_file.exists() else []:
        line = line.strip()
        if line.startswith('# off: '):
            disabled.add(line[7:])
        elif line and not line.startswith('#') and line in folders and line not in order:
            order.append(line)
    order += sorted((n for n in folders if n not in order and n not in disabled and not n.startswith('_')), key=str.lower)
    plugins = {}
    for folder in order:
        for path in (folders[folder] / 'romfs/nativeNX/plugins').glob('*.nro'):
            plugins[path.name.lower()] = path
    if replacement:
        plugins['wishlist.nro'] = replacement
    at, end, rows = 0x03200000, 0x03E00000, []
    for name, path in plugins.items():
        data = path.read_bytes()
        assert data[0x10:0x14] == b'NRO0', path
        bss = struct.unpack_from('<I', data, 0x38)[0]
        offset, size = struct.unpack_from('<II', data, 0x30)
        required = len(data) + bss + 0x10000
        assert at + required <= end, f'{name}: shared plugin region exhausted ({end-at} bytes available, {required} required)'
        image_size = (offset + size + bss + 0xFFF) & ~0xFFF
        rows.append({'plugin': name, 'base': hex(at), 'image_bytes': image_size, 'preload_required': required})
        at = (at + image_size + 0xFFFF) & ~0xFFFF
    return {'plugins': rows, 'used_bytes': at-0x03200000, 'remaining_bytes': end-at, 'limit_bytes': 0xC00000}

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mods', type=Path)
    parser.add_argument('--wishlist', type=Path, help='Check a new Wishlist NRO before installation')
    args = parser.parse_args()
    print(json.dumps(check(args.mods, args.wishlist), indent=2))
