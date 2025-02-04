use std::sync::Arc;

use tokio::sync::{broadcast, Mutex};
use avent::{Recv, Context};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone)]
pub enum Event {
    WithData(String),
    NoData
}

#[derive(Debug, Clone)]
struct State {
    test: std::sync::Arc<tokio::sync::Mutex<String>>
}

struct TestSender1 {
    tx: broadcast::Sender<Event>
}

struct TestSender2 {
    tx: broadcast::Sender<Event>
}

impl TestSender1 {
    pub fn emit(&self, event: Event) {
        self.tx.send(event).expect("emit failed");
    }
}

impl TestSender2 {
    pub fn emit(&self, event: Event) {
        self.tx.send(event).expect("emit failed");
    }
}

#[derive(Clone)]
enum ReceiverTest {
    TestReveiver1,
    TestReveiver2,
}

impl Recv for ReceiverTest {
    type EventType = Event;
    type ContextType = State;

    async fn handle(&self, event: Event, context: &State) {
        match self {
            ReceiverTest::TestReveiver1 => {
                // sleep(Duration::from_millis(100)).await;
                let st = Arc::clone(&context.test);
                let mut st = st.lock().await;
                tracing::info!("TestReveiver1 state: {:#?}", st);
                *st = "other state".into();
                tracing::info!("TestReveiver1 {:#?}", event);
            },
            ReceiverTest::TestReveiver2 => {
                let st = Arc::clone(&context.test);
                let mut st = st.lock().await;
                tracing::info!("TestReveiver2 state: {:#?}", st);
                *st = "other state2".into();
                tracing::info!("TestReveiver2 {:#?}", event);
            },
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let ctx = Context::<Event, State>::new(32, State{test: Arc::new(Mutex::new("state".into()))});

    let sender1 = TestSender1 {
        tx: ctx.get_tx()
    };
    let sender2 = TestSender2 {
        tx: ctx.get_tx()
    };

    let receiver1 = ReceiverTest::TestReveiver1;
    let receiver2 = ReceiverTest::TestReveiver2;

    let _ = tokio::spawn(async move {
        ctx.start(vec![receiver1, receiver2]).await;
    }).await;

    sender1.emit(Event::NoData);
    sender2.emit(Event::NoData);
    sender1.emit(Event::WithData("sender1".into()));
    sender2.emit(Event::WithData("sender2".into()));
    sender1.emit(Event::NoData);
    sender2.emit(Event::NoData);

    sleep(Duration::from_millis(1000)).await;
    tracing::info!("done");
}
