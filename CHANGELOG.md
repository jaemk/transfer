# Changelog

## [0.4.1]
### Added
- Release script for downloaded, unpacking, and symlinking
  the latest packaged release

### Changed
- Fixed password field validation
- Update readme with detailed deployment directions

### Removed
- deploy script


## [0.4.0]
### Added

### Changed
- Update migrant_toml
- Move `Migrant.toml` configuration file to xdg project directory
- Support sourcing `config.ron` from xdg project dir so a customized
  config can be moved out of the project directory
- Change deployment strategy
    - travis-ci will now build the server and frontend and package
      everything into a complete app that can be downloaded and rn
    - project updates should be downloaded and the entire project
      directory can be replaced as configured files are stored
      in the xdg project directory

### Removed

