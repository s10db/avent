use std::marker::PhantomData;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

pub trait Recv : 'static + std::marker::Send {
    type EventType;
    type ContextType;

    fn handle(&self, event: Self::EventType, context: &mut Self::ContextType);
}

pub struct Context<E, C>
{
    tx: broadcast::Sender<E>,
    dummy: PhantomData<C>
}

impl<E, C> Context<E, C>
    where E: 'static + Clone + std::marker::Send,
          C: 'static + std::marker::Send
{
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel::<E>(capacity);

        Self { tx, dummy: PhantomData }
    }

    pub async fn start(&self, recvs: Vec<Box<dyn Recv<EventType = E, ContextType = C>>>, mut context_data: C) {
        let mut fwd = self.tx.clone().subscribe();

        tokio::spawn(async move {
            tracing::info!("start core loop");
            loop {
                match fwd.recv().await {
                    Ok(msg) => {
                        for recv in recvs.iter() {
                            recv.handle(msg.clone(), &mut context_data);
                        }
                    },
                    Err(e) => {
                        match e {
                            broadcast::error::RecvError::Closed => break,
                            broadcast::error::RecvError::Lagged(len) => {
                                tracing::warn!("{len} message(s) were not handled, discarding");
                                sleep(Duration::from_millis(1)).await;
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