# Changelog

## Version 0.4 (unreleased)
- allow editor on non-primary window
- breaking: require `.add_plugin(EditorPlugin::new())` instead of `.add_plugin(EditorPlugin)`
- merge `EditorState` and `Editor`
- fix: run editor before transform propagation
- add translate/rotate/scale gizmos
- add indicators for cameras and lights in editor

## Version 0.3.1
- fix clear tab background
- fix editor camera viewport
- remove left alt shortcut for hiding ursor

## Version 0.3
- add `NotInScene` component to skip entity when saving to scene
- update egui etc.
- update to bevy-inspector-egui 0.16
- use `Time` pausing instead of previous hack
- add multiediting

## Version 0.2
- add right-click despawn options
- fix 3d camera
- allo marking entities as not pickable

## Version 0.1.1
- add ability to toggle game UI in editor view

## Version 0.1.0
- First release with editor cameras, inspector/hierarchy, configurable controls, schedule/render graph dump
