use std::fmt::{Debug, Formatter};
use crate::raft::protobufs::LogEntry;

#[derive(Default)]
pub struct Entry {
    pub term: u64,
    pub message: String,
}

impl Clone for Entry {
    fn clone(&self) -> Self {
        Self {
            term: self.term,
            message: self.message.clone(),
        }
    }
}

impl Entry {
    pub fn to_log_entry(&self, index: u64) -> LogEntry {
        LogEntry {
            index,
            term: self.term,
            data: self.message.clone(),
        }
    }

    pub fn from_log_entry(log_entry: LogEntry) -> Self {
        Self {
            term: log_entry.term,
            message: log_entry.data,
        }
    }
}

#[derive(Default)]
pub struct Log {
    pub commit_index: u64,
    pub last_applied: u64,
    pub entries: Vec<Entry>,
}

impl Log {
    pub fn new() -> Self {
        Self {
            commit_index: 0,
            last_applied: 0,
            entries: vec![],
        }
    }

    pub fn get_log_entries(&self, next_index: u64) -> Vec<LogEntry> {
        self.entries[next_index as usize..].iter().enumerate().map(
            |(index, entry)| entry.to_log_entry(next_index + index as u64)
        ).collect()
    }

    pub fn append_entry(&mut self, entry: Entry) {
        self.commit_index = self.entries.len() as u64;

        self.entries.push(entry);

        self.last_applied = self.commit_index;
    }

    pub fn append_entries(&mut self, index: u64, entries: Vec<LogEntry>) {
        // Set the commit index to the index of the last entry in the finished entries
        self.commit_index = index + entries.len() as u64 - 1;

        // Calculate the starting position in the existing entries
        let start_pos = index as usize;

        // Ensure the vector has enough capacity to accommodate new entries
        if start_pos + entries.len() > self.entries.len() {
            self.entries.resize(start_pos + entries.len(), Entry::default());
        }

        // Iterate over the new entries and insert/overwrite them in the existing log
        for (i, entry) in entries.into_iter().enumerate() {
            let log_index = start_pos + i;
            self.entries[log_index] = Entry::from_log_entry(entry);
        }

        self.last_applied = self.commit_index;
    }
}

impl Debug for Log {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,
            "{}",
            self.entries.iter().map(|e| e.message.clone()).reduce(|acc, s| {
                format!("{}{}", acc, s)
            }).unwrap_or(String::new())
        )
    }
}
