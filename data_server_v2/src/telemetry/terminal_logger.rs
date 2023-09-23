use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::io::Write;

use actix_web::cookie::time;
use actix_web::cookie::time::format_description::well_known::Rfc3339;
use opentelemetry::Value;
use tracing::span::{Attributes, Id};
use tracing::{Event, Level, Metadata, Subscriber};
use tracing_bunyan_formatter::JsonStorage;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Context;
use tracing_subscriber::registry::SpanRef;
use tracing_subscriber::Layer;

const LEVEL: &str = "level";
const NAME: &str = "name";
const HOSTNAME: &str = "hostname";
const PID: &str = "pid";
const TIME: &str = "time";
const MESSAGE: &str = "msg";
const _SOURCE: &str = "src";

pub struct TerminalLogger<W: for<'a> MakeWriter<'a> + 'static> {
    make_writer: W,
    pid: u32,
    hostname: String,
    bunyan_version: u8,
    name: String,
    default_fields: HashMap<String, Value>,
    skip_fields: HashSet<String>,
}

#[non_exhaustive]
#[derive(Debug)]
pub struct SkipFieldError(String);

impl fmt::Display for SkipFieldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} is a core field in the bunyan log format, it can't be skipped",
            &self.0
        )
    }
}

impl std::error::Error for SkipFieldError {}

impl<W: for<'a> MakeWriter<'a> + 'static> TerminalLogger<W> {
    pub fn new(name: String, make_writer: W) -> Self {
        Self::with_default_fields(name, make_writer, HashMap::new())
    }

    pub fn with_default_fields(
        name: String,
        make_writer: W,
        default_fields: HashMap<String, Value>,
    ) -> Self {
        Self {
            make_writer,
            name,
            pid: std::process::id(),
            hostname: "xxx".into(),
            bunyan_version: 0,
            default_fields,
            skip_fields: HashSet::new(),
        }
    }

    pub fn skip_fields<Fields, Field>(mut self, fields: Fields) -> Result<Self, SkipFieldError>
    where
        Fields: Iterator<Item = Field>,
        Field: Into<String>,
    {
        for field in fields {
            let field = field.into();

            self.skip_fields.insert(field);
        }

        Ok(self)
    }

    fn serialize_bunyan_core_fields(
        &self,
        values: &mut Vec<u8>,
        message: &str,
        level: &Level,
    ) -> Result<(), std::io::Error> {
        values.push(self.bunyan_version);
        self.name.bytes().for_each(|b| values.push(b));
        message.bytes().for_each(|b| values.push(b));
        level.as_str().bytes().for_each(|b| values.push(b));
        self.hostname.bytes().for_each(|b| values.push(b));
        values.push(self.pid as u8);

        if let Ok(time) = &time::OffsetDateTime::now_utc().format(&Rfc3339) {
            time.bytes().for_each(|b| values.push(b));
        }
        Ok(())
    }

    fn serialize_field<V>(
        &self,
        values: &mut Vec<u8>,
        key: &str,
        value: &V,
    ) -> Result<(), std::io::Error>
    where
        V: Display,
    {
        if !self.skip_fields.contains(key) {
            value.to_string().bytes().for_each(|b| values.push(b));
        }

        Ok(())
    }

    /// Given a span, it serialised it to a in-memory buffer (vector of bytes).
    fn serialize_span<S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>(
        &self,
        span: &SpanRef<S>,
        ty: Type,
    ) -> Result<Vec<u8>, std::io::Error> {
        let mut buffer: Vec<u8> = Vec::new();

        let message = format_span_context(span, ty);
        self.serialize_bunyan_core_fields(&mut buffer, &message, span.metadata().level())?;
        // Additional metadata useful for debugging
        // They should be nested under `src` (see https://github.com/trentm/node-bunyan#src )
        // but `tracing` does not support nested values yet
        self.serialize_field(&mut buffer, "target", &span.metadata().target())?;

        self.serialize_field(
            &mut buffer,
            "line",
            &span.metadata().line().unwrap_or(0).to_string(),
        )?;
        self.serialize_field(
            &mut buffer,
            "file",
            &span.metadata().file().unwrap_or_default(),
        )?;

        // Add all default fields
        for (key, value) in self.default_fields.iter() {
            self.serialize_field(&mut buffer, key, value)?;
        }

        let extensions = span.extensions();
        if let Some(visitor) = extensions.get::<JsonStorage>() {
            for (key, value) in visitor.values() {
                // Make sure this key isn't reserved. If it is reserved,
                // silently ignore

                self.serialize_field(&mut buffer, key, value)?;
            }
        }

        // We add a trailing new line.
        buffer.write_all(b"\n")?;
        Ok(buffer)
    }

    /// Given an in-memory buffer holding a complete serialised record, flush it to the writer
    /// returned by self.make_writer.
    ///
    /// If we write directly to the writer returned by self.make_writer in more than one go
    /// we can end up with broken/incoherent bits and pieces of those records when
    /// running multi-threaded/concurrent programs.
    fn emit(&self, buffer: &[u8], meta: &Metadata<'_>) -> Result<(), std::io::Error> {
        self.make_writer.make_writer_for(meta).write_all(buffer)
    }
}

