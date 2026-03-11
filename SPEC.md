# Memoria - AI-Powered Personal Memory System

## Project Overview

**Project Name:** Memoria  
**Project Type:** Desktop Application (Tauri + Rust + JavaScript)  
**Core Functionality:** An AI-powered personal memory system that continuously records and indexes a user's digital activity, enabling semantic search of their entire digital life using natural language.  
**Target Users:** Knowledge workers, developers, researchers, and anyone who needs to recall digital information they've encountered.

---

## Architecture Overview

### Technology Stack
- **Frontend:** Vanilla JavaScript, HTML, CSS (no TypeScript)
- **Backend:** Rust with Tauri
- **Database:** SQLite (metadata) + Qdrant (vector embeddings)
- **AI:** Local embedding model via ONNX Runtime
- **OCR:** macOS Vision framework (primary), Tesseract (fallback)

### Project Structure
```
memoria/
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # Library exports
│   │   ├── activity/            # Activity Capture Engine
│   │   │   ├── mod.rs
│   │   │   ├── app_monitor.rs   # Application focus monitoring
│   │   │   ├── file_monitor.rs  # File access events
│   │   │   ├── clipboard.rs     # Clipboard monitoring
│   │   │   └── browser.rs       # Browser history (optional)
│   │   ├── capture/             # Screen Understanding Pipeline
│   │   │   ├── mod.rs
│   │   │   ├── screenshot.rs    # Screenshot capture
│   │   │   └── ocr.rs           # OCR processing
│   │   ├── processing/          # Content Processing Engine
│   │   │   ├── mod.rs
│   │   │   ├── cleaner.rs       # Text cleaning
│   │   │   ├── tokenizer.rs     # Tokenization
│   │   │   └── structurer.rs    # Data structuring
│   │   ├── embedding/           # Embedding & Indexing
│   │   │   ├── mod.rs
│   │   │   ├── model.rs         # ONNX embedding model
│   │   │   ├── vector_store.rs  # Qdrant integration
│   │   │   └── indexer.rs       # Incremental indexing
│   │   ├── knowledge/           # Knowledge Graph Builder
│   │   │   ├── mod.rs
│   │   │   ├── graph.rs         # Graph structure
│   │   │   └── relations.rs     # Relationship extraction
│   │   ├── search/              # Search & Retrieval Engine
│   │   │   ├── mod.rs
│   │   │   ├── query.rs         # Query processing
│   │   │   ├── retrieval.rs     # Vector similarity search
│   │   │   └── ranking.rs       # Result ranking
│   │   ├── timeline/           # Timeline Engine
│   │   │   ├── mod.rs
│   │   │   ├── session.rs       # Session management
│   │   │   └── builder.rs       # Timeline reconstruction
│   │   ├── storage/            # Database layer
│   │   │   ├── mod.rs
│   │   │   ├── sqlite.rs        # SQLite operations
│   │   │   └── schema.rs        # Database schema
│   │   ├── config/             # Configuration management
│   │   │   ├── mod.rs
│   │   │   └── settings.rs      # User settings
│   │   ├── ipc/                # Tauri IPC handlers
│   │   │   ├── mod.rs
│   │   │   └── commands.rs      # Command definitions
│   │   └── utils/              # Utilities
│   │       ├── mod.rs
│   │       ├── logger.rs       # Logging
│   │       └── metrics.rs      # Performance metrics
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── icons/
├── src/                         # JavaScript frontend
│   ├── index.html
│   ├── main.js                 # Entry point
│   ├── styles/
│   │   ├── main.css           # Main styles
│   │   ├── skeuomorphic.css   # Skeuomorphic design
│   │   ├── components.css     # Component styles
│   │   └── views.css          # View-specific styles
│   ├── js/
│   │   ├── app.js             # Main application
│   │   ├── router.js          # Simple router
│   │   ├── views/
│   │   │   ├── search.js      # Search view
│   │   │   ├── timeline.js    # Timeline view
│   │   │   ├── preview.js     # Preview panel
│   │   │   └── graph.js       # Memory graph view
│   │   ├── components/
│   │   │   ├── search-bar.js  # Search bar component
│   │   │   ├── memory-card.js # Memory card
│   │   │   ├── timeline-item.js
│   │   │   ├── toggle.js      # Skeuomorphic toggle
│   │   │   ├── button.js      # Button component
│   │   │   └── modal.js       # Modal dialog
│   │   └── utils/
│   │       ├── api.js         # Tauri IPC wrapper
│   │       ├── format.js      # Formatting utilities
│   │       └── debounce.js    # Debounce utility
│   └── assets/
│       └── icons/             # UI icons
├── config/                     # Configuration files
│   ├── default.toml          # Default settings
│   └── modules.json          # Module enable/disable
├── models/                    # ML models (downloaded at runtime)
├── docs/                      # Documentation
├── scripts/                   # Build scripts
├── SPEC.md
├── README.md
└── package.json
```

