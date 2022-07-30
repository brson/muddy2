//! Methods that provide some higher-level interpretation of MIDI messages.

use crate::message::{self, cvm};

impl message::ChannelVoiceMessage {
    /// Returns if the note should turn off.
    ///
    /// Taking into account that NoteOn with velocity 0 means NoteOff.
    ///
    /// Returns `Some` if the note should be turned off,
    /// and the inner value is the off velocity.
    ///
    /// Reference: todo
    pub fn is_note_off_equiv(&self) -> Option<cvm::KeyVelocity> {
        todo!()
    }
}

impl cvm::PitchBendChange {
    /// Reference: todo
    pub fn is_centered(&self) -> bool {
        u16::from(self.value) == 0x2000
    }
}

