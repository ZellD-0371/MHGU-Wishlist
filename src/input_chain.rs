//! Attach each HID import once. A later wrapper may already call us.
// Native modal input is read and withheld at HID level. Only Recomp needs
// its post-remapping menu hook before attaching the HID shortcut observer.
pub fn menu_ready(recomp:bool, installed:bool)->bool{!recomp||installed}
// Forge's native keyboard snapshot is not an authoritative source of the
// emulator host's focus/key-release state. Recomp supplies that host state.
pub fn keyboard_blocks_pad(recomp:bool,keyboard:bool)->bool{recomp&&keyboard}
// Menu translator +3DE4F8 hooks form a chain in load order. A hook further in
// (KatSmithy's tree navigation, for example) reads its input before an outer
// hook sees the result, so blocking the returned word is too late. While the
// wishlist owns input, the chain below receives no D-pad/stick bits at all, and
// a direction held when ownership ends stays withheld until it is released.
pub const NAV_WORD:u32=0x00f0_00f0;
const NAV_ACTION:u32=0xf0;
// The game's translator body: stick/D-pad high bits fold into action bits 0xF0.
pub fn translate(word:u32)->u32{word|((word>>16)&NAV_ACTION)}
pub fn inner_word(word:u32,owned:bool,latch:&mut u32)->u32{
 *latch&=word;
 if owned{*latch|=word&NAV_WORD;}
 word&!*latch
}
// The withheld directions come from the raw word; other actions keep the
// chain's mapping, including the host's optional action layout.
pub fn owned_actions(mapped:u32,word:u32,raw_only:bool)->u32{
 // Raw and translated directions share one order (0x10 up, 0x20 right,
 // 0x40 down, 0x80 left), so the raw word's directions are used as they are.
 let nav=translate(word)&NAV_ACTION;
 // A navigation takeover below us (KatSmithy) clears the game's word and refills
 // it from its own Forge input, which we withhold while open: nothing comes back.
 // Then every action, not only directions, comes from the raw word.
 if raw_only{return (translate(word)&!NAV_ACTION)|nav}
 (mapped&!NAV_ACTION)|nav
}
pub fn can_attach(saved: u32, candidate: u32, unresolved: u32, own: &[u32]) -> bool {
    saved == 0
        && candidate >= 0x10000
        && candidate & 3 == 0
        && candidate != unresolved
        && !own.contains(&candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    const GAME: u32 = 0x03362028;
    const WISHLIST: u32 = 0x0395d6cc;
    const SKILLINFO: u32 = 0x03bb3c24;
    const UNRESOLVED: u32 = 0x015efd44;

    // A chained hook below us that consumes navigation, like KatSmithy's tree.
    fn nav_consumer(word:u32,moved:&mut u32)->u32{
        let mapped=translate(word);
        if mapped&NAV_ACTION!=0{*moved+=1;}
        mapped&!NAV_ACTION
    }

    #[test]
    fn owned_directions_never_reach_an_inner_navigation_hook(){
        let mut latch=0;let mut moved=0;
        for word in [0x40,0x0040_0000,0x0010_0010,0x80|0x2000]{
            let mapped=nav_consumer(inner_word(word,true,&mut latch),&mut moved);
            let actions=owned_actions(mapped,word,false);
            assert_eq!(actions&NAV_ACTION,translate(word)&NAV_ACTION);
        }
        assert_eq!(moved,0);
        // Non-direction actions still come from the chain (host layout preserved).
        assert_eq!(owned_actions(0x4000,0x2000,false)&0x6000,0x4000);
    }

    // KatSmithy-like takeover: clears the word and refills it only from its own
    // (withheld) input, so the chain returns nothing while the wishlist is open.
    fn takeover(_word:u32)->u32{0}

    #[test]
    fn takeover_below_us_leaves_every_action_to_the_raw_word(){
        let mut latch=0;
        for word in [0x2000,0x4000,0x0040_0000,0x8000]{
            let mapped=takeover(inner_word(word,true,&mut latch));
            assert_eq!(owned_actions(mapped,word,true),translate(word));
        }
    }

    #[test]
    fn direction_held_when_ownership_ends_is_withheld_until_released(){
        let mut latch=0;
        assert_eq!(inner_word(0x40|0x2000,true,&mut latch),0x2000);
        assert_eq!(inner_word(0x40,false,&mut latch),0);
        assert_eq!(inner_word(0x40|0x10,false,&mut latch),0x10);
        assert_eq!(inner_word(0,false,&mut latch),0);
        assert_eq!(inner_word(0x40,false,&mut latch),0x40);
    }

    #[test]
    fn unowned_input_passes_through_unchanged(){
        let mut latch=0;
        for word in [0,0x40,0x00f0_00f0,0xffff_ffff]{assert_eq!(inner_word(word,false,&mut latch),word);}
    }

    #[test]
    fn native_input_does_not_depend_on_a_menu_trampoline(){
        assert!(menu_ready(false,false));
        assert!(menu_ready(false,true));
        assert!(!menu_ready(true,false));
        assert!(menu_ready(true,true));
    }

    #[test]
    fn stale_native_alt_does_not_disable_pad_but_recomp_keeps_its_guard(){
        assert!(!keyboard_blocks_pad(false,true));
        assert!(!keyboard_blocks_pad(false,false));
        assert!(keyboard_blocks_pad(true,true));
        assert!(!keyboard_blocks_pad(true,false));
    }

    #[test]
    fn later_wrapper_stays_in_front_without_replacing_our_predecessor() {
        let own = [WISHLIST];
        let mut saved = 0;
        let mut got = GAME;
        if can_attach(saved, got, UNRESOLVED, &own) { saved = got; got = WISHLIST; }
        let skillinfo_previous = got;
        got = SKILLINFO;
        for _ in 0..100 {
            if can_attach(saved, got, UNRESOLVED, &own) { saved = got; got = WISHLIST; }
        }
        assert_eq!((got, skillinfo_previous, saved), (SKILLINFO, WISHLIST, GAME));
        assert_ne!(saved, SKILLINFO); // Would create Skillinfo -> Wishlist -> Skillinfo.
    }

    #[test]
    fn existing_wrapper_can_be_attached_once_and_lazy_import_can_retry() {
        assert!(!can_attach(0, UNRESOLVED, UNRESOLVED, &[WISHLIST]));
        assert!(can_attach(0, SKILLINFO, UNRESOLVED, &[WISHLIST]));
        assert!(!can_attach(SKILLINFO, WISHLIST, UNRESOLVED, &[WISHLIST]));
        assert!(!can_attach(SKILLINFO, GAME, UNRESOLVED, &[WISHLIST]));
        for bad in [0, 0xfffc, GAME + 1, WISHLIST] {
            assert!(!can_attach(0, bad, UNRESOLVED, &[WISHLIST]));
        }
    }
}