---

## UI/UX Specification

### Design Philosophy
Apple-inspired skeuomorphic design with:
- Tactile elements that feel like physical objects
- Glass layers and depth
- Subtle shadows and textures
- Smooth gradients
- Premium macOS utility aesthetic

### Color Palette
- **Background Primary:** #1E1E1E (dark charcoal)
- **Background Secondary:** #2D2D2D (elevated surface)
- **Background Tertiary:** #3A3A3A (cards/panels)
- **Glass Effect:** rgba(255, 255, 255, 0.05)
- **Accent Primary:** #007AFF (macOS blue)
- **Accent Secondary:** #30D158 (macOS green)
- **Accent Warning:** #FF9F0A (macOS orange)
- **Accent Danger:** #FF453A (macOS red)
- **Text Primary:** #FFFFFF
- **Text Secondary:** #8E8E93
- **Text Tertiary:** #636366
- **Border Light:** rgba(255, 255, 255, 0.1)
- **Border Dark:** rgba(0, 0, 0, 0.3)
- **Shadow:** rgba(0, 0, 0, 0.4)

### Typography
- **Font Family:** -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", "Helvetica Neue", sans-serif
- **Heading 1:** 28px, weight 600
- **Heading 2:** 22px, weight 600
- **Heading 3:** 18px, weight 500
- **Body:** 14px, weight 400
- **Caption:** 12px, weight 400
- **Monospace:** "SF Mono", Monaco, "Courier New", monospace

### Spacing System
- **Base Unit:** 8px
- **XS:** 4px
- **SM:** 8px
- **MD:** 16px
- **LG:** 24px
- **XL:** 32px
- **XXL:** 48px

### Visual Effects
- **Glass Panel:** backdrop-filter: blur(20px), semi-transparent background
- **Card Shadow:** 0 4px 12px rgba(0, 0, 0, 0.3), 0 1px 3px rgba(0, 0, 0, 0.2)
- **Button Shadow:** 0 2px 4px rgba(0, 0, 0, 0.3), inset 0 1px 0 rgba(255, 255, 255, 0.1)
- **Inset Shadow:** inset 0 2px 4px rgba(0, 0, 0, 0.3)
- **Border Radius:** 8px (cards), 6px (buttons), 12px (panels), 20px (search bar)
- **Gradient Light:** linear-gradient(180deg, rgba(255,255,255,0.1) 0%, transparent 100%)
- **Gradient Dark:** linear-gradient(180deg, rgba(0,0,0,0.2) 0%, transparent 100%)
- **Transition:** all 0.2s ease-out

### Components

