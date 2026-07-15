//! # Notes module
//!
//! Houses commands for updating notes

use clap::{Args, Subcommand};

/// Args for appending a note
#[derive(Clone, Debug, Args)]
#[command(about = "Append a note")]
pub(super) struct Append {
    #[arg(help = "Notes to append")]
    notes: Vec<String>,
}

impl Append {
    /// Build a new Append struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(super) fn new(notes: &[String]) -> Self {
        Self {
            notes: notes.into(),
        }
    }

    /// Turns the Append args into a Vec<String> representing notes
    pub(super) fn into_notes(self) -> Vec<String> {
        self.notes
    }
}

/// Args for replacing the contents of a note
#[derive(Clone, Debug, Args)]
#[command(about = "Replace contents of a note")]
pub(super) struct Replace {
    #[arg(help = "Index to replace")]
    index: usize,

    #[arg(help = "Note that should be used for replacement")]
    contents: String,
}

impl Replace {
    /// Build a new Replace struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(super) fn new(index: usize, contents: &str) -> Self {
        Self {
            index,
            contents: contents.into(),
        }
    }

    /// Consume the Replace struct and turn it into a tuple of (usize, String)
    /// which maps to (index, contents)
    pub(super) fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

/// Args for inserting a note
#[derive(Clone, Debug, Args)]
#[command(about = "Insert a note")]
pub(super) struct Insert {
    #[arg(help = "Index to insert note at")]
    index: usize,

    #[arg(help = "Note to insert")]
    contents: String,
}

impl Insert {
    /// Build a new Insert struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(super) fn new(index: usize, contents: &str) -> Self {
        Self {
            index,
            contents: contents.into(),
        }
    }

    /// Consume the Insert struct and turn it into a tuple of (usize, String)
    /// which maps to (index, contents)
    pub(super) fn into_parts(self) -> (usize, String) {
        (self.index, self.contents)
    }
}

/// Args for removing a note
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Remove a note")]
pub(super) struct Remove {
    #[arg(help = "Index of note to remove")]
    index: usize,
}

impl Remove {
    /// Build a new Remove struct.
    /// Only used for testing.
    #[cfg(test)]
    pub(super) fn new(index: usize) -> Self {
        Self { index }
    }

    /// Get a copy of on the index attribute
    pub(super) fn index(&self) -> usize {
        self.index
    }
}

/// Struct for clearing all notes.
/// Unit struct. Just here to follow pattern for other commands.
#[derive(Clone, Copy, Debug, Args)]
#[command(about = "Clear notes")]
pub(super) struct Clear;

/// Enum to house all commands and their argument structs
#[derive(Clone, Debug, Subcommand)]
pub(super) enum UpdateNotesCmd {
    Append(Append),
    Replace(Replace),
    Insert(Insert),
    Remove(Remove),
    Clear(Clear),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes_append_new() {
        let to_add = vec!["note".to_string()];
        let found = Append::new(&to_add);
        assert_eq!(found.notes, to_add)
    }

    #[test]
    fn notes_append_into_notes() {
        let notes = vec!["note".to_string()];
        let cmd = Append {
            notes: notes.clone(),
        };
        assert_eq!(cmd.into_notes(), notes);
    }

    #[test]
    fn notes_replace_new() {
        let to_replace = "note";
        let idx: usize = 1;
        let found = Replace::new(idx, to_replace);
        assert_eq!(found.index, idx);
        assert_eq!(found.contents, to_replace);
    }

    #[test]
    fn notes_replace_into_parts() {
        let index: usize = 1;
        let contents = "stuff".to_string();

        let cmd = Replace {
            index,
            contents: contents.clone(),
        };

        assert_eq!(cmd.into_parts(), (index, contents));
    }

    #[test]
    fn notes_insert_new() {
        let to_insert = "note";
        let idx: usize = 1;
        let found = Insert::new(idx, to_insert);
        assert_eq!(found.index, idx);
        assert_eq!(found.contents, to_insert);
    }

    #[test]
    fn notes_insert_into_parts() {
        let index: usize = 1;
        let contents = "stuff".to_string();

        let cmd = Insert {
            index,
            contents: contents.clone(),
        };

        assert_eq!(cmd.into_parts(), (index, contents));
    }

    #[test]
    fn notes_remove_index() {
        let index: usize = 1;
        let cmd = Remove { index };
        assert_eq!(cmd.index(), index);
    }
}
