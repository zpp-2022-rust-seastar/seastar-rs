use key_value_store::*;

#[seastar::main]
async fn main() {
    let distr = seastar::Distributed::start(Server::new).await;
    let futs = distr.map_all(|sharded| ShardedServer(sharded).run(5555));
    futures::future::join_all(futs).await;
    distr.stop().await;
}
