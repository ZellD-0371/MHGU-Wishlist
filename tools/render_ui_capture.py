"""Offline layout proof: run the real Rust draw code with a recording ImGui adapter.

This does not launch the game or claim to reproduce the runtime font rasterizer.
PNG text uses Noto's ascent/descent scale, matching ImGui's em-height convention.
"""
from pathlib import Path
import re,subprocess,json,struct,ast
from PIL import Image,ImageDraw,ImageFont,ImageChops
from fontTools.ttLib import TTFont
from fontTools.pens.boundsPen import BoundsPen

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'target/ui-steel-proof';OUT.mkdir(parents=True,exist_ok=True)
font=TTFont(ROOT/'assets/NotoSansCJKkr-Regular.otf');cmap=font.getBestCmap();denom=font['hhea'].ascent-font['hhea'].descent
(OUT/'advances.bin').write_bytes(b''.join(struct.pack('<f',font['hmtx'][cmap[c]][0]/denom if c in cmap else .5) for c in range(65536)))
glyphs=font.getGlyphSet();bounds={};horizontal={}
for name in set(cmap.values()):
 pen=BoundsPen(glyphs);glyphs[name].draw(pen)
 bounds[name]=(0.,0.) if pen.bounds is None else ((font['hhea'].ascent-pen.bounds[3])/denom,(font['hhea'].ascent-pen.bounds[1])/denom)
 horizontal[name]=(0.,0.) if pen.bounds is None else (pen.bounds[0]/denom,pen.bounds[2]/denom)
