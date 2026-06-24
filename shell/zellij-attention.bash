# zellij-attention — bash integration
#
# Pairs with the `arm` / `cmd_done` plugin events to give you a manual
# "in-progress → done" flow (see zellij-attention.zsh for the zsh version):
#
#   1. While ANY foreground command is running, press your keybind (configured
#      in config.kdl via `MessagePlugin { name "zellij-attention::arm" }`).
#      Zellij — not the blocked shell — handles the keypress, so the plugin
#      flags the focused pane in-progress (⚡) immediately.
#   2. When that command finishes and the shell returns to the prompt, the
#      PROMPT_COMMAND hook below pings `cmd_done`. The plugin flips the pane to
#      ✓ only if it was armed; otherwise it is a no-op.
#
# You can also arm/cancel the current pane from the prompt with `attn` /
# `attn-cancel`.
#
# Source this from your ~/.bashrc:
#   [ -f /path/to/zellij-attention/shell/zellij-attention.bash ] \
#     && source /path/to/zellij-attention/shell/zellij-attention.bash
#
# Note: unlike the zsh hook, this pings cmd_done on every prompt (including a
# bare Enter). That is harmless — cmd_done is a no-op unless the pane is armed.

# Only meaningful inside a Zellij pane.
if [ -n "$ZELLIJ_PANE_ID" ]; then
  _zattn_skip=

  # precmd-equivalent: PROMPT_COMMAND runs each time the prompt is drawn, i.e.
  # right after a foreground command returns. Fire-and-forget inside a subshell
  # so it never blocks the prompt and emits no job-control message.
  _zattn_precmd() {
    if [ -n "$_zattn_skip" ]; then _zattn_skip=; return; fi
    [ -n "$ZELLIJ_PANE_ID" ] || return
    ( zellij action pipe --name "zellij-attention::cmd_done::$ZELLIJ_PANE_ID" -- "" >/dev/null 2>&1 & )
  }

  # Append our hook to PROMPT_COMMAND without clobbering or duplicating it.
  case ";${PROMPT_COMMAND};" in
    *";_zattn_precmd;"*) ;;
    *) PROMPT_COMMAND="${PROMPT_COMMAND:+$PROMPT_COMMAND;}_zattn_precmd" ;;
  esac

  # Arm the current pane from the prompt (the keybind does this mid-command).
  # Skip the cmd_done that fires on the very next prompt so `attn` itself does
  # not immediately complete the arm — it is resolved by the NEXT command.
  attn()        { zellij action pipe --name "zellij-attention::arm::$ZELLIJ_PANE_ID" -- ""; _zattn_skip=1; }
  # Cancel a pending arm without emitting the completed icon.
  attn-cancel() { zellij action pipe --name "zellij-attention::disarm::$ZELLIJ_PANE_ID" -- ""; }
fi
