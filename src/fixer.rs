//! Fix の連鎖適用エンジン。
//!
//! 1 pass 内: range.start 昇順で sort → overlap した fix は両方 drop → 残りを降順 apply。
//! 多 pass: lint → fix を集める → apply → 再 lint をループ、収束または MAX_PASSES で停止する。

use crate::rule::Fix;

pub const MAX_PASSES: usize = 10;

/// fixes を `source` に適用し `(fixed_source, applied, dropped)` を返す。
/// overlap した fix は両方 drop して次 pass の再 lint に委ねる (データ破壊回避)。
pub(crate) fn apply_fixes(source: &str, fixes: &[Fix]) -> (String, Vec<Fix>, Vec<Fix>) {
    if fixes.is_empty() {
        return (source.to_string(), Vec::new(), Vec::new());
    }

    let mut sorted: Vec<Fix> = fixes.to_vec();
    sorted.sort_by_key(|f| f.range.start);

    let mut applied: Vec<Fix> = Vec::with_capacity(sorted.len());
    let mut dropped: Vec<Fix> = Vec::new();
    let mut i = 0;
    while i < sorted.len() {
        let mut group_end = sorted[i].range.end;
        let mut j = i + 1;
        while j < sorted.len() && sorted[j].range.start < group_end {
            group_end = group_end.max(sorted[j].range.end);
            j += 1;
        }
        if j == i + 1 {
            applied.push(sorted[i].clone());
        } else {
            for f in &sorted[i..j] {
                dropped.push(f.clone());
            }
        }
        i = j;
    }

    let mut buf = source.to_string();
    for f in applied.iter().rev() {
        let end = f.range.end.min(buf.len());
        let start = f.range.start.min(end);
        buf.replace_range(start..end, &f.replacement);
    }

    (buf, applied, dropped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fix(range: std::ops::Range<usize>, replacement: &str) -> Fix {
        Fix {
            range,
            replacement: replacement.to_string(),
        }
    }

    #[test]
    fn applies_single_fix() {
        let (s, a, d) = apply_fixes("hello world", &[fix(6..11, "rust")]);
        assert_eq!(s, "hello rust");
        assert_eq!(a.len(), 1);
        assert!(d.is_empty());
    }

    #[test]
    fn applies_non_overlapping_descending() {
        let (s, a, d) = apply_fixes("aXbXc", &[fix(1..2, "1"), fix(3..4, "2")]);
        assert_eq!(s, "a1b2c");
        assert_eq!(a.len(), 2);
        assert!(d.is_empty());
    }

    #[test]
    fn replacement_changes_length_safely() {
        let (s, _, _) = apply_fixes("AAA BBB CCC", &[fix(0..3, "x"), fix(8..11, "yyyyy")]);
        assert_eq!(s, "x BBB yyyyy");
    }

    #[test]
    fn overlap_drops_both() {
        let (s, a, d) = apply_fixes("abcdef", &[fix(0..3, "X"), fix(2..5, "Y")]);
        assert_eq!(s, "abcdef");
        assert!(a.is_empty());
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn duplicate_range_drops_both() {
        let (s, a, d) = apply_fixes("abc", &[fix(0..1, "X"), fix(0..1, "Y")]);
        assert_eq!(s, "abc");
        assert!(a.is_empty());
        assert_eq!(d.len(), 2);
    }

    #[test]
    fn empty_fixes_returns_source() {
        let (s, a, d) = apply_fixes("hello", &[]);
        assert_eq!(s, "hello");
        assert!(a.is_empty());
        assert!(d.is_empty());
    }

    #[test]
    fn adjacent_ranges_both_apply() {
        let (s, a, _) = apply_fixes("abcdef", &[fix(0..2, "X"), fix(2..4, "Y")]);
        assert_eq!(s, "XYef");
        assert_eq!(a.len(), 2);
    }
}