#### Search Bar
- Large, prominent input field with glass effect
- Subtle inner shadow for depth
- Magnifying glass icon (left)
- Clear button (right, appears when text present)
- Voice input button (optional)
- Focus state: glowing border (#007AFF)

#### Memory Cards
- Rounded corners with subtle shadow
- Glass overlay on hover
- Icon indicating activity type (file, browser, app, etc.)
- Title, timestamp, and context preview
- Subtle gradient background

#### Skeuomorphic Buttons
- Raised appearance with gradient
- Subtle inner highlight (top edge)
- Pressed state: inset shadow, slight scale down
- Hover: slight brightness increase

#### Toggle Switches
- Glossy track with metallic appearance
- Circular thumb with subtle reflection
- Smooth slide animation
- Color change based on state

#### Timeline Items
- Vertical line connector
- Circular node markers
- Activity icons
- Expandable content area
- Glass panel style

#### Graph Nodes
- Circular with gradient fill
- Connection lines with fade
- Hover: scale up, glow effect
- Selected: prominent border

### Layout Structure

#### Main Window (800x600 default, resizable)
```
+------------------------------------------+
|  [Title Bar - Custom with traffic lights]|
+------------------------------------------+
|  [Sidebar]  |  [Main Content Area]       |
|  - Search   |  - View content here        |
|  - Timeline |  - Dynamic based on route  |
|  - Graph    |                             |
|  - Settings |                             |
|             |                             |
+------------------------------------------+
|  [Status Bar - Activity indicator]        |
+------------------------------------------+
```

#### Sidebar (200px fixed)
- Logo at top
- Navigation items with icons
- Active state: accent color, subtle glow
- Collapse button

#### Content Area (flexible)
- View-specific layouts
- Smooth transitions between views

#### Status Bar (32px)
- Recording indicator (pulsing dot)
- Event count
- Last activity timestamp
- Developer mode toggle (hidden by default)

---

## Functionality Specification

### 1. Activity Capture Engine

#### Application Focus Monitoring
- Track active application changes
- Record: app bundle ID, app name, window title, timestamp
- Detect idle periods (no input for X minutes)
- Configurable sampling rate (default: 1 second)

#### File Access Events
- Monitor file open/create/modify events
- Record: file path, operation type, timestamp, app that accessed
- Use FSEvents API on macOS
- Filter by user-configured directories

#### Clipboard Monitoring
- Capture clipboard changes (text only by default)
- Record: content, source app, timestamp
- Option to exclude sensitive apps (password managers)

#### Browser History (Optional)
- Safari: Use AppleScript to query history database
- Chrome: Read from SQLite history file
- Firefox: Read from SQLite places database
- Configurable: enable/disable per browser

### 2. Screen Understanding Pipeline

#### Screenshot Capture
- Periodic capture: configurable interval (default: 30 seconds)
- Active window only (not full screen)
- Compress to JPEG (quality: 70%)
- Store in local cache with expiry
- Privacy: capture disabled during screen lock

#### OCR Processing
- Primary: macOS Vision framework (VNRecognizeTextRequest)
- Fallback: Tesseract OCR
- Extract text with bounding boxes
- Filter: remove empty/minimal text
- Associate with screenshot timestamp

### 3. Content Processing Engine

#### Text Cleaning
- Remove excessive whitespace
- Normalize Unicode
- Strip control characters
- Handle encoding issues

#### Tokenization
- Split into sentences and words
- Preserve meaningful boundaries
- Handle code blocks specially

#### Data Structuring
- Create standardized activity records
- Extract metadata (URLs, file types, etc.)
- Categorize content type

### 4. Embedding and Semantic Indexing System

#### Embedding Model
- Model: sentence-transformers (all-MiniLM-L6-v2 or similar)
- Runtime: ONNX Runtime for efficiency
- Dimensions: 384 (MiniLM-L6-v2)
- Quantization: INT8 for faster inference

#### Vector Storage
- Primary: Qdrant (local mode)
- Fallback: SQLite with vector extension
- Index type: HNSW
- Payload: metadata, timestamp, activity type

#### Incremental Indexing
- Batch size: 10 items
- Queue system for pending embeddings
- Background processing (non-blocking)
- Retry logic for failed items

### 5. Knowledge Graph Builder

#### Graph Structure
- Nodes: Activities, Applications, Files, URLs, Sessions
- Edges: Relationships between nodes
- Properties: timestamps, frequencies, context

#### Relationship Extraction
- Temporal: same session, time proximity
- Functional: same app, related files
- Content: semantic similarity (via embeddings)
- Usage: frequent co-occurrence

### 6. Search and Retrieval Engine

#### Query Processing
- Parse natural language query
- Extract intent: search type, time range, content filters
- Handle relative dates ("last Thursday", "yesterday afternoon")
- Extract entity mentions

#### Vector Similarity Search
- Convert query to embedding
- Search vector store
- Combine with metadata filters

#### Ranking Algorithm
- Base score: cosine similarity
- Recency boost: exponential decay (half-life: 7 days)
- Frequency boost: repeated access indicates importance
- Context boost: same session/activity cluster
- Final score: weighted combination

### 7. Timeline Engine

#### Session Detection
- Idle threshold: 15 minutes
- Session types: work, browsing, coding, media
- App-based classification

#### Timeline Reconstruction
- Group activities by session
- Sort chronologically
- Aggregate related events
- Generate summary

### 8. Context Builder

#### Session Management
- Detect session boundaries
- Track session metadata
- Associate activities with sessions
- Calculate session statistics

---

## Data Flow

### Activity to Memory Pipeline
```
[User Activity]
    ↓
[Activity Capture] → [SQLite: activities table]
    ↓
[Content Processing] → [Cleaned Text]
    ↓
[Embedding Generation] → [Vector + Metadata]
    ↓
[Vector Storage (Qdrant)] ←→ [Knowledge Graph Update]
    ↓
[Search Index Updated]
```

### Query Pipeline
```
[User Query]
    ↓
[Query Processing] → [Intent + Filters]
    ↓
[Embedding Generation]
    ↓
[Vector Search] → [Candidate Results]
    ↓
[Ranking Engine] → [Ranked Results]
    ↓
[Result Presentation]
```

---

## Configuration

### Module Configuration (config/modules.json)
```json
{
  "activity_capture": {
    "enabled": true,
    "app_monitoring": true,
    "file_monitoring": true,
    "clipboard_monitoring": true,
    "browser_history": false
  },
  "screen_capture": {
    "enabled": true,
    "interval_seconds": 30,
    "active_window_only": true,
    "ocr_enabled": true
  },
  "privacy": {
    "exclude_apps": ["1Password", "Bitwarden", "Keychain Access"],
    "exclude_directories": ["/Users/*/Library/Containers"],
    "pause_on_lock": true,
    "retention_days": 90
  }
}
```

### Settings (config/default.toml)
```toml
[general]
data_directory = "~/.memoria"
log_level = "info"

[performance]
max_concurrent_ocr = 2
embedding_batch_size = 10
screenshot_cache_mb = 500

[developer]
debug_mode = false
metrics_port = 9090
```

---

## Privacy Controls

### User Controls
- **Pause Tracking:** Global pause button in status bar
- **Delete Memory:** Clear all stored data option
- **Exclude Apps:** List of apps to ignore
- **Exclude Directories:** Paths to exclude from monitoring
- **Retention Period:** Auto-delete data older than X days
- **Data Export:** Export all data as JSON

### Privacy Guarantees
- All processing local (no network transmission)
- Optional: disable screenshot capture entirely
- Optional: disable clipboard monitoring
- Screen capture disabled during screen lock

---

## Developer Mode

### Debug Metrics
- Event throughput (events/second)
- Embedding generation speed (embeddings/second)
- Indexing queue status
- Vector search latency
- Memory usage
- Storage size

### Activation
- Keyboard shortcut: Cmd+Shift+D
- Hidden toggle in settings

---

## Build and Packaging

### Requirements
- Rust 1.70+
- Node.js 18+
- Xcode 14+ (for macOS)
- Tauri CLI 2.x

### Build Commands
```bash
# Development
npm run tauri dev

# Production build
npm run tauri build

# Package as .app
npm run tauri build -- --target aarch64-apple-darwin
```

### Output
- macOS: .app bundle in src-tauri/target/release/bundle/macos/
- Optional: .dmg installer

---

## Acceptance Criteria

### Core Functionality
- [ ] Application launches without errors
- [ ] Activity monitoring captures app switches
- [ ] Screenshot capture works at configured interval
- [ ] OCR extracts text from screenshots
- [ ] Embeddings generated and stored in vector DB
- [ ] Natural language search returns relevant results
- [ ] Timeline shows past activities grouped by session
- [ ] Knowledge graph visualizes connections

### UI/UX
- [ ] Skeuomorphic design matches specification
- [ ] Search bar is prominent and functional
- [ ] Memory cards display activity information
- [ ] Timeline is navigable and responsive
- [ ] Graph view shows connections
- [ ] All animations are smooth (60fps)

### Performance
- [ ] Background processing doesn't block UI
- [ ] Memory usage stays under 500MB typical
- [ ] CPU usage under 10% when idle
- [ ] Search results return within 500ms

### Privacy
- [ ] Pause button stops all monitoring
- [ ] Delete removes all stored data
- [ ] Excluded apps are not tracked
- [ ] Data never leaves local machine

### Developer Mode
- [ ] Debug metrics display correctly
- [ ] Metrics update in real-time
- [ ] Mode can be toggled on/off
