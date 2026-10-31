# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
"""Entry.py declares what it is, and its lifecycle cannot claim more."""
import apk_wireless_import
import Entry


def test_status_is_a_word_and_status_reports_it():
    assert Entry.STATUS in apk_wireless_import.STATUS_WORDS
    assert Entry.status() == Entry.STATUS


def test_the_header_row_is_the_shared_one():
    assert Entry.MARKER_HEADERS is apk_wireless_import.MARKER_HEADERS


def test_run_tests_is_false_unless_live():
    if Entry.STATUS != "live":
        assert Entry.run_tests() is False


def test_start_and_stop_are_no_ops():
    assert Entry.start() is None and Entry.stop() is None


def test_without_the_scraper_conversion_refuses():
    if Entry.STATUS == "unavailable":
        import pytest
        with pytest.raises(RuntimeError):
            Entry.convert_ias_html_to_markers("<table></table>")
