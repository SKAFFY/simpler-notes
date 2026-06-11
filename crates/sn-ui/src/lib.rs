pub mod app;
pub mod command;
pub mod workspace;
pub mod panel;
pub mod dock;
pub mod layout_state;
pub mod testing;

pub use app::SnApp;
pub use workspace::Workspace;
pub use panel::PanelView;
pub use testing::MockPanel;
pub use crate::dock::area::DockArea;
pub use dock::DockPlacement;
