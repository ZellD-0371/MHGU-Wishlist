"""Compare source package, release ZIP and every installed Wishlist copy."""
from pathlib import Path
import argparse,json,hashlib,zipfile
ROOT=Path(__file__).resolve().parents[1]
def digest(data):return hashlib.sha256(data).hexdigest()
def audit(root,low=False):
 manifest=json.loads((ROOT/'dist/build'/('low-memory' if low else 'standard')/'package_sha256.json').read_text())
 version=__import__('tomllib').loads((ROOT/'Cargo.toml').read_text())['package']['version']
 source=ROOT/('dist/native-low/Wishlist' if low else 'dist/native/Wishlist')
 archive=ROOT/'dist'/f'Wishlist_{"LowMemory_" if low else ""}v{version}.zip';errors=[]
 with zipfile.ZipFile(archive) as z:
  if set(z.namelist())!={'Wishlist/'+n for n in manifest}:errors.append('ZIP file list differs')
  for name,sha in manifest.items():
   if digest((source/name).read_bytes())!=sha or digest(z.read('Wishlist/'+name))!=sha:errors.append('Build/ZIP mismatch: '+name)
 copies=list(root.rglob('wishlist.nro'));expected=root/'Wishlist/romfs/nativeNX/plugins/wishlist.nro'
 if copies!=[expected]:errors.append('NRO copies: '+str([str(p) for p in copies]))
 dest=root/'Wishlist'
 for name,sha in manifest.items():
  p=dest/name
  if not p.is_file() or digest(p.read_bytes())!=sha:errors.append('Installed mismatch: '+name)
 extra=[str(p.relative_to(dest)) for p in dest.rglob('*') if p.is_file() and p.relative_to(dest).as_posix() not in manifest]
 if extra:errors.append('Residual files: '+str(extra))
 result={'root':str(root),'version':version,'profile':'LowMemory' if low else 'HD','files':len(manifest),'errors':errors}
 print(json.dumps(result,ensure_ascii=False,indent=2));return not errors
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--root',type=Path,required=True);p.add_argument('--low-memory',action='store_true');a=p.parse_args();raise SystemExit(0 if audit(a.root,a.low_memory) else 1)
