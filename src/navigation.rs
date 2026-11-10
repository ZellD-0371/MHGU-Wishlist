pub fn move_slot(slot:usize,direction:i32,page:i32)->usize{
 let row=(slot%8) as i32;let group=(slot/8) as i32;
 ((group+page).rem_euclid(6)*8+(row+direction).rem_euclid(8)) as usize
}

/// Eight-row pages with no selectable empty rows on the last page.
pub fn move_item(index:usize,count:usize,direction:i32,page:i32)->usize{
 if count==0{return 0}
 let index=index.min(count-1);let pages=count.div_ceil(8);
 let group=((index/8) as i32+page).rem_euclid(pages as i32) as usize;
 let start=group*8;let rows=(count-start).min(8);
 let row=(index%8).min(rows-1);
 start+(row as i32+direction).rem_euclid(rows as i32) as usize
}