(OUT/'bounds.bin').write_bytes(b''.join(struct.pack('<2f',*bounds.get(cmap.get(c),(0.,0.))) for c in range(65536)))
(OUT/'horizontal.bin').write_bytes(b''.join(struct.pack('<2f',*horizontal.get(cmap.get(c),(0.,0.))) for c in range(65536)))
source=(ROOT/'src/ui.rs').read_text(encoding='utf-8')
canvas=source[source.index('static mut FONT'):source.index('fn save(')]
draw=source[source.index('fn materials('):source.index('pub fn render()')]
render=source[source.index('pub fn render()'):source.index('pub fn menu()')]
render='pub fn render(){let cv=Canvas::new().unwrap();STATE.with(|s|{'+render.split(' STATE.with(|s|{',1)[1]
modules='\n'.join(f'#[path="{(ROOT/"src"/(n+".rs")).as_posix()}"]mod {n};' for n in ['sorting','binding_model','model','generated','locales','equipment','interaction','navigation','presentation','text_blob','name_data','hud_layout','notifications'])
stub=r'''

extern crate alloc;extern crate self as forge;
use alloc::{vec::Vec,format,string::String};
use std::cell::RefCell;
use model::{List,Recipe};use locales as l;use generated as g;
use sys::imgui::{ImVec2,ImDrawFlags};
pub mod sys{pub mod imgui{
 #[derive(Clone,Copy,Debug)]pub struct ImVec2{pub x:f32,pub y:f32}
 pub enum ImDrawFlags{None,Closed}
 pub struct Baked{size:f32}pub struct Glyph{pub AdvanceX:f32,pub X0:f32,pub X1:f32,pub Y0:f32,pub Y1:f32}
 static BOUNDS:&[u8]=include_bytes!("bounds.bin");static HORIZONTAL:&[u8]=include_bytes!("horizontal.bin");static ADV:&[u8]=include_bytes!("advances.bin");static mut BAKED:Baked=Baked{size:0.};static mut GLYPH:Glyph=Glyph{AdvanceX:0.,X0:0.,X1:0.,Y0:0.,Y1:0.};
 pub unsafe fn ImFont_GetFontBaked(_: *mut u8,size:f32,_:f32)->*mut Baked{unsafe{BAKED.size=size};&raw mut BAKED}
 pub unsafe fn ImFontBaked_FindGlyph(baked: *mut Baked,ch:u16)->*mut Glyph{let at=ch as usize*8;let adv=ch as usize*4;unsafe{GLYPH.AdvanceX=f32::from_le_bytes(ADV[adv..adv+4].try_into().unwrap())*(*baked).size;GLYPH.X0=f32::from_le_bytes(HORIZONTAL[at..at+4].try_into().unwrap())*(*baked).size;GLYPH.X1=f32::from_le_bytes(HORIZONTAL[at+4..at+8].try_into().unwrap())*(*baked).size;GLYPH.Y0=f32::from_le_bytes(BOUNDS[at..at+4].try_into().unwrap())*(*baked).size;GLYPH.Y1=f32::from_le_bytes(BOUNDS[at+4..at+8].try_into().unwrap())*(*baked).size;} &raw mut GLYPH}
}}
pub mod imgui{
 use super::*;static ADV:&[u8]=include_bytes!("advances.bin");thread_local!{static SIZE:RefCell<f32>=const{RefCell::new(25.)};}
 #[derive(Clone,Copy)]pub struct ImFont;impl ImFont{pub fn raw(&self)->*mut u8{std::ptr::null_mut()}pub fn load(_: &std::ffi::CStr)->Option<Self>{Some(Self)}}
 pub fn push_font(_: &ImFont,s:Option<f32>){let dpi=std::env::var("PROOF_HOST_FONT_SCALE").ok().and_then(|v|v.parse::<f32>().ok()).unwrap_or(1.);SIZE.with(|v|*v.borrow_mut()=s.unwrap_or(25.)*dpi)}pub fn pop_font(){}
 pub fn calc_text_size(t:&str,_:bool,_:f32)->ImVec2{let s=SIZE.with(|v|*v.borrow());ImVec2{x:t.chars().map(|c|{let at=c as usize*4;if at+4<=ADV.len(){f32::from_le_bytes(ADV[at..at+4].try_into().unwrap())*s}else{s*0.5}}).sum(),y:s}}
 pub struct Io{pub DisplaySize:ImVec2,pub DeltaTime:f32}pub fn get_io()->Io{Io{DisplaySize:ImVec2{x:std::env::var("PROOF_WIDTH").ok().and_then(|v|v.parse().ok()).unwrap_or(1280.),y:std::env::var("PROOF_HEIGHT").ok().and_then(|v|v.parse().ok()).unwrap_or(720.)},DeltaTime:0.016}}
 #[derive(Clone,Copy)]pub struct DrawList;
 impl DrawList{
  pub fn foreground()->Self{Self}pub fn raw(&self)->*mut u8{1 as *mut u8}
  pub fn add_rect_filled(&self,a:ImVec2,b:ImVec2,c:u32,_:f32){println!("[\"rect\",{},{},{},{},{}]",a.x,a.y,b.x,b.y,c)}
  pub fn add_circle_filled(&self,p:ImVec2,r:f32,c:u32,_:i32){println!("[\"circle\",{},{},{},{}]",p.x,p.y,r,c)}
  pub fn add_line(&self,a:ImVec2,b:ImVec2,c:u32,t:f32){println!("[\"line\",{},{},{},{},{},{}]",a.x,a.y,b.x,b.y,c,t)}
  pub fn add_rect(&self,a:ImVec2,b:ImVec2,c:u32,_:f32,t:f32,_:ImDrawFlags){println!("[\"outline\",{},{},{},{},{},{}]",a.x,a.y,b.x,b.y,c,t)}
  pub fn add_rect_filled_multicolor(&self,a:ImVec2,b:ImVec2,l:u32,r:u32,_:u32,_:u32){println!("[\"gradient\",{},{},{},{},{},{}]",a.x,a.y,b.x,b.y,l,r)}
  pub fn add_convex_poly_filled(&self,p:&[ImVec2],c:u32){println!("[\"polygon\",{:?},{}]",p.iter().map(|v|[v.x,v.y]).collect::<Vec<_>>(),c)}
  pub fn add_triangle_filled(&self,a:ImVec2,b:ImVec2,d:ImVec2,c:u32){println!("[\"polygon\",{:?},{}]",vec![[a.x,a.y],[b.x,b.y],[d.x,d.y]],c)}
  pub fn add_polyline(&self,p:&[ImVec2],c:u32,t:f32,_:ImDrawFlags){println!("[\"polyline\",{:?},{},{}]",p.iter().map(|v|[v.x,v.y]).collect::<Vec<_>>(),c,t)}
  pub fn push_clip_rect(&self,a:ImVec2,b:ImVec2,_:bool){println!("[\"clip\",{},{},{},{}]",a.x,a.y,b.x,b.y)}pub fn pop_clip_rect(&self){println!("[\"unclip\"]")}
  pub fn add_text_with_font(&self,_:&ImFont,s:f32,p:ImVec2,c:u32,t:&str){println!("[\"text\",{},{},{},{},{:?}]",p.x,p.y,s,c,t)}
  pub fn add_text(&self,p:ImVec2,c:u32,t:&str){self.add_text_with_font(&ImFont,25.,p,c,t)}
 }
}
mod icons{
 use super::*;include!(concat!(env!("WISHLIST_SRC"),"/icons_generated.rs"));include!(concat!(env!("WISHLIST_SRC"),"/item_text_generated.rs"));
 pub const Y:usize=110;pub const A:usize=107;pub const B:usize=108;pub const X:usize=109;pub const L3:usize=111;pub const R3:usize=112;pub const ZL:usize=113;pub const ZR:usize=114;pub const DPAD:usize=115;pub const DPAD_LEFT:usize=118;pub const L:usize=116;pub const R:usize=117;
 pub fn init(){}pub fn set_layout(_:usize){}
 pub fn draw(_:imgui::DrawList,t:usize,x:f32,y:f32,w:f32,h:f32,c:u32,f:bool){println!("[\"icon\",{},{},{},{},{},{},{}]",t,x,y,w,h,c,f)}
 pub fn equipment(k:u8)->usize{*EQUIP.get(k as usize).unwrap_or(&46)}pub fn material(id:u16)->(usize,u32){let r=ITEMS[id as usize];(r.0 as usize,PALETTE[r.1 as usize])}
}
mod text_data{pub fn init(){}pub fn description(k:u8,id:u16,l:usize)->&'static str{super::text_blob::get(include_bytes!(concat!(env!("WISHLIST_SRC"),"/../assets/ui-text.bin")),k,id,l)}}
mod native_badges{pub struct Marker{pub x:f32,pub y:f32}pub fn available()->bool{false}pub fn markers(_: &super::List,_:u32)->Vec<Marker>{vec![]}}
mod game_menu{pub fn visible(_:u32)->bool{false}}
mod sound{pub enum Cue{Open}pub fn cue(_:Cue){}}use sound::Cue;
struct State{sort:Option<sorting::Dialog>,installed:bool,loaded:bool,game_ready:bool,base:u32,mode:u8,list:List,stock:[u32;4096],stock_ready:bool,aggregate:interaction::AggregateFocus,selected:usize,focus:interaction::Focus,preview:bool,preview_panel:usize,preview_clock:f32,frozen:Option<Recipe>,detail_page:usize,tab:usize,status:u8,notices:notifications::Queue,notice_demo:Option<notifications::Demo>,notice_test_case:usize}
impl State{fn active(&self)->Option<Recipe>{None}fn at_smithy(&self)->bool{false}}
thread_local!{static STORE:RefCell<Option<State>>=const{RefCell::new(None)};}
struct Shared;static STATE:Shared=Shared;impl Shared{fn with<R>(&self,f:impl FnOnce(&mut State)->R)->R{STORE.with(|v|f(v.borrow_mut().as_mut().unwrap()))}}
fn main(){
 let args:Vec<String>=std::env::args().collect();let screen=args.get(1).map_or("register",String::as_str);let lang=args.get(2).and_then(|s|s.parse().ok()).unwrap_or(4);
 let mut list=List::default();list.language=lang;list.ui_scale=args.get(3).and_then(|s|s.parse().ok()).unwrap_or(100);
 for (i,(kind,id)) in [(1,105),(2,2),(14,1),(9,1),(22,1),(25,9),(2,4)].iter().enumerate(){if let Some(r)=g::RECIPES.iter().find(|r|r.kind==*kind&&r.id==*id){list.slots[i]=Some(model::Entry{recipe:*r,ready:None});}}
 let r=g::RECIPES.iter().find(|r|r.kind==1&&name(**r,0).contains("Rathalos")).copied().unwrap_or(g::RECIPES[0]);list.slots[0]=Some(model::Entry{recipe:r,ready:None});
 let mut stock=[99;4096];stock[r.materials[0].id as usize]=0;
 let frozen=g::RECIPES.iter().find(|r|r.kind==7).copied();
 let mut s=State{sort:None,installed:true,loaded:true,game_ready:true,base:0,mode:2,list,stock,stock_ready:true,aggregate:Default::default(),selected:0,focus:Default::default(),preview:false,preview_panel:0,preview_clock:0.,frozen,detail_page:0,tab:0,status:0,notices:notifications::Queue::default(),notice_demo:None,notice_test_case:2};
 match screen{"sort-primary"=>s.sort=Some(sorting::Dialog::default()),"sort-secondary"=>s.sort=Some(sorting::Dialog{secondary:true,criterion:1,..Default::default()}),"sort-wait"=>s.sort=Some(sorting::Dialog{secondary:true,criterion:1,message:1,..Default::default()}),"register"=>{s.mode=1;s.selected=7},"register-filled"=>s.mode=1,"materials"=>s.tab=1,"inspect"=>s.focus.material=true,"skills"=>s.detail_page=1,"confirm"=>s.mode=4,"done"=>s.mode=5,"overwrite"=>s.mode=3,"quick-remove"=>s.mode=8,"notice"=>{s.mode=0;s.notices.push(r);},"notice-test"=>start_notice_demo(&mut s),"weapon"=>s.selected=2,"category"=>{if let Some(r)=g::RECIPES.iter().find(|r|r.group>0){s.list.slots[0]=Some(model::Entry{recipe:*r,ready:None})}},_=>()}
 if screen=="materials-long"{let r=*g::RECIPES.iter().find(|r|r.kind==9&&r.id==51&&r.materials.iter().any(|m|g::ITEMDATA.get(m.id as usize*2).is_some_and(|v|v[0]=="Gore Magala Feeler+"))).unwrap();s.list.slots.fill(None);s.list.slots[0]=Some(model::Entry{recipe:r,ready:None});s.stock.fill(0);s.tab=1;}
 if screen=="braces"{let r=*g::RECIPES.iter().find(|r|r.kind==3&&r.id==3&&r.level==0).unwrap();s.list.slots.fill(None);s.list.slots[0]=Some(model::Entry{recipe:r,ready:None});}
 if screen=="long-equipment"{let r=*g::RECIPES.iter().max_by(|a,b|imgui::calc_text_size(name(**a,lang),false,0.).x.total_cmp(&imgui::calc_text_size(name(**b,lang),false,0.).x)).unwrap();s.list.slots.fill(None);s.list.slots[0]=Some(model::Entry{recipe:r,ready:None});}
 if screen=="long-material"{let r=*g::RECIPES.iter().max_by(|a,b|{let widest=|r:&Recipe|r.totals().iter().map(|m|imgui::calc_text_size(item(m.id,lang),false,0.).x as u32).max().unwrap_or(0);widest(a).cmp(&widest(b))}).unwrap();s.list.slots.fill(None);s.list.slots[0]=Some(model::Entry{recipe:r,ready:None});s.focus.material=true;}
 STORE.with(|v|*v.borrow_mut()=Some(s));unsafe{FONT=Some(imgui::ImFont)}
 if screen=="prompts"{let cv=Canvas::new().unwrap();cv.bar(1102.,528.,160.,icons::R3,l::QUICK_REMOVE[lang]);cv.bar(1102.,564.,160.,icons::R3,l::REGISTER[lang]);return}
 render();
}
'''
(OUT/'capture.rs').write_text(modules+'\n'+stub+'\n'+canvas+'\n'+draw+'\n'+render,encoding='utf-8')
import os
env=os.environ.copy();env['WISHLIST_SRC']=(ROOT/'src').as_posix()
subprocess.run(['rustc','+nightly','--edition','2024',str(OUT/'capture.rs'),'-Awarnings','-o',str(OUT/'capture.exe')],env=env,check=True)
sprite_source=(ROOT/'src/icons_generated.rs').read_text(encoding='utf-8');sprites=[tuple(map(int,m)) for m in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}',sprite_source)]
atlas=Image.frombytes('RGBA',(2048,1024),(ROOT/'assets/wishlist-atlas.rgba').read_bytes())
def color(c):return (c&255,c>>8&255,c>>16&255,c>>24&255)
def paint(commands,width=1280,height=720):
 im=Image.new('RGBA',(width,height),'#515e51');clips=[]
 for cmd in commands:
  typ,*a=cmd
  if typ=='clip':clips.append(a);continue
  if typ=='unclip':clips.pop();continue
  layer=Image.new('RGBA',im.size);d=ImageDraw.Draw(layer)
  if typ in ('rect','outline','line'):
   x,y,x2,y2,c,*width=a;box=(x,y,x2,y2)
   if typ=='rect':d.rectangle(box,fill=color(c))
   elif typ=='outline':d.rectangle(box,outline=color(c),width=max(1,round(width[0])))
   else:d.line(box,fill=color(c),width=max(1,round(width[0])))
  elif typ=='circle':
   x,y,r,c=a;d.ellipse((x-r,y-r,x+r,y+r),fill=color(c))
  elif typ=='gradient':
   x,y,x2,y2,l,r=a;l=color(l);r=color(r)
   for px in range(round(x),round(x2)):
    ratio=(px-x)/max(1,x2-x);d.line((px,y,px,y2),fill=tuple(round(l[i]+(r[i]-l[i])*ratio) for i in range(4)))
  elif typ in ('polygon','polyline'):
   p,c,*w=a;p=[tuple(v) for v in p]
   if typ=='polygon':d.polygon(p,fill=color(c))
   else:d.line(p+[p[0]],fill=color(c),width=max(1,round(w[0])))
  elif typ=='text':
   x,y,size,c,text=a;f=ImageFont.truetype(str(ROOT/'assets/NotoSansCJKkr-Regular.otf'),max(1,round(size*1000/denom)))
   d.text((x,y),text,font=f,fill=color(c))
  elif typ=='icon':
   tile,x,y,w,h,c,flip=a;sx,sy,sw,sh=sprites[tile];pic=atlas.crop((sx,sy,sx+sw,sy+sh)).resize((max(1,round(w)),max(1,round(h))),Image.Resampling.LANCZOS)
   if flip:pic=pic.transpose(Image.Transpose.FLIP_LEFT_RIGHT)
   pic=ImageChops.multiply(pic,Image.new('RGBA',pic.size,color(c)));layer.alpha_composite(pic,(round(x),round(y)))
  if clips:
   x,y,x2,y2=clips[-1];mask=Image.new('L',im.size);ImageDraw.Draw(mask).rectangle((x,y,x2,y2),fill=255);layer.putalpha(ImageChops.multiply(layer.getchannel('A'),mask))
  im.alpha_composite(layer)
 return im.convert('RGB')
