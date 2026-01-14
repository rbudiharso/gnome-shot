use super::Annotation;

#[derive(Debug, Clone)]
pub enum HistoryAction {
    Add(Annotation),
    Remove(usize, Annotation),
}

#[derive(Debug, Default)]
pub struct History {
    undo_stack: Vec<HistoryAction>,
    redo_stack: Vec<HistoryAction>,
}

impl History {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_add(&mut self, annotation: Annotation) {
        self.undo_stack.push(HistoryAction::Add(annotation));
        self.redo_stack.clear();
    }

    pub fn push_remove(&mut self, index: usize, annotation: Annotation) {
        self.undo_stack.push(HistoryAction::Remove(index, annotation));
        self.redo_stack.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo(&mut self, annotations: &mut Vec<Annotation>) -> bool {
        if let Some(action) = self.undo_stack.pop() {
            match action {
                HistoryAction::Add(annotation) => {
                    // Remove the last added annotation
                    if let Some(removed) = annotations.pop() {
                        self.redo_stack.push(HistoryAction::Add(removed));
                    }
                }
                HistoryAction::Remove(index, annotation) => {
                    // Re-add the removed annotation
                    annotations.insert(index, annotation.clone());
                    self.redo_stack.push(HistoryAction::Remove(index, annotation));
                }
            }
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self, annotations: &mut Vec<Annotation>) -> bool {
        if let Some(action) = self.redo_stack.pop() {
            match action {
                HistoryAction::Add(annotation) => {
                    annotations.push(annotation.clone());
                    self.undo_stack.push(HistoryAction::Add(annotation));
                }
                HistoryAction::Remove(index, annotation) => {
                    if index < annotations.len() {
                        annotations.remove(index);
                    }
                    self.undo_stack.push(HistoryAction::Remove(index, annotation));
                }
            }
            true
        } else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}
