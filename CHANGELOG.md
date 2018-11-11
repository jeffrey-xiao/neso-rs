# Changelog

## 0.3.0 - 2018-11-11

### Changed

 - Change `console_error_panic_hook` as optional dependency.

## Fixed

 - Fix memory leak when ROMs get reloaded.
 - Handle out-of-range CHR ROM and PRG ROM banks appropriately.
 - Handle PRG RAM related flags in MMC3.

## 0.2.0 - 2018-11-10

### Changed

 - Clean up dependencies for different platforms.

### Added

 - Add basic logging framework compatible with different platforms.

## 0.1.0 - 2018-11-10

### Added

 - Initial functional emulator.