/// The type of record we are dealing with: entering a span, exiting a span, an event.
#[derive(Clone, Debug)]
pub enum Type {
    EnterSpan,
    ExitSpan,
    Event,
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let repr = match self {
            Type::EnterSpan => "START",
            Type::ExitSpan => "END",
            Type::Event => "EVENT",
        };
        write!(f, "{}", repr)
    }
}

/// Ensure consistent formatting of the span context.
///
/// Example: "[AN_INTERESTING_SPAN - START]"
fn format_span_context<S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>(
    span: &SpanRef<S>,
    ty: Type,
) -> String {
    format!("[{} - {}]", span.metadata().name().to_uppercase(), ty)
}

/// Ensure consistent formatting of event message.
///
/// Examples:
/// - "[AN_INTERESTING_SPAN - EVENT] My event message" (for an event with a parent span)
/// - "My event message" (for an event without a parent span)
fn format_event_message<S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>>(
    current_span: &Option<SpanRef<S>>,
    event: &Event,
    event_visitor: &JsonStorage<'_>,
) -> String {
    // Extract the "message" field, if provided. Fallback to the target, if missing.
    // let mut message = event_visitor
    //     .values()
    //     .get("message")
    //     .and_then(|v| match v {
    //         Value::String(s) => Some(s.as_str()),
    //         _ => None,
    //     })
    //     .unwrap_or_else(|| event.metadata().target())
    //     .to_owned();

    // If the event is in the context of a span, prepend the span name to the message.
    // if let Some(span) = &current_span {
    //     message = format!("{} {}", format_span_context(span, Type::Event), message);
    // }

    "message".into()
}

impl<S, W> Layer<S> for TerminalLogger<W>
where
    S: Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    W: for<'a> MakeWriter<'a> + 'static,
{
    fn on_event(&self, event: &Event<'_>, ctx: Context<'_, S>) {
        // Events do not necessarily happen in the context of a span, hence lookup_current
        // returns an `Option<SpanRef<_>>` instead of a `SpanRef<_>`.
        let current_span = ctx.lookup_current();

        let mut event_visitor = JsonStorage::default();
        event.record(&mut event_visitor);

        // Opting for a closure to use the ? operator and get more linear code.
        let format = || {
            let mut buffer = Vec::new();

            // let mut serializer = serde_json::Serializer::new(&mut buffer);
            // let mut map_serializer = serializer.serialize_map(None)?;

            let message = format_event_message(&current_span, event, &event_visitor);
            self.serialize_bunyan_core_fields(&mut buffer, &message, event.metadata().level())?;
            // Additional metadata useful for debugging
            // They should be nested under `src` (see https://github.com/trentm/node-bunyan#src )
            // but `tracing` does not support nested values yet
            self.serialize_field(&mut buffer, "target", &event.metadata().target())?;
            self.serialize_field(
                &mut buffer,
                "line",
                &event.metadata().line().unwrap_or_default(),
            )?;
            self.serialize_field(
                &mut buffer,
                "file",
                &event.metadata().file().unwrap_or_default(),
            )?;

            // Add all default fields
            for (key, value) in self.default_fields.iter() {
                self.serialize_field(&mut buffer, key, value)?;
            }

            // Add all the other fields associated with the event, expect the message we already used.
            for (key, value) in event_visitor.values().iter() {
                self.serialize_field(&mut buffer, key, value)?;
            }

            // Add all the fields from the current span, if we have one.
            if let Some(span) = &current_span {
                let extensions = span.extensions();
                if let Some(visitor) = extensions.get::<JsonStorage>() {
                    for (key, value) in visitor.values() {
                        // Make sure this key isn't reserved. If it is reserved,
                        // silently ignore

                        self.serialize_field(&mut buffer, key, value)?;
                    }
                }
            }

            // We add a trailing new line.
            buffer.write_all(b"\n")?;

            Ok(buffer)
        };

        let result: std::io::Result<Vec<u8>> = format();
        if let Ok(formatted) = result {
            let _ = self.emit(&formatted, event.metadata());
        }
    }

    fn on_new_span(&self, _attrs: &Attributes, id: &Id, ctx: Context<'_, S>) {
        let span = ctx.span(id).expect("Span not found, this is a bug");
        if let Ok(serialized) = self.serialize_span(&span, Type::EnterSpan) {
            let _ = self.emit(&serialized, span.metadata());
        }
    }

    fn on_close(&self, id: Id, ctx: Context<'_, S>) {
        let span = ctx.span(&id).expect("Span not found, this is a bug");
        if let Ok(serialized) = self.serialize_span(&span, Type::ExitSpan) {
            let _ = self.emit(&serialized, span.metadata());
        }
    }
}
