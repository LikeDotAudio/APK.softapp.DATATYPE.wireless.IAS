# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Reads an IAS (Intermodulation Analysis System) HTML report and returns the marker table
# DataMigration imports from it.
#
# Owns: the IAS HTML dialect only. Row extraction itself belongs to the Rust
# scraper; this file is the Python edge that names it and shapes the result.
#
# Must not: parse HTML in Python. `oaRustCore.html_scraper` is the parser,
# and a pure-Python fallback here would silently become the real one.
#
# The import name is a contract with `src/lib.rs`. `add_submodule!` names each
# module's Python name as a STRING there — it used to `stringify!` the Rust
# module identifier, which meant renaming a directory in `src/` broke this
# line and nothing reported it: the ImportError is caught, STATUS goes to
# "unavailable", and the failure shows up only as a report with no rows.
# `Core/Tests/test_registry_names.py` reads both sides and fails when they
# disagree. Change both files together anyway.
#
# The header row is fixed by DataMigration's frame contract; `check.sh
# datamigration` is what proves it still matches the Rust record.

from importer_logging import matrix_log

MARKER_HEADERS = ["ZONE", "GROUP", "DEVICE", "NAME", "FREQ_MHZ", "PEAK"]

try:
    from oaRustCore.html_scraper import scrape_tables as rust_scrape_tables
except ImportError:
    rust_scrape_tables = None

# "live" when the Rust scraper is reachable, "unavailable" when it is not.
# Asserted by Tests/test_handler_status.py so a broken import name fails a test
# rather than a customer's import.
STATUS = "live" if rust_scrape_tables else "unavailable"


# Inline comment: Logic for convert_ias_html_to_markers
def convert_ias_html_to_markers(html_content):
    if rust_scrape_tables:
        matrix_log("ui", "importer", "convert_ias_html_to_markers", "🚀 Using HIGH-PERFORMANCE RUST HTML scraper.", "DEBUG")
        return MARKER_HEADERS, rust_scrape_tables(str(html_content))
    raise RuntimeError("Rust HTML parser required")


Marker_convert_IAShtml_report_to_csv = convert_ias_html_to_markers


# Inline comment: Logic for start
def start(): pass
# Inline comment: Logic for stop
def stop(): pass
# Inline comment: Logic for status
def status(): return STATUS
# Inline comment: Logic for run_tests
def run_tests(): return True

__all__ = ["convert_ias_html_to_markers", "Marker_convert_IAShtml_report_to_csv", "MARKER_HEADERS", "STATUS", "start", "stop", "status", "run_tests"]
