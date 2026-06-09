use serde::{Deserialize, Serialize};

use crate::errors::ServiceMetadataError;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub(crate) struct Notes {
    notes: Option<Vec<String>>,
}

impl Notes {
    pub(super) fn new(notes: Option<Vec<String>>) -> Self {
        Self { notes }
    }

    pub(super) fn try_get(&self) -> Option<&[String]> {
        self.notes.as_deref()
    }

    pub(crate) fn extend(&mut self, extension: Vec<String>) {
        match &mut self.notes {
            Some(notes) => notes.extend(extension),
            None => self.notes = Some(extension),
        }
    }

    fn try_get_notes_mut(&mut self) -> Result<&mut Vec<String>, ServiceMetadataError> {
        match &mut self.notes {
            Some(notes) => Ok(notes),
            None => Err(ServiceMetadataError::NotesNotSet),
        }
    }

    pub(crate) fn replace(
        &mut self,
        idx: usize,
        contents: &str,
    ) -> Result<(), ServiceMetadataError> {
        match self.try_get_notes_mut()?.get_mut(idx) {
            Some(val) => {
                *val = contents.into();
                Ok(())
            }
            None => Err(ServiceMetadataError::NoteIndexNotFound(idx)),
        }
    }

    pub(crate) fn remove(&mut self, idx: usize) -> Result<(), ServiceMetadataError> {
        let notes = self.try_get_notes_mut()?;
        match idx < notes.len() {
            true => {
                notes.remove(idx);
                Ok(())
            }
            false => Err(ServiceMetadataError::NoteIndexOutOfRange(idx, notes.len())),
        }
    }

    pub(crate) fn insert(
        &mut self,
        idx: usize,
        contents: &str,
    ) -> Result<(), ServiceMetadataError> {
        let notes = self.try_get_notes_mut()?;
        match idx < notes.len() {
            true => {
                notes.insert(idx, contents.into());
                Ok(())
            }
            false => Err(ServiceMetadataError::NoteIndexOutOfRange(idx, notes.len())),
        }
    }

    pub(crate) fn clear(&mut self) {
        self.notes = None
    }
}
