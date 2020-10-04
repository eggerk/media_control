# media\_control

Simple mpris-based media control utility, similar to [playerctl](https://github.com/altdesktop/playerctl), but with less features.

This tool has the ability to remember the last used *player*.
Every time this tool is used, the same *player* will be affected.
The selected *player* can be changed with the *next_player* or *previous_player* commands.
This change will be remembered the next time this program is run.

## Usage

```bash
media_control <command>
```

### Available commands

- `play`
- `pause`
- `playpause` or `play-pause`
- `next`: next song/video in the playlist.
- `previous`: previous song/video in the playlist.
- `next_player`, `previous_player`: cycle through the active media players.

### Example i3 configuration

The tool can be used in the following way in [i3wm](https://i3wm.org/):

```
bindsym XF86AudioPlay exec media_control play-pause
bindsym XF86AudioPause exec media_control pause
bindsym XF86AudioNext exec media_control next
bindsym XF86AudioPrev exec media_control previous

# Use Ctrl+Media Next/Prev keys to cycle through players.
bindsym Ctrl+XF86AudioNext exec media_control next_player
bindsym Ctrl+XF86AudioPrev exec media_control previous_player
```

## Differences to playerctl
- Remembers last used media player.
- Allows easy cycling through players.
- Misses some of the more advanced interaction that I never use.
- Shows a notification.

## References
- [playerctl](https://github.com/altdesktop/playerctl)
