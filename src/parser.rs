// FIXME: A sysex message that is repeatedly interrupted by
// system realtime messages will cause exponential parsing behavior.

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
    /// Need more bytes to parse a message.
    ///
    /// If the contained is `Some` that indicates the number of needed bytes;
    /// if `None` then the number of bytes is unknown.
    ///
    /// The number of needed bytes is unknown if the provided buffer is empty,
    /// or if parsing a SysEx message.
    NeedMoreBytes(Option<usize>),
    /// A real time message was encountered while parsing another message.
    /// This returns the message, along with the byte that contained it.
    /// The caller should remove the byte from the stream and retry.
    ///
    /// [`MessageParseOutcome::bytes_consumed`] will be 0.
    InterruptingSystemRealTimeMessage {
        message: SystemRealTimeMessage,
        byte_index: usize,
    },
    /// A non-status byte was encountered while looking for a status byte.
    ///
    /// The unexpected byte is accounted for by [`MessageParseOutcome::bytes_consumed`].
    UnexpectedDataByte,
    /// Encountered an End-of-SysEx status byte while not parsing a SysEx.
    UnexpectedEox,
    /// A status byte was encountered while parsing a message.
    ///
    /// The broken message bytes are accounted for by [`MessageParseOutcome::bytes_consumed`].
    ///
    /// TODO this is unused
    BrokenMessage,
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
                        MessageParseOutcomeStatus::UnexpectedEox => {
                            self.running_status_byte = None;
                            Ok(MessageParseOutcome {
                                bytes_consumed: 1 + outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::BrokenMessage => {
                            self.running_status_byte = None;
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
                                bytes_consumed: outcome.bytes_consumed,
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
                        MessageParseOutcomeStatus::UnexpectedEox => {
                            self.running_status_byte = None;
                            Ok(MessageParseOutcome {
                                bytes_consumed: outcome.bytes_consumed,
                                status: outcome.status,
                            })
                        },
                        MessageParseOutcomeStatus::BrokenMessage => {
                            self.running_status_byte = None;
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
                /// case: system realtime messages
                /// case: broken messages
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
                match self.0 {
                    system_status_bytes::SYSTEM_COMMON_MIDI_TIME_QUARTER_FRAME => get_data_bytes(buf, 1),
                    system_status_bytes::SYSTEM_COMMON_SONG_POSITION_POINTER => get_data_bytes(buf, 2),
                    system_status_bytes::SYSTEM_COMMON_SONG_SELECT => get_data_bytes(buf, 1),
                    system_status_bytes::SYSTEM_COMMON_UNDEFINED_1 => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_COMMON_UNDEFINED_2 => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_COMMON_TUNE_REQUEST => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_TIMING_CLOCK => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_UNDEFINED_1 => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_START => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_CONTINUE => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_STOP => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_UNDEFINED_2 => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_ACTIVE_SENSING => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_REALTIME_SYSTEM_RESET => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_END_OF_SYSTEM_EXCLUSIVE_FLAG => get_data_bytes(buf, 0),
                    system_status_bytes::SYSTEM_EXCLUSIVE => {
                        get_sysex_bytes(buf)
                    }
                    _ => {
                        unreachable!()
                    }
                }                    
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn parse_exact_number_of_bytes(&self, bytes: &[u8]) -> Result<MessageParseOutcome> {
        if self.0 != system_status_bytes::SYSTEM_EXCLUSIVE {
            // This check is potentially expensively-redundant for SysEx messages,
            // and `bytes` also contains the EOX status byte.
            for byte in bytes { assert!(!is_status_byte(*byte)) }
        }
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
                        bytes_consumed: 2,
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
                    Ok(MessageParseOutcome {
                        bytes_consumed: 2,
                        status: MessageParseOutcomeStatus::Message (
                            Message::Channel(ChannelMessage {
                                channel,
                                message: ChannelMessageType::ChannelMode(
                                    ChannelModeMessage::try_from(bytes[0]).unwrap(),
                                )
                            })
                        )
                    })
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
                assert_eq!(bytes.len(), 2);
                let bytes = <[u8; 2]>::try_from(bytes).unwrap();
                Ok(MessageParseOutcome {
                    bytes_consumed: 2,
                    status: MessageParseOutcomeStatus::Message (
                        Message::Channel(ChannelMessage {
                            channel,
                            message: ChannelMessageType::ChannelVoice(
                                ChannelVoiceMessage::PitchBendChange(cvm::PitchBendChange {
                                    value: cvm::Unsigned14::assert_from(bytes),
                                })
                            )
                        })
                    )
                })
            }
            status_nibbles::SYSTEM_MESSAGE => {
                self.parse_system_message(bytes)
            },
            _ => {
                unreachable!()
            }
        }
    }

    fn parse_system_message(&self, bytes: &[u8]) -> Result<MessageParseOutcome> {
        match self.0 {
            system_status_bytes::SYSTEM_COMMON_MIDI_TIME_QUARTER_FRAME => {
                assert_eq!(bytes.len(), 1);
                todo!()
            }
            system_status_bytes::SYSTEM_COMMON_SONG_POSITION_POINTER => {
                assert_eq!(bytes.len(), 2);
                todo!()
            }
            system_status_bytes::SYSTEM_COMMON_SONG_SELECT => {
                assert_eq!(bytes.len(), 1);
                todo!()
            }
            system_status_bytes::SYSTEM_COMMON_UNDEFINED_1 => {
                assert_eq!(bytes.len(), 0);
                todo!()
            }
            system_status_bytes::SYSTEM_COMMON_UNDEFINED_2 => {
                assert_eq!(bytes.len(), 0);
                todo!()
            }
            system_status_bytes::SYSTEM_COMMON_TUNE_REQUEST => {
                assert_eq!(bytes.len(), 0);
                todo!()
            }
            system_status_bytes::SYSTEM_REALTIME_TIMING_CLOCK => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::TimingClock
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_UNDEFINED_1 => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::Undefined1
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_START => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::Start
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_CONTINUE => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::Continue
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_STOP => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::Stop
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_UNDEFINED_2 => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::Undefined2
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_ACTIVE_SENSING => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::ActiveSensing
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_REALTIME_SYSTEM_RESET => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::Message(
                        Message::System(SystemMessage::SystemRealTime(
                            SystemRealTimeMessage::SystemReset
                        ))
                    )
                })
            }
            system_status_bytes::SYSTEM_END_OF_SYSTEM_EXCLUSIVE_FLAG => {
                assert_eq!(bytes.len(), 0);
                Ok(MessageParseOutcome {
                    bytes_consumed: 0,
                    status: MessageParseOutcomeStatus::UnexpectedEox,
                })
            }
            system_status_bytes::SYSTEM_EXCLUSIVE => {
                assert_eq!(bytes.last(), Some(&system_status_bytes::SYSTEM_END_OF_SYSTEM_EXCLUSIVE_FLAG));
                todo!()
            }
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

fn get_sysex_bytes(buf: &[u8]) -> DataBytes {
    for (index, byte) in buf.iter().enumerate() {
        if is_status_byte(*byte) {
            if *byte == system_status_bytes::SYSTEM_END_OF_SYSTEM_EXCLUSIVE_FLAG {
                // NB: bytes includes the EOX marker
                return DataBytes::Bytes(&buf[..index + 1]);
            } else {
                return DataBytes::InterruptingStatusByte { index };
            }
        }
    }

    DataBytes::NeedMore(None)
}

enum DataBytes<'buf> {
    Bytes(&'buf [u8]),
    NeedMore(Option<usize>),
    InterruptingStatusByte {
        index: usize,
    }
}

/// Reference: MIDI spec table I
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

/// Reference: MIDI spec tables V, VI, VII
mod system_status_bytes {
    pub const SYSTEM_EXCLUSIVE: u8 = 0xF0;
    pub const SYSTEM_COMMON_MIDI_TIME_QUARTER_FRAME: u8 = 0xF1;
    pub const SYSTEM_COMMON_SONG_POSITION_POINTER: u8 = 0xF2;
    pub const SYSTEM_COMMON_SONG_SELECT: u8 = 0xF3;
    pub const SYSTEM_COMMON_UNDEFINED_1: u8 = 0xF4;
    pub const SYSTEM_COMMON_UNDEFINED_2: u8 = 0xF5;
    pub const SYSTEM_COMMON_TUNE_REQUEST: u8 = 0xF6;
    pub const SYSTEM_END_OF_SYSTEM_EXCLUSIVE_FLAG: u8 = 0xF7;
    pub const SYSTEM_REALTIME_TIMING_CLOCK: u8 = 0xF8;
    pub const SYSTEM_REALTIME_UNDEFINED_1: u8 = 0xF9;
    pub const SYSTEM_REALTIME_START: u8 = 0xFA;
    pub const SYSTEM_REALTIME_CONTINUE: u8 = 0xFB;
    pub const SYSTEM_REALTIME_STOP: u8 = 0xFC;
    pub const SYSTEM_REALTIME_UNDEFINED_2: u8 = 0xFD;
    pub const SYSTEM_REALTIME_ACTIVE_SENSING: u8 = 0xFE;
    pub const SYSTEM_REALTIME_SYSTEM_RESET: u8 = 0xFF;
}
