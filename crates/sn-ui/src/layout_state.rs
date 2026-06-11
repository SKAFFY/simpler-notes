use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockItemState {
    pub variant: DockItemVariant,
    pub children: Vec<DockItemState>,
    pub sizes: Vec<Option<f32>>,
    pub active_index: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockItemVariant {
    Split {
        axis: String,
    },
    Tabs,
    Panel {
        name: String,
    },
}

impl DockItemVariant {
    pub fn name(&self) -> Option<&str> {
        match self {
            DockItemVariant::Panel { name } => Some(name.as_str()),
            _ => None,
        }
    }
}
