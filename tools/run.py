# Copyright: Ankitects Pty Ltd and contributors
# License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

import os
import platform
import sys
from pathlib import Path


def configure_windows_arm64_qt() -> None:
    if os.name != "nt" or platform.machine().upper() != "ARM64":
        return

    qt_root = Path(os.environ.get("ANKI_QT_ROOT", r"C:\vcpkg\installed\arm64-windows"))
    qt_bin = qt_root / "bin"
    qt_tools_bin = qt_root / "tools" / "Qt6" / "bin"
    qt_plugins = qt_root / "Qt6" / "plugins"

    if not qt_bin.exists() or not qt_plugins.exists():
        return

    extra_path_entries = [str(qt_bin)]
    if qt_tools_bin.exists():
        extra_path_entries.append(str(qt_tools_bin))
    os.environ["PATH"] = os.pathsep.join(extra_path_entries + [os.environ.get("PATH", "")])
    os.environ.setdefault("QT_PLUGIN_PATH", str(qt_plugins))

    if hasattr(os, "add_dll_directory"):
        os.add_dll_directory(str(qt_bin))
        if qt_tools_bin.exists():
            os.add_dll_directory(str(qt_tools_bin))

    webengine_process = qt_tools_bin / "QtWebEngineProcess.exe"
    if webengine_process.exists():
        os.environ.setdefault("QTWEBENGINEPROCESS_PATH", str(webengine_process))

    locales_path = qt_root / "translations" / "qtwebengine_locales"
    if locales_path.exists():
        os.environ.setdefault("QTWEBENGINE_LOCALES_PATH", str(locales_path))

    resources_path = qt_root / "resources"
    if resources_path.exists():
        os.environ.setdefault("QTWEBENGINE_RESOURCES_PATH", str(resources_path))

sys.path.extend(["pylib", "qt", "out/pylib", "out/qt"])

configure_windows_arm64_qt()

import aqt

if not os.environ.get("SKIP_RUN"):
    aqt.run()
