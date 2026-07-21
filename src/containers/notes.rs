//! # Notes
//!
//! Container for dealing with notes.

use serde::{Deserialize, Serialize};

use crate::errors::NotesError;

/// Notes struct. Wraps an Option<Vec<String>> and provides
/// utility methods that operate on the wrapped type.
#[derive(Clone, Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(transparent)]
pub(crate) struct Notes {
    notes: Option<Vec<String>>,
}

impl Notes {
    /// Build a new Notes struct.
    pub(super) fn new(notes: Option<Vec<String>>) -> Self {
        Self { notes }
    }

    /// Get a mutable borrow to the notes Option.
    pub(crate) fn try_get(&self) -> Option<&[String]> {
        self.notes.as_deref()
    }

    /// Extend the notes vector.
    /// If the notes vector is not set, it will be set to this extension.
    pub(crate) fn extend(&mut self, extension: Vec<String>) {
        match &mut self.notes {
            Some(notes) => notes.extend(extension),
            None => self.notes = Some(extension),
        }
    }

    /// Get a mutable borrow to the notes vector.
    /// Returns Err if the the notes vector is None
    fn try_get_notes_mut(&mut self) -> Result<&mut Vec<String>, NotesError> {
        match &mut self.notes {
            Some(notes) => Ok(notes),
            None => Err(NotesError::NotesNotSet),
        }
    }

    /// Removes a note.
    /// Notes must be set and the index must be present in the notes vector
    pub(crate) fn replace(&mut self, idx: usize, contents: &str) -> Result<(), NotesError> {
        match self.try_get_notes_mut()?.get_mut(idx) {
            Some(val) => {
                *val = contents.into();
                Ok(())
            }
            None => Err(NotesError::NoteIndexNotFound(idx)),
        }
    }

    /// Removes a note.
    /// Notes must be set and the index must be less than the length of the notes vector
    pub(crate) fn remove(&mut self, idx: usize) -> Result<(), NotesError> {
        let notes = self.try_get_notes_mut()?;
        match idx < notes.len() {
            true => {
                notes.remove(idx);
                if notes.is_empty() {
                    self.clear();
                };
                Ok(())
            }
            false => Err(NotesError::NoteIndexOutOfRange(idx, notes.len())),
        }
    }

    /// Inserts a note at the given index.
    /// Notes must be set and the index must be less than the length of the notes vector
    pub(crate) fn insert(&mut self, idx: usize, contents: &str) -> Result<(), NotesError> {
        let notes = self.try_get_notes_mut()?;
        match idx < notes.len() {
            true => {
                notes.insert(idx, contents.into());
                Ok(())
            }
            false => Err(NotesError::NoteIndexOutOfRange(idx, notes.len())),
        }
    }

    /// Sets the `.notes` attribute to `None`
    pub(crate) fn clear(&mut self) {
        self.notes = None
    }

    /// Gets the length of the notes if the notes are set (e.g. Some()).
    /// If the notes are not set (e.g. None) returns None.
    /// Only used for testing.
    #[cfg(test)]
    pub(crate) fn len(&self) -> Option<usize> {
        self.notes.as_ref().and_then(|vec| Some(vec.len()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOTE: &str = "note";

    /// Builds a Notes struct with the notes field set to a Vec<String>
    fn build_notes_some() -> Notes {
        Notes {
            notes: Some(vec![NOTE.to_string()]),
        }
    }

    /// Builds a Notes struct with the notes field set to None
    fn build_notes_none() -> Notes {
        Notes { notes: None }
    }

    #[test]
    fn notes_constructor_some() {
        let notes: Vec<String> = Vec::from(["val".to_string()]);
        let built = Notes::new(Some(notes.clone()));
        assert_eq!(built.notes.unwrap(), notes);
    }

    #[test]
    fn notes_constructor_none() {
        let built = Notes::new(None);
        assert!(built.notes.is_none());
    }

    #[test]
    fn notes_try_get_unset() {
        assert!(build_notes_none().try_get().is_none())
    }

    #[test]
    fn notes_try_get_set() {
        assert_eq!(build_notes_some().try_get().unwrap(), [NOTE])
    }

    #[test]
    fn notes_extend_unset() {
        let mut notes = build_notes_none();
        let extension = "extension";

        notes.extend(vec![extension.to_string()]);

        let notes = notes.notes.unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(&notes[0], extension);
    }

    #[test]
    fn notes_extend_set() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        let original_len = original_len.unwrap();
        assert_eq!(original_len, 1);

        let extension = "extension";

        notes.extend(vec!["extension".to_string()]);

        let notes = notes.notes.unwrap();
        let new_len = notes.len();
        assert_eq!(new_len, original_len + 1);
        assert_eq!(notes[new_len - 1], extension);
    }

    #[test]
    fn notes_try_get_notes_mut_unset() {
        let mut notes = build_notes_none();
        let found = notes.try_get_notes_mut().unwrap_err();
        match found {
            NotesError::NotesNotSet => (),
            _ => panic!("Expected NotesError::NotesNotSet"),
        }
    }

    #[test]
    fn notes_try_get_notes_mut_set() {
        let mut notes = build_notes_some();
        assert_eq!(*notes.try_get_notes_mut().unwrap(), [NOTE])
    }

    #[test]
    fn notes_replace_ok() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        assert_eq!(original_len.unwrap(), 1);

        let idx: usize = 0;
        let contents = "stuff";
        notes.replace(idx, contents).unwrap();

        assert_eq!(notes.notes.unwrap()[0], contents);
    }

    #[test]
    fn notes_replace_idx_not_found() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        let target_idx: usize = 1000;
        assert!(original_len.unwrap() < target_idx);

        let found = notes.replace(target_idx, "some_val").unwrap_err();
        match found {
            NotesError::NoteIndexNotFound(_) => (),
            _ => panic!("Expected NotesError::NoteIndexNotFound"),
        }
    }

