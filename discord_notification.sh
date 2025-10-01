#!/bin/bash

# Discord notification script for release workflow
# Simple and clean approach

echo "🎯 Discord Notification Processing"
echo "=================================="

# Use environment variables from GitHub Actions
VERSION="${RELEASE_VERSION:-1.0.0-beta.1}"
IS_PRERELEASE="${IS_PRERELEASE:-true}"
REPO="${GITHUB_REPOSITORY:-mira-organization/born-in-hell}"

echo "Version: $VERSION"
echo "Is Prerelease: $IS_PRERELEASE"
echo "Repository: $REPO"

# Get release notes from GitHub API if in CI environment
if [[ -n "$GITHUB_TOKEN" && -n "$GITHUB_REPOSITORY" ]]; then
  echo "🔄 Fetching release notes from GitHub API..."
  
  # Wait a moment for the release to be fully created
  sleep 10
  
  # Get the release notes from the GitHub API with retry
  for i in {1..3}; do
    RELEASE_NOTES=$(curl -s -H "Authorization: token $GITHUB_TOKEN" \
      "https://api.github.com/repos/$REPO/releases/tags/v$VERSION" \
      | jq -r '.body // "No changelog available"')
    
    if [[ "$RELEASE_NOTES" != "No changelog available" ]] && [[ "$RELEASE_NOTES" != "null" ]]; then
      break
    fi
    echo "Attempt $i: Release notes not ready yet, waiting..."
    sleep 5
  done
else
  echo "❌ Error: GitHub token or repository not available"
  echo "This script must be run in a GitHub Actions environment with proper tokens"
  exit 1
fi

echo "📝 Processing changelog for Discord..."

# Simple processing - extract and format sections
format_section() {
  local title="$1"
  local emoji="$2"
  local content="$3"
  
  if [[ -n "$content" ]]; then
    echo "$emoji **$title:**"
    echo "$content" | while read -r line; do
      if [[ "$line" =~ ^\*.*\*\*#[0-9] ]]; then
        # Remove issue numbers and links, clean up
        clean_line=$(echo "$line" | sed 's/\* \*\*#[0-9]*:\*\* /  • /' | sed 's/\[[^]]*\]([^)]*)//g' | sed 's/, closes.*$//' | sed 's/ ()$//')
        echo "$clean_line"
      elif [[ "$line" =~ ^\* ]]; then
        # Simple bullet points
        clean_line=$(echo "$line" | sed 's/^\* /  • /' | sed 's/\[[^]]*\]([^)]*)//g' | sed 's/ ()$//')
        echo "$clean_line"
      fi
    done
    echo ""  # Add spacing after each section
  fi
}

# Extract features and bug fixes
FEATURES=$(echo "$RELEASE_NOTES" | sed -n '/### Features/,/### Bug Fixes/p' | sed '1d;$d')
BUGFIXES=$(echo "$RELEASE_NOTES" | sed -n '/### Bug Fixes/,$p' | sed '1d')

# Format sections
CHANGELOG=""
if [[ -n "$FEATURES" ]]; then
  FEATURES_FORMATTED=$(format_section "NEW FEATURES" "🚀" "$FEATURES")
  CHANGELOG+="$FEATURES_FORMATTED"
fi
if [[ -n "$BUGFIXES" ]]; then
  BUGFIXES_FORMATTED=$(format_section "BUG FIXES" "🐛" "$BUGFIXES")
  CHANGELOG+="$BUGFIXES_FORMATTED"
fi

# Remove trailing newlines and clean up
CHANGELOG=$(echo "$CHANGELOG" | sed -e :a -e '/^\s*$/{ $d; N; ba' -e '}' | sed 's/^[[:space:]]*$//')

# Add proper spacing between sections (smaller gap)
CHANGELOG=$(echo "$CHANGELOG" | sed 's/🐛 \*\*BUG FIXES:\*\*/\n🐛 **BUG FIXES:**/')

echo "📝 Processed changelog:"
echo "$CHANGELOG"

# Check if too long and use summary
if [[ ${#CHANGELOG} -gt 800 ]]; then
  echo "⚠️  Too long (${#CHANGELOG} chars) - using summary"
  CHANGELOG="🚀 **NEW FEATURES:**
  • Several new features and improvements

🐛 **BUG FIXES:**
  • Various bug fixes and optimizations"
fi

# Determine release type emoji
if [[ "$IS_PRERELEASE" == "true" ]]; then
  release_emoji="🧪"
else
  release_emoji="✅"
fi

# Send to Discord if webhook is available
if [[ -n "$DISCORD_WEBHOOK" ]]; then
  echo "📨 Sending Discord notification..."
  
  # Create simple Discord embed - put everything in description for better spacing
  JSON_PAYLOAD=$(cat << EOF
{
  "embeds": [
    {
      "title": "🎉 Born in Hell ${VERSION}",
      "description": "**📋 What's New:**\n\n${CHANGELOG//$'\n'/\\n}\n\n**📦 Downloads**\n**Desktop:**\n🐧 [Linux](https://github.com/${REPO}/releases/download/v${VERSION}/born-in-hell-linux-${VERSION}.zip) • 💻 [Windows](https://github.com/${REPO}/releases/download/v${VERSION}/born-in-hell-windows-${VERSION}.zip)\n\n**macOS:**\n🖥️ [Intel](https://github.com/${REPO}/releases/download/v${VERSION}/born-in-hell-macOS-intel-${VERSION}.dmg) • 💻 [Apple Silicon](https://github.com/${REPO}/releases/download/v${VERSION}/born-in-hell-macOS-apple-silicon-${VERSION}.dmg)\n\n🔗 [View Full Release](https://github.com/${REPO}/releases/tag/v${VERSION})",
      "color": $([[ "$IS_PRERELEASE" == "true" ]] && echo "16776960" || echo "5763719"),
      "footer": {
        "text": "$([[ "$IS_PRERELEASE" == "true" ]] && echo "🧪 Pre-Release" || echo "✅ Stable Release") • $(date +'%B %d, %Y')"
      },
      "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    }
  ]
}
EOF
)
  
  # Send to Discord
  response=$(echo "$JSON_PAYLOAD" | curl -s -w "\nHTTP_CODE:%{http_code}" -H "Content-Type: application/json" -X POST -d @- "$DISCORD_WEBHOOK")
  
  if echo "$response" | grep -q "HTTP_CODE:2"; then
    echo "✅ Discord notification sent successfully!"
  else
    echo "❌ Discord error: $response"
  fi
else
  echo "⚠️  No Discord webhook configured"
fi

echo ""
echo "✅ Discord notification processing completed!"