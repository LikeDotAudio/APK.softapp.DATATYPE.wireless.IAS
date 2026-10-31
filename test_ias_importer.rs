// Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
// MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
//! Standalone Test Runner for IAS Importer & MQTT pipeline.

use std::path::Path;

mod ias_importer;

fn main() {
    // The fixture lives beside this file, in DATATYPE.wireless.IAS/test_data — not under the
    // DataMigration crate that compiles this runner.
    let report_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../DATATYPE/DATATYPE.wireless.IAS/test_data/report.html");
    println!("[IAS Test Runner] Testing standalone IAS Importer with file: {}", report_path);

    match ias_importer::parse_ias_report(Path::new(report_path)) {
        Ok(result) => {
            println!("\n=== IAS PARSE SUCCESS ===");
            println!("Status: {}", result.status);
            println!("Source Format: {}", result.source_format);
            println!("Total Channels Extracted: {}", result.total_channels);
            
            println!("\n--- Sample Extracted Channels (First 10) ---");
            for (idx, ch) in result.channels.iter().take(10).enumerate() {
                println!("[{:02}] Zone: '{}' | Group: '{}' | Device: '{}' | Name: '{}' | Freq: {:.3} MHz",
                    idx + 1, ch.zone, ch.group, ch.device, ch.name, ch.freq_mhz);
            }
            println!("==========================");
        }
        Err(err) => {
            eprintln!("[IAS Test Runner] ERROR: {}", err);
            std::process::exit(1);
        }
    }
}
