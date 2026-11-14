//! Opt-in diagnostics use Forge's existing log sink, never another filesystem mount.
use core::sync::atomic::{AtomicBool,AtomicU32,Ordering};
#[cfg(test)]
static EMITTED:AtomicU32=AtomicU32::new(0);
#[cfg(test)]
pub fn emitted()->u32{EMITTED.load(Ordering::Relaxed)}
static ENABLED:AtomicBool=AtomicBool::new(false);
static INIT_OK:AtomicU32=AtomicU32::new(0);
static INIT_FAILED:AtomicU32=AtomicU32::new(0);
pub fn init_status(bit:u32,ok:bool){if ok{INIT_FAILED.fetch_and(!bit,Ordering::Relaxed);INIT_OK.fetch_or(bit,Ordering::Relaxed);}else{INIT_OK.fetch_and(!bit,Ordering::Relaxed);INIT_FAILED.fetch_or(bit,Ordering::Relaxed);}}
static CLOCK:AtomicU32=AtomicU32::new(0);
struct Channel{seen:AtomicBool,key:AtomicU32,time:AtomicU32,suppressed:AtomicU32}
static CHANNELS:[Channel;16]=[const{Channel{seen:AtomicBool::new(false),key:AtomicU32::new(0),time:AtomicU32::new(0),suppressed:AtomicU32::new(0)}};16];
/// First sample immediately; changed failures at most once/second, identical ones once/30s.
pub fn due(seen:bool,old:u32,key:u32,elapsed:u32)->bool{!seen||elapsed>=if old==key{30_000}else{1_000}}
pub fn enabled()->bool{ENABLED.load(Ordering::Relaxed)}
// Forge defaults to ERROR on some hosts. An explicit opt-in must still produce
// diagnostics, without changing the global level (and other plugins' output).
#[cfg(not(test))]
fn write(args:core::fmt::Arguments<'_>){
 match unsafe{forge::sys::log::forge_log_getLevel()}{
  forge::sys::log::Level::Error=>forge::log::prelude::error!("[wishlist-diag] {}",args),
  forge::sys::log::Level::Warn=>forge::log::prelude::warn!("[wishlist-diag] {}",args),
  _=>forge::log::prelude::info!("[wishlist-diag] {}",args),
 }
}

pub fn tick(dt:f32){if enabled()&&dt.is_finite(){CLOCK.fetch_add((dt.clamp(0.,1.)*1000.) as u32,Ordering::Relaxed);}}
pub fn set_enabled(on:bool)->bool{
 if ENABLED.swap(on,Ordering::Relaxed)==on{return false}
 for c in &CHANNELS{c.seen.store(false,Ordering::Relaxed);c.suppressed.store(0,Ordering::Relaxed);}
 #[cfg(not(test))]
 if on{write(format_args!("v{} diagnostic logging enabled (diagnostic records are not necessarily errors)",env!("CARGO_PKG_VERSION")));}
 if on{event(15,"initialization bits: native=1, badges=2, completion=4, menu=8, session=16, input=32, atlas=64, font=128; succeeded / failed",[INIT_OK.load(Ordering::Relaxed),INIT_FAILED.load(Ordering::Relaxed),0,0]);}
 true
}
/// Values are technical IDs/counts/results only; no names, save contents, or user paths.
pub fn event(channel:usize,message:&str,data:[u32;4]){
 if !enabled(){return}let Some(c)=CHANNELS.get(channel)else{return};
 let key=data.iter().fold(2166136261u32,|h,v|(h^v).wrapping_mul(16777619));let now=CLOCK.load(Ordering::Relaxed);
 if !due(c.seen.load(Ordering::Relaxed),c.key.load(Ordering::Relaxed),key,now.wrapping_sub(c.time.load(Ordering::Relaxed))){c.suppressed.fetch_add(1,Ordering::Relaxed);return}
 c.seen.store(true,Ordering::Relaxed);c.key.store(key,Ordering::Relaxed);c.time.store(now,Ordering::Relaxed);let skipped=c.suppressed.swap(0,Ordering::Relaxed);
 #[cfg(test)]
 EMITTED.fetch_add(1,Ordering::Relaxed);
 #[cfg(not(test))]
 write(format_args!("{}: {:?}; hex={:08X?}; suppressed={}",message,data,data,skipped));
}
