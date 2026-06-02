//! Markdown document → segment 抽出。
//!
//! textlint 互換のため、各 segment は **markdown source の生 slice** を保持する。
//! 上位 rule は segment.text 内 byte offset を Document::pos_at で絶対 (line, column) に解決する。

use comrak::nodes::{AstNode, NodeValue, Sourcepos};
use comrak::{Arena, Options, parse_document};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentKind {
    Paragraph,
    Heading,
    BlockQuote,
    ListItem,
    TableCell,
    CodeBlock,
}

#[derive(Debug, Clone)]
pub struct TextSegment {
    /// markdown source の該当 block 範囲をそのまま slice。
    pub text: String,
    /// source 内 byte offset (segment 起点)。
    pub start_byte: usize,
    /// source 内 line (1-based, segment 起点)。
    pub start_line: usize,
    /// source 内 column (1-based, char ベース, segment 起点)。
    pub start_column: usize,
    pub kind: SegmentKind,
    /// segment が BlockQuote 配下にあるか。引用を対象外にする rule が参照する。
    pub in_block_quote: bool,
    /// segment 内 inline code span の byte 範囲 (segment 相対, 区切り文字含む)。
    /// prh 等が text ノード相当だけを見るために、この範囲のマッチをスキップする。
    pub code_ranges: Vec<(usize, usize)>,
    /// segment 内 Link/Image node の URL 部分の byte 範囲 (segment 相対)。
    /// `[label](url)` の URL や `<url>` autolink、GFM bare autolink を含む。
    /// URL 内の `?` を textlint と同様に lint 対象外にするための除外範囲。
    pub link_url_ranges: Vec<(usize, usize)>,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub source: String,
    pub segments: Vec<TextSegment>,
    /// 各 line (1-based) の先頭 byte offset。
    line_starts: Vec<usize>,
}

impl Document {
    pub fn parse(source: &str) -> Self {
        let arena = Arena::new();
        let mut opts = Options::default();
        opts.extension.table = true;
        opts.extension.strikethrough = true;
        opts.extension.autolink = true;
        opts.extension.tasklist = true;
        opts.render.sourcepos = true;
        let root = parse_document(&arena, source, &opts);

        let line_starts = build_line_starts(source);
        let mut segments = Vec::new();
        collect(root, false, false, source, &line_starts, &mut segments);

        let fm_end = frontmatter_end(source);
        if fm_end > 0 {
            segments.retain(|s| s.start_byte >= fm_end);
        }

        Document {
            source: source.to_string(),
            segments,
            line_starts,
        }
    }

    /// 任意 source byte offset を絶対 (line, column, 1-based, char ベース) に解決する。
    /// 出力 column は textlint の出力に合わせて char count (multi-byte 1 文字 = column 1) で返す。
    pub fn locate(&self, byte_offset: usize) -> (usize, usize) {
        let line_idx = match self.line_starts.binary_search(&byte_offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        let line_slice = &self.source[line_start..byte_offset.min(self.source.len())];
        let col = line_slice.chars().count() + 1;
        (line_idx + 1, col)
    }

    /// segment 内 byte offset を絶対 (line, column) に解決する。
    pub fn pos_at(&self, segment: &TextSegment, segment_byte_offset: usize) -> (usize, usize) {
        let abs = segment.start_byte + segment_byte_offset.min(segment.text.len());
        self.locate(abs)
    }
}

/// `---` で囲まれた YAML frontmatter の末尾バイト (exclusive) を返す。
/// frontmatter が存在しない場合は 0 を返す。
fn frontmatter_end(source: &str) -> usize {
    if !source.starts_with("---\n") {
        return 0;
    }
    let body = &source[4..];
    if let Some(pos) = body.find("\n---\n") {
        4 + pos + 5
    } else if let Some(pos) = body.find("\n---\r\n") {
        4 + pos + 6
    } else if body.ends_with("\n---") {
        source.len()
    } else {
        0
    }
}

fn build_line_starts(source: &str) -> Vec<usize> {
    let mut v = vec![0usize];
    for (i, b) in source.bytes().enumerate() {
        if b == b'\n' {
            v.push(i + 1);
        }
    }
    v
}

/// comrak の (line, byte-column) を source 全体の byte offset に変換する。
/// comrak の column は **1-based byte index within the line**。
fn byte_offset_start(source: &str, line_starts: &[usize], line: usize, col: usize) -> usize {
    if line == 0 || line > line_starts.len() {
        return source.len();
    }
    let line_start = line_starts[line - 1];
    let line_end = line_starts.get(line).copied().unwrap_or(source.len());
    (line_start + col.saturating_sub(1)).min(line_end)
}

/// end.column (1-based byte column of **last byte of last char**) を exclusive offset に変換。
/// multi-byte char の last byte を指してくるため、その char の end byte (exclusive) を返す。
fn byte_offset_end_exclusive(
    source: &str,
    line_starts: &[usize],
    line: usize,
    col: usize,
) -> usize {
    if line == 0 || line > line_starts.len() {
        return source.len();
    }
    let line_start = line_starts[line - 1];
    let line_end = line_starts.get(line).copied().unwrap_or(source.len());
    let i = (line_start + col.saturating_sub(1)).min(line_end.saturating_sub(1).max(line_start));
    let line_slice = &source[line_start..line_end];
    let target_off = i.saturating_sub(line_start);
    // i 以下の最大 char boundary を見つけ、その char の end を返す。
    let mut last_start = 0usize;
    let mut last_len = 0usize;
    for (off, c) in line_slice.char_indices() {
        if off > target_off {
            break;
        }
        last_start = off;
        last_len = c.len_utf8();
    }
    line_start + last_start + last_len
}

fn segment_slice(
    source: &str,
    line_starts: &[usize],
    pos: Sourcepos,
) -> Option<(String, usize, usize, usize)> {
    let start = byte_offset_start(source, line_starts, pos.start.line, pos.start.column);
    let end = byte_offset_end_exclusive(source, line_starts, pos.end.line, pos.end.column);
    if start >= end {
        return None;
    }
    let text = source[start..end].to_string();
    Some((text, start, pos.start.line, pos.start.column))
}

fn push_segment<'a>(
    out: &mut Vec<TextSegment>,
    source: &str,
    line_starts: &[usize],
    node: &'a AstNode<'a>,
    pos: Sourcepos,
    kind: SegmentKind,
    in_block_quote: bool,
) {
    if let Some((text, start_byte, start_line, start_column)) =
        segment_slice(source, line_starts, pos)
    {
        let mut code_ranges = Vec::new();
        collect_code_ranges(
            node,
            source,
            line_starts,
            start_byte,
            text.len(),
            &mut code_ranges,
        );
        let mut link_url_ranges = Vec::new();
        collect_link_url_ranges(
            node,
            source,
            line_starts,
            start_byte,
            text.len(),
            &mut link_url_ranges,
        );
        out.push(TextSegment {
            text,
            start_byte,
            start_line,
            start_column,
            kind,
            in_block_quote,
            code_ranges,
            link_url_ranges,
        });
    }
}

