//! Bounded, value-only font metrics and paragraph layouts. No glyph pointers survive a call.
extern crate alloc;
use alloc::{string::String,vec::Vec};
use core::{cell::UnsafeCell,sync::atomic::{AtomicBool,Ordering}};
pub const METRIC_LIMIT:usize=128;pub const LAYOUT_LIMIT:usize=16;pub const TEXT_BUDGET:usize=16*1024;
#[derive(Clone,Copy,Default,Debug,PartialEq)]pub struct Metrics{pub width:f32,pub ink:Option<[f32;4]>}
#[derive(Clone,Copy,PartialEq,Eq)]pub struct MetricKey{pub font:usize,pub pixels:u32}
#[derive(Clone,Copy,PartialEq,Eq)]pub struct LayoutKey{pub font:usize,pub scale:u32,pub width:u32,pub height:u32,pub size:u32}
pub struct Layout{pub size:f32,pub lines:Vec<String>}
struct MetricEntry{key:MetricKey,text:String,value:Metrics,used:u64}
struct LayoutEntry{key:LayoutKey,text:String,value:Layout,used:u64}
#[derive(Default)]pub struct MetricsCache{entries:Vec<MetricEntry>,clock:u64}
impl MetricsCache{
 pub fn get(&mut self,key:MetricKey,text:&str,build:impl FnOnce()->Metrics)->Metrics{
  self.clock=self.clock.wrapping_add(1);if self.clock==0{for e in &mut self.entries{e.used=0;}self.clock=1;}
  if let Some(entry)=self.entries.iter_mut().find(|e|e.key==key&&e.text==text){entry.used=self.clock;return entry.value}
  let value=build();if text.len()>TEXT_BUDGET{return value}
  while self.entries.len()>=METRIC_LIMIT||self.entries.iter().map(|e|e.text.len()).sum::<usize>()+text.len()>TEXT_BUDGET{let at=self.entries.iter().enumerate().min_by_key(|(_,e)|e.used).unwrap().0;self.entries.swap_remove(at);}
  self.entries.push(MetricEntry{key,text:text.into(),value,used:self.clock});value
 }
}
#[derive(Default)]pub struct LayoutsCache{entries:Vec<LayoutEntry>,clock:u64}
impl LayoutsCache{
 pub fn with<R>(&mut self,key:LayoutKey,text:&str,build:impl FnOnce()->Layout,draw:impl FnOnce(&Layout)->R)->R{
  self.clock=self.clock.wrapping_add(1);if self.clock==0{for e in &mut self.entries{e.used=0;}self.clock=1;}
  if let Some(entry)=self.entries.iter_mut().find(|e|e.key==key&&e.text==text){entry.used=self.clock;return draw(&entry.value)}
  let value=build();let bytes=text.len()+value.lines.iter().map(|s|s.len()).sum::<usize>();if bytes>TEXT_BUDGET{return draw(&value)}
  while self.entries.len()>=LAYOUT_LIMIT||self.entries.iter().map(|e|e.text.len()+e.value.lines.iter().map(|s|s.len()).sum::<usize>()).sum::<usize>()+bytes>TEXT_BUDGET{let at=self.entries.iter().enumerate().min_by_key(|(_,e)|e.used).unwrap().0;self.entries.swap_remove(at);}
  self.entries.push(LayoutEntry{key,text:text.into(),value,used:self.clock});draw(&self.entries.last().unwrap().value)
 }
}
struct Shared<T>{lock:AtomicBool,value:UnsafeCell<T>}
unsafe impl<T:Send> Sync for Shared<T>{}
impl<T> Shared<T>{fn with<R>(&self,f:impl FnOnce(&mut T)->R)->R{while self.lock.compare_exchange_weak(false,true,Ordering::Acquire,Ordering::Relaxed).is_err(){core::hint::spin_loop()}let result=f(unsafe{&mut*self.value.get()});self.lock.store(false,Ordering::Release);result}}
static METRICS:Shared<MetricsCache>=Shared{lock:AtomicBool::new(false),value:UnsafeCell::new(MetricsCache{entries:Vec::new(),clock:0})};
static LAYOUTS:Shared<LayoutsCache>=Shared{lock:AtomicBool::new(false),value:UnsafeCell::new(LayoutsCache{entries:Vec::new(),clock:0})};
pub fn clear(){METRICS.with(|c|*c=MetricsCache::default());LAYOUTS.with(|c|*c=LayoutsCache::default());}
pub fn metrics(key:MetricKey,text:&str,build:impl FnOnce()->Metrics)->Metrics{METRICS.with(|cache|cache.get(key,text,build))}
pub fn paragraph<R>(key:LayoutKey,text:&str,build:impl FnOnce()->Layout,draw:impl FnOnce(&Layout)->R)->R{LAYOUTS.with(|cache|cache.with(key,text,build,draw))}
