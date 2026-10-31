// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! The compose healthcheck: 0 when the runner's tick wrote its health file
//! recently, 1 otherwise (reason on stderr). The logic is the runner's.
fn main() {
    std::process::exit(apk_plugin_runner::health_check())
}
