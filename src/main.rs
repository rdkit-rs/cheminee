#![deny(clippy::print_stdout)]

use cheminee::{command_line, rest_api};
use clap::*;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

fn init_tracing() -> eyre::Result<Option<opentelemetry_sdk::trace::SdkTracerProvider>> {
    let env_filter = if std::env::var_os("RUST_LOG").is_some() {
        tracing_subscriber::EnvFilter::from_default_env()
    } else {
        tracing_subscriber::EnvFilter::new("info")
    };

    let fmt_layer = tracing_subscriber::fmt::layer();

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        log::info!("OpenTelemetry enabled, exporting to {}", endpoint);

        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .build()?;

        let tracer_provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
            .with_batch_exporter(exporter)
            .with_resource(
                opentelemetry_sdk::Resource::builder()
                    .with_service_name(
                        std::env::var("OTEL_SERVICE_NAME")
                            .unwrap_or_else(|_| "cheminee".to_string()),
                    )
                    .build(),
            )
            .build();

        let otel_layer =
            tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer("cheminee"));

        registry.with(otel_layer).init();

        Ok(Some(tracer_provider))
    } else {
        registry.init();
        Ok(None)
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let tracer_provider = init_tracing()?;

    let app = Command::new("cheminee")
        .subcommand_required(true)
        .subcommand(command_line::indexing::bulk_index::command())
        .subcommand(command_line::indexing::bulk_delete::command())
        .subcommand(command_line::indexing::create_index::command())
        .subcommand(command_line::indexing::delete_index::command())
        .subcommand(command_line::indexing::index_sdf::command())
        .subcommand(command_line::indexing::merge_segments::command())
        .subcommand(command_line::pubchem::fetch_pubchem::command())
        .subcommand(command_line::pubchem::stream_pubchem_sdf::command())
        .subcommand(command_line::search::basic_search::command())
        .subcommand(command_line::search::basic_search_count::command())
        .subcommand(command_line::search::substructure_search::command())
        .subcommand(command_line::search::superstructure_search::command())
        .subcommand(command_line::search::similarity_search::command())
        .subcommand(command_line::search::identity_search::command())
        .subcommand(rest_api::command());

    let matches = app.get_matches();
    let matches = match matches.subcommand().unwrap() {
        (command_line::indexing::bulk_index::NAME, matches) => {
            command_line::indexing::bulk_index::action(matches)
        }
        (command_line::indexing::bulk_delete::NAME, matches) => {
            command_line::indexing::bulk_delete::action(matches)
        }
        (command_line::indexing::create_index::NAME, matches) => {
            command_line::indexing::create_index::action(matches)
        }
        (command_line::indexing::delete_index::NAME, matches) => {
            command_line::indexing::delete_index::action(matches)
        }
        (command_line::indexing::index_sdf::NAME, matches) => {
            command_line::indexing::index_sdf::action(matches)
        }
        (command_line::indexing::merge_segments::NAME, matches) => {
            command_line::indexing::merge_segments::action(matches)
        }
        (command_line::pubchem::fetch_pubchem::NAME, matches) => {
            command_line::pubchem::fetch_pubchem::action(matches).await
        }
        (command_line::pubchem::stream_pubchem_sdf::NAME, matches) => {
            command_line::pubchem::stream_pubchem_sdf::action(matches)
        }
        (command_line::search::basic_search::NAME, matches) => {
            command_line::search::basic_search::action(matches)
        }
        (command_line::search::basic_search_count::NAME, matches) => {
            command_line::search::basic_search_count::action(matches)
        }
        (command_line::search::substructure_search::NAME, matches) => {
            command_line::search::substructure_search::action(matches)
        }
        (command_line::search::superstructure_search::NAME, matches) => {
            command_line::search::superstructure_search::action(matches)
        }
        (command_line::search::similarity_search::NAME, matches) => {
            command_line::search::similarity_search::action(matches)
        }
        (command_line::search::identity_search::NAME, matches) => {
            command_line::search::identity_search::action(matches)
        }
        (rest_api::NAME, matches) => rest_api::action(matches).await,
        (unknown, _) => panic!("🤨: {}", unknown),
    };

    matches?;

    if let Some(provider) = tracer_provider {
        provider.shutdown()?;
    }

    Ok(())
}
