# feedrv3-tui

Interactive terminal UI for the feedrv3 server API.

## Features
- Login with username/password.
- Browse feeds, favorites, and folders.
- Refresh data on demand.

## Usage
```
FEEDRV3_SERVER_URL=http://localhost:8091 cargo run -p feedrv3-tui
```

Controls:
- Login: type username/password, Tab to switch field, Enter to login.
- Tabs: `1` Feeds, `2` Favorites, `3` Folders, or Left/Right arrows.
- Navigation: Up/Down or `j`/`k`.
- Refresh: `r`.
- Quit: `q`.
