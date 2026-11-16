//! Read-only view for Better Skillinfo: which armor piece the detail page 2 shows.
//! No hooks, no imports: Better Skillinfo finds this block once by its magic in
//! Wishlist's data segment and reads it. Rewritten every rendered frame.
//!
//! Both draw on ImGui's foreground list, which ImGui windows can never cover, and
//! other plugins (KatSmithy) draw there too. So Better Skillinfo leaves a draw
//! callback in `draw`, and Wishlist calls it right after its own window: the panel
//! lands just above Wishlist whatever the plugin order.
use core::{cell::UnsafeCell,ptr::addr_of_mut};
pub const MAGIC:[u8;16]=*b"MHGU-WL-SKILLV01";
#[repr(C)]
pub struct View{
 magic:[u8;16],version:u32,
 /// Wishlist's own window is on screen (any tab/page).
 open:u32,
 /// Detail page 2 of an armor piece is on screen; kind/id/level are valid.
 active:u32,kind:u32,id:u32,level:u32,
 /// Screen pixels (ImGui display coordinates): the panel's right edge, top and bottom.
 right:f32,top:f32,bottom:f32,
 serial:u32,
 /// Written by Better Skillinfo: `extern "C" fn()` drawing its panel, or 0.
 draw:u32,
}
struct Shared(UnsafeCell<View>);
unsafe impl Sync for Shared{}
#[used]
static VIEW:Shared=Shared(UnsafeCell::new(View{magic:MAGIC,version:2,open:0,active:0,kind:0,id:0,level:0,right:0.,top:0.,bottom:0.,serial:0,draw:0}));
fn with<R>(f:impl FnOnce(*mut View)->R)->R{f(VIEW.0.get())}
/// Start of every render: nothing is shown until this frame says so.
pub fn begin_frame(){with(|v|unsafe{
 addr_of_mut!((*v).active).write_volatile(0);addr_of_mut!((*v).open).write_volatile(0);
 let n=addr_of_mut!((*v).serial);n.write_volatile(n.read_volatile().wrapping_add(1));
})}
pub fn open(){with(|v|unsafe{addr_of_mut!((*v).open).write_volatile(1)})}
pub fn hide(){with(|v|unsafe{addr_of_mut!((*v).active).write_volatile(0)})}
pub fn show(kind:u8,id:u16,level:u8,right:f32,top:f32,bottom:f32){
 if !matches!(kind,1..=5){return}
 with(|v|unsafe{
  addr_of_mut!((*v).kind).write_volatile(kind as u32);addr_of_mut!((*v).id).write_volatile(id as u32);addr_of_mut!((*v).level).write_volatile(level as u32);
  addr_of_mut!((*v).right).write_volatile(right);addr_of_mut!((*v).top).write_volatile(top);addr_of_mut!((*v).bottom).write_volatile(bottom);
  // Fields first, the flag last: a reader never sees a half-written piece.
  addr_of_mut!((*v).active).write_volatile(1);
 })
}
/// After Wishlist's window is complete: let the linked panel draw above it.
pub fn overlay(){
 let (active,draw)=with(|v|unsafe{(addr_of_mut!((*v).active).read_volatile(),addr_of_mut!((*v).draw).read_volatile())});
 if active!=1||draw==0{return}
 let callback:extern "C" fn()=unsafe{core::mem::transmute(draw as usize)};
 callback();
}
