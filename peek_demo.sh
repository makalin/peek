#!/bin/bash
#
#  Peek - A simple TUI system dashboard
#  Requires: tmux, btop

# --- Configuration ---
SESSION_NAME="Peek"
NOTES_FILE="$HOME/peek_notes.txt"

# --- Prerequisite Check ---
command -v tmux >/dev/null 2>&1 || { echo >&2 "Error: 'tmux' is not installed. Please install it first (e.g., brew install tmux)."; exit 1; }
command -v btop >/dev/null 2>&1 || { echo >&2 "Error: 'btop' is not installed. Please install it first (e.g., brew install btop)."; exit 1; }

# --- Create notes file if it doesn't exist ---
touch "$NOTES_FILE"

# --- Define Commands for Panes ---

# 1. Port watcher (uses a loop instead of 'watch' for no dependencies)
PORT_CMD="while true; do clear; echo '--- Listening Ports (refreshing every 2s) ---'; lsof -i -P -n | grep LISTEN; sleep 2; done"

# 2. Info pane (cron + notes)
INFO_CMD="echo '--- Cron Jobs ---'; crontab -l; echo; echo '--- Notes ($NOTES_FILE) ---'; cat $NOTES_FILE; echo; echo \"(To edit notes, open a new terminal and run: nano $NOTES_FILE)\""

# --- Tmux Session Logic ---

# Check if the session already exists
tmux has-session -t $SESSION_NAME 2>/dev/null

if [ $? != 0 ]; then
  # Session does not exist. Create and configure it.
  echo "Creating new '$SESSION_NAME' session..."

  # 1. Create a new detached session named "Peek" with a window named "Dashboard"
  tmux new-session -d -s $SESSION_NAME -n 'Dashboard'

  # 2. Split the window vertically (top/bottom). Creates pane 0 (top) and 1 (bottom).
  tmux split-window -v -t $SESSION_NAME:Dashboard -p 60 # Top pane gets 60%

  # 3. Split the top pane (0) horizontally (left/right). Creates pane 0 (left) and 1 (right).
  #    The bottom pane automatically becomes pane 2.
  tmux split-window -h -t $SESSION_NAME:Dashboard.0

  # 4. Send commands to each pane
  # Pane 0 (Top-Left): btop
  tmux send-keys -t $SESSION_NAME:Dashboard.0 'btop' C-m

  # Pane 1 (Top-Right): Port Watcher
  tmux send-keys -t $SESSION_NAME:Dashboard.1 "$PORT_CMD" C-m

  # Pane 2 (Bottom): Cron & Notes
  tmux send-keys -t $SESSION_NAME:Dashboard.2 "$INFO_CMD" C-m

  # 5. Select the main 'btop' pane for immediate interaction
  tmux select-pane -t $SESSION_NAME:Dashboard.0

else
  # Session exists.
  echo "Attaching to existing '$SESSION_NAME' session..."
fi

# Attach to the session
tmux attach-session -t $SESSION_NAME