fn main() {
    let mut total: i64 = 0;
    let amount = 9223372036854775807;
    total = total.checked_add(amount).unwrap();
    println!("{}", total);
}