screens=['register','register-filled','list','braces','materials','materials-long','prompts','inspect','skills','confirm','done','overwrite','notice','notice-test','weapon','category','quick-remove','sort-primary','sort-secondary','sort-wait']
for screen in screens:
 raw=subprocess.check_output([str(OUT/'capture.exe'),screen,'4'],text=True,encoding='utf-8');commands=[json.loads(s) for s in raw.splitlines()]
 (OUT/(screen+'.json')).write_text(json.dumps(commands,ensure_ascii=False),encoding='utf-8');paint(commands).save(OUT/(screen+'.png'))
 print(screen,len(commands),'draw commands')
print('Offline draw proofs:',OUT)
for screen in ['register','confirm','done','overwrite','notice','list','braces','materials-long','prompts','sort-primary','sort-secondary','sort-wait']:
 raw=subprocess.check_output([str(OUT/'capture.exe'),screen,'0'],text=True,encoding='utf-8')
 commands=[json.loads(s) for s in raw.splitlines()]
 (OUT/(screen+'.en.json')).write_text(json.dumps(commands),encoding='utf-8');paint(commands).save(OUT/(screen+'.en.png'))
adv=struct.unpack('<65536f',(OUT/'advances.bin').read_bytes());problems=[]
locale_source=(ROOT/'src/locales.rs').read_text(encoding='utf-8')
sort_labels=[json.loads(re.search(r'pub const '+key+r':\[Label;[36]\]=(\[.*?\]);',locale_source,re.S).group(1)) for key in ['SORT_CRITERIA','SORT_CATEGORIES']]
tab_labels=[json.loads(re.search(r'pub const '+key+r':Label=(\[.*?\]);',locale_source).group(1)) for key in ['LIST','TOTAL']]
def label(key):return json.loads(re.search(r'pub const '+key+r':Label=(\[.*?\]);',locale_source).group(1))
labels={key:label(key) for key in ['REGISTER_ACTION','NO_EQUIPMENT','DONE','CLOSE','YES','NO','NOTICE','TITLE','DELETE_QUESTION','REPLACE_QUESTION']}
def ink_box(x,y,size,text):
 pen=0.;left=float('inf');right=float('-inf');top=float('inf');bottom=float('-inf')
 for ch in text:
  key=cmap.get(ord(ch));by=bounds.get(key,(0.,0.));bx=horizontal.get(key,(0.,0.))
  if by[1]>by[0]:left=min(left,x+(pen+bx[0])*size);right=max(right,x+(pen+bx[1])*size);top=min(top,y+by[0]*size);bottom=max(bottom,y+by[1]*size)
  pen+=adv[ord(ch)] if ord(ch)<65536 else .5
 return (left,top,right,bottom) if left<=right else (x,y,x+pen*size,y+size)
