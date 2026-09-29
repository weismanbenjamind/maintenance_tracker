//! # Diff
//!
//! Houses to level logic for the diff command

mod cmd;
mod diff_calculator;
mod metadata_filter;
mod subcmds;

pub(crate) use cmd::Diff;

#[cfg(test)]
pub(crate) mod testing {
    pub(crate) use cmd::Cmd;
    pub(crate) use subcmds::Threshold;

    use super::{cmd, subcmds};
}
