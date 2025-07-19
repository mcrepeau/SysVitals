use std::collections::VecDeque;

const DEFAULT_HISTORY_LENGTH: usize = 100;

#[derive(Debug, Clone)]
pub struct HistoricalMetric<T> {
    current: T,
    history: VecDeque<T>,
    max_len: usize,
}

impl<T: Clone> HistoricalMetric<T> {
    pub fn new(initial: T) -> Self {
        Self::with_capacity(initial, DEFAULT_HISTORY_LENGTH)
    }

    pub fn with_capacity(initial: T, max_len: usize) -> Self {
        let mut history = VecDeque::with_capacity(max_len);
        history.push_back(initial.clone());
        Self {
            current: initial,
            history,
            max_len,
        }
    }

    pub fn update(&mut self, value: T) {
        self.current = value.clone();
        if self.history.len() == self.max_len {
            self.history.pop_front();
        }
        self.history.push_back(value);
    }

    pub fn current(&self) -> &T {
        &self.current
    }

    pub fn history(&self) -> &VecDeque<T> {
        &self.history
    }
}
