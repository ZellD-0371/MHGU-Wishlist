"""Generate single-language text packs and one metric-preserving Noto subset.

The original font and multilingual blobs remain build inputs. Shipping files are
under assets/languages; unchanged generated files can be verified without fontTools.
"""
from pathlib import Path
import argparse,hashlib,json,re,struct,zlib
ROOT=Path(__file__).resolve().parents[1]
CODES=['en','zh-Hans','zh-Hant','ja','ko','de','fr','it','es']
STR=r'"(?:[^"\\]|\\.)*"'
ROW=re.compile(r'\[(?:\s*'+STR+r'\s*,){8}\s*'+STR+r'\s*\]')

def records(data):
    assert data[:4]==b'WLTX' and struct.unpack_from('<I',data,4)[0]==1
    count=struct.unpack_from('<I',data,8)[0]
    for i in range(count):
        row=struct.unpack_from('<10I',data,12+i*40)
        yield row[0],[data[at:data.index(0,at)].decode('utf-8') for at in row[1:]]

def pack(rows,language):
    # Missing keys already return "" at runtime; preserve IDs, omit empty rows.
    rows=[(key,values) for key,values in rows if values[language]]
    header=16+len(rows)*8;pool=bytearray();offsets={};index=bytearray()
    for key,values in rows:
        text=values[language]
        if text not in offsets:offsets[text]=header+len(pool);pool.extend(text.encode()+b'\0')
        index.extend(struct.pack('<II',key,offsets[text]))
    return b'WLL1'+struct.pack('<III',1,language,len(rows))+index+(pool or b'\0')

def description_pack(rows,language):
    rows=[(key,values[language]) for key,values in rows if values[language]]
    header=16+len(rows)*8;pool=bytearray();offsets={};index=bytearray()
    for key,text in rows:
        raw=text.encode('utf-8');assert len(raw)<=4096
        if text not in offsets:
            offsets[text]=header+len(pool)
            pool.extend(struct.pack('<II',len(raw),crc(raw))+raw)
        index.extend(struct.pack('<II',key,offsets[text]))
    prefix=b'WLD1'+struct.pack('<III',1,language,len(rows))+index
    return prefix+pool,{'bytes':len(prefix),'crc32':crc(prefix)}

def required_characters(names,descriptions):
    common=set(range(32,127))|{0xA0}
    chars=[set(common) for _ in CODES]
    for _,values in names+descriptions:
        for i,text in enumerate(values):chars[i].update(ord(c) for c in text if ord(c)>=32)
    # Include localized category/skill/settings strings. The language picker must
    # show every language's own name even while a different font is active.
    special={'locales.rs','categories_generated.rs','equipment_generated.rs'}
    for path in (ROOT/'src').glob('*.rs'):
        source=path.read_text(encoding='utf-8')
        if path.name in special:
            for row in ROW.finditer(source):
                values=json.loads(row[0])
                for i,text in enumerate(values):chars[i].update(ord(c) for c in text if ord(c)>=32)
            if path.name=='locales.rs':
                languages=json.loads(re.search(r'pub const LANGUAGES:.*?=(\[.*?\]);',source).group(1))
                for text in languages:common.update(ord(c) for c in text)
        elif path.name!='generated.rs' and not path.name.endswith('_generated.rs'):
            for literal in re.findall(STR,source):
                try:text=json.loads(literal)
                except ValueError:continue
                common.update(ord(c) for c in text if ord(c)>=32)
    for s in chars:s.update(common)
    return chars

def crc(data):return zlib.crc32(data)&0xffffffff

