# Scoreboard Engine Change Log

## [v0.10.0] - 2026-05-24 - Team Release

### Breaking Changes

- Timers and Counters used `amount` on some actions for thier `value`, these are now `value`.
- Updated companion module and web dashboard to use `value` instead.

### New Features

- New Team Widget (value/short name, name, primary colour, secondary colour)
- Add `data-foreground` and `data-background` attributes to scoreboard.js helper
- Add colors to example HTML
- Update example configurations to use Team Widget
- Add Team Widget support to Companion module
- Fixed but in reset counter using first increment value instead of initial\_value
- Expand widget documentation and move to github wiki.

### Bug Fixes

- Added missing actions for Timers (set\_min, set\_max, set\_initial, set\_direction)
- Clock running companion feedback broken in last release, fixed.


## [v0.9.0] - 2026-05-23 - Pause Release

- Added `paused` and `paused\_time` state to Timer widgets.
- Remove `test` widget from example rugby league configuration
- Refactor code for flat json with new `extra_values` function on Widgets.
- Add `paused\_formatted` state to Timer widgets

## [v0.8.1] - 2026-05-21 

Fix issue in github workflow preventing automated builds.

## [v0.8.0] - 2026-05-21 - What's My Name Release

### BREAKING CHANGES

These changes require an update to your XML configuration.

- Renamed MappedList to List - Make widget names more sensible
- Renamed StaticText to Text - Make widget names more sensible

### Updates

- Add `*_running` state to flat JSON output for Timers

## [v0.7.0] - 2026-05-19 - Pocket Calculator Release

- Added Calculation type widget using rust evalexpr
- Added example AFL Configuration using Calculation type for `(home_goals * 6) + home_behinds`
- Documentation Updates for Calculation Field
- UI Tweaks - condensing spacing

## [v0.6.0] - 2026-05-18 Itsy Bitsy Spider Release

- OBS (Web title) Support!
- Added support for internal static HTML server to host web titles on the engine itself.
- Added scoreboard.js helper library for easy HTML title creation.
- Provide pages/example.html showing how to use the server & library

## [v0.5.0] - 2026-05-27 - You're My Best Friend Release

- *Bug*: Companion module was not returning the correct value for a MappedList. Fixed.
- Companion module renamed to scoreboard-engine (was scoreboard-scoreboard)
- Remove version numbers from READMEs and other documentation
- Major documentation enhancements
- Fixes to flat output for use in vMix
- Major refactor using polymorphic data structures and widget factory to
  decouple widget functionality.
- Add dashboard-ui option to hide widgets in the default generated webUI
- Updated code to use cargo version to remove redundant updates

## [v0.4.0] - 2026-05-16 - Faster Better Stronger Release

* Major refactor using polymorphic data structures and widget factory to 
  decouple widget functionality.
* Add dashboard-ui option to hide widgets in the default generated webUI
* Updated code to use cargo version to remove redundant updates

## [v0.3.0] - 2026-05-07 - Smoke on the Water Release

* Add formatted output to flat JSON for timers
* Convert flat output to a list to keep vMix Data sources happy

## [v0.2.0] - 2026-03-01 - Celebration Release

* Clean up for public release

## [v0.1.0 ] - Development - Careless Whisper Release

* Initial Release

