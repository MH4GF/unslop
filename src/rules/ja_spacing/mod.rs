pub mod ja_no_space_between_full_width;
pub mod ja_space_between_half_and_full_width;

/// 全角文字の判定。upstream の `ZenRegExpStr` + 句読点 + `々〇〻` 相当を扱う。
///
/// upstream は `[、。]|[㐀-䶿一-鿿豈-﫿]|[\uD840-\uD87F][\uDC00-\uDFFF]|[ぁ-んァ-ヶ]` に
/// `々〇〻` を加えた集合を扱う (`ja-no-space-between-full-width` 側は `々〇〻` を含む)。
/// unslop では char 単位の判定なので surrogate pair は CJK Ext B 以降の単一 char 範囲に展開する。
pub(crate) fn is_full_width(c: char) -> bool {
    matches!(
        c,
        '々' | '〇' | '〻' | '、' | '。'
        | '\u{3400}'..='\u{4DBF}'
        | '\u{4E00}'..='\u{9FFF}'
        | '\u{F900}'..='\u{FAFF}'
        | '\u{20000}'..='\u{2A6DF}'
        | '\u{2A700}'..='\u{2EBEF}'
        | '\u{3041}'..='\u{3093}'
        | '\u{30A1}'..='\u{30F6}'
    )
}

/// upstream の `katakakana = /[ァ-ヶ]( )[ァ-ヶ]/` 例外用。
pub(crate) fn is_katakana_in_compound(c: char) -> bool {
    matches!(c, '\u{30A1}'..='\u{30F6}')
}

/// `[A-Za-z0-9]` の判定。upstream の `space: "always"` で `alphabets: true, numbers: true` に対応する。
pub(crate) fn is_half_width_alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// 句読点 `、。` の判定。`exceptPunctuation: true` (always のデフォルト) で除外する。
pub(crate) fn is_zen_punctuation(c: char) -> bool {
    matches!(c, '、' | '。')
}

/// segment 相対 byte 範囲 `[s, e)` が code span / link URL に重なるかを返す。
pub(crate) fn range_in_excluded(seg: &crate::document::TextSegment, s: usize, e: usize) -> bool {
    seg.code_ranges.iter().any(|&(cs, ce)| s < ce && cs < e)
        || seg.link_url_ranges.iter().any(|&(cs, ce)| s < ce && cs < e)
}
