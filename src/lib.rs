use futures::Future;
use rand::Rng;
use tokio::{select, sync::mpsc};

async fn send_item_requests<I>(items: Vec<I>, item_sender: async_channel::Sender<I>) {
    for item in items {
        let send_response = item_sender.send(item).await;
        if send_response.is_err() {
            return;
        }
    }
}

async fn process_item_requests<I, T, E, F, Fut>(
    item_receiver: async_channel::Receiver<I>,
    checker: F,
    response_sender: mpsc::Sender<Result<T, E>>,
) where
    F: (FnOnce(I) -> Fut) + Copy,
    Fut: Future<Output = Result<T, E>>,
{
    loop {
        let item = item_receiver.recv().await;
        match item {
            Ok(item) => {
                select! {
                    result = checker(item) => {
                        let _ = response_sender.send(result).await;
                    }
                    _ = response_sender.closed() => {
                        return;
                    }
                }
            }
            Err(_) => {
                break;
            }
        }
    }
}

async fn process_item_responses<T, E>(
    length: usize,
    mut response_receiver: mpsc::Receiver<Result<T, E>>,
) -> Option<Result<T, E>> {
    let mut result: Option<Result<T, E>> = None;
    let mut rng = rand::thread_rng();
    for nth in 1..=length {
        let option_result_cur = response_receiver.recv().await;
        match option_result_cur {
            Some(result_cur) => match result_cur {
                ok @ Ok(_) => {
                    result = Some(ok);
                    break;
                }
                err @ Err(_) => {
                    if rng.gen::<f64>() * nth as f64 <= 1.0 {
                        result = Some(err)
                    };
                }
            },
            None => {
                break;
            }
        }
    }
    result
}

/// Returns the first non-error result from a function `checker` applied to each entry in a list of `items`.
/// If the list of items is empty, it returns `None`.
/// If all of the results are errors, it randoms a random error from the error results.
/// There are `concurrent` workers to apply the `checker` function.
/// If `concurrent` is 0, then it will create `len(items)` workers.
pub async fn get_first_ok_bounded<I, T, E, F, Fut>(
    items: Vec<I>,
    mut concurrent: usize,
    checker: F,
) -> Option<Result<T, E>>
where
    F: (FnOnce(I) -> Fut) + Send + Copy + 'static,
    Fut: Future<Output = Result<T, E>> + Send,
    I: Send + 'static,
    T: Send + 'static,
    E: Send + 'static,
{
    if concurrent == 0 {
        concurrent = items.len()
    };
    let length = items.len();
    let (item_sender, item_receiver) = async_channel::bounded::<I>(1);
    let (response_sender, response_receiver) = mpsc::channel::<Result<T, E>>(1);
    for _ in 0..concurrent {
        let item_receiver = item_receiver.clone();
        let response_sender = response_sender.clone();
        tokio::task::spawn(async move {
            process_item_requests(item_receiver, checker, response_sender).await
        });
    }
    tokio::task::spawn(send_item_requests(items, item_sender));
    process_item_responses(length, response_receiver).await
}
