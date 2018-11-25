# Changelog

## 0.4.0 - 2018-11-24

### Added

 - Support for save files and save states.
 - Interface for changing APU sample rate. This can be used to have accurate audio playback at
   different speeds.
 - Interface for object attribute memory and checking if tall sprites are enabled.
 - More PPU tests.

### Changed

 - Cartridge information is now logged with `info!`/`console.log` instead of
   `debug!`/`console.debug`.

### Fixed

 - Mute ultrasonic frequencies in triangle channel. Since triangle channel clocks faster than the
   square channels, it can produce ultrasonic frequencies that produce undesirable audible sounds
   when passed into the mixer.
 - Fix small issues with palette mirroring and sprite-0 hit.

## 0.3.0 - 2018-11-11

### Changed

 - Change `console_error_panic_hook` as optional dependency.

### Fixed

 - Fix memory leak when ROMs get reloaded.
 - Handle out-of-range CHR ROM and PRG ROM banks appropriately.
 - Handle PRG RAM related flags in MMC3.

## 0.2.0 - 2018-11-10

### Added

 - Add basic logging framework compatible with different platforms.

### Changed

 - Clean up dependencies for different platforms.

## 0.1.0 - 2018-11-10

### Added

 - Initial functional emulator.
