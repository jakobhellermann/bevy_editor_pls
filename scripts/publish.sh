#!/bin/sh

set -eu

cargo publish -p bevy_editor_pls_core
cargo publish -p bevy_editor_pls_default_windows
cargo publish -p bevy_editor_pls