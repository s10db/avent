use tokio::sync::broadcast;
use std::sync::Arc;

pub trait Recv : 'static + std::marker::Send {
    type EventType;
    type ContextType;

    fn handle(&self, event: Self::EventType, context: &Self::ContextType) -> impl std::future::Future<Output = ()> + Send;
}

pub struct Context<E, C>
{
    tx: broadcast::Sender<E>,
    state: C,
}

impl<E, C> Context<E, C>
    where E: 'static + Clone + std::marker::Send,
          C: 'static + Clone + std::marker::Send + std::marker::Sync
{
    pub fn new(capacity: usize, state: C) -> Self {
        let (tx, _) = broadcast::channel::<E>(capacity);

        Self { tx, state }
    }

    pub async fn start(&self, recvs: Vec<impl Recv<EventType = E, ContextType = C> + std::marker::Sync+ std::clone::Clone>) {
        let mut fwd = self.tx.clone().subscribe();
        let state = Arc::new(self.state.clone());

        tokio::spawn(async move {
            tracing::info!("start core loop");
            loop {
                match fwd.recv().await {
                    Ok(msg) => {
                        for recv in recvs.iter().cloned() {
                            let msg = msg.clone();
                            let st = Arc::clone(&state);
                            tokio::spawn(async move {
                                recv.handle(msg, &st).await
                            });
                        }
                    },
                    Err(e) => {
                        match e {
                            broadcast::error::RecvError::Closed => break,
                            broadcast::error::RecvError::Lagged(len) => {
                                tracing::warn!("{len} message(s) were not handled, discarding");
                            },
                        }
                    },
                }
            }
            tracing::info!("close core loop");
        });
    }

    pub fn get_tx(&self) -> broadcast::Sender<E> {
        self.tx.clone()
    }
}