/// block ノード配下の inline `Code` ノードを集め、segment 相対 byte 範囲に変換する。
/// emphasis 等にネストした code span も拾うため subtree 全体を走査する。
fn collect_code_ranges<'a>(
    node: &'a AstNode<'a>,
    source: &str,
    line_starts: &[usize],
    seg_start: usize,
    seg_len: usize,
    out: &mut Vec<(usize, usize)>,
) {
    for descendant in node.descendants() {
        let data = descendant.data.borrow();
        if !matches!(data.value, NodeValue::Code(_)) {
            continue;
        }
        let pos = data.sourcepos;
        let abs_start = byte_offset_start(source, line_starts, pos.start.line, pos.start.column);
        let abs_end = byte_offset_end_exclusive(source, line_starts, pos.end.line, pos.end.column);
        if abs_end <= abs_start || abs_start < seg_start {
            continue;
        }
        let rel_start = abs_start - seg_start;
        let rel_end = (abs_end - seg_start).min(seg_len);
        if rel_start < rel_end {
            out.push((rel_start, rel_end));
        }
    }
}

/// block ノード配下の `Link` / `Image` ノードを集め、URL 部分の segment 相対 byte 範囲を返す。
/// `[label](url)` / `![alt](url)` / `<url>` / GFM bare autolink を扱う。
fn collect_link_url_ranges<'a>(
    node: &'a AstNode<'a>,
    source: &str,
    line_starts: &[usize],
    seg_start: usize,
    seg_len: usize,
    out: &mut Vec<(usize, usize)>,
) {
    for descendant in node.descendants() {
        let data = descendant.data.borrow();
        let is_image = matches!(data.value, NodeValue::Image(_));
        let is_link = matches!(data.value, NodeValue::Link(_));
        if !is_link && !is_image {
            continue;
        }
        let pos = data.sourcepos;
        let abs_start = byte_offset_start(source, line_starts, pos.start.line, pos.start.column);
        let abs_end = byte_offset_end_exclusive(source, line_starts, pos.end.line, pos.end.column);
        if abs_end <= abs_start || abs_start < seg_start {
            continue;
        }
        let link_src = &source[abs_start..abs_end];
        // Image は構文全体 (先頭 `!` 含む) を除外。alt text の取り扱いは upstream と差分が出るが、
        // 実用上 alt に `?`/`!` を含む例は稀で、誤検知の方が問題になる。
        let range = if is_image {
            Some((0, link_src.len()))
        } else {
            extract_url_range(link_src)
        };
        let Some((us, ue)) = range else {
            continue;
        };
        let rel_start = (abs_start + us).saturating_sub(seg_start);
        let rel_end = ((abs_start + ue).saturating_sub(seg_start)).min(seg_len);
        if rel_start < rel_end {
            out.push((rel_start, rel_end));
        }
    }
}

