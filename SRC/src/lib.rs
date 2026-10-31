// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The IAS report importer as a worker (LOOSE-workers-convert-rest).
//!
//! ```text
//!   browser ── APK.audio/System/Protocols/wireless_ias/outgoing/job ──▶ worker
//!   worker  ── APK.audio/System/Protocols/wireless_ias/incoming/result ─▶ browser
//! ```
//!
//! ONE JOB IN, ONE RESULT OUT, MATCHED BY `job_id`. Neither leaf retains (see
//! `contracts/rust/src/baremetal_io.rs`): a retained job would run again on
//! every reconnect, and a retained result would answer a question the next
//! reader never asked.
//!
//! [`handle_job`] is the whole tool and has no bus in it, so the tests below
//! run it on the fixture in `../test_data/` with no broker anywhere near.
//! [`run_job_agent`] hands it to the runner's job loop
//! (`apk_plugin_runner::jobs::JobWorker`); the envelope — `job_id`, `text` |
//! `content_b64`, the size limit, the versioned answer — is the wireless
//! kernel's (`apkaudio_wireless_kernel::job`), shared with the other three
//! wireless workers.

pub use apkaudio_wireless_ias as ias_importer;
pub mod self_test;

use apkaudio_wireless_kernel::job;
use serde_json::Value;

/// The token the bus knows this worker by. The same word as `[wireless_ias]`.
pub const MODULE: &str = "wireless_ias";

/// The tool the result came from, for a reader that listens to several.
pub const TOOL: &str = "DATATYPE.wireless.IAS";

/// The compiled-in config, the same text the runner publishes at `/config`.
pub const CONFIG_INI: &str = include_str!("../config.ini");

/// Used when `config.ini` does not say.
const DEFAULT_MAX_JOB_BYTES: usize = 8 * 1024 * 1024;

/// `max_job_bytes` from `config.ini`, or the default.
pub fn max_job_bytes(config_ini: &str) -> usize {
    job::max_job_bytes(config_ini, MODULE, DEFAULT_MAX_JOB_BYTES)
}

/// Answer one job. Always returns a result document, never panics on input.
pub fn handle_job(payload: &[u8], max_bytes: usize) -> Value {
    match job::read_file_job(payload, TOOL, "report.html", "the report", max_bytes) {
        Ok(file) => file.answer(TOOL, ias_importer::classify_ias_str(&file.text(), &file.filename)),
        Err(failure) => failure,
    }
}

/// The job loop: the runner's. Blocking; the runner starts it on its own
/// thread and exits the process if it ever returns, so Docker restarts it.
pub fn run_job_agent(host: &str, port: u16) {
    let max = max_job_bytes(CONFIG_INI);
    apk_plugin_runner::jobs::JobWorker::new(MODULE, TOOL, move |payload: &[u8]| handle_job(payload, max))
        // A job is a file, and base64 grows it by a third.
        .max_packet(max / 3 * 4 + 64 * 1024)
        .describe(|a| format!("ok={} channels={}", a["ok"], a["result"]["total_channels"]))
        .run(host, port);
}

#[cfg(test)]
mod tests {
    use super::*;
    use apkaudio_contracts::baremetal_io;
    use base64::Engine as _;
    use serde_json::json;

    const REPORT: &str = include_str!("../../test_data/report.html");

    fn job(body: Value) -> Vec<u8> {
        body.to_string().into_bytes()
    }

    #[test]
    fn a_text_job_is_parsed_and_answered_under_its_id() {
        let out = handle_job(&job(json!({"job_id": "j1", "filename": "report.html", "text": REPORT})), 1 << 20);
        assert_eq!(out["job_id"], "j1");
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["result"]["total_channels"], 168);
        assert_eq!(out["result"]["status"], "success");
        assert_eq!(out["result"]["file_path"], "report.html");
    }

    #[test]
    fn a_base64_job_is_the_same_as_its_text() {
        let b64 = base64::engine::general_purpose::STANDARD.encode(REPORT);
        let a = handle_job(&job(json!({"job_id": "j2", "filename": "report.html", "content_b64": b64})), 1 << 20);
        let b = handle_job(&job(json!({"job_id": "j2", "filename": "report.html", "text": REPORT})), 1 << 20);
        assert_eq!(a["ok"], true, "{a}");
        assert_eq!(a["result"], b["result"]);
    }

    #[test]
    fn a_bad_job_is_answered_not_dropped() {
        let out = handle_job(b"not json", 1 << 20);
        assert_eq!(out["ok"], false);
        assert!(out["job_id"].is_null());

        let out = handle_job(&job(json!({"text": REPORT})), 1 << 20);
        assert_eq!(out["ok"], false, "no job_id");

        let out = handle_job(&job(json!({"job_id": "j3"})), 1 << 20);
        assert_eq!(out["job_id"], "j3");
        assert_eq!(out["ok"], false, "no content");

        let out = handle_job(&job(json!({"job_id": "j4", "content_b64": "%%%"})), 1 << 20);
        assert_eq!(out["ok"], false, "bad base64");
    }

    #[test]
    fn an_oversized_job_is_refused_by_size() {
        let out = handle_job(&job(json!({"job_id": "j5", "text": REPORT})), 100);
        assert_eq!(out["ok"], false);
        assert!(out["error"].as_str().unwrap().contains("at most 100"));
    }

    #[test]
    fn the_limit_comes_from_config_ini() {
        assert_eq!(max_job_bytes(CONFIG_INI), 8_388_608);
        assert_eq!(max_job_bytes("[wireless_ias]\n"), DEFAULT_MAX_JOB_BYTES);
    }

    #[test]
    fn job_and_result_topics_are_this_modules() {
        let (job, retain) = baremetal_io::module_job(MODULE).unwrap();
        assert_eq!(job, "APK.audio/System/Protocols/wireless_ias/outgoing/job");
        assert!(!retain);
        let (result, retain) = baremetal_io::module_result(MODULE).unwrap();
        assert_eq!(result, "APK.audio/System/Protocols/wireless_ias/incoming/result");
        assert!(!retain);
    }

    /// The wire shape a browser reads. A parser field renamed is a failure
    /// here first; change it on purpose, together with `job::SCHEMA`.
    #[test]
    fn the_result_shape_is_pinned() {
        let out = handle_job(&job(json!({"job_id": "s", "filename": "report.html", "content_b64": base64::engine::general_purpose::STANDARD.encode(REPORT)})), 1 << 24);
        assert_eq!(out["ok"], true, "{out}");
        assert_eq!(out["schema"], job::SCHEMA);
        let keys = |v: &Value| {
            let mut k: Vec<String> = v.as_object().unwrap().keys().cloned().collect();
            k.sort();
            k
        };
        assert_eq!(keys(&out["result"]), ["channels", "file_path", "skipped_rows", "source_format", "status", "total_channels"]);
        assert_eq!(keys(&out["result"]["channels"][0]), ["band_info", "device", "freq_mhz", "group", "name", "zone"]);
        assert_eq!(job::SCHEMA, 1);
    }
}