centred=0;long_checked=False;prompt_checked=False;modal_checks=0;notice_checks=0;pager_checks=0;colour_checks=0
for lang in range(9):
 for screen in screens:
  raw=subprocess.check_output([str(OUT/'capture.exe'),screen,str(lang)],text=True,encoding='utf-8');clips=[]
  commands=list(map(json.loads,raw.splitlines()))
  if screen.startswith('sort-'):
   secondary=screen!='sort-primary';left=298. if secondary else 418.;right=662.
   for i,label in enumerate(sort_labels[0]):
    row=next(a for typ,*a in commands if typ=='text' and a[4]==label[lang])
    icon=next(a for typ,*a in commands if typ=='icon' and a[1]==left+10. and a[2]==286.+i*62.)
    assert row[0]>=icon[1]+icon[3]+10.,(lang,screen,'primary icon overlaps label')
   if secondary:
    for i,label in enumerate(sort_labels[1]):
     row=next(a for typ,*a in commands if typ=='text' and a[4]==label[lang])
     icon=next(a for typ,*a in commands if typ=='icon' and a[1]==right+(9. if i==0 else 8.) and abs(a[2]-(281.+i*44. if i==0 else 271.+i*44.))<.01)
     assert row[0]>=icon[1]+icon[3]+8.,(lang,screen,'secondary icon overlaps label')
   else:assert all(label[lang] not in [a[-1] for a in commands if a[0]=='text'] for label in sort_labels[1]),(lang,'secondary choices shown too early')
  for command in commands:
   typ,*a=command
   if typ=='clip':clips.append(a)
   elif typ=='unclip':clips.pop()
   elif typ=='text':
    x,y,size,c,text=a;width=sum(adv[ord(ch)] if ord(ch)<65536 else .5 for ch in text)*size
    if clips and x+width>clips[-1][2]+2:problems.append({'screen':screen,'language':lang,'text':text,'excess':round(x+width-clips[-1][2],2)})
    ink=[bounds[cmap[ord(ch)]] for ch in text if ord(ch) in cmap and not ch.isspace() and bounds[cmap[ord(ch)]][1]>bounds[cmap[ord(ch)]][0]]
    box=ink_box(x,y,size,text);ink_center=(box[1]+box[3])*.5;ink_x=(box[0]+box[2])*.5
    for tab,expected_x in [(0,355.5),(1,710.)]:
     if text==tab_labels[tab][lang] and 120<y<160:
      assert abs(ink_x-expected_x)<.6,(screen,lang,text,'horizontal centre')
      assert abs(ink_center-142.5)<.6,(screen,lang,text,'vertical centre')
      centred+=1
    if screen=='category' and clips and abs(clips[-1][1]-380.)<.01 and abs(clips[-1][3]-419.)<.01:
     assert abs(ink_center-399.5)<.6,(lang,text,'category material row centre');centred+=1
    if screen=='materials-long' and lang==0 and x==188 and text=='Gore Magala Feeler+':
     assert abs((ink_center-215)/35-round((ink_center-215)/35))<.01,'Long material left its row centre'
     centred+=1;long_checked=True
    if screen=='prompts' and lang==0 and text=='Remove':
     assert size>=23 and abs(ink_center-542)<.6,'Removal prompt is small or not centred'
     centred+=1;prompt_checked=True
    if screen=='register' and text==labels['REGISTER_ACTION'][lang]:
     assert abs(ink_x-714)<.6,(lang,'registration action')
     icon=next(a for typ,*a in commands if typ=='icon' and a[0]==107 and a[2]==362.)
     assert abs(icon[1]+icon[3]*.5-ink_x)<.6,(lang,'A icon')
     modal_checks+=1
    if screen=='register' and text==labels['NO_EQUIPMENT'][lang]:
     assert abs(ink_x-1117)<.6 and abs(ink_center-344.5)<.6,(lang,'empty equipment')
     modal_checks+=1
    if screen in ['confirm','overwrite','done','quick-remove']:
     question=labels['DONE'][lang] if screen=='done' else labels['REPLACE_QUESTION'][lang] if screen=='overwrite' else labels['DELETE_QUESTION'][lang]
     if text in question.splitlines():assert abs(ink_x-640)<.6,(lang,screen,'question');modal_checks+=1
     for key,expected_x in [('YES',536),('NO',744),('CLOSE',640)]:
      if text==labels[key][lang]:assert abs(ink_x-expected_x)<.6,(lang,screen,key);modal_checks+=1
    if screen=='notice' and text==labels['NOTICE'][lang]:
     toast=next(a for typ,*a in commands if typ=='rect' and abs(a[3]-a[1]-80)<.1 and a[4]==0xe0282620)
     assert abs(ink_x-(toast[0]+toast[2])*.5)<.6,(lang,'notification centre')
     title=next(a for typ,*a in commands if typ=='text' and a[4].startswith(labels['TITLE'][lang]+' \"'))
     title_width=sum(adv[ord(ch)] for ch in title[4])*title[2]
     expected=min(max(title_width+40,width),876)+32
     assert abs(toast[2]-toast[0]-expected)<1,(lang,'excess notification padding')
     notice_checks+=1
    if re.fullmatch(r'◀ \d+ / \d+ ▶',text):
     top=580. if ink_center>550 else 484.;expected=306. if top==580. else 336.
     left=next(a for typ,*a in commands if typ=='icon' and a[0]==118 and a[2]==top)
     right=next(a for typ,*a in commands if typ=='icon' and a[0]==115 and a[2]==top)
     assert abs(ink_x-expected)<.6 and abs(ink_center-top-12.5)<.6,(lang,screen,'page centre')
     assert abs((box[0]-left[1]-left[3])-(right[1]-box[2]))<1,(lang,screen,'page icon gaps')
     pager_checks+=1
  if screen=='braces':
   gear=[a for typ,*a in commands if typ=='icon' and a[0]==73 and a[3] in (24.,27.)]
   assert len(gear)==2 and all(a[5]==0xffe6e6e6 for a in gear),(lang,'Bherna Braces icon colour')
   colour_checks+=len(gear)
