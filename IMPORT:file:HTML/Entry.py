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

# MARKER_HEADERS, the lifecycle and the logging call come from
# `apk_wireless_import` (DATATYPE.wireless.CSV/SRC/python, package
# `apk-wireless-import`, pinned in requirements.txt beside this file) — one copy
# for the IAS, SoundBase-PDF and Wireless Workbench edges. This edge no longer
# imports NODE.BareMetal's `importer_logging`, so it needs no other repository on
# PYTHONPATH.

from apk_wireless_import import MARKER_HEADERS, Lifecycle, log

try:
    from oaRustCore.html_scraper import scrape_tables as rust_scrape_tables
except ImportError:
    rust_scrape_tables = None

# "live" when the Rust scraper is reachable, "unavailable" when it is not.
# Asserted by Tests/test_handler_status.py so a broken import name fails a test
# rather than a customer's import.
STATUS = "live" if rust_scrape_tables else "unavailable"


def convert_ias_html_to_markers(html_content):
    if rust_scrape_tables:
        log("convert_ias_html_to_markers", "🚀 Using HIGH-PERFORMANCE RUST HTML scraper.")
        return MARKER_HEADERS, rust_scrape_tables(str(html_content))
    raise RuntimeError("Rust HTML parser required")


Marker_convert_IAShtml_report_to_csv = convert_ias_html_to_markers


def _self_test():
    """Scrape a one-table report through the Rust scraper and check what comes back."""
    html = (
        "<html><body><table>"
        "<tr><th>Name</th><th>Frequency</th></tr>"
        "<tr><td>Ch 1</td><td>470.125</td></tr>"
        "</table></body></html>"
    )
    headers, tables = convert_ias_html_to_markers(html)
    tables = [list(t) for t in tables]
    return headers == MARKER_HEADERS and tables == [[{"Name": "Ch 1", "Frequency": "470.125"}]]


# The importer interface's lifecycle, from the shared module: start/stop are
# no-ops (stateless), run_tests() is False unless STATUS is "live" and the
# scrape above answers right — never True for having run nothing.
_life = Lifecycle(STATUS, self_test=_self_test)
start, stop, status, run_tests = _life.start, _life.stop, _life.status, _life.run_tests


__all__ = ["convert_ias_html_to_markers", "Marker_convert_IAShtml_report_to_csv", "MARKER_HEADERS", "STATUS", "start", "stop", "status", "run_tests"]
