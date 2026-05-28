//! lindera (IPADIC) ラッパ。Phase 1b の rule から共通利用する。

use lindera::dictionary::load_dictionary;
use lindera::mode::Mode;
use lindera::segmenter::Segmenter;
use lindera::tokenizer::Tokenizer;
use once_cell::sync::Lazy;

static TOKENIZER: Lazy<Tokenizer> = Lazy::new(|| {
    let dict =
        load_dictionary("embedded://ipadic").expect("failed to load embedded IPADIC dictionary");
    let segmenter = Segmenter::new(Mode::Normal, dict, None);
    Tokenizer::new(segmenter)
});

#[derive(Debug, Clone)]
pub struct Token {
    pub surface: String,
    pub byte_start: usize,
    pub byte_end: usize,
    pub pos: String,
    pub pos_detail_1: String,
    pub pos_detail_2: String,
    pub pos_detail_3: String,
    pub base_form: String,
    pub conjugated_form: String,
    pub reading: String,
}

pub fn tokenize(text: &str) -> Vec<Token> {
    let mut raw = match TOKENIZER.tokenize(text) {
        Ok(t) => t,
        Err(_) => return Vec::new(),
    };
    let mut out = Vec::with_capacity(raw.len());
    for token in raw.iter_mut() {
        let details = token.details();
        let pos = details.first().map(|s| s.to_string()).unwrap_or_default();
        let p1 = details.get(1).map(|s| s.to_string()).unwrap_or_default();
        let p2 = details.get(2).map(|s| s.to_string()).unwrap_or_default();
        let p3 = details.get(3).map(|s| s.to_string()).unwrap_or_default();
        let conj_form = details.get(5).map(|s| s.to_string()).unwrap_or_default();
        let base = details.get(6).map(|s| s.to_string()).unwrap_or_default();
        let reading = details.get(7).map(|s| s.to_string()).unwrap_or_default();
        out.push(Token {
            surface: token.surface.to_string(),
            byte_start: token.byte_start,
            byte_end: token.byte_end,
            pos,
            pos_detail_1: p1,
            pos_detail_2: p2,
            pos_detail_3: p3,
            base_form: base,
            conjugated_form: conj_form,
            reading,
        });
    }
    out
}
