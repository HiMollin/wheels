/// 如何正确食用Actor
use tokio::sync::{mpsc, oneshot};

#[tokio::main]
async fn main() {
    let handle = ScoreHandle::new();

    handle.add(5).await;
    handle.sub(9).await;

    println!("score = {}", handle.get().await);
}

struct Score {
    score: i32,
}

impl Score {
    fn set(num: i32) -> Self {
        Self { score: num }
    }

    fn new() -> Self {
        Score::set(0)
    }

    fn get(&self) -> i32 {
        self.score
    }
}

/// 统一不同情况
enum ScoreMessage {
    Add(i32),
    Sub(i32),
    Get { response_to: oneshot::Sender<i32> },
}

// actor，包含维护的数据和接收操作的通道
struct ScoreActor {
    score: Score,
    receiver: mpsc::Receiver<ScoreMessage>,
}

impl ScoreActor {
    fn new(receiver: mpsc::Receiver<ScoreMessage>) -> Self {
        Self {
            score: Score::new(),
            receiver,
        }
    }

    async fn run(mut self) {
        while let Some(msg) = self.receiver.recv().await {
            match msg {
                ScoreMessage::Add(op) => {
                    self.score.score += op;
                }
                ScoreMessage::Sub(op) => {
                    self.score.score -= op;
                }
                ScoreMessage::Get { response_to } => {
                    let _ = response_to.send(self.score.get());
                }
            }
        }
    }
}

/// 供外部使用的handle
struct ScoreHandle {
    sender: mpsc::Sender<ScoreMessage>,
}

impl ScoreHandle {
    fn new() -> Self {
        let (sender, recv) = mpsc::channel(32);

        let actor = ScoreActor::new(recv);

        tokio::spawn(actor.run());

        Self { sender }
    }

    async fn add(&self, num: i32) {
        let _ = self.sender.send(ScoreMessage::Add(num)).await;
    }

    async fn sub(&self, num: i32) {
        let _ = self.sender.send(ScoreMessage::Sub(num)).await;
    }

    async fn get(&self) -> i32 {
        let (sender, recv) = oneshot::channel();

        let _ = self
            .sender
            .send(ScoreMessage::Get {
                response_to: sender,
            })
            .await;

        recv.await.expect("error")
    }
}
