# zellij-attention — zsh integration
#
# Pairs with the `arm` / `cmd_done` plugin events to give you a manual
# "in-progress → done" flow:
#
#   1. While ANY foreground command is running, press your keybind (configured
#      in config.kdl via `MessagePlugin { name "zellij-attention::arm" }`).
#      Zellij — not the blocked shell — handles the keypress, so the plugin
#      flags the focused pane in-progress (⚡) immediately.
#   2. When that command finishes and the shell returns to the prompt, the
#      precmd hook below pings `cmd_done`. The plugin flips the pane to ✓ only
#      if it was armed; otherwise it is a no-op.
#
# You can also arm/cancel the current pane from the prompt with `attn` /
# `attn-cancel`.
#
# Source this from your ~/.zshrc:
#   [ -f /path/to/zellij-attention/shell/zellij-attention.zsh ] \
#     && source /path/to/zellij-attention/shell/zellij-attention.zsh

# Only meaningful inside a Zellij pane.
if [[ -n "$ZELLIJ_PANE_ID" ]]; then
  zmodload -F zsh/datetime b:EPOCHSECONDS 2>/dev/null
  autoload -Uz add-zsh-hook

  # Minimum command duration (seconds) before pinging cmd_done.
  # 0 (default) = ping after every command, so an armed pane always completes.
  # Set higher (e.g. 3) to skip quick commands and reduce background pings —
  # you typically only arm long-running commands anyway.
  : ${ZATTN_MIN_SECONDS:=0}

  _zattn_preexec() {
    _zattn_ran=1
    _zattn_start=$EPOCHSECONDS
  }

  _zattn_precmd() {
    # Skip bare prompts (no command ran) so repeated Enter doesn't fire.
    [[ -n "$ZELLIJ_PANE_ID" && -n "$_zattn_ran" ]] || return
    _zattn_ran=
    if (( ZATTN_MIN_SECONDS > 0 )); then
      (( EPOCHSECONDS - ${_zattn_start:-0} < ZATTN_MIN_SECONDS )) && return
    fi
    # Fire-and-forget inside a subshell: never blocks the prompt and emits no
    # job-control message.
    ( zellij action pipe --name "zellij-attention::cmd_done::$ZELLIJ_PANE_ID" -- "" &>/dev/null & )
  }

  add-zsh-hook preexec _zattn_preexec
  add-zsh-hook precmd _zattn_precmd

  # Arm the current pane from the prompt (the keybind does this mid-command).
  # Unset _zattn_ran so the precmd firing right after this command does NOT
  # immediately emit cmd_done — the arm should be resolved by the NEXT command
  # you run, not by `attn` itself.
  attn()        { zellij action pipe --name "zellij-attention::arm::$ZELLIJ_PANE_ID" -- ""; _zattn_ran=; }
  # Cancel a pending arm without emitting the completed icon.
  attn-cancel() { zellij action pipe --name "zellij-attention::disarm::$ZELLIJ_PANE_ID" -- ""; }
fi
