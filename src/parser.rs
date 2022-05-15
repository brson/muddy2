use anyhow::Result;
use crate::message::*;

pub struct MessageParseOutcome<'buf> {
    remaining_buf: &'buf [u8],
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
    fn parse<'buf>(&mut self, buf: &'buf [u8]) -> Result<MessageParseOutcome<'buf>> {
        let mut buf_iter = buf.iter();

        match buf_iter.next().copied() {
            None => {
                Ok(MessageParseOutcome {
                    remaining_buf: buf_iter.as_slice(),
                    status: MessageParseOutcomeStatus::NeedMoreBytes(None),
                })
            }
            Some(first_byte) => {
                const STATUS_BYTE_MASK: u8 = 0b10000000;
                if first_byte & STATUS_BYTE_MASK != 0 {
                    let status_byte = StatusByte(first_byte);
                    todo!()
                } else if let Some(running_status_byte) = self.running_status_byte {
                    todo!()
                } else {
                    Ok(MessageParseOutcome {
                        remaining_buf: buf_iter.as_slice(),
                        status: MessageParseOutcomeStatus::UnexpectedDataByte,
                    })
                }
            }
        }
    }
}

const CHANNEL_VOICE_MESSAGE_NOTE_OFF: u8 = 0b1000;
const CHANNEL_VOICE_MESSAGE_NOTE_ON: u8 = 0b1001;
const CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_OR_AFTERTOUCH: u8 = 0b1010;
const CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE: u8 = 0b1011;
const CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE: u8 = 0b1100;
const CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_OR_AFTERTOUCH: u8 = 0b1101;
const CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE: u8 = 0b1110;
const CHANNEL_MODE_MESSAGE_SELECT: u8 = 0b1011;



#[derive(Copy, Clone)]
struct StatusByte(u8);

impl StatusByte {
    
}
