# media-launcher

Media Launcher automatically matches TV episodes with their corresponding external audio and subtitle tracks and generates launch scripts for playback in your preferred video player.

Cross-platform, scriptable, and built for **MPV**/**VLC** players

## How to use
- Download executable application file from [releases](https://github.com/akoidan/media-launcher/releases)
- Run it either by opening it, either passing folder with TV series path as first argument
- THe app will scan **TV season folder** and matche each episode with its external audio and subtitle tracks using smart `\d\d` pattern.
- It will generater one launcher per episode (`01`, `02`, …):
  - **Linux / macOS:** `01.bash`, `02.bash`, …
  - **Windows:** `01.cmd`, `02.cmd`, …
- Open the corresponding launcher by double clicking on it and the player should open automatically.

## Playback

- Uses **mpv** or **VLC** (must be available in `$PATH`)
- If both are found, **mpv is preferred**
- If only one is found, it is used automatically
- All matching audio and subtitle tracks are attached

## OS
- Windows: Download `.exe` file from [releases](https://github.com/akoidan/media-launcher/releases)
- Arhclinux `yay -S media-launcher` or `paru -S media-launcher`
- Other Linux distro: : Download `.elf` file from [releases](https://github.com/akoidan/media-launcher/releases)

## Development
See [CONTRIBUTING.md](./CONTRIBUTING.md)
