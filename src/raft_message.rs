use crate::raft_id::RaftId;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct RaftMessageBody {
    pub id: RaftId,
    pub current_term: usize,
    pub log_length: usize,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum RaftMessage {
    RequestVote(RaftMessageBody),
    VoteForCandidate(RaftMessageBody),
    RejectCandidateVote(RaftMessageBody),
}
