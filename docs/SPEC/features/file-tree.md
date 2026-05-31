---
priority: P1
layer: gui
depends: [workspace-layout]
---

- [x]

# File Tree

Отображение .md файлов в vault в виде дерева. Часть Project Panel (левая панель workspace).

## Поведение

- Показывает все .md файлы рекурсивно
- Директории раскрываются/схлопываются (клик по папке)
- Клик по файлу → открыть в редакторе (вкладка)
- Корень vault не показывается (сразу содержимое)

## UI

- Список из gpui-component или кастомный
- Иконка файла: обычный файл или `FileWarning` (жёлтая) если `vault.get_diagnostics(path)` непустой
- Только имя файла без расширения `.md`
- Отступы для вложенности директорий
- Изменяемая ширина

### Diagnostics

Каждый элемент дерева запрашивает `vault.get_diagnostics(path)`. Если есть diagnostics — иконка меняется на `FileWarning`. При наведении — tooltip со списком сообщений.

```rust
fn render_file_row(path: &Path, vault: &Vault) -> FileRow {
    let diagnostics = vault.get_diagnostics(path);
    let icon = if diagnostics.is_empty() {
        Icon::File
    } else {
        Icon::FileWarning
    };
    let tooltip = if diagnostics.is_empty() {
        None
    } else {
        Some(diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join("\n"))
    };
    // ...
}
```

## Resize

Project Panel (и FileTree внутри) имеет изменяемую ширину. Дефолт 250px.

Drag handle — вертикальная полоса на правой границе Project Panel:

```
┌──────────────────┬─────────────────────┐
│  Project Panel   ║  Editor Panel       │
│                  ║                     │
│  file1.md        ║                     │
│  file2.md        ║                     │
│                  ║                     │
│        ⇤ drag ⇥  ║                     │
└──────────────────┴─────────────────────┘
```

- При наведении на handle — курсор меняется на `col-resize`
- Drag = изменение ширины Project Panel
- Минимальная ширина: 150px
- Максимальная ширина: 40% от окна
- Ширина сохраняется в `Settings.project_panel_width`
