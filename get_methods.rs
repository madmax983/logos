use aletheiadb::core::hlc::HybridTimestamp;
fn main() {
    let t: HybridTimestamp = HybridTimestamp::new(0, 0).unwrap();
    let i: i64 = t.wallclock() as i64;
    println!("{:?}", i);
}
