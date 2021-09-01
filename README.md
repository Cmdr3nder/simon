# Simon (the Pi man)

A TUI Media Launcher originally designed to launch OMXPlayer on an old Raspberry Pi.
My Pi couldn't handle running a full GUI media center like Kodi.
I haven't used this project since mid 2019 when the network stack on my Raspberry Pi
stopped working.

## Configuration

This project uses https://crates.io/crates/directories to determine the director to expect a `simon.config.toml` configuration file at.
Typically for a linux box this would be `/home/username/.config/simon/simon.config.toml`.
In that file it expects a configuration like [this](example.config.toml)

## Usage

Once you launch `simon` from the command line you will be presented with a TUI based on your configuration.

Arrow-Keys control the cursor
Pressing Enter will move from region selection to item selection and back
Pressing P will start playing a media item
Pressing Q will quit out of Simon.

## Cross-Compilation

I've lost the instructions I was using to accomplish this,
but I used to manage to cross compile for the old Raspberry Pi ARM platform.
I'm sure there are several tutorials on how to cross-compile Rust projects for ARM platforms on the internet.
Good Luck!
