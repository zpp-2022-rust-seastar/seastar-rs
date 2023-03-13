async fn frobnicate() -> i32 {
    42
}

#[seastar::main]
async fn main() {
    println!("Frobnication result: {}", frobnicate().await);
}