(OUT/'text-audit.json').write_text(json.dumps({'screens':len(screens)*9,'width_overflows':problems},ensure_ascii=False,indent=2),encoding='utf-8')
print('Nine-language text audit:',len(problems),'width overflows')
assert not problems,problems
assert centred>=100,centred
assert long_checked and prompt_checked,'Required regression fixtures were not rendered'
print('Alignment audit:',centred,'tab, material and prompt checks passed')

assert modal_checks>=100 and notice_checks==18,(modal_checks,notice_checks)
print('Dialog/empty-slot/notification audit:',modal_checks+notice_checks,'checks passed')
# Reproduce Recomp's additional host font scale, which caused the live-only drift.
import os
probes=0
for lang in range(9):
 for screen in ['register','confirm','overwrite','done','notice','notice-test']:
  baseline=subprocess.check_output([str(OUT/'capture.exe'),screen,str(lang)])
  for factor in ['1.25','1.5','2.0']:
   environment=os.environ.copy();environment['PROOF_HOST_FONT_SCALE']=factor
   scaled=subprocess.check_output([str(OUT/'capture.exe'),screen,str(lang)],env=environment)
   assert baseline==scaled,(screen,lang,factor,'host font scale changed overlay layout')
   probes+=1
print('Host font scale audit:',probes,'comparisons passed')

