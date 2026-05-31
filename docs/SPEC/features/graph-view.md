---
priority: P2
layer: gui
depends: [vault]
---

- [x]

# Graph View (MindMap)

Визуализация связей между заметками через `[[вики-ссылки]]`. Нижняя панель в workspace, альтернатива Timeline.

## Поведение

- Каждая заметка — узел (node) на графе
- `[[Note A]]` в заметке B → ребро между B и A
- Force-directed layout — узлы отталкиваются, связи притягивают
- **Кластеризация**: на основе связности (connected components или community detection) — плотно связанные заметки группируются вместе визуально
- Клик по узлу → открыть заметку
- Перетаскивание узлов не поддерживается в MVP

## Данные

### Кластеризация

Граф строится как forest связных компонент. Каждый connected component — отдельный кластер. Внутри кластера — force-directed layout. Кластеры располагаются с отступом друг от друга.

```rust
fn build_graph(vault: &Vault) -> Graph {
    let mut nodes: Vec<PathBuf> = vault.list_md_files();
    let mut edges: Vec<(PathBuf, PathBuf)> = Vec::new();

    for path in &nodes {
        let outgoing = vault.get_outgoing_links(path);
        for link in outgoing {
            edges.push((link.source, link.target));
        }
    }

    // Connected components для кластеризации
    let clusters = find_connected_components(&nodes, &edges);

    Graph { clusters, edges }
}

struct Cluster {
    nodes: Vec<PathBuf>,
    position: Point,  // центр кластера
}

struct Graph {
    clusters: Vec<Cluster>,
    edges: Vec<(PathBuf, PathBuf)>,
}
```

Force-directed layout применяется внутри каждого кластера. Центры кластеров равномерно распределяются по canvas.

### Рёбра

- Узлы: все .md файлы в vault
- Рёбра: из `LinkIndex` — `vault.get_backlinks(target)` для обратных связей, `vault.get_outgoing_links(source)` для прямых
- Направление: от файла к файлу, на который ссылаются

## Динамическое сужение

При поиске (в будущем) граф показывает только узлы и связи, релевантные запросу.

## Ограничения

- Для MVP: простой force-directed layout (без GPU)
- Без интерактивного перетаскивания
- Без зума
