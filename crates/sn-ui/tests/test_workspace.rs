use sn_ui::workspace::Workspace;

#[gpui::test]
fn test_workspace_new(cx: &mut gpui::TestAppContext) {
    let _workspace = cx.update(Workspace::new);
}

#[gpui::test]
fn test_workspace_state(cx: &mut gpui::TestAppContext) {
    let workspace = cx.update(Workspace::new);
    cx.update(|app| {
        let state = workspace.state.read(app);
        assert!(!state.title.is_empty());
    });
}
