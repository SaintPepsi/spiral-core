fn factorial(n: u64) -> u64 {
    match n {
        0 | 1 => 1,
        _ => n * factorial(n - 1),
    }
}

fn main() {
    println!("5! = {}", factorial(5));
    println!("10! = {}", factorial(10));
    println!("20! = {}", factorial(20));
}