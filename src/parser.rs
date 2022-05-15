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

#[derive(Copy, Clone)]
struct StatusByte(u8);

impl StatusByte {
    
}
