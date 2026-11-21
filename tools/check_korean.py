"""Check both runtime Korean inputs, including separately stored equipment descriptions."""
from pathlib import Path
import struct,json
from prepare_languages import records,pack,description_pack
ROOT=Path(__file__).resolve().parents[1]
NAMES=list(records((ROOT/'assets/ui-names.bin').read_bytes()));DESC=list(records((ROOT/'assets/ui-text.bin').read_bytes()))
old=('가과','생고기','섬광구슬','귀환구슬','개옥','쟈기','란포스','팡고','이빌죠','생명의 분진','화속성')
for key,values in NAMES+DESC:
 assert not any(term in values[4] for term in old),(key,values[4])
items={v[0]:v[4] for key,v in NAMES if key>>16==1}
assert items['Lifecrystals']=='생명의 결정'
assert items['Lifepowder']=='생명의 가루'
expected_names=pack([r for r in NAMES if r[0]>>16!=31],4)
expected_text,_=description_pack(sorted(DESC+[r for r in NAMES if r[0]>>16==31]),4)
assert (ROOT/'assets/languages/names.ko.bin').read_bytes()==expected_names
assert (ROOT/'assets/languages/text.ko.bin').read_bytes()==expected_text
print('Korean names, equipment/material descriptions and shipped KO packs verified')

# This UI stores skill-tree IDs, not activated-skill IDs. Preserve GU's actual
# point/effect relationships while validating every displayed tree label.
import re,sys
sys.path.insert(0,str(ROOT.parent/'tools/korean_port'))
from wording_fixes import polish
source=(ROOT/'src/equipment_generated.rs').read_text(encoding='utf-8')
block=re.search(r'pub static SKILLS:.*?=&\[\n(.*?)\n\];',source,re.S)[1]
trees={row['id']:row for row in json.loads((ROOT.parent/'Database/mhxx_skill_trees_all_languages.json').read_text(encoding='utf-8'))}
for m in re.finditer(r'^\((\d+),(\[.*\])\),$',block,re.M):
 ident=int(m[1]);values=json.loads(m[2]);assert values[4]==trees[ident]['name']['ko'],(ident,values[4])
 assert not any(c in values[4] for c in 'ＳＰ％０１２３４５６７８９'),(ident,values[4])
for text in ['감기에 걸리기 쉽다.', '머리를 감기', '차지 단축', '기력', '치우침 없이']:
 assert polish(text)==text,('context-sensitive wording changed',text)
print('Skill-tree IDs / canonical labels / contextual exceptions verified')

# Prefix-aware: correct Yian Kut-Ku must not be renamed a second time.
from monster_terms import canonical as canonical_monsters
pairs=[('게료스','게리오스'),('얀쿡','얀쿡크'),('도도브랑고','도도블랑고'),
       ('브랑고','블랑고'),('격앙된 라잔','격앙 라잔'),('분노를 머금은 이블조','미친 이블조')]
for old_name,new_name in pairs:
 assert canonical_monsters(old_name)==new_name
 assert canonical_monsters(new_name)==new_name
 assert canonical_monsters(canonical_monsters(old_name))==new_name
assert canonical_monsters('라잔 / 격앙된 라잔 / 이블조 / 분노를 머금은 이블조')=='라잔 / 격앙 라잔 / 이블조 / 미친 이블조'
legacy_monsters=re.compile(r'게료스|얀쿡(?!크)|얀쿡크크|도도브랑고|브랑고|격앙된 라잔|분노를 머금은 이블조')
for key,values in NAMES+DESC:
 assert not legacy_monsters.search(values[4]),(key,values[4])
for filename in ['generated.rs','item_text_generated.rs','categories_generated.rs']:
 for match in re.finditer(r'\[(?:\s*'+r'"(?:[^"\\]|\\.)*"'+r'\s*,){8}\s*'+r'"(?:[^"\\]|\\.)*"'+r'\s*\]',(ROOT/'src'/filename).read_text(encoding='utf-8')):
  assert not legacy_monsters.search(json.loads(match[0])[4]),filename
print('Monster spellings, special forms and repeated normalization verified')

# Database Itm_ID and GU material ID are separate identity spaces.
from sync_korean import ITEM_NAME_OVERRIDES
item_db={r['Itm_ID']:r for r in json.loads((ROOT.parent/'Database/mhxx_all_tables.json').read_text(encoding='utf-8'))['ID_Itm_Name']}
gu_items={values[0]:((key&65535)//2,values[4]) for key,values in NAMES if key>>16==1 and key&1==0}
for ident,(english,korean) in ITEM_NAME_OVERRIDES.items():
 assert (item_db[ident]['Itm_Name_0'],item_db[ident]['Itm_Name_4'])==(english,korean)
 assert gu_items[english][1]==korean,(ident,english,gu_items.get(english))
assert item_db[2139]['Itm_Name_4']=='괴조의 상편린'
assert gu_items[item_db[2139]['Itm_Name_0']][1]=='괴조의 상편린'
assert gu_items[item_db[2139]['Itm_Name_0']][0]!=gu_items[item_db[2140]['Itm_Name_0']][0]
assert gu_items['Flash Bomb'][0]!=gu_items['Flash S'][0]
assert gu_items['Flash Bomb'][1]=='섬광옥' and gu_items['Flash S'][1]=='섬광탄'
assert polish('고기굽기\n세트')=='바베큐\n세트'
print('Database ID aliases, material grades, Flash Bomb/S and Korean line breaks verified')
