# Troubleshooting

## Notifications not appearing

1. **Check plugin is loaded:**
   - Verify `load_plugins` is configured in your `config.kdl`
   - Check Zellij logs for `zellij-attention: v… loaded` (log dir is OS-specific):
     - Linux: `tail -f /tmp/zellij-*/zellij-log/zellij.log`
     - macOS: `tail -f $TMPDIR/zellij-*/zellij-log/zellij.log`

2. **Verify pipe commands work:**
   ```bash
   echo $ZELLIJ_PANE_ID  # Should print a number
   zellij pipe --name "zellij-attention::working::$ZELLIJ_PANE_ID"
   # Tab name should change immediately
   ```
   > **Nushell:** `$ZELLIJ_PANE_ID` is *not* expanded inside `"…"`. Use string
   > interpolation: `zellij pipe --name $"zellij-attention::working::($env.ZELLIJ_PANE_ID)"`.

3. **Clear the compiled-plugin cache** (then start a fresh session — `load_plugins` loads at session start):
   ```bash
   # Zellij caches compiled WASM by path; clear if the plugin isn't updating.
   # Linux:
   find ~/.cache/zellij -path "*zellij-attention*" -delete
   # macOS:
   find ~/Library/Caches/org.Zellij-Contributors.Zellij -path "*zellij-attention*" -delete
   ```

## Plugin not loading

- Verify the plugin is in `load_plugins` in your `config.kdl` with the correct file path
- Check that the `.wasm` file exists at `~/.config/zellij/plugins/zellij-attention.wasm`

## Pipe command hangs or does nothing

- Ensure you're using the `--name` flag (broadcast), NOT `--plugin` (targeted)
- Check `$ZELLIJ_PANE_ID` is set: `echo $ZELLIJ_PANE_ID`
- Verify the format uses double-colon separators: `zellij-attention::EVENT_TYPE::PANE_ID`

### Wrong format examples

**Correct:**
```bash
zellij pipe --name "zellij-attention::working::5"
```

**Common mistakes:**
```bash
# WRONG: Single colon
zellij pipe --name "zellij-attention:working:5"

# WRONG: Missing plugin name prefix
zellij pipe --name "working::5"

# WRONG: Using --plugin instead of --name
zellij pipe --plugin "zellij-attention" --message "working::5"
```

## Tabs not restoring original names

- This is expected if notifications are still active on other panes in the same tab
- Focus the pane with the notification to clear it — the tab name restores automatically
- To force-clear all notifications, restart the Zellij session (state is memory-only)
