pub enum Message {
    Channel(ChannelMessage),
    System(SystemMessage),
}

pub enum ChannelMessage {
    ChannelVoice(ChannelVoiceMessage),
    ChannelMode(ChannelModeMessage),
}

pub enum SystemMessage {
    SystemCommon(SystemCommonMessage),
    SystemRealTime(SystemRealTimeMessage),
    SystemExclusive(SystemExclusiveMessage),
}

pub struct ChannelVoiceMessage;
pub struct ChannelModeMessage;

pub struct SystemCommonMessage;
pub struct SystemRealTimeMessage;
pub struct SystemExclusiveMessage;
