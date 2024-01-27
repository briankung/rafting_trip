use crate::raft_message::RaftMessage;

#[derive(PartialEq, Eq, Clone, Default)]
pub struct RaftChannel {
    pub queue: Vec<RaftMessage>,
}

pub trait Channel {
    fn push(&mut self, message: RaftMessage);
    fn pop(&mut self) -> Option<RaftMessage>;
    fn all_messages(&mut self) -> Vec<RaftMessage>;
}

impl Channel for RaftChannel {
    fn push(&mut self, message: RaftMessage) {
        todo!()
    }

    fn pop(&mut self) -> Option<RaftMessage> {
        todo!()
    }

    fn all_messages(&mut self) -> Vec<RaftMessage> {
        todo!()
    }
}
