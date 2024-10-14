#!/bin/sh

set -eu

cargo publish -p bevy_editor_pls_core --features bevy/x11
cargo publish -p bevy_editor_pls_default_windows --features bevy/x11
cargo publish -p bevy_editor_pls
