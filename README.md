# media-launcher

Media Launcher automatically matches TV episodes with their corresponding external audio and subtitle tracks and generates launch scripts for playback in your preferred video player.

Cross-platform, scriptable, and built for **[MPV](https://mpv.io/)**, **[VLC](https://www.videolan.org/)**, **[POT](https://potplayer.tv/)** players


## Installation

### Windows
Download `.exe` file from [releases](https://github.com/akoidan/media-launcher/releases)

### Archlinux
```bash
yay -S media-launcher
``` 
or 
```bash
paru -S media-launcher
```

### MacOS

```bash
brew tap akoidan/media-launcher https://github.com/akoidan/media-launcher
brew trust akoidan/media-launcher
brew install media-launcher
```

### Other Linux distro
Download `.elf` file from [releases](https://github.com/akoidan/media-launcher/releases)


## How to use
- Download executable application file from [releases](https://github.com/akoidan/media-launcher/releases)
- Run it either by opening it, either passing folder with TV series path as first argument
- THe app will scan **TV season folder** and matche each episode with its external audio and subtitle tracks using smart `\d\d` pattern.
- It will generater one launcher per episode (`01`, `02`, …):
  - **Linux:** `01.bash`, `02.bash`, …
  - **Windows:** `01.cmd`, `02.cmd`, …
  - **Mac:** `01.command`, `02.command`, …
- Open the corresponding launcher by double clicking on it and the player should open automatically.

## Playback

- Uses **mpv**, **VLC**, or **PotPlayer** (Windows only; must be available in `$PATH` or its default install location)
- If multiple are found, priority is **mpv > VLC > PotPlayer**
- If only one is found, it is used automatically
- All matching audio and subtitle tracks are attached


## Development
See [CONTRIBUTING.md](./CONTRIBUTING.md)
