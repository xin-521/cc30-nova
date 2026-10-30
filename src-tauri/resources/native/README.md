# Native runtime component

`InsydeDCHU.dll` is a third-party hardware component and is not distributed
with this repository.

For local development with real hardware, copy a legitimately obtained
`InsydeDCHU.dll` into this directory before building the Tauri application.
Alternatively, set `CC30_DCHU_PATH` to its absolute path.

Without the DLL, CC30 Nova starts in its clearly identified simulation mode
and will not attempt hardware writes.