def build(check=False):
    names=list(records((ROOT/'assets/ui-names.bin').read_bytes()))
    descriptions=list(records((ROOT/'assets/ui-text.bin').read_bytes()))
    chars=required_characters(names,descriptions)
    original=ROOT/'assets/NotoSansCJKkr-Regular.otf'
    inputs=hashlib.sha256(original.read_bytes()+(ROOT/'assets/ui-names.bin').read_bytes()+(ROOT/'assets/ui-text.bin').read_bytes()+json.dumps([sorted(s) for s in chars]).encode()+Path(__file__).read_text(encoding='utf-8-sig').encode()).hexdigest()
    out=ROOT/'assets/languages';out.mkdir(exist_ok=True)
    manifest_file=out/'manifest.json'
    previous=json.loads(manifest_file.read_text(encoding='utf-8')) if manifest_file.exists() else {}
    if previous.get('inputs_sha256')==inputs:
        good=all((out/name).exists() and len((data:=(out/name).read_bytes()))==meta['bytes'] and crc(data)==meta['crc32'] for name,meta in previous['files'].items())
        if good:
            emit_rust(previous)
            print('Language resources verified:',len(previous['files']),'files')
            return previous
    if check:raise SystemExit('Language resources are stale or damaged; regenerate with Python + fontTools.')
    try:
        from fontTools import subset
        from fontTools.ttLib import TTFont
    except ImportError:raise SystemExit('Generating language fonts requires fontTools. Run this script with a Python interpreter that has fontTools installed.')
    font=TTFont(original);cmap=font.getBestCmap();missing=[];files={};description_indexes={}
    name_rows=[row for row in names if row[0]>>16!=31]
    description_rows=sorted(descriptions+[row for row in names if row[0]>>16==31])
    assert len({key for key,_ in description_rows})==len(description_rows)
    for language,code in enumerate(CODES):
        for prefix in ['names','text']:
            filename=f'{prefix}.{code}.bin'
            if prefix=='names':data=pack(name_rows,language)
            else:data,description_indexes[code]=description_pack(description_rows,language)
            (out/filename).write_bytes(data);files[filename]={'bytes':len(data),'crc32':crc(data)}
        unsupported=sorted(chars[language]-cmap.keys());missing.append(unsupported)
        assert not unsupported,(code,'font source lacks required characters',unsupported)
    # Keep one shared subset: this avoids unsafe font deletion in current Recomp
    # bridges and preserves all nine languages with no font switch at runtime.
    required=set.union(*chars)
    current=TTFont(original,recalcTimestamp=False)
    options=subset.Options();options.layout_features=['*'];options.recalc_timestamp=False;options.notdef_glyph=True;options.notdef_outline=True;options.recommended_glyphs=True
    worker=subset.Subsetter(options=options);worker.populate(unicodes=required);worker.subset(current)
    family='Wishlist UI';ps='WishlistUI-Regular'
    current['name'].names=[record for record in current['name'].names if record.nameID not in (1,2,3,4,6,16,17)]
    for name_id,value in [(1,family),(2,'Regular'),(3,ps+'-1'),(4,family+' Regular'),(6,ps),(16,family),(17,'Regular')]:current['name'].setName(value,name_id,3,1,0x409)
    if 'CFF ' in current:
        cff=current['CFF '].cff;cff.fontNames=[ps];cff.topDictIndex[0].FamilyName=family;cff.topDictIndex[0].FullName=family+' Regular'
    filename='font.shared.otf';current.save(out/filename)
    verified=TTFont(out/filename);submap=verified.getBestCmap();assert required<=submap.keys()
    assert all(verified['hmtx'][submap[c]]==font['hmtx'][cmap[c]] for c in required),'font metrics changed'
    assert (verified['hhea'].ascent,verified['hhea'].descent)==(font['hhea'].ascent,font['hhea'].descent)
    data=(out/filename).read_bytes();files[filename]={'bytes':len(data),'crc32':crc(data)}
    for code in CODES:
        obsolete=out/f'font.{code}.otf'
        if obsolete.is_file():obsolete.unlink()
    print('Shared font:',len(data),'bytes;',len(submap),'characters; missing: 0',flush=True)
    result={'format':2,'inputs_sha256':inputs,'codes':CODES,'files':files,'description_indexes':description_indexes,'previously_unsupported_characters':missing}
    manifest_file.write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
    emit_rust(result)
    return result

def emit_rust(manifest):
    lines=['// Generated by tools/prepare_languages.py. Language resource integrity checks.','pub const FILES:[[(usize,u32);3];9]=[']
    for code in CODES:
        names=[f'names.{code}.bin',f'text.{code}.bin','font.shared.otf']
        lines.append('['+','.join(f'({manifest["files"][name]["bytes"]},{manifest["files"][name]["crc32"]})' for name in names)+'],')
    lines.append('];')
    lines.append('pub const DESCRIPTION_INDEX:[(usize,u32);9]=[')
    for code in CODES:
        meta=manifest['description_indexes'][code];lines.append(f'({meta["bytes"]},{meta["crc32"]}),')
    lines.append('];');(ROOT/'src/language_assets.rs').write_text('\n'.join(lines)+'\n',encoding='utf-8')

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--check',action='store_true');args=parser.parse_args();build(args.check)
