use std::{env, time::Duration};

use anyhow::{anyhow, Context};

fn main() -> anyhow::Result<()> {
    let port = env::args()
        .nth(1)
        .context("port not found")?
        .parse::<usize>()?;
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .trust_dns(true)
            .timeout(Duration::from_secs(5))
            .build()?;
        let items = (2..=255u8).map(move |elem| (elem, client.clone()));
        let url = first_ok::get_first_ok_bounded(items, 0, move |(item, client)| async move {
            let url = format!("http://192.168.1.{}:{}", item, port);
            let response = client.get(&url).send().await?;
            if response.status().as_u16() != 200 {
                return Err(anyhow!(format!("{}", response.status())));
            }
            Ok(url)
        })
        .await
        .context("nothing reported")??;
        println!("{}", url);
        Ok(())
    })
}
