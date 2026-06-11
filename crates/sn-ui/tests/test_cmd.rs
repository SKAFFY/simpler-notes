use sn_ui::command::handler::{OpenVault, CloseTab, SaveFile};

#[gpui::test]
fn test_action_types_compile(cx: &mut gpui::TestAppContext) {
    cx.update(|_app| {
        let _open = OpenVault;
        let _close_tab = CloseTab;
        let _save = SaveFile;
    });
}
