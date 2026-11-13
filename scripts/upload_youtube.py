#!/usr/bin/env python3
"""Upload the built hackathon video to YouTube via YouTube Data API v3.

One-time setup in Google Cloud Console:
  1. Enable YouTube Data API v3 (`gcloud services enable youtube.googleapis.com`).
  2. Create an OAuth 2.0 Client ID of type "Desktop app" and save the JSON as:
     ~/.config/aaa-protracker/client_secrets.json
     (or pass --client-secrets /path/to/client_secrets.json).

Run from anywhere:
  uv run --no-project --with google-api-python-client --with google-auth-oauthlib \
    python ~/kod/007_hackathons/002_kiroween_2026/ai-agent-assisted-protracker/scripts/upload_youtube.py
"""
import argparse
import sys
from pathlib import Path

from google.auth.transport.requests import Request
from google.oauth2.credentials import Credentials
from google_auth_oauthlib.flow import InstalledAppFlow
from googleapiclient.discovery import build
from googleapiclient.http import MediaFileUpload

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_VIDEO = ROOT / "data" / "video" / "cut" / "AAA-PROTRACKER-PREVIEW.mp4"
DEFAULT_THUMB = ROOT / "data" / "video" / "cut" / "build" / "B1-card.png"
CONFIG_DIR = Path.home() / ".config" / "aaa-protracker"
DEFAULT_SECRETS = CONFIG_DIR / "client_secrets.json"
DEFAULT_TOKEN = CONFIG_DIR / "youtube_token.json"

SCOPES = [
    "https://www.googleapis.com/auth/youtube.upload",
    "https://www.googleapis.com/auth/youtube",
]

DEFAULT_TITLE = "AI-Agent-Assisted (AAA) ProTracker – Kiroween 2026 (Resurrection)"
DEFAULT_DESCRIPTION = (
    "AI-Agent-Assisted (AAA) ProTracker resurrects the 1987 Commodore Amiga 500 and MOS 8364 Paula "
    "4-channel .MOD tracker format for the AI agent era. Built from scratch for Kiroween 2026 "
    "(Resurrection category) using Kiro Spec-Driven Development, it lets you inspect, co-compose, "
    "and verify byte-exact 31-sample ProTracker modules that play natively in ProTracker 2 on macOS "
    "or on real Amiga hardware.\n\n"
    "0:00 Commodore Amiga 500 & MOS 8364 Paula (4 hardware DMA channels)\n"
    "0:10 The Birth of .MOD (Ultimate Soundtracker to ProTracker)\n"
    "0:20 The European Demoscene & 0xy Arpeggio Tricks\n"
    "0:30 AI-Agent-Assisted (AAA) ProTracker (Kiroween 2026 Resurrection)\n"
    "1:32 Round-Trip in ProTracker 2 Clone v1.93 (Turrican II Remix)\n"
    "1:55 Verified Against Kiro EARS Requirements\n\n"
    "GitHub: https://github.com/kizouker/ai-agent-assisted-protracker\n\n"
    "#Kiroween #Kiroween2026 #Kiro #Amiga #Amiga500 #ProTracker #MOD #Demoscene #Chiptune #Paula8364 #RetroComputing #AIAgents #SpecDrivenDevelopment"
)
DEFAULT_TAGS = [
    "Amiga",
    "ProTracker",
    "MOD",
    "Demoscene",
    "Paula8364",
    "Kiroween",
    "Kiro",
    "Chiptune",
    "RetroComputing",
]


def get_youtube_service(secrets_path: Path, token_path: Path):
    if not secrets_path.exists():
        sys.exit(
            f"Missing OAuth client secrets file: {secrets_path}\n"
            "Download an OAuth 2.0 Desktop client JSON from Google Cloud Console "
            "(APIs & Services -> Credentials) and save it to that path."
        )
    creds = None
    if token_path.exists():
        creds = Credentials.from_authorized_user_file(str(token_path), SCOPES)
    if not creds or not creds.valid:
        if creds and creds.expired and creds.refresh_token:
            creds.refresh(Request())
        else:
            flow = InstalledAppFlow.from_client_secrets_file(str(secrets_path), SCOPES)
            creds = flow.run_local_server(port=0)
        token_path.parent.mkdir(parents=True, exist_ok=True)
        token_path.write_text(creds.to_json())
    return build("youtube", "v3", credentials=creds)


def main() -> None:
    parser = argparse.ArgumentParser(description="Upload AAA ProTracker video to YouTube.")
    parser.add_argument("--file", type=Path, default=DEFAULT_VIDEO, help="MP4 file to upload")
    parser.add_argument("--title", default=DEFAULT_TITLE, help="YouTube video title")
    parser.add_argument("--description", default=DEFAULT_DESCRIPTION, help="Video description")
    parser.add_argument(
        "--privacy",
        choices=["unlisted", "public", "private"],
        default="unlisted",
        help="Privacy status (default: unlisted)",
    )
    parser.add_argument("--thumbnail", type=Path, default=DEFAULT_THUMB, help="Optional PNG/JPG thumbnail")
    parser.add_argument("--client-secrets", type=Path, default=DEFAULT_SECRETS, help="OAuth2 client_secrets.json")
    parser.add_argument("--token-file", type=Path, default=DEFAULT_TOKEN, help="Cached OAuth2 token path")
    args = parser.parse_args()

    if not args.file.exists():
        sys.exit(f"Video file not found: {args.file}")

    youtube = get_youtube_service(args.client_secrets, args.token_file)

    body = {
        "snippet": {
            "title": args.title,
            "description": args.description,
            "tags": DEFAULT_TAGS,
            "categoryId": "28",  # Science & Technology
            "defaultLanguage": "en",
            "defaultAudioLanguage": "en",
        },
        "status": {
            "privacyStatus": args.privacy,
            "selfDeclaredMadeForKids": False,
        },
    }

    media = MediaFileUpload(str(args.file), mimetype="video/mp4", chunksize=4 * 1024 * 1024, resumable=True)
    request = youtube.videos().insert(part="snippet,status", body=body, media_body=media)

    print(f"Uploading {args.file.name} ({args.privacy})...")
    response = None
    while response is None:
        status, response = request.next_chunk()
        if status:
            print(f"  Progress: {int(status.progress() * 100)}%")

    video_id = response["id"]
    print(f"Uploaded successfully: https://youtu.be/{video_id}")

    if args.thumbnail and args.thumbnail.exists():
        try:
            youtube.thumbnails().set(
                videoId=video_id,
                media_body=MediaFileUpload(str(args.thumbnail), mimetype="image/png"),
            ).execute()
            print(f"Set custom thumbnail: {args.thumbnail}")
        except Exception as exc:
            print(f"Note: custom thumbnail skipped ({exc})")


if __name__ == "__main__":
    main()
