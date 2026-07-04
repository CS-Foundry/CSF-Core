use entity::logs;
use sea_orm::{ActiveValue::Set, DbConn, EntityTrait};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{interval, Duration};
use tracing::field::{Field, Visit};
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::layer::Context;
use tracing_subscriber::Layer;
use uuid::Uuid;

const FLUSH_INTERVAL: Duration = Duration::from_secs(1);
const FLUSH_BATCH_SIZE: usize = 100;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogClassification {
    Security,
    Performance,
    Audit,
    System,
    Network,
    Storage,
}

impl LogClassification {
    fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Audit => "audit",
            Self::System => "system",
            Self::Network => "network",
            Self::Storage => "storage",
        }
    }

    fn from_target(target: &str) -> Self {
        if target.contains("auth") || target.contains("rbac") || target.contains("jwt") {
            Self::Security
        } else if target.contains("scheduler") || target.contains("metrics") {
            Self::Performance
        } else if target.contains("network")
            || target.contains("sdn")
            || target.contains("wireguard")
        {
            Self::Network
        } else if target.contains("volume") || target.contains("storage") {
            Self::Storage
        } else if target.contains("registry") {
            Self::Audit
        } else {
            Self::System
        }
    }
}

pub struct LogRecord {
    pub service: &'static str,
    pub level: &'static str,
    pub classification: &'static str,
    pub message: String,
    pub agent_id: Option<Uuid>,
    pub workload_id: Option<Uuid>,
    pub organization_id: Option<Uuid>,
}

#[derive(Default)]
struct EventVisitor {
    message: String,
    agent_id: Option<Uuid>,
    workload_id: Option<Uuid>,
    organization_id: Option<Uuid>,
}

impl Visit for EventVisitor {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        self.record_field(field.name(), format!("{value:?}"));
    }

    fn record_str(&mut self, field: &Field, value: &str) {
        self.record_field(field.name(), value.to_string());
    }
}

impl EventVisitor {
    fn record_field(&mut self, name: &str, value: String) {
        match name {
            "message" => self.message = trim_quotes(value),
            "agent_id" => self.agent_id = Uuid::parse_str(&trim_quotes(value)).ok(),
            "workload_id" => self.workload_id = Uuid::parse_str(&trim_quotes(value)).ok(),
            "organization_id" => self.organization_id = Uuid::parse_str(&trim_quotes(value)).ok(),
            _ => {}
        }
    }
}

fn trim_quotes(value: String) -> String {
    value.trim_matches('"').to_string()
}

fn level_as_str(level: &Level) -> &'static str {
    match *level {
        Level::TRACE | Level::DEBUG => "DEBUG",
        Level::INFO => "INFO",
        Level::WARN => "WARN",
        Level::ERROR => "ERROR",
    }
}

pub struct DbLogLayer {
    service: &'static str,
    sender: UnboundedSender<LogRecord>,
}

impl DbLogLayer {
    pub fn new(service: &'static str) -> (Self, UnboundedReceiver<LogRecord>) {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        (Self { service, sender }, receiver)
    }
}

const NOISY_ACCESS_TARGET: &str = "csfx::http_access";
const NOISY_TARGET_PREFIXES: [&str; 2] = ["sea_orm", "sqlx"];
const NOISY_MESSAGE_SUBSTRINGS: [&str; 1] = ["heartbeat processed"];

fn is_routine_access_event(event: &Event<'_>) -> bool {
    event.metadata().target() == NOISY_ACCESS_TARGET && *event.metadata().level() == Level::INFO
}

fn is_sql_trace_event(event: &Event<'_>) -> bool {
    let target = event.metadata().target();
    NOISY_TARGET_PREFIXES
        .iter()
        .any(|prefix| target.starts_with(prefix))
}

fn is_performance_event(event: &Event<'_>) -> bool {
    *event.metadata().level() == Level::INFO
        && LogClassification::from_target(event.metadata().target())
            == LogClassification::Performance
}

fn is_noisy_message_event(visitor: &EventVisitor) -> bool {
    NOISY_MESSAGE_SUBSTRINGS
        .iter()
        .any(|needle| visitor.message.contains(needle))
}

impl<S: Subscriber> Layer<S> for DbLogLayer {
    fn on_event(&self, event: &Event<'_>, _ctx: Context<'_, S>) {
        if *event.metadata().level() == Level::TRACE {
            return;
        }

        if is_routine_access_event(event)
            || is_sql_trace_event(event)
            || is_performance_event(event)
        {
            return;
        }

        let mut visitor = EventVisitor::default();
        event.record(&mut visitor);

        if is_noisy_message_event(&visitor) {
            return;
        }

        let record = LogRecord {
            service: self.service,
            level: level_as_str(event.metadata().level()),
            classification: LogClassification::from_target(event.metadata().target()).as_str(),
            message: visitor.message,
            agent_id: visitor.agent_id,
            workload_id: visitor.workload_id,
            organization_id: visitor.organization_id,
        };

        let _ = self.sender.send(record);
    }
}

pub fn spawn_log_writer(mut receiver: UnboundedReceiver<LogRecord>, db: DbConn) {
    tokio::spawn(async move {
        let mut buffer = Vec::with_capacity(FLUSH_BATCH_SIZE);
        let mut ticker = interval(FLUSH_INTERVAL);

        loop {
            tokio::select! {
                record = receiver.recv() => {
                    match record {
                        Some(record) => {
                            buffer.push(record);
                            if buffer.len() >= FLUSH_BATCH_SIZE {
                                flush(&db, &mut buffer).await;
                            }
                        }
                        None => {
                            flush(&db, &mut buffer).await;
                            break;
                        }
                    }
                }
                _ = ticker.tick() => {
                    flush(&db, &mut buffer).await;
                }
            }
        }
    });
}

async fn flush(db: &DbConn, buffer: &mut Vec<LogRecord>) {
    if buffer.is_empty() {
        return;
    }

    let models: Vec<logs::ActiveModel> = buffer
        .drain(..)
        .map(|record| logs::ActiveModel {
            id: Set(Uuid::new_v4()),
            service: Set(record.service.to_string()),
            level: Set(record.level.to_string()),
            classification: Set(record.classification.to_string()),
            message: Set(record.message),
            agent_id: Set(record.agent_id),
            workload_id: Set(record.workload_id),
            organization_id: Set(record.organization_id),
            created_at: Set(chrono::Utc::now().into()),
        })
        .collect();

    if let Err(error) = logs::Entity::insert_many(models).exec(db).await {
        tracing::warn!(target: "shared::db_log_layer", error = %error, "failed to flush logs to database");
    }
}
