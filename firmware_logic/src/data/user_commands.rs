
use ringbuffer::ConstGenericRingBuffer;

use crate::UserCommand;

// 34 results into a core lockup state. 33 is the maximum value that works without issues.
// Note that the size does not have to be a power of two, but that not using a power of two might be significantly (up to 3 times) slower.
const COMMANDS_BUFFER_SIZE: usize = 32;

#[derive(Default)]
pub struct UserCommands {
    commands: ConstGenericRingBuffer<UserCommand, COMMANDS_BUFFER_SIZE>,
}