#![allow(unused)]
use std::cell::RefCell;
use std::cmp::Ordering;
use std::{collections::BTreeMap, rc::Rc};

mod raft_log;
mod raft_buddy;
mod raft_channel;
mod raft_id;
mod raft_message;
mod raft_temporal;
mod raft_topology;
mod raft_type_aliases;

use raft_log::RaftLog;
use raft_buddy::{RaftBuddy, Role};
use raft_channel::{Channel, RaftChannel};
use raft_id::RaftId;
use raft_message::{RaftMessage, RaftMessageBody};
use raft_temporal::RaftTimer;
use raft_temporal::Temporal;
use raft_topology::Topology;
use raft_type_aliases::RcMutChannel;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Eq, Clone, Default)]
    struct TestRaftChannel {
        queue: Vec<RaftMessage>,
    }

    impl Channel for TestRaftChannel {
        fn push(&mut self, message: RaftMessage) {
            self.queue.push(message);
        }

        fn pop(&mut self) -> Option<RaftMessage> {
            self.queue.pop()
        }

        fn all_messages(&mut self) -> Vec<RaftMessage> {
            self.queue.clone()
        }
    }

    fn default_topology() -> Topology {
        Topology::from_iter([
            (
                RaftId(0),
                Rc::new(RefCell::new(TestRaftChannel::default())) as RcMutChannel,
            ),
            (
                RaftId(1),
                Rc::new(RefCell::new(TestRaftChannel::default())) as RcMutChannel,
            ),
            (
                RaftId(2),
                Rc::new(RefCell::new(TestRaftChannel::default())) as RcMutChannel,
            ),
            (
                RaftId(3),
                Rc::new(RefCell::new(TestRaftChannel::default())) as RcMutChannel,
            ),
            (
                RaftId(4),
                Rc::new(RefCell::new(TestRaftChannel::default())) as RcMutChannel,
            ),
        ])
    }

    #[test]
    fn test_init_servers() {
        let buddies: Vec<RaftBuddy> = default_topology().into();

        assert!(buddies.len() == 5)
    }

    #[test]
    fn test_buddies_run_out_of_patience() {
        let mut buddy = RaftBuddy {
            role: Role::Follower,
            id: RaftId(0),
            topology: default_topology(),
            timer: 1.into(),
            ..Default::default()
        };

        buddy.tick();

        assert!(buddy.timer.ticks_left == 0);
    }

    #[test]
    fn test_buddies_run_out_of_patience_then_restart() {
        let mut buddy = RaftBuddy {
            role: Role::Follower,
            id: RaftId(0),
            topology: default_topology(),
            timer: 1.into(),
            ..Default::default()
        };

        buddy.tick();
        buddy.tick();

        assert!(buddy.timer.ticks_left == 1);
    }

    #[test]
    fn test_a_candidate_arises() {
        let mut buddies: Vec<RaftBuddy> = default_topology().into();

        for _tick in (0..100) {
            buddies.iter_mut().for_each(|buddy| {
                buddy.tick();
            });
        }

        let buddy = buddies.first().unwrap();

        assert_eq!(buddy.role, Role::Candidate);
        assert!(buddies[1..].iter().all(|b| b.role == Role::Follower));
    }

    #[test]
    fn test_the_candidate_sends_messages_to_other_buddies() {
        let topology = default_topology();
        let mut buddies: Vec<RaftBuddy> = topology.clone().into();
        let (candidate, followers) = buddies.split_first_mut().unwrap();

        for tick in (1..=100) {
            candidate.tick()
        }

        let messages: Vec<Option<RaftMessage>> = topology
            .values()
            .map(|channel| channel.borrow_mut().pop())
            .collect();

        assert_eq!(
            messages,
            [
                None,
                Some(RaftMessage::RequestVote(RaftMessageBody {
                    id: RaftId(0),
                    current_term: 0,
                    log_length: 1
                })),
                Some(RaftMessage::RequestVote(RaftMessageBody {
                    id: RaftId(0),
                    current_term: 0,
                    log_length: 1
                })),
                Some(RaftMessage::RequestVote(RaftMessageBody {
                    id: RaftId(0),
                    current_term: 0,
                    log_length: 1
                })),
                Some(RaftMessage::RequestVote(RaftMessageBody {
                    id: RaftId(0),
                    current_term: 0,
                    log_length: 1
                })),
            ]
        )
    }

    #[test]
    fn test_peers_respond_to_solicitation() {
        let topology = default_topology();
        let mut buddies: Vec<RaftBuddy> = topology.clone().into();
        let (candidate, followers) = buddies.split_first_mut().unwrap();

        for follower in followers.iter_mut() {
            follower
                .channel()
                .borrow_mut()
                .push(RaftMessage::RequestVote(RaftMessageBody {
                    id: RaftId(0),
                    current_term: 0,
                    log_length: 1,
                }));

            follower.tick()
        }

        let messages: Vec<_> = topology
            .values()
            .map(|channel| channel.borrow_mut().all_messages())
            .collect();

        assert_eq!(
            messages,
            [
                vec![
                    RaftMessage::VoteForCandidate(RaftMessageBody {
                        id: 1.into(),
                        current_term: 0,
                        log_length: 1
                    }),
                    RaftMessage::VoteForCandidate(RaftMessageBody {
                        id: 2.into(),
                        current_term: 0,
                        log_length: 1
                    }),
                    RaftMessage::VoteForCandidate(RaftMessageBody {
                        id: 3.into(),
                        current_term: 0,
                        log_length: 1
                    }),
                    RaftMessage::VoteForCandidate(RaftMessageBody {
                        id: 4.into(),
                        current_term: 0,
                        log_length: 1
                    })
                ],
                vec![],
                vec![],
                vec![],
                vec![]
            ]
        )
    }

    #[test]
    fn test_candidate_becomes_leader() {
        let topology = default_topology();
        let mut buddies: Vec<RaftBuddy> = topology.clone().into();
        let (mut candidate, followers) = buddies.split_first_mut().unwrap();

        candidate.timer.ticks_left = 1;
        candidate.tick();

        for follower in followers.iter_mut() {
            follower.tick()
        }

        candidate.tick();

        assert_eq!(candidate.role, Role::Leader);
    }

    #[test]
    fn test_without_majority_candidate_does_not_become_leader() {
        let topology = default_topology();
        let mut buddies: Vec<RaftBuddy> = topology.clone().into();
        let (mut candidate, followers) = buddies.split_first_mut().unwrap();

        candidate.timer.ticks_left = 1;
        candidate.tick();

        for follower in followers[0..2].iter_mut() {
            follower.tick()
        }

        candidate.tick();

        assert_eq!(candidate.role, Role::Candidate);
    }

    #[test]
    fn test_with_bare_majority_candidate_does_become_leader() {
        let topology = default_topology();
        let mut buddies: Vec<RaftBuddy> = topology.clone().into();
        let (mut candidate, followers) = buddies.split_first_mut().unwrap();

        candidate.timer.ticks_left = 1;
        candidate.tick();

        for follower in followers[0..3].iter_mut() {
            follower.tick()
        }

        candidate.tick();

        assert_eq!(candidate.role, Role::Leader);
    }
}
