//! Attach each HID import once. A later wrapper may already call us.
// Native modal input is read and withheld at HID level. Only Recomp needs
// its post-remapping menu hook before attaching the HID shortcut observer.
pub fn menu_ready(recomp:bool, installed:bool)->bool{!recomp||installed}
// Forge's native keyboard snapshot is not an authoritative source of the
// emulator host's focus/key-release state. Recomp supplies that host state.
pub fn keyboard_blocks_pad(recomp:bool,keyboard:bool)->bool{recomp&&keyboard}
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
