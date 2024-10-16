# IDDisplay
## about

This is a gui for song recognition using [ShazamIO](https://github.com/shazamio/ShazamIO) written in rust. The gui library i used is [iced](https://github.com/iced-rs/iced/tree/master).
 Currently it only recocnizes and displays the current song but i am also working on a spotify integration.

## Install

curently you need to compile the code yourself. I already created executables for the ShazamIO python code. The default recording device will be used to lisen for music.
The curently suported platforms are x86_64 windows and linux and aarch64 linux. My setup is a raspberry pi 4 with a touchscreen and a microphone.
With minor tweaks i am sure any platfornm supported by rust and python can work.

## usage

press the detedct button and wait until a song is recognized. It will try detect the song until you exit.

## quirks and features

Some genres of music like Techno are verry hard to detect so the detected song might not be the song that is playing. This will happen more with songs witout lyrics.
Currently the recording time increases until it finds a song starting at 3s maxing out at 24s. I am working on settings page to tweak the configuration.