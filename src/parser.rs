use anyhow::Result;
use crate::message::*;

pub struct MessageParseOutcome {
    bytes_consumed: u8,
    status: MessageParseOutcomeStatus,
}

pub enum MessageParseOutcomeStatus {
    Message(Message),
    NeedMoreBytes(Option<u8>),
    /// A real time message was encountered while parsing another message.
    /// This returns the message, along with the byte that contained it.
    /// The caller should remove the byte from the stream and retry.
    InterruptingSystemRealTimeMessage {
        message: SystemRealTimeMessage,
        byte_index: usize,
    },
    UnexpectedDataByte,
}

pub struct Parser {
    running_status_byte: Option<StatusByte>,
}

impl Parser {
    pub fn parse(&mut self, buf: &[u8]) -> Result<MessageParseOutcome> {
        let mut buf_iter = buf.iter();

        match buf_iter.next().copied() {
            None => {
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::NeedMoreBytes(None),
                })
            }
            Some(first_byte) => {
                const STATUS_BYTE_MASK: u8 = 0b10000000;
                if first_byte & STATUS_BYTE_MASK != 0 {
                    let status_byte = StatusByte(first_byte);
                    let outcome = status_byte.parse(buf_iter.as_slice())?;
                    match outcome.status {
                        MessageParseOutcomeStatus::Message(Message::Channel(_)) => {
                            self.running_status_byte = Some(status_byte);
                            todo!()
                        },
                        _ => {
                            todo!()
                        }
                    }
                } else if let Some(running_status_byte) = self.running_status_byte {
                    todo!()
                } else {
                    Ok(MessageParseOutcome {
                        bytes_consumed: 1,
                        status: MessageParseOutcomeStatus::UnexpectedDataByte,
                    })
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
struct StatusByte(u8);

impl StatusByte {
    pub fn parse(&self, buf: &[u8]) -> Result<MessageParseOutcome> {
        let status_nibble = self.0 >> 4;
        todo!()
    }
}

mod status_nibbles {
    const CHANNEL_VOICE_MESSAGE_NOTE_OFF: u8 = 0b1000;
    const CHANNEL_VOICE_MESSAGE_NOTE_ON: u8 = 0b1001;
    const CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_OR_AFTERTOUCH: u8 = 0b1010;
    const CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE: u8 = 0b1011;
    const CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE: u8 = 0b1100;
    const CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_OR_AFTERTOUCH: u8 = 0b1101;
    const CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE: u8 = 0b1110;
    const CHANNEL_MODE_MESSAGE_SELECT: u8 = 0b1011;
    const SYSTEM_MESSAGE: u8 = 0b1111;
}


