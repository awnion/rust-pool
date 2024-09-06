use deadpool_postgres::{Config, ManagerConfig, RecyclingMethod, Runtime};
use testcontainers_modules::postgres;
use testcontainers_modules::testcontainers::runners::AsyncRunner;
use testcontainers_modules::testcontainers::ImageExt;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let mut p = Vec::new();
    for i in 11..=16 {
        p.push(tokio::spawn(async move {
            let container = postgres::Postgres::default()
                .with_tag(format!("{i}"))
                .start()
                .await
                .unwrap();
            let host_port = container.get_host_port_ipv4(5432).await.unwrap();
            let host = container.get_host().await.unwrap();
            let connection_string =
                format!("postgres://postgres:postgres@{host}:{host_port}/postgres");

            container.stdout(true);

            let mut cfg = Config::new();
            cfg.url = Some(connection_string);
            cfg.manager = Some(ManagerConfig {
                recycling_method: RecyclingMethod::Fast,
            });
            let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();
            for i in 1..10i32 {
                println!("i = {i}");
                let client = pool.get().await.unwrap();
                let stmt = client.prepare_cached("SELECT 1 + $1").await.unwrap();
                let rows = client.query(&stmt, &[&i]).await.unwrap();
                let value: i32 = rows[0].get(0);
                assert_eq!(value, i + 1);
            }
            println!("Container {:?}", container);
            container.stop().await.unwrap();
            container.rm().await.unwrap();
        }));
    }
    for x in p {
        x.await.unwrap();
    }
}
