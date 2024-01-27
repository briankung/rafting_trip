use std::ops::{Deref, DerefMut};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum LogEntry {
    Root,
    Node {
        term: usize,
        index: usize,
        // Might be nice for the contents to be an Option or something
        contents: String,
    },
}

#[derive(Debug)]
pub struct RaftLog(Vec<LogEntry>);

trait RaftAppendable {
    fn append_entries(&mut self, prev_index: usize, prev_term: usize, entries: &[LogEntry])
        -> bool;
}

/// If any of the exit conditions are false, we can exit without mutating the
/// existing log, but if they are true we have to wait to the end to see if we
/// should mutate. Ergo, `return false` is fine, but `return true` will return
/// too early.
impl RaftAppendable for RaftLog {
    fn append_entries(
        &mut self,
        prev_index: usize,
        prev_term: usize,
        entries: &[LogEntry],
    ) -> bool {
        let Some(prev_entry) = self.get(prev_index) else {
            eprintln!("Could not get log entry for index {prev_index}");
            return false;
        };

        let prev_term_matches = match prev_entry {
            LogEntry::Root => true,
            LogEntry::Node { term, index, .. } => *term == prev_term && *index == prev_index,
        };

        let new_entry = entries.first();

        if new_entry.is_none() {
            return true;
        }

        let new_entry_is_contiguous = match new_entry.unwrap() {
            LogEntry::Node { index, .. } => *index == prev_index + 1,
            LogEntry::Root => false,
        };

        if prev_term_matches && new_entry_is_contiguous {
            let mut entries = entries.to_vec();
            self.truncate(prev_index + 1);
            self.append(&mut entries);
        }

        prev_term_matches && new_entry_is_contiguous
    }
}

impl Deref for RaftLog {
    type Target = Vec<LogEntry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RaftLog {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Default for RaftLog {
    fn default() -> Self {
        Self(vec![LogEntry::Root])
    }
}

impl From<&[LogEntry]> for RaftLog {
    fn from(entries: &[LogEntry]) -> Self {
        RaftLog(entries.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_has_an_empty_log_at_the_root() {
        let log = RaftLog::default();

        assert!(log[0] == LogEntry::Root)
    }

    #[test]
    fn it_appends() {
        let mut log = RaftLog::default();
        let log_entry = LogEntry::Node {
            term: 1,
            index: 1,
            contents: "set x 42".to_owned(),
        };

        let can_append = log.append_entries(0, 0, &[log_entry.clone()]);

        assert!(can_append);
        assert!(log.len() == 2);
        assert!(log[1] == log_entry);
    }

    #[test]
    fn it_does_not_append_with_gap_in_index() {
        let mut log = RaftLog::default();

        let cannot_append = !log.append_entries(
            0,
            0,
            &[LogEntry::Node {
                term: 1,
                index: 2,
                contents: "set x 42".to_owned(),
            }],
        );

        assert!(cannot_append);
    }

    #[test]
    // This test was green to start with so either I'm doing something wrong or
    // I'm doing something right, but clearly I don't know which
    fn it_does_not_append_when_current_term_is_higher() {
        let mut log = RaftLog::from(
            [
                LogEntry::Root,
                LogEntry::Node {
                    term: 1,
                    index: 1,
                    contents: "set x 42".to_owned(),
                },
                LogEntry::Node {
                    term: 2,
                    index: 2,
                    contents: "set x 24".to_owned(),
                },
            ]
            .as_slice(),
        );

        let cannot_append = !log.append_entries(
            2,
            0,
            &[LogEntry::Node {
                term: 1,
                index: 3,
                contents: "set x 42".to_owned(),
            }],
        );

        assert!(cannot_append);
    }

    #[test]
    fn it_returns_true_for_empty_appends() {
        let mut log = RaftLog::default();

        let can_append = log.append_entries(0, 0, &[]);

        assert!(can_append);
    }

    #[test]
    // Ok this one also...passed...
    fn it_returns_true_for_matching_log_entries() {
        let mut log = RaftLog::from(
            [
                LogEntry::Root,
                LogEntry::Node {
                    term: 1,
                    index: 1,
                    contents: "set x 1".to_owned(),
                },
                LogEntry::Node {
                    term: 2,
                    index: 2,
                    contents: "set x 2".to_owned(),
                },
                LogEntry::Node {
                    term: 3,
                    index: 3,
                    contents: "set x 3".to_owned(),
                },
            ]
            .as_slice(),
        );

        let can_append = log.append_entries(
            0,
            0,
            [
                LogEntry::Node {
                    term: 1,
                    index: 1,
                    contents: "set x 1".to_owned(),
                },
                LogEntry::Node {
                    term: 2,
                    index: 2,
                    contents: "set x 2".to_owned(),
                },
                LogEntry::Node {
                    term: 3,
                    index: 3,
                    contents: "set x 3".to_owned(),
                },
            ]
            .as_slice(),
        );

        assert!(can_append);
    }

    #[test]
    fn it_truncates_logs() {
        let mut log = RaftLog::from(
            [
                LogEntry::Root,
                LogEntry::Node {
                    term: 1,
                    index: 1,
                    contents: "set x 1".to_owned(),
                },
                LogEntry::Node {
                    term: 2,
                    index: 2,
                    contents: "set x 2".to_owned(),
                },
                LogEntry::Node {
                    term: 3,
                    index: 3,
                    contents: "set x 3".to_owned(),
                },
            ]
            .as_slice(),
        );

        let can_append = log.append_entries(
            0,
            0,
            [
                LogEntry::Node {
                    term: 4,
                    index: 1,
                    contents: "set x 1".to_owned(),
                },
                LogEntry::Node {
                    term: 5,
                    index: 2,
                    contents: "set x 2".to_owned(),
                },
                LogEntry::Node {
                    term: 6,
                    index: 3,
                    contents: "set x 3".to_owned(),
                },
            ]
            .as_slice(),
        );

        assert!(can_append);
        assert!(
            log.len() == 4,
            "Expected log length of 4, log actually had length of {}",
            log.len()
        );

        assert!(
            log.last()
                == Some(&LogEntry::Node {
                    term: 6,
                    index: 3,
                    contents: "set x 3".to_owned(),
                })
        );
    }
}
