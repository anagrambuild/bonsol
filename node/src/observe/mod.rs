pub use {
    metrics::{counter, gauge, histogram, Unit},
    std::time::Instant,
    tracing::{info, instrument},
};

#[derive(strum_macros::Display)]
pub enum MetricEvents {
    ProofExpired,
    ProvingFailed,
    ProvingSucceeded,
    ClaimAttempt,
    ClaimMissed,
    ClaimReceived,
    ImageDeployment,
    ImageDownload,
    ImageCompressed,
    ImageLoaded,
    ImageComputeEstimate,
    ExecutionRequest,
    ProofGeneration,
    ProofCompression,
    ProofConversion,
    InputDownload,
    ProofCycles,
    ProofSegments,
    BonsolStartup,
    SignaturesInFlight,
    IncompatibleProverVersion,
    ProofSubmissionError,
    TransactionExpired,
}

macro_rules! emit_event {
  ($event:expr, $($field_name:ident => $field_value:expr),* $(,)?) => {
      info!(event = $event.to_string(), $($field_name = $field_value),*, "Event: {}", $event);
      let c = counter!("events", "event" => $event.to_string());
      c.increment(1);
  };
}

macro_rules! emit_event_with_duration {
    ($event:expr, $op:block, $($field_name:ident => $field_value:expr),*) => {{
        let start = Instant::now();
        let result = $op;
        let duration_ms = start.elapsed().as_millis();
        info!(
            event = $event.to_string(),
            $($field_name = $field_value),*,
            "Duration: {} = {} ms", $event, duration_ms
        );
        let h = histogram!("durations", "duration" => $event.to_string());
        h.record(duration_ms as f64);
        result
    }};
}

macro_rules! emit_counter {
    ($event:expr, $value:expr, $($field_name:expr => $field_value:expr)*) => {
      info!(event = $event.to_string(), $($field_name = $field_value),*, "{} = {}", $event, $value);
      let c = counter!("counters", "counter" => $event.to_string());
      c.increment($value);
    };
}

macro_rules! emit_gauge {
    ($event:expr, $value:expr, $($field_name:expr => $field_value:expr)*) => {
        info!(event = $event.to_string(), $($field_name = $field_value),* "{} = {}", $event, $value);
        let g = gauge!("gauges", "gauge" => $event.to_string());
        g.set($value);
    };
}

macro_rules! emit_histogram {
    ($event:expr, $value:expr, $($field_name:ident => $field_value:expr),*) => {{
        info!(event = $event.to_string(), $($field_name = $field_value),*, "{} = {}", $event, $value);
        let h = histogram!("histograms", "histogram" => $event.to_string());
        h.record($value);
    }};
}