assert pager_checks>=100 and colour_checks==18,(pager_checks,colour_checks)
print('Pager spacing / native icon colour audit:',pager_checks+colour_checks,'checks passed')

if '--matrix' in __import__('sys').argv:
 import html,math
 sizes=[(1280,720),(1920,1080),(2560,1600),(3440,1440),(5120,1440),(1024,768)]
 scales=[50,60,70,80,90,100]
 cases=['register','materials','category','confirm','notice','long-equipment','long-material','sort-primary','sort-secondary','sort-wait']
 failures=[];runs=0;gallery=[]
 for lang in range(9):
  for percent in scales:
   for width,height in sizes:
    for screen in cases:
     environment=os.environ.copy();environment['PROOF_WIDTH']=str(width);environment['PROOF_HEIGHT']=str(height)
     raw=subprocess.check_output([str(OUT/'capture.exe'),screen,str(lang),str(percent)],env=environment,text=True,encoding='utf-8')
     commands=list(map(json.loads,raw.splitlines()));clips=[]
     for typ,*a in commands:
      if typ=='clip':clips.append(a)
      elif typ=='unclip':assert clips;clips.pop()
      elif typ=='text':
       x,y,font_size,colour,text=a;box=ink_box(x,y,font_size,text)
       if not all(math.isfinite(v) for v in box) or box[0]<-3 or box[1]<-3 or box[2]>width+3 or box[3]>height+3 or (clips and (box[0]<clips[-1][0]-3 or box[2]>clips[-1][2]+3 or box[1]<clips[-1][1]-3 or box[3]>clips[-1][3]+3)):
        failures.append({'language':lang,'screen':screen,'scale':percent,'size':[width,height],'text':text,'ink':box,'clip':clips[-1] if clips else None})
      elif typ=='icon':
       _,x,y,w,h,*_=a
       if x < -3 or y < -3 or x+w>width+3 or y+h>height+3:failures.append({'language':lang,'screen':screen,'scale':percent,'size':[width,height],'icon_bounds':[x,y,w,h]})
     assert not clips,(lang,screen,'unbalanced clips')
     runs+=1
     # One real-data long-name sample per language, distributed across scales and aspect ratios.
     if screen=='long-equipment' and percent==scales[lang%len(scales)] and (width,height)==sizes[lang%len(sizes)]:
      filename=f'matrix-{lang}.png';im=paint(commands,width,height);im.thumbnail((1280,900));im.save(OUT/filename);gallery.append((filename,lang,percent,width,height))
  print('UI matrix language',lang+1,'/ 9 completed',flush=True)
 report={'kind':'offline production draw-code regression, not a live-game test','cases':runs,'languages':9,'resolutions':sizes,'scales':scales,'screens':cases,'failure_count':len(failures),'failures':failures[:100]}
 (OUT/'regression-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
 cards=''.join(f'<section><h2>{html.escape(language_name)} · {percent}% · {width}×{height}</h2><img src="{filename}" loading="lazy"></section>' for filename,lang,percent,width,height in gallery for language_name in [['English','简体中文','繁體中文','日本語','한국어','Deutsch','Français','Italiano','Español'][lang]])
 (OUT/'regression-gallery.html').write_text('<!doctype html><meta charset="utf-8"><title>Wishlist regression previews</title><style>body{background:#182020;color:#eee;font:16px sans-serif;margin:32px}section{margin:30px 0}img{max-width:100%;border:1px solid #697474}</style><h1>Wishlist UI regression previews</h1><p>Offline rendering of production draw code. These images do not prove native game hooks or live-game rendering. See regression-report.json and the live-game checklist.</p>'+cards,encoding='utf-8')
 print('UI matrix:',runs,'cases;',len(failures),'failures')
 assert not failures,failures[:5]
