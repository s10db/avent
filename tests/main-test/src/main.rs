use tokio::sync::broadcast;
use s10_avent::{Recv, Context};

// user code
//################
#[derive(Debug, Clone)]
pub enum Event {
    WithData(String),
    NoData
}

#[derive(Debug)]
struct State {
    test: String
}

struct TestSender1 {
    tx: broadcast::Sender<Event>
}

struct TestSender2 {
    tx: broadcast::Sender<Event>
}

struct TestReveiver1;
struct TestReveiver2;

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

impl Recv for TestReveiver1 {
    type EventType = Event;
    type ContextType = State;

    fn handle(&self, event: Event, context: &mut State) {
        tracing::info!("TestReveiver1 state: {:#?}", context);
        context.test = "other state".into();
        tracing::info!("TestReveiver1 {:#?}", event);
    }
}

impl Recv for TestReveiver2 {
    type EventType = Event;
    type ContextType = State;

    fn handle(&self, event: Event, context: &mut State) {
        tracing::info!("TestReveiver2 state: {:#?}", context);
        context.test = "other state2".into();
        tracing::info!("TestReveiver2 {:#?}", event);
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let ctx = Context::<Event, State>::new(32);

    let sender1 = TestSender1 {
        tx: ctx.get_tx()
    };
    let sender2 = TestSender2 {
        tx: ctx.get_tx()
    };

    let receiver1 = TestReveiver1;
    let receiver2 = TestReveiver2;

    let _ = tokio::spawn(async move {
        ctx.start(vec![Box::new(receiver1), Box::new(receiver2)], State{test: "state".into()}).await;
    }).await;

    sender1.emit(Event::NoData);
    sender2.emit(Event::NoData);
    sender1.emit(Event::WithData("sender1".into()));
    sender2.emit(Event::WithData("sender2".into()));
    sender1.emit(Event::NoData);
    sender2.emit(Event::NoData);

    tracing::info!("done");
}