"""Generate native quest -> possible item IDs (rewards, monsters, gathering).
Only native GU quest/item IDs enter the runtime table. No translated-name matching.
"""
from pathlib import Path
import csv,json,re,struct,sys,hashlib
ROOT=Path(__file__).resolve().parents[1];SHARED=ROOT.parent
DATA=SHARED/'tools/MHGU-Save-editor/data'
ROM=SHARED/'UnpakingGU/backup/romfs/nativeNX'
sys.path.insert(0,str(SHARED/'tools/korean_port'))
from inspect_inputs import gmd_info

def csvrows(name):return list(csv.DictReader((DATA/name).open(encoding='utf-8-sig')))
def build(check=False):
 db=json.loads((SHARED/'Database/mhxx_all_tables.json').read_text(encoding='utf-8'))
 source=(ROOT/'src/generated.rs').read_text(encoding='utf-8')
 block=re.search(r'pub static ITEMDATA:.*?=&\[\n(.*?)\n\];',source,re.S)[1]
 items={}
 for i,line in enumerate(block.splitlines()):
  if line.strip().rstrip(',')=='["";9]':continue
  en=json.loads(line.strip().rstrip(','))[0]
  if en:
   assert i%2==0, (i,en) # GMD has name/description pairs; item ID is half the name index.
   items.setdefault(en,[]).append(i//2)
 item_ids={}
 for row in db['ID_Itm_Name']:
  ids=items.get(row['Itm_Name_0'],[])
  if len(ids)==1:item_ids[row['Itm_ID']]=ids[0]
 monsters={r['Mon_Name_0']:r['Mon_ID'] for r in db['ID_Mon_Name'] if r['Mon_ID']>0}
 # White Fatalis is the GU name of the Database's Old Fatalis, not normal Fatalis.
 if 'Old Fatalis' in monsters:monsters['White Fatalis']=monsters['Old Fatalis']
 native_names=csvrows('monster-index.csv')
 assert not [r['monster'] for r in native_names if r['monster'] and r['monster'] not in monsters], 'Unmapped native monster identity'
 native_mon={int(r['index']):monsters[r['monster']] for r in native_names if r['monster'] in monsters}
 boss={}
 for r in csvrows('quest-monsters.csv'):
  if int(r['monster']) in native_mon:boss.setdefault(int(r['quest_id']),set()).add(native_mon[int(r['monster'])])
 rewards={}
 for r in csvrows('rewards.csv'):
  if int(r['chance'])>0 and int(r['count'])>0:rewards.setdefault(int(r['rem']),set()).add(int(r['item']))
 quest_rewards={}
 for r in csvrows('quest-rewards.csv'):quest_rewards.setdefault(int(r['quest_id']),set()).update(rewards.get(int(r['rem']),()))
 drops={};gather={}
 for r in db['DB_ItmtoMon']:
  if r['Itm_ID'] in item_ids and r['Pct']>0 and r['Qty']>0:drops.setdefault((r['Mon_ID'],r['Rank_ID']),set()).add(item_ids[r['Itm_ID']])
 for r in db['DB_ItmtoLoc']:
  if r['Itm_ID'] in item_ids:gather.setdefault((r['Loc_ID'],r['Rank_ID']),set()).add(item_ids[r['Itm_ID']])
 # Native stage -> Database location, joined by explicit location names.
 stage_names={1:'J. Frontier',2:'V. Hills',3:'A. Ridge',4:'M. Peaks',5:'Dunes',6:'D. Island',7:'Marshlands',8:'Volcano',9:'Arena',10:'V. Slayground',11:'A. Steppe',12:'V. Hollow',13:'Primal Forest',14:'F. Seaway',15:'F. Slayground',16:'Sanctuary',17:'Forlorn Arena',18:'S. Pinnacle',19:'Ingle Isle',20:'Polar Field',21:"Wyvern’s End",22:'Desert',23:'Jungle',24:'R. Pinnacle',25:'Castle Schrade',26:'Fortress',27:'Forlorn Citadel'}
 locs={r['Loc_Name_0']:r['Loc_ID'] for r in db['ID_Loc_Name']}
 # Do not silently accept a spelling change in the canonical location table.
 aliases={'R. Pinnacle':'Ruined Pinnacle',"Wyvern’s End":"Wyvern's End"}
 stage_locs={s:locs.get(n,locs.get(aliases.get(n,''))) for s,n in stage_names.items()}
 assert all(v is not None for v in stage_locs.values()), 'Unmapped native stage'
 # Deviant level ranges are specific to the material table (two early-rank schemes).
 special={4:(1,3),5:(4,5),6:(6,8),7:(1,4),8:(5,8),9:(9,10),10:(11,12),11:(13,15),12:(16,16)}
 records={};unmapped=set();extra_monsters=set()
 used={r['Itm_ID'] for key in ('DB_ItmtoMon','DB_ItmtoLoc') for r in db[key]}
 missing_items=[(r['Itm_ID'],r['Itm_Name_0']) for r in db['ID_Itm_Name'] if r['Itm_ID'] in used and r['Itm_ID'] not in item_ids]
 for r in csvrows('quest-index.csv'):
  quest=int(r['quest_id']);qfile=ROM/f'loc/quest/questData/questData_{quest:07d}.ext'
  if not qfile.exists():continue
  q=qfile.read_bytes();assert len(q) in (329,369) and struct.unpack_from('<I',q,12)[0]==quest,(quest,len(q),struct.unpack_from('<I',q,12)[0])
  rank={0:1,1:1,3:2,5:3}.get(q[0x13]);assert rank,(quest,q[0x13])
  available=set(quest_rewards.get(quest,()))
  mon=set(boss.get(quest,()))
  textfile=ROM/f'eng/quest/questData/questData_{quest:07d}_eng.gmd'
  if textfile.exists():
   text=gmd_info(textfile.read_bytes())['strings']
   for name in text[3].splitlines():
    name=name.strip()
    if name in monsters:mon.add(monsters[name]);extra_monsters.add(name)
    elif name not in ('None','-','', '???'):unmapped.add(name)
  level=q[0x12]-100
  for monster in mon:
   ranks=[rank]
   if any(m==monster and k>3 for m,k in drops):ranks=[k for k,(lo,hi) in special.items() if lo<=level<=hi]
   for k in ranks:available.update(drops.get((monster,k),()))
  stage=q[0x14];stage=stage-100 if 101<=stage<=104 else stage
  loc=stage_locs.get(stage)
  if loc is not None:available.update(gather.get((loc,rank),()))
  ids=sorted(i for i in available if 0<i<2992)
  assert records.setdefault(quest,ids)==ids
 assert not missing_items, ('Unmapped acquisition item IDs',missing_items)
 assert unmapped<= {'DUMMY','dummy'}, ('Unmapped other monsters',unmapped)
 records=sorted(records.items())
 pool=bytearray();index=bytearray()
 for quest,ids in records:index+=struct.pack('<III',quest,len(pool)//2,len(ids));pool+=struct.pack('<'+'H'*len(ids),*ids)
 data=b'WLQM'+struct.pack('<II',1,len(records))+index+pool
 out=ROOT/'assets/quest-materials.bin'
 if not out.exists() or out.read_bytes()!=data:
  if check:raise SystemExit('Quest material resources stale: run tools/prepare_quest_materials.py')
  out.write_bytes(data)
 report={'unmappedSourceItems':missing_items,'quests':len(records),'bytes':len(data),'itemMappings':len(item_ids),'stageMappings':stage_locs,'unmappedOtherMonsterLabels':sorted(unmapped),'otherMonsters':sorted(extra_monsters),'sha256':hashlib.sha256(data).hexdigest()}
 (ROOT/'target/quest-materials-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
 print(json.dumps(report,ensure_ascii=False))
if __name__=='__main__':build('--check' in sys.argv)
