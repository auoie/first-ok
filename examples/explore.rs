use anyhow::anyhow;
use futures::future::join_all;
use tokio::{select, sync::mpsc, time};

fn main() -> anyhow::Result<()> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async {
        wait_for_first_task().await?;
        run_many_tasks().await?;
        let _res = wait_for_first_ok_task().await;
        let _res = all_tasks_errors().await;
        let res = first_ok::get_first_ok_bounded(1..=99, 0, move |_idx: u16| async move {
            let milliseconds = 100. * 100. * rand::random::<f64>();
            let duration = tokio::time::Duration::from_millis(milliseconds as u64);
            tokio::time::sleep(duration).await;
            Ok::<tokio::time::Duration, tokio::time::Duration>(duration)
        })
        .await;
        println!("get_first_ok result: {:?}", res);
        let res = first_ok::get_first_ok_bounded(1..=99, 0, move |idx: u16| async move {
            let milliseconds = 10 * (idx as u64);
            let duration = tokio::time::Duration::from_millis(milliseconds);
            tokio::time::sleep(duration).await;
            Err::<tokio::time::Duration, tokio::time::Duration>(duration)
        })
        .await;
        println!("get_first_ok result: {:?}", res);
        Ok(())
    })
}

async fn run_many_tasks() -> anyhow::Result<()> {
    let length = 10;
    let mut tasks = Vec::with_capacity(length);
    for index in 1..=length {
        let task = tokio::task::spawn(async move {
            let milliseconds = 10 * index;
            let duration = tokio::time::Duration::from_millis(milliseconds as u64);
            tokio::time::sleep(duration).await;
            println!("Slept for {} milliseconds", milliseconds);
        });
        tasks.push(task);
    }
    join_all(tasks).await;
    Ok(())
}

async fn wait_for_first_task() -> anyhow::Result<()> {
    let length = 10;
    let value = {
        let (sender, mut receiver) = mpsc::channel::<tokio::time::Duration>(1);
        for index in 1..=length {
            let sender = sender.clone();
            tokio::task::spawn(async move {
                let duration = tokio::time::Duration::from_millis(1);
                let sleep_completion_closure = || async {
                    println!("Slept for {} milliseconds", 100 * index);
                    let _successful_send = sender.send(duration).await;
                };
                select! {
                    _ = testing_sleep(index, duration, index * 10) => {
                        sleep_completion_closure().await;
                    }
                    _ = sender.closed() => {}
                }
            });
        }
        receiver.recv().await.ok_or(anyhow!("none"))
    }?;
    println!("Waited {:?} duration", value);
    Ok(())
}

async fn testing_sleep(id: usize, duration: tokio::time::Duration, steps: usize) {
    for _ in 0..steps {
        tokio::time::sleep(duration).await;
        println!("{id} slept for {:?}", duration);
    }
}

async fn wait_for_first_ok_task() -> Option<Result<time::Duration, time::Duration>> {
    let length = 10;
    let half = length / 2;
    let value = {
        let (sender, mut receiver) =
            mpsc::channel::<Result<tokio::time::Duration, tokio::time::Duration>>(1);
        for index in 1..half {
            let sender = sender.clone();
            tokio::task::spawn(async move {
                let milliseconds = 10 * index;
                let duration = tokio::time::Duration::from_millis(milliseconds as u64);
                let sleep_completion_closure = || async {
                    println!("Slept for {} milliseconds", milliseconds);
                    let _successful_send = sender.send(Err(duration)).await;
                };
                select! {
                    _ = tokio::time::sleep(duration) => {
                        sleep_completion_closure().await;
                    }
                    _ = sender.closed() => {}
                }
            });
        }
        for index in half..=length {
            let sender = sender.clone();
            tokio::task::spawn(async move {
                let milliseconds = 10 * index;
                let duration = tokio::time::Duration::from_millis(milliseconds as u64);
                let sleep_completion_closure = || async {
                    println!("Slept for {} milliseconds", milliseconds);
                    let _successful_send = sender.send(Ok(duration)).await;
                };
                select! {
                    _ = tokio::time::sleep(duration) => {
                        sleep_completion_closure().await;
                    }
                    _ = sender.closed() => {}
                }
            });
        }
        let mut result: Option<Result<time::Duration, time::Duration>> = None;
        for idx in 1..=length {
            let option_result_cur = receiver.recv().await;
            match option_result_cur {
                Some(result_cur) => match result_cur {
                    Ok(ok_cur) => {
                        result = Some(Ok(ok_cur));
                        break;
                    }
                    Err(err_cur) => {
                        if rand::random::<f64>() * idx as f64 <= idx as f64 {
                            result = Some(Err(err_cur))
                        };
                    }
                },
                None => {
                    break;
                }
            }
        }
        result
    };
    println!("Waited {:?} duration", value);
    value
}

async fn all_tasks_errors() -> Option<Result<tokio::time::Duration, tokio::time::Duration>> {
    let length = 10;
    let value = {
        let (sender, mut receiver) =
            mpsc::channel::<Result<tokio::time::Duration, tokio::time::Duration>>(1);
        for index in 1..=length {
            let sender = sender.clone();
            tokio::task::spawn(async move {
                let milliseconds = 10 * index;
                let duration = tokio::time::Duration::from_millis(milliseconds as u64);
                let sleep_completion_closure = || async {
                    println!("Slept for {} milliseconds", milliseconds);
                    let _successful_send = sender.send(Err(duration)).await;
                };
                select! {
                    _ = tokio::time::sleep(duration) => {
                        sleep_completion_closure().await;
                    }
                    _ = sender.closed() => {}
                }
            });
        }
        let mut result: Option<Result<tokio::time::Duration, tokio::time::Duration>> = None;
        for idx in 1..=length {
            let option_result_cur = receiver.recv().await;
            match option_result_cur {
                Some(result_cur) => match result_cur {
                    Ok(ok_cur) => {
                        result = Some(Ok(ok_cur));
                        break;
                    }
                    Err(err_cur) => {
                        if rand::random::<f64>() * idx as f64 <= idx as f64 {
                            result = Some(Err(err_cur))
                        };
                    }
                },
                None => {
                    break;
                }
            }
        }
        result
    };
    println!("Waited {:?} duration", value);
    value
}
