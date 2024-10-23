//! Enemy behavior definition

use std::ops::Range;

use action::EnemyBehavior;

pub mod action;

/// An enemy and all of its related behavioral metadata
pub struct Freak {
    /// The (range in) amount of ticks before another behavior is performed (randomly selected each
    /// time)
    cooldown: Range<u32>,
    /// An enemies behavior is specific to them
    behavior: Box<dyn EnemyBehavior>
}

