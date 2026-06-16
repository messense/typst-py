use comemo::Track;
use ecow::{EcoString, eco_format};
use serde::Serialize;
use typst::World;
use typst::diag::{StrResult, Warned, bail};
use typst::engine::Sink;
use typst::foundations::{
    Content, Context, IntoValue, LocatableSelector, Scope, StyleChain, Value,
};
use typst::introspection::{EmptyIntrospector, Introspector};
use typst::routines::SpanMode;
use typst::syntax::Span;
use typst::syntax::SyntaxMode;
use typst_eval::eval_string;
use typst_layout::PagedDocument;

use crate::world::SystemWorld;

/// Processes an input file to extract provided metadata
#[derive(Debug, Clone)]
pub struct QueryCommand {
    /// Defines which elements to retrieve
    pub selector: String,

    /// Extracts just one field from all retrieved elements
    pub field: Option<String>,

    /// Expects and retrieves exactly one element
    pub one: bool,

    /// The format to serialize in
    pub format: SerializationFormat,
}

/// Processes an input file to evaluate a Typst expression
#[derive(Debug, Clone)]
pub struct EvalCommand {
    /// The piece of Typst code to evaluate.
    pub expression: String,

    /// The format to serialize in.
    pub format: SerializationFormat,

    /// Whether to pretty-print JSON output.
    pub pretty: bool,
}

// Output file format for query command
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SerializationFormat {
    Json,
    Yaml,
}

/// Execute a query command.
pub fn query(world: &mut SystemWorld, command: &QueryCommand) -> StrResult<String> {
    let Warned { output, warnings } = typst::compile(world);

    match output {
        // Retrieve and print query results.
        Ok(document) => {
            let data = retrieve(world, command, &document)?;
            let serialized = format(data, command)?;
            Ok(serialized)
        }
        // Print errors and warnings.
        Err(errors) => Err(compile_error_message(errors, warnings)),
    }
}

/// Evaluate a Typst expression.
pub fn eval(world: &mut SystemWorld, command: &EvalCommand) -> StrResult<String> {
    let Warned { output, warnings } = typst::compile::<PagedDocument>(world);

    match output {
        Ok(document) => {
            let data = evaluate(world, command, document.introspector().as_ref())?;
            serialize_with_pretty(&data, command.format, command.pretty)
        }
        Err(errors) => Err(compile_error_message(errors, warnings)),
    }
}

fn compile_error_message(
    errors: ecow::EcoVec<typst::diag::SourceDiagnostic>,
    warnings: ecow::EcoVec<typst::diag::SourceDiagnostic>,
) -> EcoString {
    let mut message = EcoString::from("failed to compile document");
    for (i, error) in errors.into_iter().enumerate() {
        message.push_str(if i == 0 { ": " } else { ", " });
        message.push_str(&error.message);
    }
    for warning in warnings {
        message.push_str(": ");
        message.push_str(&warning.message);
    }
    message
}

/// Retrieve the matches for the selector.
fn retrieve(
    world: &dyn World,
    command: &QueryCommand,
    document: &PagedDocument,
) -> StrResult<Vec<Content>> {
    let mut sink = Sink::new();
    let selector = eval_string(
        world.track(),
        world.library(),
        sink.track_mut(),
        EmptyIntrospector.track(),
        Context::none().track(),
        &command.selector,
        SpanMode::Uniform(Span::detached()),
        SyntaxMode::Code,
        Scope::default(),
    )
    .map_err(|errors| {
        let mut message = EcoString::from("failed to evaluate selector");
        for (i, error) in errors.into_iter().enumerate() {
            message.push_str(if i == 0 { ": " } else { ", " });
            message.push_str(&error.message);
        }
        message
    })?
    .cast::<LocatableSelector>()
    .map_err(|e| e.message().clone())?;

    Ok(document
        .introspector()
        .query(&selector.0)
        .into_iter()
        .collect::<Vec<_>>())
}

/// Evaluate the expression with document introspection available.
fn evaluate(
    world: &dyn World,
    command: &EvalCommand,
    introspector: &dyn Introspector,
) -> StrResult<Value> {
    let mut sink = Sink::new();
    let library = world.library();

    eval_string(
        world.track(),
        library,
        sink.track_mut(),
        introspector.track(),
        Context::new(None, Some(StyleChain::new(&library.styles))).track(),
        &command.expression,
        SpanMode::Uniform(Span::detached()),
        SyntaxMode::Code,
        Scope::default(),
    )
    .map_err(|errors| {
        let mut message = EcoString::from("failed to evaluate expression");
        for (i, error) in errors.into_iter().enumerate() {
            message.push_str(if i == 0 { ": " } else { ", " });
            message.push_str(&error.message);
        }
        message
    })
}

/// Format the query result in the output format.
fn format(elements: Vec<Content>, command: &QueryCommand) -> StrResult<String> {
    if command.one && elements.len() != 1 {
        bail!("expected exactly one element, found {}", elements.len());
    }

    let mapped: Vec<_> = elements
        .into_iter()
        .filter_map(|c| match &command.field {
            Some(field) => c.get_by_name(field).ok(),
            _ => Some(c.into_value()),
        })
        .collect();

    if command.one {
        let Some(value) = mapped.first() else {
            bail!("no such field found for element");
        };
        serialize(value, command.format)
    } else {
        serialize(&mapped, command.format)
    }
}

/// Serialize data to the output format.
fn serialize(data: &impl Serialize, format: SerializationFormat) -> StrResult<String> {
    serialize_with_pretty(data, format, true)
}

/// Serialize data to the output format, optionally pretty-printing JSON.
fn serialize_with_pretty(
    data: &impl Serialize,
    format: SerializationFormat,
    pretty: bool,
) -> StrResult<String> {
    match format {
        SerializationFormat::Json => {
            if pretty {
                serde_json::to_string_pretty(data).map_err(|e| eco_format!("{e}"))
            } else {
                serde_json::to_string(data).map_err(|e| eco_format!("{e}"))
            }
        }
        SerializationFormat::Yaml => serde_yaml::to_string(&data).map_err(|e| eco_format!("{e}")),
    }
}
