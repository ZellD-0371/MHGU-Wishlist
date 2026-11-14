
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Failure {
    pub source: u32,
    pub slot: u32,
    pub id: u16,
    pub count: u16,
}

pub fn collect(
    stock: &mut [u32; 4096],
    mut read: impl FnMut(u32, u32) -> (u16, u16),
) -> Option<Failure> {
    stock.fill(0);

    let mut first_rejected = None;

    for (source, slots) in [(0, 2300), (1, 50)] {
        for slot in 0..slots {
            let (id, count) = read(source, slot);

            if id >= 2992 || count > 99 {
                if first_rejected.is_none() {
                    first_rejected = Some(Failure {
                        source,
                        slot,
                        id,
                        count,
                    });
                }
                continue;
            }

            if id > 0 {
                stock[id as usize] += count as u32;
            }
        }
    }

    first_rejected
}
