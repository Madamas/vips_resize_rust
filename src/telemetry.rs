// use std::sync::Mutex;
// use std::collections::HashMap;
// use std::convert::Infallible;

// use opentelemetry::sdk;
// use opentelemetry::exporter::metrics::prometheus::{Encoder, IntCounterVec, HistogramVec, TextEncoder, default_registry};
// use opentelemetry::api::{MetricOptions, metrics::{Meter, measure::Measure},Counter};

// lazy_static! {
//     static ref METER: Mutex<TelemetryMeter> = Mutex::new(TelemetryMeter::new());
// }

// // TODO: Think a way to make counters and measures HashMap with generic values
// // traits, maybe?
// pub struct TelemetryMeter {
//     instance: sdk::Meter,
//     counters: HashMap<String, IntCounterVec>,
//     measures: HashMap<String, HistogramVec>
// }

// impl TelemetryMeter {
//     fn new() -> TelemetryMeter {
//         return TelemetryMeter{
//             instance: sdk::Meter::new("thumbnailer_rust"),
//             counters: HashMap::default(),
//             measures: HashMap::default()
//         }
//     }

//     fn ensure_i64_counter(&mut self, name: &str) {
//         if self.counters.contains_key(name) {
//             return;
//         }

//         let counter = self.instance.new_i64_counter(name, MetricOptions::default());
//         self.counters.insert(String::from(name), counter);
//     }

//     pub fn increment_i64_counter(name: &str, val: i64, labels: sdk::LabelSet) {
//         METER.lock().unwrap().ensure_i64_counter(name);

//         match METER.lock().unwrap().counters.get(name) {
//             Some(counter) => counter.add(val, &labels),
//             None => return
//         }
//     }

//     fn ensure_f64_measure(&mut self, name: &str) {
//         if self.counters.contains_key(name) {
//             return;
//         }

//         let measure = self.instance.new_f64_measure(name, MetricOptions::default());
//         self.measures.insert(String::from(name), measure);
//     }

//     pub fn make_f64_measure(name: &str, val: f64) {}

//     pub fn collect() -> String {
//         let mut buffer = vec![];
//         let encoder = TextEncoder::new();
//         let metric_families = default_registry().gather();
//         encoder.encode(&metric_families, &mut buffer).unwrap();

//         String::from_utf8(buffer).unwrap()
//     }
// }

// fn metrics_function(req: hyper::Request<hyper::Body>) -> Result<hyper::Response<hyper::Body>, hyper::Error> {
//     let uri = req.uri();

//     match (req.method(), uri.path()) {
//         (&hyper::Method::GET, "/metrics") => {
//             let metrics = TelemetryMeter::collect();

//             let response = hyper::Response::builder()
//                 .header("Content-Type", "application/json")
//                 .header("content-length", metrics.len())
//                 .status(hyper::StatusCode::OK)
//                 .body(hyper::Body::from(metrics))
//                 .unwrap();

//             Ok(response)
//         },
//         _ => {
//             let mut not_found = hyper::Response::default();
//             *not_found.status_mut() = hyper::StatusCode::NOT_FOUND;
//             Ok(not_found)
//         }
//     }
// }

// pub async fn telemetry_export_server() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     let make_svc = hyper::service::make_service_fn(|_conn| {
//         futures::future::ok::<_, Infallible>(hyper::service::service_fn(|req| async move {
//             metrics_function(req)
//         }))
//     });

//     let addr = ([0, 0, 0, 0], 9464).into();
//     let server = hyper::Server::bind(&addr)
//     .serve(make_svc);

//     println!("Telemetry exporter listening on http://{}/metrics", addr);

//     server.await?;

//     Ok(())
// }
