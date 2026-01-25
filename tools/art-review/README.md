# Art Review Tool

A dynamic web application for reviewing and marking Essence Wars card artwork for regeneration.

## Features

- **Visual Gallery**: Display all card artwork in a responsive grid
- **Card Information**: Shows card name, ID, and faction
- **Mark for Regeneration**: Click any card or use the button to mark it
- **Filtering**: Filter by faction (Argentum, Symbiote, Obsidion, Neutral)
- **Search**: Search cards by name or ID
- **Status Filters**: View only marked or unmarked cards
- **Persistent Storage**: Marks are saved in browser localStorage
- **Export**: Export marked cards to JSON with name and file path
- **Statistics**: Real-time count of total and marked cards

## Quick Start

### Option 1: Python HTTP Server (Recommended)

```bash
# From the art-review directory
python3 -m http.server 8000
```

Then open: http://localhost:8000

### Option 2: Using the launch script

```bash
# From the project root
./tools/art-review/launch.sh
```

## Usage

1. **Browse Cards**: Scroll through the gallery to view all artwork
2. **Mark Cards**: Click a card or the "Mark for Regeneration" button
3. **Filter**: Use faction buttons to filter by faction
4. **Search**: Type in the search box to find specific cards
5. **Export**: Click "Export Marked Cards" to download a JSON file
6. **Clear**: Click "Clear All Marks" to reset all selections

## Export Format

The exported JSON includes:

```json
{
  "exported_at": "2026-01-25T10:30:00.000Z",
  "total_marked": 5,
  "cards": [
    {
      "id": 1000,
      "name": "Brass Sentinel",
      "faction": "argentum",
      "file_path": "../../crates/essence-wars-ui/static/cards/core_set/1000.webp",
      "absolute_path": "/home/chris/ai-cardgame/crates/essence-wars-ui/static/cards/core_set/1000.webp"
    }
  ]
}
```

## Technical Details

- **Pure HTML/CSS/JavaScript**: No build step required
- **Local Storage**: Marks persist across sessions
- **YAML Parser**: Loads card names from YAML definitions
- **Responsive Design**: Works on desktop and tablet
- **Image Fallback**: Shows placeholder for missing images

## File Structure

```
tools/art-review/
├── index.html       # Main HTML page
├── app.js          # Application logic
├── launch.sh       # Launch script
└── README.md       # This file
```

## Notes

- The tool automatically loads card data from `data/cards/core_set/*.yaml`
- Images are loaded from `crates/essence-wars-ui/static/cards/core_set/*.webp`
- Marks are saved in browser localStorage and persist across sessions
- The export button downloads a timestamped JSON file to your Downloads folder
