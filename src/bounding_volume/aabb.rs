use std::cmp::{min, max};
use std::num::Zero;
use nalgebra::traits::scalar_op::{ScalarAdd, ScalarSub};
use utils::default::Default;
use bounding_volume::bounding_volume::{BoundingVolume, LooseBoundingVolume};

#[deriving(ToStr)]
pub struct AABB<V>
{
  priv mins: V,
  priv maxs: V
}

impl<V: Copy + Ord> AABB<V>
{
  pub fn new(&mins: &V, &maxs: &V) -> AABB<V>
  {
    assert!(mins <= maxs);

    AABB {
      mins: mins,
      maxs: maxs
    }
  }
}

impl<V: Ord + Copy> BoundingVolume for AABB<V>
{
  #[inline]
  fn intersects(&self, other: &AABB<V>) -> bool
  { !(self.mins > other.maxs || other.mins > self.maxs) }

  #[inline]
  fn contains(&self, other: &AABB<V>) -> bool
  { self.mins <= other.mins && self.maxs >= other.maxs }

  #[inline]
  fn merge(&mut self, other: &AABB<V>)
  {
    self.mins = min(copy self.mins, copy other.mins);
    self.maxs = max(copy self.maxs, copy other.maxs);
  }

  #[inline]
  fn merged(&self, other: &AABB<V>) -> AABB<V>
  {
    AABB {
      mins: min(copy self.mins, copy other.mins),
      maxs: max(copy self.maxs, copy other.maxs)
    }
  }
}

impl<V: Ord + ScalarAdd<N> + ScalarSub<N>, N> LooseBoundingVolume<N> for AABB<V>
{
  #[inline]
  fn loosen(&mut self, amount: N)
  {
    self.mins.scalar_sub_inplace(&amount);
    self.maxs.scalar_add_inplace(&amount);
  }

  #[inline]
  fn loosened(&self, amount: N) -> AABB<V>
  {
    AABB {
      mins: self.mins.scalar_sub(&amount),
      maxs: self.maxs.scalar_add(&amount)
    }
  }
}

impl<V: Zero> Default for AABB<V>
{
  fn default() -> AABB<V>
  {
    AABB {
      mins: Zero::zero(),
      maxs: Zero::zero()
    }
  }
}
