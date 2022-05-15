use anyhow::Result;
use crate::message::*;

pub struct MessageParseOutcome<'buf> {
    remaining_buf: &'buf [u8],
    status: MessageParseOutcomeStatus,
}

pub enum MessageParseOutcomeStatus {
    Message(Message),
    NeedMoreBytes(Option<u8>),
}

pub fn parse<'buf>(buf: &'buf [u8]) -> Result<MessageParseOutcome<'buf>> {
    let mut buf_iter = buf.iter();

    match buf_iter.next() {
        None => {
            Ok(MessageParseOutcome {
                remaining_buf: buf_iter.as_slice(),
                status: MessageParseOutcomeStatus::NeedMoreBytes(None),
            })
        }
        Some(status_byte) => {
            todo!()
        }
    }
}
