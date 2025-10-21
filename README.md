# Chippy - A Cross-Platform CHIP-8 Interpreter

Chippy is a Chip-8 interpreter for Windows, Linux, and MacOS. It's licensed under the terms of the GNU General Public License, version 3 (GPLv3).

Currently, this interpreter only supports the original CHIP-8 specification for the COSMAC VIP, however various quirk settings can be edited in the config.toml configuration file.

## Build Instructions

Build this project like any other Rust project.

## Configuration Instructions

Edit the config.toml file before running the interpreter to specify the desired configuration.

Setting a particular preset other than "custom" will overwrite various settings to match a particular CHIP-8 specification (e.g. the "chip8" preset uses the original CHIP-8 specification for the COSMAC VIP).

## Run Instructions

Run the interpreter from the command line, passing the path of the
Chip-8 program as an argument.