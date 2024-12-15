use rafka_broker::Broker;
use rafka_storage::db::RetentionPolicy;
use std::time::Duration;

#[tokio::main]
async fn main() {
    let partition_id = 0;
    let total_partitions = 1;
    let retention_policy = RetentionPolicy {
        max_age: Duration::from_secs(1000000),
        max_bytes: 1024 * 1024 * 1024,
    };
    let broker = Broker::new(partition_id, total_partitions, Some(retention_policy));
    broker.serve("0.0.0.0:8000").await;
}
