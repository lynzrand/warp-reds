use std::{
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{self, AtomicU64},
        Arc,
    },
    time::Duration,
};

use argh::FromArgs;
use warp::Filter;

#[derive(FromArgs)]
/// Rynco's ⚡ BLAZING FAST ⚡ `/hitreds` implementation.
struct Params {
    /// the port to bind. Defaults to 8880
    #[argh(option, default = "8880")]
    port: u16,

    /// the file to store the counter
    #[argh(option, default = "\"hitreds.txt\".into()")]
    file: PathBuf,
}

#[tokio::main]
async fn main() {
    let Params { port, file } = argh::from_env();

    let v = tokio::fs::read_to_string(&file).await.unwrap_or_default();
    let v = u64::from_str(&v).unwrap_or(0);

    let counter = Arc::new(AtomicU64::new(v));

    let server = warp::any().and(warp::header("user-agent")).map({
        let counter = counter.clone();
        move |agent: String| {
            let c = counter.fetch_add(1, atomic::Ordering::Relaxed);
            let minimum_2_pow = c.next_power_of_two();
            if agent.contains("KHTML") {
                format!(
                    "<html><body>打红人计数器 ({}/{})</body></html>",
                    c, minimum_2_pow
                )
            } else {
                format!("打红人计数器 ({}/{})", c, minimum_2_pow)
            }
        }
    });

    tokio::spawn(async move {
        let mut t = tokio::time::interval(Duration::from_secs(1));
        let mut last_v = 0;
        loop {
            t.tick().await;
            let c = counter.load(atomic::Ordering::Relaxed);
            if c != last_v {
                let res = tokio::fs::write(&file, format!("{}", c)).await;
                if let Err(r) = res {
                    eprintln!("{}", r);
                }
                last_v = c;
            }
        }
    });

    warp::serve(server).bind(([0, 0, 0, 0], port)).await;
}
