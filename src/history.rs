pub struct UndoRedo<T> {
    past: Vec<T>,
    future: Vec<T>,
}

impl<T> Default for UndoRedo<T> {
    fn default() -> Self {
        Self {
            past: Vec::new(),
            future: Vec::new(),
        }
    }
}

impl<T> UndoRedo<T> {
    pub fn can_undo(&self) -> bool {
        !self.past.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.future.is_empty()
    }

    pub fn push(&mut self, state: T) {
        self.past.push(state);
        self.future.clear();
    }

    pub fn undo(&mut self, current: T) -> Option<T> {
        let previous = self.past.pop()?;
        self.future.push(current);
        Some(previous)
    }

    pub fn redo(&mut self, current: T) -> Option<T> {
        let next = self.future.pop()?;
        self.past.push(current);
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::UndoRedo;

    #[test]
    fn undo_and_redo_restore_state_in_order() {
        let mut history = UndoRedo::default();
        history.push("one");
        history.push("two");
        history.push("three");

        let current = "four";
        let current = history.undo(current).expect("undo one");
        assert_eq!(current, "three");

        let current = history.undo(current).expect("undo two");
        assert_eq!(current, "two");

        let current = history.redo(current).expect("redo one");
        assert_eq!(current, "three");
    }

    #[test]
    fn push_clears_redo_branch() {
        let mut history = UndoRedo::default();
        history.push(1);
        history.push(2);
        let current = history.undo(3).expect("undo");
        assert_eq!(current, 2);

        history.push(4);
        assert!(!history.can_redo());
    }
}
