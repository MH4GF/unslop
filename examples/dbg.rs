use unslop::morph::tokenize;

fn main() {
    for t in tokenize("また、しかし、また、行きましょう。") {
        println!(
            "{:>10} {:>4}/{:>4}  pos={} detail={}/{}/{}",
            t.surface,
            t.byte_start,
            t.byte_end,
            t.pos,
            t.pos_detail_1,
            t.pos_detail_2,
            t.pos_detail_3
        );
    }
}
