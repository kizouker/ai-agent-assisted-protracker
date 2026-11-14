#!/bin/bash
# Verify and upload the Kiroween 2026 hackathon video to YouTube (API or Studio fallback).
# Runs on: M4 (local macOS). Modes: apply | check
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/.." && pwd)"
. "$HERE/ui.sh"
MODE="${1:-apply}"

VIDEO="$ROOT/data/video/cut/AAA-PROTRACKER-PREVIEW.mp4"
THUMB="$ROOT/data/video/cut/build/B1-card.png"
URL_FILE="$ROOT/data/video/cut/youtube-url.txt"
CONFIG_DIR="$HOME/.config/aaa-protracker"
SECRETS="$CONFIG_DIR/client_secrets.json"
TOKEN="$CONFIG_DIR/youtube_token.json"

title "AAA ProTracker – YouTube Upload ($MODE)"

step "Preflight: tools and video file"
what "Verify ffprobe, uv, and that AAA-PROTRACKER-PREVIEW.mp4 exists and meets hackathon limits."

command -v ffprobe >/dev/null 2>&1 || fail "ffprobe is missing. Install ffmpeg with: brew install ffmpeg"
command -v uv >/dev/null 2>&1 || fail "uv is missing. Install uv with: brew install uv"
ok "Required CLI tools found (ffprobe, uv)."

[ -f "$VIDEO" ] || fail "Video file missing: $VIDEO — build it first with scripts/build_video_cut.py --tts"
ok "Found video file: $VIDEO"

WIDTH="$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 "$VIDEO")"
HEIGHT="$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 "$VIDEO")"
SRATE="$(ffprobe -v error -select_streams a:0 -show_entries stream=sample_rate -of csv=p=0 "$VIDEO")"
DUR="$(ffprobe -v error -show_entries format=duration -of csv=p=0 "$VIDEO")"
DUR_INT="${DUR%.*}"

[ "$WIDTH" = "1920" ] && [ "$HEIGHT" = "1080" ] || fail "Expected 1920x1080 video, got ${WIDTH}x${HEIGHT}"
[ "$SRATE" = "48000" ] || fail "Expected 48000 Hz audio, got ${SRATE} Hz"
[ "$DUR_INT" -lt 180 ] || fail "Video duration ${DUR}s exceeds the 180s (3 min) hackathon limit"
ok "Video verified: ${WIDTH}x${HEIGHT}, ${SRATE} Hz audio, ${DUR}s (< 180s)."

step "Preflight: YouTube OAuth credentials"
what "Check ~/.config/aaa-protracker/client_secrets.json or auto-detect a downloaded client_secret_*.json in ~/Downloads."

LATEST_DL="$(ls -t "$HOME"/Downloads/client_secret*.json 2>/dev/null | head -n 1 || true)"
if [ ! -f "$SECRETS" ] && [ -n "$LATEST_DL" ]; then
  info "Found downloaded OAuth client file in ~/Downloads: $LATEST_DL"
  if [ "$MODE" = "apply" ]; then
    mkdir -p "$CONFIG_DIR"
    cp "$LATEST_DL" "$SECRETS"
    chmod 600 "$SECRETS"
  fi
fi

if [ -f "$SECRETS" ]; then
  ok "OAuth client secrets ready: $SECRETS"
else
  warn "No OAuth client_secrets.json found at $SECRETS"
  info "To enable 1-click CLI upload: download an OAuth 2.0 Desktop client JSON from"
  info "https://console.cloud.google.com/apis/credentials into ~/Downloads and re-run this script."
fi

if [ "$MODE" = "check" ]; then
  title "Check only: nothing changed"
  exit 0
fi

if [ ! -f "$SECRETS" ]; then
  step "Fallback: prepare manual YouTube Studio upload"
  what "Copy video title + description to clipboard and reveal MP4 in Finder for instant drag-and-drop."
  cat <<'EOF' | pbcopy
AI-Agent-Assisted (AAA) ProTracker – Kiroween 2026 (Resurrection)

AI-Agent-Assisted (AAA) ProTracker resurrects the 1987 Commodore Amiga 500 and MOS 8364 Paula 4-channel .MOD tracker format for the AI agent era. Built from scratch for Kiroween 2026 (Resurrection category) using Kiro Spec-Driven Development, it lets you inspect, co-compose, and verify byte-exact 31-sample ProTracker modules that play natively in ProTracker 2 on macOS or on real Amiga hardware.

0:00 Commodore Amiga 500 & MOS 8364 Paula (4 hardware DMA channels)
0:10 The Birth of .MOD (Ultimate Soundtracker to ProTracker)
0:20 The European Demoscene & 0xy Arpeggio Tricks
0:30 AI-Agent-Assisted (AAA) ProTracker (Kiroween 2026 Resurrection)
1:32 Round-Trip in ProTracker 2 Clone v1.93 (Turrican II Remix)
1:55 Verified Against Kiro EARS Requirements

GitHub: https://github.com/kizouker/ai-agent-assisted-protracker

#Kiroween #Kiroween2026 #Kiro #Amiga #Amiga500 #ProTracker #MOD #Demoscene #Chiptune #Paula8364 #RetroComputing #AIAgents #SpecDrivenDevelopment
EOF
  CLIP_LEN="$(pbpaste | wc -c | tr -d ' ')"
  [ "$CLIP_LEN" -gt 100 ] || fail "Failed to copy title and description to clipboard"
  ok "Copied title and timestamped description to clipboard (${CLIP_LEN} bytes)."
  open -R "$VIDEO"
  ok "Revealed $VIDEO in Finder."
  title "Ready: drag AAA-PROTRACKER-PREVIEW.mp4 into YouTube Studio and press Cmd+V"
  exit 0
fi

step "Upload video via YouTube Data API v3"
what "Run scripts/upload_youtube.py and save the resulting YouTube URL."

OUT_LOG="$(mktemp -t aaa_yt_upload.XXXXXX)"
run "uv run --no-project --with google-api-python-client --with google-auth-oauthlib python $HERE/upload_youtube.py"
if ! uv run --no-project --with google-api-python-client --with google-auth-oauthlib \
  python "$HERE/upload_youtube.py" --file "$VIDEO" --thumbnail "$THUMB" 2>&1 | tee "$OUT_LOG"; then
  rm -f "$OUT_LOG"
  fail "YouTube API upload failed. Check OAuth consent screen and YouTube Data API v3 status."
fi

YT_URL="$(grep -Eo 'https://youtu\.be/[A-Za-z0-9_-]+' "$OUT_LOG" | tail -n 1 || true)"
rm -f "$OUT_LOG"

step "Verify uploaded YouTube URL"
what "Confirm a valid youtu.be URL was returned, save it to data/video/cut/youtube-url.txt, and copy to clipboard."

[ -n "$YT_URL" ] || fail "Upload finished but no https://youtu.be/... URL was returned"
printf '%s\n' "$YT_URL" > "$URL_FILE"
SAVED_URL="$(cat "$URL_FILE")"
[ "$SAVED_URL" = "$YT_URL" ] || fail "Could not verify saved URL in $URL_FILE"
printf '%s' "$YT_URL" | pbcopy
ok "YouTube URL saved to $URL_FILE and copied to clipboard: $YT_URL"

title "Done – YouTube URL: $YT_URL"
