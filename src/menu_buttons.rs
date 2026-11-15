//! Normalize native HID shoulder bits and Recomp's final menu-action word.
//! Recomp emits MenuTabL/R as 0x100/0x200 and MenuPageL/R as 0x400/0x800.
//! On native Forge read physical HID bits, independent of game control settings.
pub fn shoulders(pressed: u32, recomp: bool) -> (bool, bool, bool) {
    let (left, right, detail) = if recomp {
        (0x100, 0x200, 0xc00)
    } else {
        (0x40, 0x80, 0x300)
    };
    (pressed & left != 0, pressed & right != 0, pressed & detail != 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shoulders_match_tabs_and_details_in_both_environments() {
        for (recomp, l, r, zl, zr) in [
            (false, 0x40, 0x80, 0x100, 0x200),
            (true, 0x100, 0x200, 0x400, 0x800),
        ] {
            assert_eq!(shoulders(l, recomp), (true, false, false));
            assert_eq!(shoulders(r, recomp), (false, true, false));
            assert_eq!(shoulders(zl, recomp), (false, false, true));
            assert_eq!(shoulders(zr, recomp), (false, false, true));
            assert_eq!(shoulders(l | zr, recomp), (true, false, true));
            assert_eq!(shoulders(0xf00f, recomp), (false, false, false));
        }
    }
}
