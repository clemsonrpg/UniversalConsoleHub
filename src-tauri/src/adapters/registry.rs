use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::emulator::EmulatorAdapter;

use super::rpcs3::Rpcs3Adapter;

#[derive(Clone, Default)]
pub struct AdapterRegistry {
    adapters: HashMap<&'static str, Arc<dyn EmulatorAdapter>>,
}

impl AdapterRegistry {
    pub fn new() -> Self {
        Self {
            adapters: HashMap::new(),
        }
    }

    pub fn with_phase_one_adapters() -> Self {
        let mut reg = Self::new();
        reg.register(Arc::new(Rpcs3Adapter::new()));
        reg
    }

    pub fn register(&mut self, adapter: Arc<dyn EmulatorAdapter>) {
        self.adapters.insert(adapter.slug(), adapter);
    }

    pub fn get(&self, slug: &str) -> Option<Arc<dyn EmulatorAdapter>> {
        self.adapters.get(slug).cloned()
    }

    pub fn list_slugs(&self) -> Vec<&'static str> {
        let mut keys: Vec<&'static str> = self.adapters.keys().copied().collect();
        keys.sort_unstable();
        keys
    }
}
