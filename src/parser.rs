use anyhow::Result;
use crate::message::*;

pub struct MessageParseOutcome {
    /// Caller should shift buffer by this number of bytes.
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
    BrokenMessage(Vec<u8>),
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
                let first_byte_is_status_byte = first_byte & STATUS_BYTE_MASK != 0;
                if first_byte_is_status_byte {
                    let remaining_bytes = buf_iter.as_slice();
                    let status_byte = StatusByte(first_byte);
                    let outcome = status_byte.parse(remaining_bytes)?;
                    assert!(outcome.bytes_consumed as usize <= remaining_bytes.len());
                    match outcome.status {
                        MessageParseOutcomeStatus::Message(Message::Channel(_)) => {
                            self.running_status_byte = Some(status_byte);
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::Message(Message::System(SystemMessage::SystemRealTime(_))) => {
                            self.running_status_byte = self.running_status_byte;
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::Message(Message::System(_)) => {
                            self.running_status_byte = None;
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::NeedMoreBytes(_) => {
                            assert_eq!(0, outcome.bytes_consumed);
                            Ok(outcome)
                        },
                        MessageParseOutcomeStatus::InterruptingSystemRealTimeMessage {
                            message, byte_index,
                        } => {
                            assert_eq!(0, outcome.bytes_consumed);
                            Ok(MessageParseOutcome {
                                bytes_consumed: 0,
                                status: MessageParseOutcomeStatus::InterruptingSystemRealTimeMessage {
                                    message,
                                    byte_index: 1 + byte_index,
                                }
                            })
                        },
                        MessageParseOutcomeStatus::UnexpectedDataByte => {
                            unreachable!()
                        },
                        MessageParseOutcomeStatus::BrokenMessage(_) => {
                            // todo think harder about this case
                            self.running_status_byte = self.running_status_byte;
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        }
                    }
                } else if let Some(running_status_byte) = self.running_status_byte {
                    let remaining_bytes = buf;
                    let status_byte = running_status_byte;
                    let outcome = status_byte.parse(remaining_bytes)?;
                    assert!(outcome.bytes_consumed as usize <= remaining_bytes.len());
                    match outcome.status {
                        MessageParseOutcomeStatus::Message(Message::Channel(_)) => {
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::Message(_) => {
                            unreachable!()
                        },
                        MessageParseOutcomeStatus::NeedMoreBytes(_) => {
                            assert_eq!(0, outcome.bytes_consumed);
                            Ok(outcome)
                        },
                        MessageParseOutcomeStatus::InterruptingSystemRealTimeMessage {
                            message, byte_index,
                        } => {
                            assert_eq!(0, outcome.bytes_consumed);
                            Ok(MessageParseOutcome {
                                bytes_consumed: 0,
                                status: MessageParseOutcomeStatus::InterruptingSystemRealTimeMessage {
                                    message,
                                    byte_index: byte_index,
                                }
                            })
                        },
                        MessageParseOutcomeStatus::UnexpectedDataByte => {
                            unreachable!()
                        },
                        MessageParseOutcomeStatus::BrokenMessage(_) => {
                            Ok(MessageParseOutcome {
                                bytes_consumed: outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        }
                    }
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
        let data_bytes = self.data_bytes(buf);
        match data_bytes {
            DataBytes::Bytes(bytes) => {
                todo!()
            }
            DataBytes::NeedMore(more) => {
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::NeedMoreBytes(more),
                })
            }
        }
    }

    fn data_bytes(&self, buf: &[u8]) -> DataBytes {
        todo!()
    }
}

enum DataBytes<'buf> {
    Bytes(&'buf [u8]),
    NeedMore(Option<u8>)
}

mod status_nibbles {
    const CHANNEL_VOICE_MESSAGE_NOTE_OFF: u8 = 0b1000;
    const CHANNEL_VOICE_MESSAGE_NOTE_ON: u8 = 0b1001;
    const CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_OR_AFTERTOUCH: u8 = 0b1010;
    const CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE_OR_CHANNEL_MODE_MESSAGE: u8 = 0b1011;
    const CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE: u8 = 0b1100;
    const CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_OR_AFTERTOUCH: u8 = 0b1101;
    const CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE: u8 = 0b1110;
    const SYSTEM_MESSAGE: u8 = 0b1111;
}


