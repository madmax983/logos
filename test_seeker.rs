fn main() {
    let mut low = 0_i64;
    let target_cents = i64::MAX;
    let initial_net_worth_cents = 5_000_000_i64;
    let target_months = 10_i64;

    let diff = target_cents.saturating_sub(initial_net_worth_cents);
    let mut high = diff.saturating_add(1_000_000_000);

    let mut best_savings = None;

    let mut iter = 0;
    while low <= high {
        iter += 1;
        let mid = low.saturating_add((high.saturating_sub(low)) / 2);

        let mut final_nw = initial_net_worth_cents;
        for _ in 1..=target_months {
            final_nw = final_nw.saturating_add(mid);
        }

        if final_nw >= target_cents {
            best_savings = Some(mid);
            high = mid.saturating_sub(1);
        } else {
            low = mid.saturating_add(1);
        }

        if iter > 100 {
            println!("Inf loop!");
            break;
        }
    }
    println!("best: {:?}, iters: {}", best_savings, iter);
}
