#[derive(Debug, Clone)]
pub(crate) enum Document {
    Nodes(Vec<Block>),
}

#[derive(Debug, Clone)]
pub(crate) enum Block {
    Paragraph {
        inlines: Vec<inline::Inline>,
    },
    Heading {
        level: u8,
        inlines: Vec<inline::Inline>,
    },
    List {
        ordered: bool,
        start: usize,
        tight: bool,
        items: Vec<ListItem>,
    },
    BlockQuote {
        children: Vec<Block>,
    },
    CodeBlock {
        info: Option<String>,
        content: String,
    },
    ThematicBreak,
    Table {
        aligns: Vec<Option<TableAlignment>>,
        header: Vec<Vec<inline::Inline>>,
        rows: Vec<Vec<Vec<inline::Inline>>>,
    },
    HtmlBlock(String),
}

#[derive(Debug, Clone)]
pub(crate) struct ListItem {
    pub(crate) children: Vec<Block>,
    pub(crate) task: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TableAlignment {
    Left,
    Center,
    Right,
}

pub(crate) mod inline {
    #[derive(Debug, Clone)]
    pub(crate) enum Inline {
        Text(String),
        RawHtml(String),
        SoftBreak,
        HardBreak,
        Code(String),
        Em(Vec<Inline>),
        Strong(Vec<Inline>),
        Del(Vec<Inline>),
        Link {
            label: Vec<Inline>,
            href: String,
            title: Option<String>,
        },
        Image {
            alt: Vec<Inline>,
            src: String,
            title: Option<String>,
        },
    }
}
