use anyhow::Result;
use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind, KeyModifiers};
use futures::StreamExt;
use ratatui::DefaultTerminal;
use tokio::sync::mpsc;

use qrag_core::{RagAnswer, RagEngine, RetrievedChunk};

use crate::ui;

#[derive(Clone, Copy)]
pub enum Role {
    You,
    Bot,
    System,
}

pub struct Message {
    pub role: Role,
    pub text: String,
}

/// Whether we're waiting on the engine.
#[derive(Clone, Copy, PartialEq)]
pub enum Status {
    Idle,
    Thinking,
}

pub struct App {
    engine: RagEngine,
    pub input: String,
    pub messages: Vec<Message>,
    pub sources: Vec<RetrievedChunk>,
    pub status: Status,
    /// Lines scrolled *up* from the bottom (0 = pinned to newest).
    pub scroll: u16,
    should_quit: bool,
}

impl App {
    pub fn new(engine: RagEngine) -> Self {
        Self {
            engine,
            input: String::new(),
            messages: vec![Message {
                role: Role::System,
                text: "Ask a question about your documents. \
                       Enter to send · ↑/↓ scroll · Esc or Ctrl-C to quit."
                    .to_string(),
            }],
            sources: Vec::new(),
            status: Status::Idle,
            scroll: 0,
            should_quit: false,
        }
    }

    pub async fn run(mut self, ternimal: &mut DefaultTerminal) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<Result<RagAnswer, String>>(8);
        let mut events = EventStream::new();
        loop {
            ternimal.draw(|frame| ui::render(frame, &self))?;
            if self.should_quit {
                break;
            }
            tokio::select! {
                maybe_event = events.next() => {
                    if let Some(Ok(event)) = maybe_event{
                        self.handle_event(event, &tx)
                    }
                }
                Some(answer) = rx.recv() =>{
                    self.on_answer(answer);
                }
            }
        }
        Ok(())
    }

    fn handle_event(&mut self, event: Event, tx: &mpsc::Sender<Result<RagAnswer, String>>) {
        let Event::Key(key) = event else { return };
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            KeyCode::Esc => self.should_quit = true,
            KeyCode::Enter => self.submit(tx),
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => self.input.push(c),
            KeyCode::Up => self.scroll = self.scroll.saturating_add(1),
            KeyCode::Down => self.scroll = self.scroll.saturating_sub(1),
            _ => {}
        }
    }

    fn submit(&mut self, tx: &mpsc::Sender<Result<RagAnswer, String>>) {
        let question = self.input.trim().to_string();
        if question.is_empty() || self.status == Status::Thinking {
            return;
        }

        self.messages.push(Message {
            role: Role::You,
            text: question.clone(),
        });
        self.input.clear();
        self.status = Status::Thinking;
        self.scroll = 0;

        let engine = self.engine.clone();
        let tx = tx.clone();
        tokio::spawn(async move {
            let result = engine
                .ask_question(question)
                .await
                .map_err(|e| format!("{e:#}"));
            let _ = tx.send(result).await;
        });
    }

    fn on_answer(&mut self, answer: Result<RagAnswer, String>) {
        self.status = Status::Idle;
        self.scroll = 0;
        match answer {
            Ok(a) => {
                self.messages.push(Message {
                    role: Role::Bot,
                    text: a.answer,
                });
                self.sources = a.sources;
            }
            Err(e) => {
                self.messages.push(Message {
                    role: Role::System,
                    text: format!("error: {e}"),
                });
            }
        }
    }
}