/// Link/Image の sourcepos slice から URL 部分の (start, end) byte offset を返す。
/// inline link `[label](url)` と image `![alt](url)` は `](` 以降 `)` 直前まで。
/// angle autolink `<url>` は `<` と `>` の間。
/// GFM bare autolink は slice 全体が URL。
/// reference link `[a][r]` 等 URL が inline source に出ないものは `None`。
pub(crate) fn extract_url_range(s: &str) -> Option<(usize, usize)> {
    if s.is_empty() {
        return None;
    }
    let bytes = s.as_bytes();
    if bytes.first() == Some(&b'<') && bytes.last() == Some(&b'>') && s.len() >= 2 {
        return Some((1, s.len() - 1));
    }
    if bytes.last() == Some(&b')')
        && let Some(open) = s.rfind("](")
    {
        let url_start = open + 2;
        let url_end = s.len() - 1;
        if url_start < url_end {
            return Some((url_start, url_end));
        }
    }
    if s.starts_with("http://")
        || s.starts_with("https://")
        || s.starts_with("ftp://")
        || s.starts_with("www.")
        || s.starts_with("mailto:")
    {
        return Some((0, s.len()));
    }
    None
}

fn collect<'a>(
    node: &'a AstNode<'a>,
    parent_is_list_item: bool,
    in_block_quote: bool,
    source: &str,
    line_starts: &[usize],
    out: &mut Vec<TextSegment>,
) {
    let data = node.data.borrow();
    let pos = data.sourcepos;
    let value = data.value.clone();
    drop(data);

    let mut child_parent_is_list_item = false;
    let mut child_in_block_quote = in_block_quote;
    match &value {
        NodeValue::Paragraph => {
            if !parent_is_list_item {
                push_segment(
                    out,
                    source,
                    line_starts,
                    node,
                    pos,
                    SegmentKind::Paragraph,
                    in_block_quote,
                );
            }
            // textlint と同じく、ListItem 直下 Paragraph は ListItem 側で扱うのでスキップ。
            return;
        }
        NodeValue::Heading(_) => {
            push_segment(
                out,
                source,
                line_starts,
                node,
                pos,
                SegmentKind::Heading,
                in_block_quote,
            );
            return;
        }
        NodeValue::TableCell => {
            push_segment(
                out,
                source,
                line_starts,
                node,
                pos,
                SegmentKind::TableCell,
                in_block_quote,
            );
            return;
        }
        NodeValue::Item(_) => {
            push_segment(
                out,
                source,
                line_starts,
                node,
                pos,
                SegmentKind::ListItem,
                in_block_quote,
            );
            // 入れ子の list は内部も再帰する (各 Item を個別 segment 化)。
            child_parent_is_list_item = true;
        }
        NodeValue::BlockQuote => {
            // BlockQuote 自体は segment にしない。中の Paragraph で拾われる。
            child_in_block_quote = true;
        }
        NodeValue::CodeBlock(_) | NodeValue::HtmlBlock(_) | NodeValue::ThematicBreak => {
            return;
        }
        _ => {}
    }

    for child in node.children() {
        collect(
            child,
            child_parent_is_list_item,
            child_in_block_quote,
            source,
            line_starts,
            out,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_url_range_angle_autolink() {
        let s = "<https://x.test/a?b=1>";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "https://x.test/a?b=1");
    }

    #[test]
    fn extract_url_range_inline_link() {
        let s = "[label](https://x.test/?z=1)";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "https://x.test/?z=1");
    }

    #[test]
    fn extract_url_range_label_with_bracket_uses_rightmost() {
        // label に `]` を含むケースは rfind("](") が右端を採用する
        let s = "[a]b](https://x.test/?q=1)";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "https://x.test/?q=1");
    }

    #[test]
    fn extract_url_range_url_with_balanced_paren() {
        let s = "[a](http://x.test/(y))";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "http://x.test/(y)");
    }

    #[test]
    fn extract_url_range_image() {
        let s = "![alt](/p/q.png?x=1)";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "/p/q.png?x=1");
    }

    #[test]
    fn extract_url_range_bare_autolink() {
        let s = "https://x.test/?z=1";
        let (us, ue) = extract_url_range(s).unwrap();
        assert_eq!(&s[us..ue], "https://x.test/?z=1");
    }

    #[test]
    fn extract_url_range_reference_link_returns_none() {
        // [a][r] は URL が inline source に出てこない
        assert_eq!(extract_url_range("[a][r]"), None);
    }

    #[test]
    fn extract_url_range_empty_returns_none() {
        assert_eq!(extract_url_range(""), None);
    }
}
