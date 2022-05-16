use anyhow::{Result, anyhow};

pub enum Message {
    Channel(ChannelMessage),
    System(SystemMessage),
}

pub struct ChannelMessage {
    channel: MidiChannelId,
    message: ChannelMessageType,
}

pub enum ChannelMessageType {
    ChannelVoice(ChannelVoiceMessage),
    ChannelMode(ChannelModeMessage),
}

pub enum ChannelVoiceMessage {
    NoteOff,
    NoteOn,
    PolyphonicKeyPressureOrAftertouch,
    ControlChange,
    ProgramChange,
    ChannelPressureOrAftertouch,
    PitchBendChange,
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