    #[test]
    fn notes_replace_not_set() {
        let mut notes = build_notes_none();

        let found = notes.replace(1, "stuff").unwrap_err();
        match found {
            NotesError::NotesNotSet => (),
            _ => panic!("Expected NotesError::NotesNotSet"),
        }
    }

    #[test]
    fn notes_remove_one() {
        let mut notes = build_notes_some();
        notes.notes.as_mut().unwrap().push("note_2".into());
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        assert_eq!(original_len.unwrap(), 2);

        notes.remove(0).unwrap();
        assert_eq!(notes.notes.as_deref().unwrap().len(), 1);
    }

    #[test]
    fn notes_remove_all() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        assert_eq!(original_len.unwrap(), 1);

        notes.remove(0).unwrap();
        assert!(notes.notes.is_none());
    }

    #[test]
    fn notes_remove_out_of_range() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        let target_idx: usize = 1000;
        assert!(original_len.unwrap() < target_idx);

        let found = notes.remove(target_idx).unwrap_err();
        match found {
            NotesError::NoteIndexOutOfRange(_, _) => (),
            _ => panic!("Expected NotesError::NoteIndexOutOfRange"),
        }
    }

    #[test]
    fn notes_remove_not_set() {
        let mut notes = build_notes_none();
        let found = notes.remove(1).unwrap_err();
        match found {
            NotesError::NotesNotSet => (),
            _ => panic!("Expected NotesError::NotesNotSet"),
        }
    }

    #[test]
    fn notes_insert_ok() {
        let mut notes = build_notes_some();
        let mut original_length: Option<usize> = None;
        notes
            .notes
            .as_deref()
            .inspect(|notes| original_length = Some(notes.len()));
        let original_length = original_length.unwrap();

        let contents = "new_note";
        notes.insert(0, contents).unwrap();

        let found = notes.notes.unwrap();
        assert!(found.len() > original_length);
        assert_eq!(&found[0], contents);
    }

    #[test]
    fn notes_insert_idx_out_of_range() {
        let mut notes = build_notes_some();
        let mut original_len: Option<usize> = None;

        notes
            .notes
            .as_deref()
            .inspect(|notes| original_len = Some(notes.len()));

        let target_idx: usize = 1000;
        assert!(original_len.unwrap() < target_idx);

        let found = notes.insert(target_idx, "stuff").unwrap_err();
        match found {
            NotesError::NoteIndexOutOfRange(_, _) => (),
            _ => panic!("Expected NotesError::NoteIndexOutOfRange"),
        }
    }

    #[test]
    fn notes_insert_not_set() {
        let mut notes = build_notes_none();
        let found = notes.insert(1, "stuff").unwrap_err();
        match found {
            NotesError::NotesNotSet => (),
            _ => panic!("Expected NotesError::NotesNotSet"),
        }
    }

    #[test]
    fn notes_clear() {
        let mut notes = build_notes_some();
        assert!(notes.notes.is_some());
        notes.clear();
        assert!(notes.notes.is_none());
    }

    #[test]
    fn notes_len_some() {
        assert_eq!(build_notes_some().len().unwrap(), 1);
    }

    #[test]
    fn notes_len_none() {
        assert!(build_notes_none().len().is_none());
    }
}
