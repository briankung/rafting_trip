use std::cell::RefCell;
use std::cmp::Ordering;
use std::{collections::BTreeMap, rc::Rc};

use crate::raft_channel::RaftChannel;
use crate::raft_id::RaftId;
use crate::raft_log::RaftLog;
use crate::raft_message::{RaftMessage as Message, RaftMessageBody as Body};
use crate::raft_temporal::{RaftTimer, Temporal};
use crate::raft_topology::Topology;
use crate::raft_type_aliases::RcMutChannel;

#[derive(PartialEq, Eq, Debug)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

#[derive(Debug)]
pub struct RaftBuddy {
    pub role: Role,
    pub id: RaftId,
    /// BTreeMap<RaftId, RcMutChannel>
    pub topology: Topology,
    pub timer: RaftTimer,
    pub current_term: usize,
    pub log: RaftLog,
    /// BTreeMap<RaftId, Message>
    pub votes_received: BTreeMap<RaftId, Message>,
}

impl Default for RaftBuddy {
    fn default() -> Self {
        Self {
            role: Role::Follower,
            id: RaftId(0),
            topology: Topology::from_iter([(
                RaftId(0),
                Rc::new(RefCell::new(RaftChannel::default())) as RcMutChannel,
            )]),
            timer: RaftTimer {
                default_timeout: 100,
                ticks_left: 100,
            },
            current_term: 0,
            log: RaftLog::default(),
            votes_received: BTreeMap::default(),
        }
    }
}

impl From<Topology> for Vec<RaftBuddy> {
    fn from(topology: Topology) -> Self {
        topology
            .iter()
            .enumerate()
            .map(|(index, (id, _))| RaftBuddy {
                id: *id,
                topology: topology.clone(),
                timer: RaftTimer {
                    default_timeout: 100 + (index * 10),
                    ticks_left: 100 + (index * 10),
                },
                ..Default::default()
            })
            .collect()
    }
}

impl RaftBuddy {
    fn process_inbox(&mut self) {
        let inbox = self.channel().clone();
        let mut inbox = inbox.borrow_mut();

        while let Some(message) = inbox.pop() {
            match message {
                Message::RequestVote(Body {
                    id,
                    current_term,
                    log_length,
                }) => match self
                    .current_term
                    .cmp(&current_term)
                    .then_with(|| self.log.len().cmp(&log_length))
                {
                    Ordering::Less => self.reject_candidate(id),
                    Ordering::Equal | Ordering::Greater => self.accept_candidate(id),
                },
                // Received a vote
                vote @ Message::VoteForCandidate(Body {
                    id,
                    current_term,
                    log_length,
                }) => {
                    if self.is_candidate() {
                        self.votes_received.entry(id).or_insert(vote);

                        if self.received_majority_votes() {
                            self.become_leader();
                        }
                    }
                }
                rejection @ Message::RejectCandidateVote(Body {
                    id,
                    current_term,
                    log_length,
                }) => {
                    match self
                        .current_term
                        .cmp(&current_term)
                        .then_with(|| self.log.len().cmp(&log_length))
                    {
                        Ordering::Less => todo!(),
                        Ordering::Equal => todo!(),
                        Ordering::Greater => todo!(),
                    }
                }
            }
        }
    }

    fn become_candidate(&mut self) {
        self.role = Role::Candidate;
    }

    fn follower_channels(&mut self) -> Vec<&RcMutChannel> {
        self.topology
            .iter()
            .filter_map(|(&peer_id, channel)| {
                if (peer_id == self.id) {
                    None
                } else {
                    Some(channel)
                }
            })
            .collect()
    }

    fn solicit_votes(&mut self) {
        let id = self.id;
        let log_length = self.log.len();
        let term = self.current_term;

        for channel in self.follower_channels() {
            channel.borrow_mut().push(Message::RequestVote(Body {
                id,
                current_term: term,
                log_length,
            }))
        }
    }

    fn get_channel(&self, id: RaftId) -> &RcMutChannel {
        let (peer_id, channel) = self
            .topology
            .iter()
            .find(|(peer_id, channel)| **peer_id == id)
            .expect("Invalid candidate selected");

        channel
    }

    pub fn channel(&self) -> &RcMutChannel {
        let (peer_id, channel) = self
            .topology
            .iter()
            .find(|(peer_id, channel)| **peer_id == self.id)
            .expect("Invalid candidate selected");

        channel
    }

    fn accept_candidate(&self, candidate_id: RaftId) {
        self.get_channel(candidate_id)
            .borrow_mut()
            .push(Message::VoteForCandidate(Body {
                id: self.id,
                current_term: self.current_term,
                log_length: self.log.len(),
            }))
    }

    fn reject_candidate(&self, candidate_id: RaftId) {
        self.get_channel(candidate_id)
            .borrow_mut()
            .push(Message::RejectCandidateVote(Body {
                id: self.id,
                current_term: self.current_term,
                log_length: self.log.len(),
            }))
    }

    fn has_messages(&self) -> bool {
        !self.channel().borrow_mut().all_messages().is_empty()
    }

    fn is_leader(&self) -> bool {
        self.role == Role::Leader
    }

    fn is_candidate(&self) -> bool {
        self.role == Role::Candidate
    }

    fn received_majority_votes(&self) -> bool {
        let majority = (self.topology.len() / 2) + 1;

        let received: usize = self
            .votes_received
            .values()
            .filter(|vote| matches!(vote, Message::VoteForCandidate(..)))
            .count();

        received >= majority
    }

    fn become_leader(&mut self) {
        self.role = Role::Leader;
    }
}

impl Temporal for RaftBuddy {
    fn tick(&mut self) {
        self.timer.tick();

        if self.has_messages() {
            self.process_inbox();
        }

        if self.timer.ticks_left == 0 {
            self.become_candidate();
            self.solicit_votes();
        }
    }
}
