use axum::extract::ws::{CloseFrame};
use axum::extract::ws::CloseCode;
use std::borrow::Cow;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("decode error")]
    DecodeError,
    #[error("invalid api version")]
    InvalidApiVersion,
    #[error("unknown opcode {0}")]
    UnknownOpcode(u8),
}

impl GatewayError {
    pub fn close_frame(&self) -> CloseFrame<'static> {
        CloseFrame {
            code: match self {
                GatewayError::DecodeError => CloseCode::from(4002u16),
                GatewayError::InvalidApiVersion => CloseCode::from(4012u16),
                GatewayError::UnknownOpcode(_) => CloseCode::from(4001u16),
            },
            reason: Cow::from(self.to_string()),
        }
    }
}
