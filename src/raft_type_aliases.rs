use crate::raft_channel::Channel;
use crate::raft_log::RaftLog;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::{collections::BTreeMap, rc::Rc};

pub type RcMutChannel = Rc<RefCell<dyn Channel>>;
