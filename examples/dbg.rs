use comrak::nodes::NodeValue;
use comrak::{Arena, Options, parse_document};

fn walk<'a>(node: &'a comrak::nodes::AstNode<'a>, depth: usize, src: &str) {
    let d = node.data.borrow();
    let name = format!("{:?}", d.value).chars().take(20).collect::<String>();
    println!(
        "{}{} sp=({},{})-({},{})",
        "  ".repeat(depth),
        name,
        d.sourcepos.start.line,
        d.sourcepos.start.column,
        d.sourcepos.end.line,
        d.sourcepos.end.column
    );
    drop(d);
    for c in node.children() {
        walk(c, depth + 1, src);
    }
}

fn show(src: &str) {
    println!("=== {:?}", src);
    println!("source bytes: {}, chars: {}", src.len(), src.chars().count());
    let arena = Arena::new();
    let mut opts = Options::default();
    opts.render.sourcepos = true;
    let root = parse_document(&arena, src, &opts);
    walk(root, 0, src);
}

fn main() {
    show("これは普通の文章です。");
    show("- 通常のリストアイテム");
    show("- **注意**: 重要な情報\n- 🔥 ホットな話題");
}
