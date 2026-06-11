use std::collections::HashMap;

use gpui::*;

use crate::panel::PanelView;

type Builder = Box<dyn Fn(&mut App) -> Box<dyn PanelView> + Send + Sync>;

pub struct PanelRegistry {
    builders: HashMap<&'static str, Builder>,
}

impl PanelRegistry {
    pub fn new() -> Self {
        Self {
            builders: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &'static str, builder: Builder) {
        self.builders.insert(name, builder);
    }

    pub fn build(&self, name: &str, _cx: &mut App) -> Option<Box<dyn PanelView>> {
        self.builders.get(name).map(|b| b(_cx))
    }

    pub fn contains(&self, name: &str) -> bool {
        self.builders.contains_key(name)
    }
}

impl Default for PanelRegistry {
    fn default() -> Self {
        Self::new()
    }
}
