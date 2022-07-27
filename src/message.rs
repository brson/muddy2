use anyhow::{Result, anyhow};

#[derive(Debug)]
pub enum Message {
    Channel(ChannelMessage),
    System(SystemMessage),
}

#[derive(Debug)]
pub struct ChannelMessage {
    pub channel: MidiChannelId,
    pub message: ChannelMessageType,
}

#[derive(Debug)]
pub enum ChannelMessageType {
    ChannelVoice(ChannelVoiceMessage),
    ChannelMode(ChannelModeMessage),
}

#[derive(Debug)]
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
    #[derive(Debug)]
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

    impl From<Unsigned7> for u8 {
        fn from(other: Unsigned7) -> u8 {
            other.0
        }
    }
}

pub mod cvm {
    pub use super::u7::Unsigned7;

    #[derive(Debug)]
    pub struct NoteNumber(pub Unsigned7);
    #[derive(Debug)]
    pub struct KeyVelocity(pub Unsigned7);
    #[derive(Debug)]
    pub struct ControlNumber(pub Unsigned7); // todo restrict range to < 120
    #[derive(Debug)]
    pub struct ProgramNumber(pub Unsigned7);

    #[derive(Debug)]
    pub struct NoteOff {
        pub note_number: NoteNumber,
        pub velocity: KeyVelocity,
    }

    #[derive(Debug)]
    pub struct NoteOn {
        pub note_number: NoteNumber,
        pub velocity: KeyVelocity,
    }

    #[derive(Debug)]
    pub struct PolyphonicKeyPressureAftertouch {
        pub note_number: NoteNumber,
        pub value: Unsigned7,
    }

    #[derive(Debug)]
    pub struct ControlChange {
        pub control_number: ControlNumber,
        pub value: Unsigned7,
    }

    #[derive(Debug)]
    pub struct ProgramChange {
        pub program_number: ProgramNumber,
    }

    #[derive(Debug)]
    pub struct ChannelPressureAftertouch {
        pub value: Unsigned7,
    }

    #[derive(Debug)]
    pub struct PitchBendChange {
    }
}

#[derive(Debug)]
pub struct ChannelModeMessage {
}

#[derive(Debug)]
pub enum SystemMessage {
    SystemCommon(SystemCommonMessage),
    SystemRealTime(SystemRealTimeMessage),
    SystemExclusive(SystemExclusiveMessage),
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SystemCommonMessage;
#[derive(Debug)]
pub struct SystemRealTimeMessage;
#[derive(Debug)]
pub struct SystemExclusiveMessage;
