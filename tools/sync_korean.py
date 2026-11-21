"""Synchronize Korean-only source columns from the shared Database and KoreanPatch.
Run --write deliberately; builds and packaging run the read-only stale check.
"""
from pathlib import Path
import sys,re,json,struct,argparse,ast
ROOT=Path(__file__).resolve().parents[1];SHARED=ROOT.parent
sys.path.insert(0,str(SHARED/'tools/korean_port'))
from wording_fixes import polish,particles
from official_terms import renamer_for
from inspect_inputs import gmd_info
# Read the shared ID table as data without importing the patch build toolchain.
_alias_ast=ast.parse((SHARED/'tools/korean_port/sync_database_terms.py').read_text(encoding='utf-8-sig'))
ITEM_NAME_OVERRIDES=ast.literal_eval(next(node.value for node in _alias_ast.body if isinstance(node,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='ITEM_NAME_OVERRIDES' for t in node.targets)))
Q=r'"(?:[^"\\]|\\.)*"';ROW=re.compile(r'\[\s*'+Q+r'(?:\s*,\s*'+Q+r'){8}\s*\]')
def sync(write=False):
 db=json.loads((SHARED/'Database/mhxx_all_tables.json').read_text(encoding='utf-8'))
 base=SHARED/'MHGU-KoreanPatch/MHGU_Korean_Combined/zz_MHGU_Korean/romfs/nativeNX/eng/table'
 legacy=json.loads((SHARED/'MHGU-KoreanPatch/MHGU_Korean_Font/legacy_hangul_map.json').read_text(encoding='utf-8'))
 tables={p.stem.removesuffix('_eng').lower():[''.join(legacy.get(c,c) for c in t) for t in gmd_info(p.read_bytes())['strings']] for p in base.glob('*_eng.gmd')}
 texts=[t for rows in tables.values() for t in rows];rename=renamer_for(texts)
 def fix(t):return particles(rename(polish(t)))
 def clean(t):
  # Apply line-break-sensitive wording before whitespace handling. Keep the
  # Korean GMD's explicit line breaks; only horizontal spacing is normalized.
  return re.sub(r'[^\S\n]+',' ',re.sub(r'<[^>]+>','',fix(t))).strip()
 item_ids={row['Itm_ID']:row for row in db['ID_Itm_Name']}
 items={}
 for row in item_ids.values():
  en,ko=row.get('Itm_Name_0'),row.get('Itm_Name_4')
  if not en or not ko:continue
  if en in items:assert items[en]==ko,('ambiguous Database/GU item identity',en)
  items[en]=ko
 for ident,(en,ko) in ITEM_NAME_OVERRIDES.items():
  row=item_ids[ident]
  assert (row['Itm_Name_0'],row['Itm_Name_4'])==(en,ko),('canonical item ID mismatch',ident)
 # English bridges the XX Database IDs to GU GMD positions; it never replaces
 # the saved GU ID and never performs a material-name substring replacement.
 assert item_ids[2139]['Itm_Name_4']=='괴조의 상편린'

 # Skill trees and activated skills have different IDs and effects. Wishlist's
 # SKILLS table stores tree IDs; never join these records by translated name.
 trees={row['SklTree_ID']:row for row in db['ID_SklTree_Name'] if row['SklTree_ID']>=0}
 exports={name:json.loads((SHARED/'Database'/name).read_text(encoding='utf-8')) for name in
  ['mhxx_skill_trees_all_languages.json','mhxx_skills_all_languages.json','mhxx_decorations_all_languages.json']}
 for row in exports['mhxx_skill_trees_all_languages.json']:
  assert trees[row['id']]['SklTree_Name_4']==row['name']['ko'],('tree export mismatch',row['id'])
 for row in exports['mhxx_skills_all_languages.json']:
  assert trees[row['skill_tree_id']]['SklTree_Name_4']==row['skill_tree_name']['ko'],('activated-skill tree mismatch',row['id'])
 for row in exports['mhxx_decorations_all_languages.json']:
  for point in row['skill_points']:
   assert trees[point['skill_tree_id']]['SklTree_Name_4']==point['skill_tree_name']['ko'],('decoration tree mismatch',row['id'])

 changes=[]
 def output(path,data):
  old=path.read_bytes()
  if old!=data:
   changes.append(str(path.relative_to(ROOT)))
   if write:path.write_bytes(data)
 def replace(m,v):
  t=list(re.finditer(Q,m[0]))[4];v=fix(v);assert fix(v)==v,('non-idempotent Korean wording',v)
  return m[0][:t.start()]+json.dumps(v,ensure_ascii=False)+m[0][t.end():]
 p=ROOT/'src/generated.rs';s=p.read_text(encoding='utf-8')
 def block(m):
  name=m[1].lower();i=0
  def row(a):
   nonlocal i
   v=json.loads(a[0]);index=i;i+=1
   target=tables.get(name,[]);ko=target[index] if index<len(target) else v[4]
   if name=='itemdata':ko=items.get(v[0],ko)
   return replace(a,ko)
  # Empty description placeholders also occupy an index.
  def line(line):
   nonlocal i
   if line=='["";9],':i+=1;return line
   return ROW.sub(row,line)
  return m[0].replace(m[2],'\n'.join(line(l) for l in m[2].splitlines()),1)
 s=re.sub(r'pub static (\w+):&\[\[&str;9\]\]=&\[\n(.*?)\n\];',block,s,flags=re.S);output(p,s.encode())
 p=ROOT/'src/item_text_generated.rs';s=p.read_text(encoding='utf-8');i=0
 def desc(m):
  nonlocal i
  v=clean(tables['itemdata'][i*2+1]);i+=1;return replace(m,v)
 output(p,ROW.sub(desc,s).encode())
 for name in ['categories_generated.rs','equipment_generated.rs']:
  p=ROOT/'src'/name
  def row(m):
   v=json.loads(m[0]);ko=v[4]
   return replace(m,ko)
  text=ROW.sub(row,p.read_text(encoding='utf-8'))
  if name=='equipment_generated.rs':
   def tree_block(block):
    def tree_row(m):
     ident=int(m[1]);values=json.loads(m[2]);tree=trees[ident]
     assert values[0]==tree['SklTree_Name_0'],('tree ID/name mismatch',ident)
     return '('+m[1]+','+ROW.sub(lambda a:replace(a,tree['SklTree_Name_4']),m[2])+'),'
    return block[0].replace(block[1],re.sub(r'^\((\d+),(\[.*\])\),$',tree_row,block[1],flags=re.M),1)
   text=re.sub(r'pub static SKILLS:.*?=&\[\n(.*?)\n\];',tree_block,text,flags=re.S)
  output(p,text.encode())
 # External equipment descriptions are a separate runtime input, not generated.rs.
 p=ROOT/'assets/ui-text.bin';data=p.read_bytes();n=struct.unpack_from('<I',data,8)[0];rows=[]
 specs={k:('armorseriesdata',10,k-1+5) for k in range(1,6)}
 specs.update({22:('otweapondata',2,1),23:('otarmordata',4,2),24:('otarmordata',4,3)})
 specs.update({c+7:(f'weapon{c:02}msgdata',6,1) for c in [0,1,2,3,4,6,7,8,9,10,11,12,13,14]})
 for i in range(n):
  row=struct.unpack_from('<10I',data,12+i*40);key=row[0];v=[data[a:data.index(0,a)].decode() for a in row[1:]]
  name,stride,offset=specs[key>>16];v[4]=fix(clean(tables[name][(key&65535)*stride+offset]));assert fix(v[4])==v[4];rows.append((key,v))
 out=bytearray(b'WLTX'+struct.pack('<II',1,n)+bytes(n*40));pool={}
 for i,(key,v) in enumerate(rows):
  offsets=[]
  for text in v:
   if text not in pool:pool[text]=len(out);out.extend(text.encode()+b'\0')
   offsets.append(pool[text])
  struct.pack_into('<10I',out,12+i*40,key,*offsets)
 output(p,bytes(out))
 if changes and not write:raise SystemExit('Korean inputs stale; run tools/sync_korean.py --write: '+', '.join(changes))
 print('Korean canonical inputs:',', '.join(changes) if changes else 'verified (idempotent)')
if __name__=='__main__':
 a=argparse.ArgumentParser(description=__doc__);a.add_argument('--write',action='store_true');sync(a.parse_args().write)
