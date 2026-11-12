//! Serializable bindings and edge detection, independent of platform input APIs.
pub const COUNT:usize=2;
pub const EVENTS:[u32;COUNT]=[2,1];
pub const DEFAULT_PAD:[u32;COUNT]=[48,32];
pub const DEFAULT_KEYS:[u32;COUNT]=[53,62];
pub const PAD_MASK:u32=0xffffff;
pub fn valid_key(v:u32)->bool{let k=v&255;v&!0x700ff==0&&((4..=101).contains(&k)||k==103)&&k!=73}
pub fn matches(raw:u32,chord:u32)->bool{chord!=0&&raw&chord==chord}
// Only the most specific matching chord fires (L3+R3 suppresses plain R3).
pub fn active(raw:u32,chords:&[u32;COUNT])->u32{let mut out=0;for i in 0..COUNT{if matches(raw,chords[i])&&!chords.iter().any(|other|*other!=chords[i]&&*other&chords[i]==chords[i]&&matches(raw,*other)){out|=1<<i;}}out}
#[derive(Clone,Copy,Default)]
pub struct Edges{held:u32}
impl Edges{pub const fn new()->Self{Self{held:0}}pub fn step(&mut self,current:u32)->u32{let e=current&!self.held;self.held=current;e}pub fn block(&mut self){self.held=u32::MAX;}}
pub fn events(bits:u32)->u32{EVENTS.iter().enumerate().filter(|(i,_)|bits&(1<<i)!=0).fold(0,|a,(_,v)|a|*v)}
#[derive(Clone,Copy,Default)]
pub struct KeyboardGuard{waiting:bool}
impl KeyboardGuard{pub fn waiting(&self)->bool{self.waiting}pub const fn new()->Self{Self{waiting:false}}pub fn filter(&mut self,raw:u32,keyboard:bool)->u32{if keyboard{self.waiting=true;}if self.waiting{if !keyboard&&raw==0{self.waiting=false;}0}else{raw}}}

#[derive(Clone,Copy)]
pub struct Capture{pub target:u8,pub ready:bool,pub candidate:u32,pub done:bool,pub last:u32}
impl Capture{
    pub const EMPTY:Self=Self{target:0,ready:false,candidate:0,done:false,last:0};
    pub fn has_binding(&self)->bool{self.candidate!=0&&(self.target<=COUNT as u8||self.candidate&255!=0)}
    pub fn sample(&mut self,raw:u32){
        if raw==0{self.ready=true;self.done=self.candidate!=0;self.last=0;return}
        if !self.ready{return}
        let changed=if self.target<=COUNT as u8{raw&!self.last!=0}else{
            (raw&255!=0&&raw&255!=self.last&255)||raw&!self.last&0x70000!=0
        };
        // Adding a key updates the chord. Releasing part of it keeps the full
        // candidate; starting another gesture replaces it instead of locking it.
        if self.done||self.candidate==0||changed{self.candidate=raw;self.done=false;}
        self.last=raw;
    }
}

