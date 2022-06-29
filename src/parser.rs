use anyhow::Result;
use crate::message::*;
use crate::assert_from::AssertFrom;        

pub struct MessageParseOutcome {
    /// Caller should shift buffer by this number of bytes.
    pub bytes_consumed: u8,
    pub status: MessageParseOutcomeStatus,
}

#[derive(Debug)]
pub enum MessageParseOutcomeStatus {
    Message(Message),
    NeedMoreBytes(Option<usize>),
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
    pub fn new() -> Parser {
        Parser {
            running_status_byte: None,
        }
    }

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
                let first_byte_is_status_byte = is_status_byte(first_byte);
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

fn is_status_byte(byte: u8) -> bool {
    const STATUS_BYTE_MASK: u8 = 0b10000000;
    byte & STATUS_BYTE_MASK != 0
}

#[derive(Copy, Clone)]
struct StatusByte(u8);

impl StatusByte {
    pub fn parse(&self, buf: &[u8]) -> Result<MessageParseOutcome> {
        let status_nibble = self.0 >> 4;
        let data_bytes = self.data_bytes(buf);
        match data_bytes {
            DataBytes::Bytes(bytes) => {
                self.parse_exact_number_of_bytes(bytes)
            }
            DataBytes::NeedMore(more) => {
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::NeedMoreBytes(more),
                })
            }
            DataBytes::InterruptingStatusByte { index } => {
                todo!()
            }
        }
    }

    fn data_bytes<'buf>(&self, buf: &'buf [u8]) -> DataBytes<'buf> {
        let status_nibble = self.0 >> 4;
        match status_nibble {
            status_nibbles::CHANNEL_VOICE_MESSAGE_NOTE_OFF => get_data_bytes(buf, 2),
            status_nibbles::CHANNEL_VOICE_MESSAGE_NOTE_ON => get_data_bytes(buf, 2),
            status_nibbles::CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_AFTERTOUCH => get_data_bytes(buf, 2),
            status_nibbles::CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE_OR_CHANNEL_MODE_MESSAGE => get_data_bytes(buf, 2),
            status_nibbles::CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE => get_data_bytes(buf, 1),
            status_nibbles::CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_AFTERTOUCH => get_data_bytes(buf, 1),
            status_nibbles::CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE => get_data_bytes(buf, 2),
            status_nibbles::SYSTEM_MESSAGE => {
                todo!()
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn parse_exact_number_of_bytes(&self, bytes: &[u8]) -> Result<MessageParseOutcome> {
        for byte in bytes { assert!(!is_status_byte(*byte)) }
        let status_nibble = self.0 >> 4;
        let channel = MidiChannelId::assert_from(self.0 & 0b1111);
        match status_nibble {
            status_nibbles::CHANNEL_VOICE_MESSAGE_NOTE_OFF => {
                assert_eq!(bytes.len(), 2);
                Ok(MessageParseOutcome {
                    bytes_consumed: 2,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::NoteOff(cvm::NoteOff {
                                    note_number: cvm::NoteNumber(cvm::Unsigned7::assert_from(bytes[0])),
                                    velocity: cvm::KeyVelocity(cvm::Unsigned7::assert_from(bytes[1])),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_NOTE_ON => {
                assert_eq!(bytes.len(), 2);
                Ok(MessageParseOutcome {
                    bytes_consumed: 2,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::NoteOn(cvm::NoteOn {
                                    note_number: cvm::NoteNumber(cvm::Unsigned7::assert_from(bytes[0])),
                                    velocity: cvm::KeyVelocity(cvm::Unsigned7::assert_from(bytes[1])),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_AFTERTOUCH => {
                assert_eq!(bytes.len(), 2);
                Ok(MessageParseOutcome {
                    bytes_consumed: 2,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::PolyphonicKeyPressureAftertouch(cvm::PolyphonicKeyPressureAftertouch {
                                    note_number: cvm::NoteNumber(cvm::Unsigned7::assert_from(bytes[0])),
                                    value: cvm::Unsigned7::assert_from(bytes[1]),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE_OR_CHANNEL_MODE_MESSAGE => {
                assert_eq!(bytes.len(), 2);
                let is_mode_message = bytes[0] >= 120 && bytes[0] <= 127;
                if !is_mode_message {
                    Ok(MessageParseOutcome {
                        bytes_consumed: 1,
                        status: MessageParseOutcomeStatus::Message (
                            Message::Channel(ChannelMessage {
                                channel,
                                message: ChannelMessageType::ChannelVoice(
                                    ChannelVoiceMessage::ControlChange(cvm::ControlChange {
                                        control_number: cvm::ControlNumber(cvm::Unsigned7::assert_from(bytes[0])),
                                        value: cvm::Unsigned7::assert_from(bytes[1]),
                                    })
                                )
                            })
                        )
                    })
                } else {
                    todo!()
                }
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE => {
                assert_eq!(bytes.len(), 1);
                Ok(MessageParseOutcome {
                    bytes_consumed: 1,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::ProgramChange(cvm::ProgramChange {
                                    program_number: cvm::ProgramNumber(cvm::Unsigned7::assert_from(bytes[0])),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_AFTERTOUCH => {
                assert_eq!(bytes.len(), 1);
                Ok(MessageParseOutcome {
                    bytes_consumed: 1,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::ChannelPressureAftertouch(cvm::ChannelPressureAftertouch {
                                    value: cvm::Unsigned7::assert_from(bytes[0]),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE => {
                todo!()
            }
            status_nibbles::SYSTEM_MESSAGE => {
                todo!()
            },
            _ => {
                unreachable!()
            }
        }
    }
}

fn get_data_bytes(buf: &[u8], num: usize) -> DataBytes {
    if let Some(needed) = num.checked_sub(buf.len()) {
        if needed > 0 {
            return DataBytes::NeedMore(Some(needed));
        }
    }

    let bytes = &buf[0..num];
    for (index, byte) in bytes.iter().enumerate() {
        if is_status_byte(*byte) {
            return DataBytes::InterruptingStatusByte { index };
        }
    }
    DataBytes::Bytes(bytes)
}

enum DataBytes<'buf> {
    Bytes(&'buf [u8]),
    NeedMore(Option<usize>),
    InterruptingStatusByte {
        index: usize,
    }
}

mod status_nibbles {
    pub const CHANNEL_VOICE_MESSAGE_NOTE_OFF: u8 = 0b1000;
    pub const CHANNEL_VOICE_MESSAGE_NOTE_ON: u8 = 0b1001;
    pub const CHANNEL_VOICE_MESSAGE_POLYPHONIC_KEY_PRESSURE_AFTERTOUCH: u8 = 0b1010;
    pub const CHANNEL_VOICE_MESSAGE_CONTROL_CHANGE_OR_CHANNEL_MODE_MESSAGE: u8 = 0b1011;
    pub const CHANNEL_VOICE_MESSAGE_PROGRAM_CHANGE: u8 = 0b1100;
    pub const CHANNEL_VOICE_MESSAGE_CHANNEL_PRESSURE_AFTERTOUCH: u8 = 0b1101;
    pub const CHANNEL_VOICE_MESSAGE_PITCH_BEND_CHANGE: u8 = 0b1110;
    pub const SYSTEM_MESSAGE: u8 = 0b1111;
}


