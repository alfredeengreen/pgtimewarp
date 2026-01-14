use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Node {
    id: Arc<str>,
}

impl Node {
    pub fn new(id: &str) -> Self {
        Self { id: Arc::from(id) }
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}
