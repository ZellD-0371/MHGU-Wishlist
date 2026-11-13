"""Package only the romfs tree. Does not install or edit existing mods."""
import pathlib,shutil,zipfile,hashlib,json,tomllib,argparse
parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--low-memory',action='store_true');args=parser.parse_args()
ROOT=pathlib.Path(__file__).resolve().parents[1]
OUT=ROOT/('dist/native-low/Wishlist' if args.low_memory else 'dist/native/Wishlist')
VERSION=tomllib.loads((ROOT/'Cargo.toml').read_text(encoding='utf-8'))['package']['version']
ARCHIVE=ROOT/'dist'/(f'Wishlist_LowMemory_v{VERSION}.zip' if args.low_memory else f'Wishlist_v{VERSION}.zip')
files={
 'romfs/nativeNX/plugins/wishlist.nro':ROOT/'wishlist.nro',
 'romfs/nativeNX/plugins/wishlist/wishlist-atlas.rgba':ROOT/('assets/low-memory/wishlist-atlas.rgba' if args.low_memory else 'assets/hd/wishlist-atlas.rgba'),
 'romfs/nativeNX/plugins/wishlist/NOTO_LICENSE.txt':ROOT/'assets/NOTO_LICENSE.txt',
 'romfs/nativeNX/plugins/wishlist/LICENSE.txt':ROOT/'LICENSE',
}
resource_manifest=json.loads((ROOT/'assets/languages/manifest.json').read_text())
for name in resource_manifest['files']:files['romfs/nativeNX/plugins/wishlist/languages/'+name]=ROOT/'assets/languages'/name
assert len(files)==23,'Expected 19 language resources and 4 common files'
# Match the runtime's actual pre-load check, including BSS and next-module alignment.
import struct
nro=(ROOT/'wishlist.nro').read_bytes();required=len(nro)+struct.unpack_from('<I',nro,0x38)[0]+0x10000
assert required<=0xC00000,f'NRO exceeds Recomp plugin region: {required} > 12582912'
assert required<=0x300000,f'NRO exceeds 3 MiB compatibility budget: {required}'
assert all(source.is_file() for source in files.values()),'Missing package input'
# Remove metadata left by older packages, within this generated output only.
assert OUT.resolve() in [ROOT.resolve()/'dist'/'native'/'Wishlist',ROOT.resolve()/'dist'/'native-low'/'Wishlist'],'Unexpected package output path'
if OUT.exists():
 for child in OUT.iterdir():
  if child.name=='romfs':continue
  if child.is_symlink():child.unlink();continue
  assert child.resolve().parent==OUT.resolve(),'Cleanup target outside package output'
  if child.is_dir():shutil.rmtree(child)
  else:child.unlink()
for name in ['ui-text.bin','ui-names.bin','NotoSansCJKkr-Regular.otf']:
 stale=OUT/'romfs/nativeNX/plugins/wishlist'/name
 if stale.is_file():stale.unlink()
manifest={}
for name,source in files.items():
 dest=OUT/name;dest.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,dest)
 manifest[name]=hashlib.sha256(dest.read_bytes()).hexdigest()
audit=ROOT/'dist'/'build'/('low-memory' if args.low_memory else 'standard')/'package_sha256.json'
audit.parent.mkdir(parents=True,exist_ok=True);audit.write_text(json.dumps(manifest,indent=2),encoding='utf-8')
with zipfile.ZipFile(ARCHIVE,'w',zipfile.ZIP_DEFLATED) as z:
 for name in files:z.write(OUT/name,'Wishlist/'+name)
print(json.dumps({'version':VERSION,'profile':'LowMemory' if args.low_memory else 'HD','archive':str(ARCHIVE),'files':len(files),'bytes':ARCHIVE.stat().st_size}))
