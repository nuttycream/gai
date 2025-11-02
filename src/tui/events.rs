use anyhow::Result;
use crossterm::event::{
    Event as CrosstermEvent, KeyEvent, KeyEventKind,
};
use futures::{FutureExt, StreamExt};
use tokio::{sync::mpsc, task::JoinHandle, time::interval};
use tokio_util::sync::CancellationToken;

// ripped straight from
// https://ratatui.rs/templates/component/tui-rs/#additional-improvements

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Error,
    AppTick,
    Key(KeyEvent),
}

#[derive(Debug)]
pub struct EventHandler {
    _tx: mpsc::UnboundedSender<Event>,
    rx: mpsc::UnboundedReceiver<Event>,
    task: Option<JoinHandle<()>>,
    stop_cancellation_token: CancellationToken,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate =
            std::time::Duration::from_millis(tick_rate_ms);
        let (tx, rx) = mpsc::unbounded_channel();
        let _tx = tx.clone();

        let stop_cancellation_token = CancellationToken::new();
        let _stop_cancelllation_token =
            stop_cancellation_token.clone();

        let task = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick_interval = interval(tick_rate);

            loop {
                let delay = tick_interval.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    _ = _stop_cancelllation_token.cancelled() => {
                        break;
                    }

                    maybe_event = crossterm_event => {
                        match maybe_event {
                            Some(Ok(evt)) => {
                                if let CrosstermEvent::Key(key) = evt
                                    && key.kind == KeyEventKind::Press {
                                        let _ = tx.send(Event::Key(key));
                                    }
                            }
                            Some(Err(_)) => {
                                let _ = tx.send(Event::Error);
                            }
                            None => {}
                        }
                    }

                    _ = delay => {
                        let _ = tx.send(Event::AppTick);
                    }
                }
            }
        });

        Self {
            _tx,
            rx,
            task: Some(task),
            stop_cancellation_token,
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.stop_cancellation_token.cancel();
        if let Some(handle) = self.task.take() {
            handle.await?;
        }
        Ok(())
    }
}

impl Drop for EventHandler {
    fn drop(&mut self) {
        self.stop_cancellation_token.cancel();
    }
}
