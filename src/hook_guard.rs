//! Accept an intact entry or an existing Forge absolute-jump hook with intact tail.
pub fn matches(words:[u32;4],expected:[u32;4],site:u32)->bool{
 if words==expected{return true}
 words[0]==0xe51ff004&&(0x03200000..0xeffff000).contains(&words[1])&&words[1]&3==0&&words[1]!=site&&words[2..]==expected[2..]
}
