//! # Update
//!
//! Houses top level logic for the update command

mod cmd;
mod notes;
mod prev_services;

pub(crate) use cmd::Update;

#[cfg(test)]
pub(crate) mod testing {
    pub(crate) use super::cmd::Cmd;
    // Note - only expose UpdateName here
    // Only case that's going to be tested outside the update module
    pub(crate) use super::cmd::UpdateName;
}
