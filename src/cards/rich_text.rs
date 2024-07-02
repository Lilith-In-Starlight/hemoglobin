use serde::de::Visitor;

use super::CardId;

enum RichElement {
    String(String),
    CardId { display: String, card_id: CardId },
    SpecificCard { display: String, id: String },
    Saga(Vec<RichString>),
    LineBreak,
}

struct RichString {
    elements: Vec<RichElement>,
}
