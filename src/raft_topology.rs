use crate::raft_id::RaftId;
use crate::raft_log::RaftLog;
use crate::raft_type_aliases::RcMutChannel;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::{collections::BTreeMap, rc::Rc};

#[derive(Clone)]
pub struct Topology(BTreeMap<RaftId, RcMutChannel>);

impl std::ops::Deref for Topology {
    type Target = BTreeMap<RaftId, RcMutChannel>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Topology {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl FromIterator<(RaftId, RcMutChannel)> for Topology {
    fn from_iter<I: IntoIterator<Item = (RaftId, RcMutChannel)>>(iter: I) -> Self {
        Topology(BTreeMap::from_iter(iter))
    }
}

impl From<BTreeMap<RaftId, RcMutChannel>> for Topology {
    fn from(value: BTreeMap<RaftId, RcMutChannel>) -> Self {
        Topology::from_iter(value)
    }
}

impl std::fmt::Debug for Topology {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ids: Vec<&RaftId> = self.0.iter().map(|(id, channel)| id).collect();
        f.write_fmt(format_args!("{ids:?}"))
    }
}
