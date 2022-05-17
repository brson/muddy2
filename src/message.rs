use anyhow::{Result, anyhow};

pub enum Message {
    Channel(ChannelMessage),
    System(SystemMessage),
}

pub struct ChannelMessage {
    pub channel: MidiChannelId,
    pub message: ChannelMessageType,
}

pub enum ChannelMessageType {
    ChannelVoice(ChannelVoiceMessage),
    ChannelMode(ChannelModeMessage),
}

pub enum ChannelVoiceMessage {
    NoteOff(cvm::NoteOff),
    NoteOn(cvm::NoteOn),
    PolyphonicKeyPressureAftertouch(cvm::PolyphonicKeyPressureAftertouch),
    ControlChange(cvm::ControlChange),
    ProgramChange(cvm::ProgramChange),
    ChannelPressureAftertouch(cvm::ChannelPressureAftertouch),
    PitchBendChange(cvm::PitchBendChange),
}

pub mod u7 {
    pub struct Unsigned7(u8);

    impl TryFrom<u8> for Unsigned7 {
        type Error = anyhow::Error;

        fn try_from(value: u8) -> anyhow::Result<Unsigned7> {
            if value <= 127 {
                Ok(Unsigned7(value))
            } else {
                Err(anyhow::anyhow!("out of range"))
            }
        }
    }
}

pub mod cvm {
    pub use super::u7::Unsigned7;
    pub struct NoteNumber(pub Unsigned7);
    pub struct KeyVelocity(pub Unsigned7);
    pub struct ControlNumber(pub Unsigned7); // todo restrict range to < 120
    pub struct ProgramNumber(pub Unsigned7);

    pub struct NoteOff {
        pub note_number: NoteNumber,
        pub velocity: KeyVelocity,
    }

    pub struct NoteOn {
        pub note_number: NoteNumber,
        pub velocity: KeyVelocity,
    }

    pub struct PolyphonicKeyPressureAftertouch {
        pub note_number: NoteNumber,
        pub value: Unsigned7,
    }

    pub struct ControlChange {
        pub control_number: ControlNumber,
        pub value: Unsigned7,
    }

    pub struct ProgramChange {
        pub program_number: ProgramNumber,
    }

    pub struct ChannelPressureAftertouch {
        pub value: Unsigned7,
    }

    pub struct PitchBendChange {
    }
}

pub struct ChannelModeMessage {
}

pub enum SystemMessage {
    SystemCommon(SystemCommonMessage),
    SystemRealTime(SystemRealTimeMessage),
    SystemExclusive(SystemExclusiveMessage),
}

pub struct MidiChannelId(u8);

impl TryFrom<u8> for MidiChannelId {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<MidiChannelId> {
        if value < 16 {
            Ok(MidiChannelId(value))
        } else {
            Err(anyhow!("Invalid midi channel {}", value))
        }
    }
}

pub struct SystemCommonMessage;
pub struct SystemRealTimeMessage;
pub struct SystemExclusiveMessage;
