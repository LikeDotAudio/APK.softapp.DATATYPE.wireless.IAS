# Part of the APK.audio project — http://APK.audio — made by Anthony Kuzub
# MIT Licence. Free, for everyone, for ever. Full text in LICENSE at the root.
# Tests/conftest.py — make Entry.py and its one dependency importable.
#
# The folder name is not a Python identifier, so the edge's own folder goes on
# sys.path and the tests `import Entry`. `apk_wireless_import` is a package
# (requirements.txt pins it at a DATATYPE.wireless.CSV rev): an installed copy
# wins; otherwise APK_WIRELESS_IMPORT_DIR, otherwise the sibling checkout's
# DATATYPE.wireless.CSV/SRC/python when this repo sits in the APK.audio tree.
import os
import sys
from pathlib import Path

EDGE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(EDGE))

try:
    import apk_wireless_import  # noqa: F401
except ImportError:
    here = os.environ.get("APK_WIRELESS_IMPORT_DIR")
    cand = Path(here) if here else EDGE.parent.parent / "DATATYPE.wireless.CSV" / "SRC" / "python"
    sys.path.insert(0, str(cand))
