# Storekeeper

Stamina resource tracking desktop tray application for various gacha games.

## Features

- **Real-time Stamina Tracking**: Monitor your stamina/energy resources across multiple gacha games
- **System Tray Application**: Runs quietly in the background with a convenient system tray interface
- **Desktop Notifications**: Get notified when your stamina reaches configurable thresholds or is full
- **Auto Daily Rewards**: Automatically claim daily check-in rewards for HoYoLab games (optional)
- **Multi-Game Support**:
  - Genshin Impact (Original Resin, Realm Currency, Parametric Transformer, Expeditions)
  - Honkai: Star Rail (Trailblaze Power)
  - Zenless Zone Zero (Battery)
  - Wuthering Waves (Waveplates)

## Installation

### Pre-built Releases

Download the latest release for your platform from the [Releases](https://github.com/kvnxiao/storekeeper/releases) page.

### Building from Source

See [DEVELOPMENT.md](DEVELOPMENT.md) for build instructions.

## Configuration Setup

Storekeeper uses two configuration files stored in your system's config directory:

| Platform | Config Directory                              |
|----------|-----------------------------------------------|
| Windows  | `%APPDATA%\storekeeper\`                      |
| macOS    | `~/Library/Application Support/storekeeper/` |
| Linux    | `~/.config/storekeeper/`                      |

### 1. Main Configuration (`config.toml`)

Copy `config.example.toml` to your config directory as `config.toml` and customize:

```toml
[general]
poll_interval_secs = 300  # How often to check resources (5 minutes)
start_minimized = true    # Start minimized to system tray
log_level = "info"        # Logging verbosity

[notifications]
enabled = true
cooldown_minutes = 30     # Minimum time between notifications

# Enable games you play
[games.genshin_impact]
enabled = true
uid = "YOUR_UID_HERE"

[games.honkai_star_rail]
enabled = true
uid = "YOUR_UID_HERE"

[games.zenless_zone_zero]
enabled = true
uid = "YOUR_UID_HERE"

[games.wuthering_waves]
enabled = true
player_id = "YOUR_PLAYER_ID_HERE"
```

### 2. Secrets Configuration (`secrets.toml`)

Copy `secrets.example.toml` to your config directory as `secrets.toml` and add your credentials:

#### HoYoLab Games (Genshin, HSR, ZZZ)

1. Go to [HoYoLab](https://www.hoyolab.com) and log in
2. Open browser Developer Tools (F12) > Application > Cookies
3. Copy the values for `ltmid_v2`, `ltoken_v2`, and `ltuid_v2`

```toml
[hoyolab]
ltmid_v2 = "YOUR_LTMID_V2_HERE"
ltoken_v2 = "YOUR_LTOKEN_V2_HERE"
ltuid_v2 = "YOUR_LTUID_V2_HERE"
```

#### Wuthering Waves

Credentials are automatically loaded from the Kuro launcher cache at:
`%APPDATA%\KR_G153\A1730\KRSDKUserLauncherCache.json`

No manual configuration required if you've logged into the game launcher.

## Development

See [DEVELOPMENT.md](DEVELOPMENT.md) for architecture details, development setup, and contribution guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details.
