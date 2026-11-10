#[derive(Clone,Copy,Default)]
pub struct Focus{pub material:bool,pub index:usize,pub yes:bool}
impl Focus{
 pub fn enter(&mut self,count:usize){if count>0{self.material=true;self.index=0}}
 pub fn back(&mut self)->bool{if self.material{self.material=false;true}else{false}}
 pub fn step(&mut self,count:usize,d:i32){if count>0{self.index=(self.index as i32+d).rem_euclid(count as i32) as usize}}
}

#[derive(Clone,Copy,Default)]
pub struct AggregateFocus{pub material:usize}
impl AggregateFocus{
 pub fn material_step(&mut self,count:usize,direction:i32,page:i32){self.material=crate::navigation::move_item(self.material,count,direction,page);}
}
