"""Offline layout proof: run the real Rust draw code with a recording ImGui adapter.

This does not launch the game or claim to reproduce the runtime font rasterizer.
PNG text uses Noto's ascent/descent scale, matching ImGui's em-height convention.
"""
from pathlib import Path
import re,subprocess,json,struct,ast
from PIL import Image,ImageDraw,ImageFont,ImageChops
from fontTools.ttLib import TTFont

ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'target/ui-steel-proof';OUT.mkdir(parents=True,exist_ok=True)
font=TTFont(ROOT/'assets/NotoSansCJKkr-Regular.otf');cmap=font.getBestCmap();denom=font['hhea'].ascent-font['hhea'].descent
(OUT/'advances.bin').write_bytes(b''.join(struct.pack('<f',font['hmtx'][cmap[c]][0]/denom if c in cmap else .5) for c in range(65536)))
source=(ROOT/'src/ui.rs').read_text(encoding='utf-8')
canvas=source[source.index('static mut FONT'):source.index('fn save(')]
draw=source[source.index('fn materials('):source.index('pub fn render()')]
render=source[source.index('pub fn render()'):source.index('pub fn menu()')]
render='pub fn render(){let cv=Canvas::new().unwrap();STATE.with(|s|{'+render.split(' STATE.with(|s|{',1)[1]
modules='\n'.join(f'#[path="{(ROOT/"src"/(n+".rs")).as_posix()}"]mod {n};' for n in ['model','generated','locales','equipment','interaction','navigation','presentation','text_blob','notifications'])
stub=r'''

extern crate alloc;extern crate self as forge;
use alloc::{vec::Vec,format,string::String};
use std::cell::RefCell;
use model::{List,Recipe};use locales as l;use generated as g;
use sys::imgui::{ImVec2,ImDrawFlags};
pub mod sys{pub mod imgui{
 #[derive(Clone,Copy,Debug)]pub struct ImVec2{pub x:f32,pub y:f32}
 pub enum ImDrawFlags{None,Closed}
 pub struct Baked;pub struct Glyph{pub Y0:f32,pub Y1:f32}
 pub unsafe fn ImFont_GetFontBaked(_: *mut u8,_:f32,_:f32)->*mut Baked{std::ptr::null_mut()}
 pub unsafe fn ImFontBaked_FindGlyph(_: *mut Baked,_:u16)->*mut Glyph{std::ptr::null_mut()}
}}
pub mod imgui{
 use super::*;static ADV:&[u8]=include_bytes!("advances.bin");thread_local!{static SIZE:RefCell<f32>=const{RefCell::new(25.)};}
 #[derive(Clone,Copy)]pub struct ImFont;impl ImFont{pub fn raw(&self)->*mut u8{std::ptr::null_mut()}pub fn load(_: &std::ffi::CStr)->Option<Self>{Some(Self)}}
 pub fn push_font(_: &ImFont,s:Option<f32>){SIZE.with(|v|*v.borrow_mut()=s.unwrap_or(25.))}pub fn pop_font(){}
 pub fn calc_text_size(t:&str,_:bool,_:f32)->ImVec2{let s=SIZE.with(|v|*v.borrow());ImVec2{x:t.chars().map(|c|{let at=c as usize*4;if at+4<=ADV.len(){f32::from_le_bytes(ADV[at..at+4].try_into().unwrap())*s}else{s*0.5}}).sum(),y:s}}
 pub struct Io{pub DisplaySize:ImVec2,pub DeltaTime:f32}pub fn get_io()->Io{Io{DisplaySize:ImVec2{x:1280.,y:720.},DeltaTime:0.016}}
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
 pub const A:usize=107;pub const B:usize=108;pub const X:usize=109;pub const L3:usize=111;pub const R3:usize=112;pub const ZL:usize=113;pub const ZR:usize=114;pub const DPAD:usize=115;pub const DPAD_LEFT:usize=118;pub const L:usize=116;pub const R:usize=117;
 pub fn init(){}pub fn set_layout(_:usize){}
 pub fn draw(_:imgui::DrawList,t:usize,x:f32,y:f32,w:f32,h:f32,c:u32,f:bool){println!("[\"icon\",{},{},{},{},{},{},{}]",t,x,y,w,h,c,f)}
 pub fn equipment(k:u8)->usize{*EQUIP.get(k as usize).unwrap_or(&46)}pub fn material(id:u16)->(usize,u32){let r=ITEMS[id as usize];(r.0 as usize,PALETTE[r.1 as usize])}
}
mod text_data{pub fn init(){}pub fn description(k:u8,id:u16,l:usize)->&'static str{super::text_blob::get(include_bytes!(concat!(env!("WISHLIST_SRC"),"/../assets/ui-text.bin")),k,id,l)}}
mod native_badges{pub struct Marker{pub x:f32,pub y:f32}pub fn available()->bool{false}pub fn markers(_: &super::List,_:u32)->Vec<Marker>{vec![]}}
mod game_menu{pub fn visible(_:u32)->bool{false}}
mod sound{pub enum Cue{Open}pub fn cue(_:Cue){}}use sound::Cue;
struct State{installed:bool,loaded:bool,game_ready:bool,base:u32,mode:u8,list:List,stock:[u32;4096],stock_ready:bool,aggregate:interaction::AggregateFocus,selected:usize,focus:interaction::Focus,preview:bool,preview_panel:usize,preview_clock:f32,frozen:Option<Recipe>,detail_page:usize,tab:usize,status:u8,notices:notifications::Queue,notice_demo:Option<notifications::Demo>,notice_test_case:usize}
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
 let mut s=State{installed:true,loaded:true,game_ready:true,base:0,mode:2,list,stock,stock_ready:true,aggregate:Default::default(),selected:0,focus:Default::default(),preview:false,preview_panel:0,preview_clock:0.,frozen,detail_page:0,tab:0,status:0,notices:notifications::Queue::default(),notice_demo:None,notice_test_case:2};
 match screen{"register"=>{s.mode=1;s.selected=7},"register-filled"=>s.mode=1,"materials"=>s.tab=1,"inspect"=>s.focus.material=true,"skills"=>s.detail_page=1,"confirm"=>s.mode=4,"quick-remove"=>s.mode=8,"notice"=>{s.mode=0;s.notices.push(r);},"notice-test"=>start_notice_demo(&mut s),"weapon"=>s.selected=2,"category"=>{if let Some(r)=g::RECIPES.iter().find(|r|r.group>0){s.list.slots[0]=Some(model::Entry{recipe:*r,ready:None})}},_=>()}
 STORE.with(|v|*v.borrow_mut()=Some(s));unsafe{FONT=Some(imgui::ImFont)}render();
}
'''
(OUT/'capture.rs').write_text(modules+'\n'+stub+'\n'+canvas+'\n'+draw+'\n'+render,encoding='utf-8')
import os
env=os.environ.copy();env['WISHLIST_SRC']=(ROOT/'src').as_posix()
subprocess.run(['rustc','+nightly','--edition','2024',str(OUT/'capture.rs'),'-Awarnings','-o',str(OUT/'capture.exe')],env=env,check=True)
sprite_source=(ROOT/'src/icons_generated.rs').read_text(encoding='utf-8');sprites=[tuple(map(int,m)) for m in re.findall(r'Sprite\{x:(\d+),y:(\d+),w:(\d+),h:(\d+)\}',sprite_source)]
atlas=Image.frombytes('RGBA',(2048,1024),(ROOT/'assets/wishlist-atlas.rgba').read_bytes())
def color(c):return (c&255,c>>8&255,c>>16&255,c>>24&255)
def paint(commands):
 im=Image.new('RGBA',(1280,720),'#515e51');clips=[]
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
screens=['register','register-filled','list','materials','inspect','skills','confirm','notice','notice-test','weapon','category','quick-remove']
for screen in screens:
 raw=subprocess.check_output([str(OUT/'capture.exe'),screen,'4'],text=True,encoding='utf-8');commands=[json.loads(s) for s in raw.splitlines()]
 (OUT/(screen+'.json')).write_text(json.dumps(commands,ensure_ascii=False),encoding='utf-8');paint(commands).save(OUT/(screen+'.png'))
 print(screen,len(commands),'draw commands')
print('Offline draw proofs:',OUT)
adv=struct.unpack('<65536f',(OUT/'advances.bin').read_bytes());problems=[]
for lang in range(9):
 for screen in screens:
  raw=subprocess.check_output([str(OUT/'capture.exe'),screen,str(lang)],text=True,encoding='utf-8');clips=[]
  for command in map(json.loads,raw.splitlines()):
   typ,*a=command
   if typ=='clip':clips.append(a)
   elif typ=='unclip':clips.pop()
   elif typ=='text':
    x,y,size,c,text=a;width=sum(adv[ord(ch)] if ord(ch)<65536 else .5 for ch in text)*size
    if clips and x+width>clips[-1][2]+2:problems.append({'screen':screen,'language':lang,'text':text,'excess':round(x+width-clips[-1][2],2)})
(OUT/'text-audit.json').write_text(json.dumps({'screens':len(screens)*9,'width_overflows':problems},ensure_ascii=False,indent=2),encoding='utf-8')
print('Nine-language text audit:',len(problems),'width overflows')
