//! Accept an intact entry or an existing Forge absolute-jump hook with intact tail.
//! Another mod may have hooked the site first (Ryujinx loads mods by folder name), and
//! Forge chains original!() through that detour, so its jump is accepted wherever the
//! plugin was loaded: any non-zero, word-aligned target other than the site itself.
fn forge_jump(first:u32,target:u32,site:u32)->bool{first==0xe51ff004&&target!=0&&target&3==0&&target!=site}
pub fn matches(words:[u32;4],expected:[u32;4],site:u32)->bool{
 if words==expected{return true}
 forge_jump(words[0],words[1],site)&&words[2..]==expected[2..]
}
/// A site checked by its first word only: that word, or a Forge jump in its place.
pub fn entry(words:[u32;2],expected:u32,site:u32)->bool{words[0]==expected||forge_jump(words[0],words[1],site)}
