//! Native modal ownership must not depend on the game's current menu/vtable.
#[derive(Default)]
pub struct Gate { held:u32, release:u32 }
impl Gate {
    pub const fn new()->Self{Self{held:0,release:0}}
    /// Returns new UI presses and physical inputs withheld from the game.
    pub fn step(&mut self,raw:u32,modal:bool,overlay:bool)->(u32,u32){
        let pressed=raw&!self.held;self.held=raw;
        if overlay{self.release=0;return (0,0)}
        self.release&=raw;
        if modal{self.release=raw;(pressed,u32::MAX)}else{(0,self.release)}
    }
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test]fn focus_loss_neutral_and_return_keep_modal_ownership(){
  let mut gate=Gate::default();
  assert_eq!(gate.step(1,true,false),(1,u32::MAX));
  assert_eq!(gate.step(1,true,false),(0,u32::MAX));
  assert_eq!(gate.step(0,true,false),(0,u32::MAX)); // Unfocused/disconnected poll.
  assert_eq!(gate.step(1<<13,true,false),(1<<13,u32::MAX));
  assert_eq!(gate.step(0,true,false),(0,u32::MAX));
  assert_eq!(gate.step(2,true,false),(2,u32::MAX));
 }
 #[test]fn closing_press_is_blocked_until_release_and_forge_can_take_over(){
  let mut gate=Gate::default();gate.step(0,false,false);
  gate.step(2,true,false);
  assert_eq!(gate.step(2,false,false),(0,2));
  assert_eq!(gate.step(0,false,false),(0,0));
  assert_eq!(gate.step(2,false,false),(0,0));
  assert_eq!(gate.step(2,true,true),(0,0));
  assert_eq!(gate.step(2,true,false),(0,u32::MAX));
 }
}
