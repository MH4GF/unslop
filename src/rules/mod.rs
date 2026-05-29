pub mod ai_writing;
pub mod jt;
pub mod prh;
pub mod unslop;

/// jt rule で共通の「Str を含む」block kind 判定。
pub(crate) fn is_str_bearing(kind: crate::document::SegmentKind) -> bool {
    use crate::document::SegmentKind::*;
    matches!(kind, Paragraph | Heading | ListItem | TableCell)
}
