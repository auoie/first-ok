use anyhow::{anyhow, Context};

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        let client = reqwest::Client::builder().trust_dns(true).build()?;
        let items = (1024..=65535u16).map(move |elem| (elem, client.clone()));
        let url = first_ok::get_first_ok_bounded(items, 0, move |(port, client)| async move {
            let url = format!("http://127.0.0.1:{}", port);
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
