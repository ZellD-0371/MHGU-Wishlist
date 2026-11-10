"""Package the standalone Forge overlay. Does not install or edit existing mods."""
import pathlib,shutil,zipfile,hashlib,json,tomllib
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/'dist/Wishlist'
VERSION=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
ARCHIVE=ROOT/f'Wishlist_v{VERSION}.zip'
files={
 'romfs/nativeNX/plugins/wishlist.nro':ROOT/'wishlist.nro',
 'romfs/nativeNX/plugins/wishlist/ui-text.bin':ROOT/'assets/ui-text.bin',
 'romfs/nativeNX/plugins/wishlist/wishlist-atlas.rgba':ROOT/'assets/wishlist-atlas.rgba',
 'romfs/nativeNX/plugins/wishlist/NotoSansCJKkr-Regular.otf':ROOT/'assets/NotoSansCJKkr-Regular.otf',
 'NOTO_LICENSE.txt':ROOT/'assets/NOTO_LICENSE.txt',
 'README_KO.md':ROOT/'README_KO.md',
 'VALIDATION.md':ROOT/'VALIDATION.md',
 'docs/crash-v0.2.1.md':ROOT/'docs/crash-v0.2.1.md',
 'tools/fix_recomp_imgui.py':ROOT/'tools/fix_recomp_imgui.py',
 'docs/forge_imgui_cursor.patch':ROOT/'docs/forge_imgui_cursor.patch',
}
# Match the runtime's actual pre-load check, including BSS and next-module alignment.
import struct
nro=(ROOT/'wishlist.nro').read_bytes();required=len(nro)+struct.unpack_from('<I',nro,0x38)[0]+0x10000
assert required<=0xC00000,f'NRO exceeds Recomp plugin region: {required} > 12582912'
manifest={}
for name,source in files.items():
 dest=OUT/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
 manifest[name]=hashlib.sha256(dest.read_bytes()).hexdigest()
(ROOT/'dist/manifest_sha256.json').write_text(json.dumps(manifest,indent=2),encoding='utf-8')
with zipfile.ZipFile(ARCHIVE,'w',zipfile.ZIP_DEFLATED) as z:
 for name in files:z.write(OUT/name,'Wishlist/'+name)
print(json.dumps({'version':VERSION,'archive':str(ARCHIVE),'files':len(files),'bytes':ARCHIVE.stat().st_size}))
