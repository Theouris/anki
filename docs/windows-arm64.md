\# Windows ARM64 bring-up notes



\## What worked

\- MSVC ARM64 compiler installed and working

\- Qt6 ARM64 built via vcpkg and verified with a minimal Qt app

\- Python ARM64 detected correctly

\- Anki booted natively on Windows ARM64 from source



\## Observations

\- Native ARM64 Anki felt significantly less laggy than emulated x64 Anki

\- Emulated x64 showed noticeable delay between cards

\- Native boot/run appears promising for heavy decks like AnKing



\## Build notes

\- Repo path used: `C:\\src\\anki`

\- Branch: `windows-arm64`

\- `rsync` and `ninja` were added to PATH

\- Rust target issue appeared for `x86\_64-pc-windows-msvc`

\- Despite that, Anki still booted



\## Next steps

\- Add missing Rust targets cleanly

\- Measure card-to-card latency more systematically

\- Test add-ons and AnKing deck behavior

\- Investigate packaging a Windows ARM64 build

