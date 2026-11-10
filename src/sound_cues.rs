//! User-selected native UI sound IDs, shared by rendering and input handling.
#[derive(Clone,Copy)]
pub enum Cue{Open,Close,Cursor,Page,Tab,Inspect,Registered}
impl Cue{pub const fn id(self)->u32{match self{Self::Open=>1,Self::Close=>2,Self::Cursor=>4,Self::Page=>7,Self::Tab=>27,Self::Inspect=>6,Self::Registered=>31}}}
