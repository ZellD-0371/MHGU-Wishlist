"""Build a pointer-free external name/material-description table from source data."""
from pathlib import Path
import json, re, struct

ROOT = Path(__file__).resolve().parents[1]
TABLES = ['ITEMDATA', 'ARMORSERIESDATA', 'OTWEAPONDATA', 'OTARMORDATA',
          'WEAPON00MSGDATA', 'WEAPON01MSGDATA', 'WEAPON02MSGDATA', 'WEAPON03MSGDATA',
          'WEAPON04MSGDATA', 'WEAPON06MSGDATA', 'WEAPON07MSGDATA', 'WEAPON08MSGDATA',
          'WEAPON09MSGDATA', 'WEAPON10MSGDATA', 'WEAPON11MSGDATA', 'WEAPON12MSGDATA',
          'WEAPON13MSGDATA', 'WEAPON14MSGDATA']

def build():
    source = (ROOT / 'src/generated.rs').read_text(encoding='utf-8')
    records = []
    for kind, table in enumerate(TABLES, 1):
        block = re.search(r'pub static ' + table + r':.*?=&\[\n(.*?)\n\];', source, re.S)
        assert block, table
        for index, line in enumerate(block[1].splitlines()):
            row = line.strip().removesuffix(',')
            values = [''] * 9 if row == '["";9]' else json.loads(row)
            assert len(values) == 9 and index <= 65535, (table, index)
            records.append((kind << 16 | index, values))
    descriptions = (ROOT / 'src/item_text_generated.rs').read_text(encoding='utf-8')
    for index, match in enumerate(re.finditer(r'^\(\d+,(\[.*\])\),$', descriptions, re.M)):
        values = json.loads(match[1])
        assert len(values) == 9
        records.append((31 << 16 | index, values))
    assert index + 1 == 2991, 'Unexpected material description count'
    pool = bytearray()
    offsets = {}
    header_size = 12 + len(records) * 40
    rows = bytearray()
    for key, values in records:
        row = [key]
        for text in values:
            assert '\0' not in text
            if text not in offsets:
                offsets[text] = header_size + len(pool)
                pool.extend(text.encode('utf-8') + b'\0')
            row.append(offsets[text])
        rows.extend(struct.pack('<10I', *row))
    data = b'WLTX' + struct.pack('<II', 1, len(records)) + rows + pool
    output = ROOT / 'assets/ui-names.bin'
    if not output.exists() or output.read_bytes() != data:
        output.write_bytes(data)
    print(f'External names: {len(records)} rows, {len(data)} bytes, 9 languages')

if __name__ == '__main__':
    build()
