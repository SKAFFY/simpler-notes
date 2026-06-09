use comrak::nodes::{ListType, NodeValue};
use comrak::{parse_document, Arena, Options};
use iced::widget::{scrollable, Column};
use iced::{Color, Element, Fill, Font};

use crate::app::{App, Message};

#[derive(Debug)]
enum MdNode {
    Heading { level: u8, children: Vec<MdNode> },
    Paragraph(Vec<MdNode>),
    Text(String),
    Code(String),
    CodeBlock(String),
    List(bool, Vec<Vec<MdNode>>),
    BlockQuote(Vec<MdNode>),
    ThematicBreak,
    Strong(Vec<MdNode>),
    Emph(Vec<MdNode>),
    Link { text: String, url: String },
}

fn convert_ast<'a>(node: &'a comrak::nodes::AstNode<'a>) -> Vec<MdNode> {
    let mut result = Vec::new();

    for child in node.children() {
        let data = child.data.borrow();
        let value = &data.value;

        match value {
            NodeValue::Heading(heading) => {
                let children = convert_ast(child);
                result.push(MdNode::Heading {
                    level: heading.level,
                    children,
                });
            }
            NodeValue::Paragraph => {
                let children = convert_ast(child);
                result.push(MdNode::Paragraph(children));
            }
            NodeValue::Text(ref t) => {
                result.push(MdNode::Text(t.to_string()));
            }
            NodeValue::CodeBlock(ref cb) => {
                result.push(MdNode::CodeBlock(cb.literal.to_string()));
            }
            NodeValue::Code(ref c) => {
                result.push(MdNode::Code(c.literal.to_string()));
            }
            NodeValue::List(list) => {
                let ordered = list.list_type == ListType::Ordered;
                let mut items = Vec::new();
                for item_node in child.children() {
                    let item_children = convert_ast(item_node);
                    items.push(item_children);
                }
                result.push(MdNode::List(ordered, items));
            }
            NodeValue::Item(_) => {
                let children = convert_ast(child);
                result.extend(children);
            }
            NodeValue::BlockQuote => {
                let children = convert_ast(child);
                result.push(MdNode::BlockQuote(children));
            }
            NodeValue::ThematicBreak => {
                result.push(MdNode::ThematicBreak);
            }
            NodeValue::Strong => {
                let children = convert_ast(child);
                result.push(MdNode::Strong(children));
            }
            NodeValue::Emph => {
                let children = convert_ast(child);
                result.push(MdNode::Emph(children));
            }
            NodeValue::Link(link) => {
                let mut link_text = String::new();
                for inner in child.children() {
                    let inner_data = inner.data.borrow();
                    if let NodeValue::Text(ref t) = &inner_data.value {
                        link_text.push_str(t.as_str());
                    }
                }
                result.push(MdNode::Link {
                    text: link_text,
                    url: link.url.to_string(),
                });
            }
            _ => {
                let children = convert_ast(child);
                result.extend(children);
            }
        }
    }

    result
}

fn build_widgets(nodes: Vec<MdNode>) -> Vec<Element<'static, Message>> {
    let mut elements = Vec::new();

    for node in nodes {
        match node {
            MdNode::Heading { level: _level, children } => {
                let inner = build_widgets(children);
                elements.push(
                    iced::widget::row(inner).into(),
                );
            }
            MdNode::Paragraph(children) => {
                let inner = build_widgets(children);
                if inner.is_empty() {
                    elements.push(iced::widget::text("").into());
                } else {
                    elements.push(iced::widget::container(iced::widget::row(inner)).padding([0, 4]).into());
                }
            }
            MdNode::Text(t) => {
                elements.push(iced::widget::text(t).size(14).into());
            }
            MdNode::Code(code) => {
                elements.push(
                    iced::widget::container(iced::widget::text(code).font(Font::MONOSPACE).size(13))
                        .padding([2, 4])
                        .style(|_: &iced::Theme| {
                            iced::widget::container::Style::default()
                                .background(Color::from_rgb(0.15, 0.15, 0.15))
                        })
                        .into(),
                );
            }
            MdNode::CodeBlock(code) => {
                elements.push(
                    iced::widget::container(scrollable(iced::widget::text(code).font(Font::MONOSPACE).size(13)).height(Fill))
                        .padding(8)
                        .style(|_: &iced::Theme| {
                            iced::widget::container::Style::default()
                                .background(Color::from_rgb(0.12, 0.12, 0.12))
                        })
                        .into(),
                );
            }
            MdNode::List(ordered, items) => {
                let mut item_widgets = Vec::new();
                for (i, item_nodes) in items.into_iter().enumerate() {
                    let prefix = if ordered {
                        format!("{}. ", i + 1)
                    } else {
                        "• ".to_string()
                    };
                    let inner = build_widgets(item_nodes);
                    item_widgets.push(
                        iced::widget::row(
                            std::iter::once(iced::widget::text(prefix).size(14).into())
                                .chain(inner)
                                .collect::<Vec<_>>(),
                        )
                        .into(),
                    );
                }
                elements.push(Column::with_children(item_widgets).spacing(2).into());
            }
            MdNode::BlockQuote(children) => {
                let inner = build_widgets(children);
                elements.push(
                    iced::widget::container(Column::with_children(inner).spacing(4))
                        .padding([4, 8])
                        .style(|_: &iced::Theme| {
                            iced::widget::container::Style::default()
                                .background(Color::from_rgb(0.15, 0.15, 0.15))
                        })
                        .into(),
                );
            }
            MdNode::ThematicBreak => {
                elements.push(
                    iced::widget::container(iced::widget::text("───")).style(|_: &iced::Theme| {
                        iced::widget::container::Style::default()
                            .background(Color::from_rgb(0.3, 0.3, 0.3))
                    }).into(),
                );
            }
            MdNode::Strong(children) => {
                let inner = build_widgets(children);
                elements.extend(inner);
            }
            MdNode::Emph(children) => {
                let inner = build_widgets(children);
                elements.extend(inner);
            }
            MdNode::Link { text: link_text, url } => {
                elements.push(
                    iced::widget::text(format!("{} ({})", link_text, url))
                        .size(14)
                        .color(Color::from_rgb(0.4, 0.6, 1.0))
                        .into(),
                );
            }
        }
    }

    elements
}

pub fn view(app: &App) -> Element<'_, Message> {
    let markdown = app.active_editor_text();
    if markdown.is_empty() {
        return iced::widget::container(iced::widget::text("No content to preview")).into();
    }

    let processed = preprocess_wikilinks(&markdown);
    let arena = Arena::new();
    let root = parse_document(&arena, &processed, &Options::default());
    let nodes = convert_ast(root);

    let widgets = build_widgets(nodes);

    let content = Column::with_children(widgets).spacing(6).padding(8);

    scrollable(iced::widget::container(content).width(Fill)).height(Fill).into()
}

fn preprocess_wikilinks(input: &str) -> String {
    let re = regex::Regex::new(r"\[\[([^\]]+)\]\]").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        let target = &caps[1];
        format!("[{}]({}.md)", target, target)
    })
    .to_string()
}
