//! Counter leaves its old IDs/layout allocated when returning to the level list.
pub fn quest_list(substate:u8)->bool{substate==5}
pub fn group_drawn(flags:u32)->bool{flags&9==9}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn difficulty_and_transition_states_do_not_use_retained_quest_ids(){
  assert!(quest_list(5));for state in [0,1,2,3,4,6,7,8,9,10,16]{assert!(!quest_list(state));}
 }
 #[test]fn retained_nodes_require_draw_visibility_as_well_as_allocation(){
  assert!(group_drawn(0x0100001b));assert!(group_drawn(0x0100000b));
  assert!(!group_drawn(0x01000013));assert!(!group_drawn(0x01000003));
  assert!(!group_drawn(8));
 }
}

/// Hold the first two rows for 1.5 s, scroll to the last two without pauses,
/// hold there for 1.5 s, then jump back to the first view and repeat.
const HOLD:f64=1.5;
const ROWS_PER_SECOND:f64=0.96;
#[derive(Default)]pub struct Scroll{quest:u32,slots:u64,time:f64}
impl Scroll{
 pub fn reset(&mut self){*self=Self::default();}
 pub fn step(&mut self,quest:u32,slots:u64,dt:f32)->(usize,f32){
  if self.quest!=quest||self.slots!=slots{self.quest=quest;self.slots=slots;self.time=0.;return (0,0.)}
  let count=slots.count_ones() as usize;if count<=2{self.time=0.;return (0,0.)}
  let distance=(count-2) as f64;let travel=distance/ROWS_PER_SECOND;let cycle=HOLD+travel+HOLD;
  if dt.is_finite(){self.time+=dt.clamp(0.,0.1) as f64;}
  if self.time>=cycle{self.time-=cycle;}
  let offset=((self.time-HOLD)*ROWS_PER_SECOND).clamp(0.,distance);
  let top=offset as usize;(top,(offset-top as f64) as f32)
 }
}
#[cfg(test)]mod scroll_tests{
 use super::*;
 fn advance(s:&mut Scroll,seconds:f64)->(usize,f32){let mut result=(0,0.);for _ in 0..(seconds*100.).round() as usize{result=s.step(1,15,0.01);}result}
 fn offset(v:(usize,f32))->f32{v.0 as f32+v.1}
 #[test]fn holds_one_and_a_half_seconds_then_moves_continuously_including_row_boundaries(){
  let mut s=Scroll::default();s.step(1,15,0.);
  assert_eq!(advance(&mut s,1.49),(0,0.));
  let a=offset(advance(&mut s,0.11));assert!((a-0.096).abs()<0.001);
  let b=offset(advance(&mut s,0.6));assert!((b-a-0.576).abs()<0.001);
  let c=offset(advance(&mut s,0.8));assert!((c-b-0.768).abs()<0.001,"no hold when crossing a row boundary");
 }
 #[test]fn last_two_rows_hold_then_jump_to_first_and_wait_again(){
  let mut s=Scroll::default();s.step(1,15,0.);
  assert_eq!(advance(&mut s,3.7),(2,0.));
  assert_eq!(advance(&mut s,1.2),(2,0.),"stop at the end instead of wrapping to row zero");
  assert_eq!(advance(&mut s,0.3),(0,0.));
  assert_eq!(advance(&mut s,1.3),(0,0.));
  assert!(offset(advance(&mut s,0.2))>0.);
 }
 #[test]fn quest_results_and_reset_restart_and_two_rows_stay_fixed(){
  let mut s=Scroll::default();s.step(1,15,0.);advance(&mut s,3.);
  assert_eq!(s.step(2,15,0.1),(0,0.));assert_eq!(s.step(2,7,0.1),(0,0.));
  assert_eq!(s.step(2,3,0.1),(0,0.));for _ in 0..100{assert_eq!(s.step(2,3,0.1),(0,0.));}
  s.reset();assert_eq!(s.step(1,15,0.1),(0,0.));
 }
}
