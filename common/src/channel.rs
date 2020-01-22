//
// ReZe.Rs - Common
//   Copyright (C) 2018-2020 Toshiaki Takada
//
// Channel Event Manager/Handler
//

use super::error::*;

/// Channel Manager trait.
pub trait ChannelManager
{
    fn poll_channel(&self) -> Result<(), CoreError>;
}

