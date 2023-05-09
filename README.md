# first-ok

This provides the function `first_ok::get_first_ok_bounded` which takes an async function and a set of items.
It applies the function to all of the items and returns the first `Ok` result.

```rust
// examples/port.rs
use anyhow::{anyhow, Context};

#[tokio::main]
fn main() -> anyhow::Result<()> {
    let client = reqwest::Client::builder().build()?;
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
}
```
