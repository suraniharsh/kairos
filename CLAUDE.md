# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Ritual is a terminal-based task logger and time tracker built with TypeScript, React, and Ink. It features a three-pane layout (Calendar, Tasks, Timeline) with keyboard-centric workflows, infinite nested subtasks, and local JSON persistence.

**Tech Stack**: TypeScript + React + Ink v6.6.0

## Development Commands

### Core Development

```bash
pnpm dev              # Development mode with hot reload (tsx)
pnpm build            # Compile TypeScript to dist/
pnpm start            # Run compiled production version
```

### Testing

```bash
pnpm test             # Run tests in watch mode (Vitest)
pnpm test:run         # Run tests once (CI mode)
pnpm test:coverage    # Run tests with coverage report
pnpm test:ui          # Run tests with UI interface
```

### Code Quality

```bash
pnpm format           # Format all files with Prettier
pnpm format:check     # Check formatting without modifying
pnpm sonar            # Run SonarQube analysis (requires SONAR_TOKEN env var)
```

### CLI Commands

```bash
ritual                # Launch interactive TUI
ritual export         # Export data to sync server
ritual import         # Import data from sync server
ritual --version      # Show version
ritual --help         # Show help
```

### Server Development (in `server/` directory)

```bash
cd server
npm run dev           # Development mode with hot reload
npm run build         # Build for production
npm start             # Run production build
npm run typecheck     # TypeScript type checking
```

## Architecture

### Three-Pane Layout System

The app uses a three-column layout with keyboard navigation:

- **Calendar Pane**: Date selection and task count visualization
- **Tasks Pane**: Recursive task list with infinite nesting
- **Timeline Pane**: Activity log with timestamps

```
┌─────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  Calendar   │  │      Tasks       │  │    Timeline      │
│             │  │                  │  │                  │
│ (navigable) │  │ (add/edit/check) │  │ (activity log)   │
│             │  │                  │  │                  │
└─────────────┘  └──────────────────┘  └──────────────────┘
```

### Context-Based State Management

Global state is managed through React Context providers (no Redux):

- `ThemeContext`: Theme switching and color schemes
- `StorageContext`: File I/O with auto-save (500ms debounce)
- `AppContext`: Selected date, tasks, timeline, UI state
- `UndoContext`: Undo/redo functionality for task operations

### Service Layer Pattern

Business logic is separated into services in `src/services/`:

- `taskService`: CRUD operations, tree manipulation, state transitions
- `timelineService`: Event creation and formatting
- `calendarService`: Month generation and navigation
- `storageService`: JSON file persistence with date hydration
- `recurringTaskService`: Recurring task generation and management
- `taskMoveService`: Task moving between dates

### Component Structure

```
src/components/
├── calendar/          # Calendar pane and navigation
├── tasks/             # Task list, items, and editing
├── timeline/          # Timeline pane and activity log
├── common/            # Shared dialogs (Help, Theme, Settings)
├── layout/            # Layout components (ThreeColumnLayout)
└── overview/          # Overview screen
```

### Data Models

**Task** (`src/types/task.ts`):

```typescript
{
  id: string;                    // UUID
  title: string;
  state: 'todo' | 'completed' | 'delegated' | 'delayed';
  createdAt: Date;
  updatedAt: Date;
  startTime?: Date;              // When task started
  endTime?: Date;                // When task completed/delegated/delayed
  children: Task[];              // Infinite nesting support
  parentId?: string;
  date: string;                  // YYYY-MM-DD
  recurrence?: RecurrencePattern;
  isRecurringInstance?: boolean;
  recurringParentId?: string;
}
```

**Timeline Event** (`src/types/timeline.ts`):

```typescript
{
  id: string;
  taskId: string;
  taskTitle: string;
  type: 'created' | 'started' | 'completed' | 'delegated' | 'delayed' | 'updated';
  timestamp: Date;
  previousState?: TaskState;
  newState?: TaskState;
}
```

**Storage Schema** (`src/types/storage.ts`):

```typescript
{
  version: string;
  tasks: { [date: string]: Task[] };
  timeline: { [date: string]: TimelineEvent[] };
  settings: UserSettings;
}
```

**UserSettings** (`src/types/storage.ts`):

```typescript
{
  theme: string;
  defaultStartTime: 'now' | 'custom';
  dateFormat: string;
  timeFormat: '12h' | '24h';
  skippedVersion?: string;
  autoMoveUnfinishedTasks: boolean;
}
```

### Theme System

Themes are defined in `src/themes/` with 30+ built-in themes:

- **Dark themes**: dark, terminal, catppuccin, nord, github-dark, cursor, etc.
- **Light themes**: light, claude, github-light, catppuccin-latte, cursor-light, etc.
- Theme registration in `src/themes/index.ts`

**Dark Theme (Default)**:

- Background: #1e1e1e (VS Code dark)
- Calendar selected: #c586c0 (purple)
- Task completed: #4ec9b0 (teal)
- Task delegated: #dcdcaa (yellow)
- Task delayed: #f48771 (red)

**Light Theme**:

- Background: #ffffff
- Calendar selected: #9933cc (purple)
- Task completed: #00aa77 (green)
- Task delegated: #ff9900 (orange)
- Task delayed: #cc3333 (red)

## Key Design Decisions

- **Ink vs Other TUIs**: React component model familiar to React developers
- **JSON vs Database**: Simple, portable, human-readable, single-file persistence
- **Context vs Redux**: Simpler for app this size, no boilerplate
- **Recursive TaskList**: Elegant infinite nesting using React's strength
- **Auto-save with debounce**: Prevents excessive I/O while keeping data current
- **Date hydration**: Storage converts dates to ISO strings, hydrates back to Date objects on load
- **date-fns**: Smaller than moment.js, functional API

## Data Persistence

- **Location**: Platform-specific config directory
  - **macOS**: `~/Library/Application Support/ritual/data.json`
  - **Linux**: `~/.local/share/ritual/data.json` (or `$XDG_DATA_HOME`)
  - **Windows**: `%APPDATA%\ritual\data.json`
- **Auto-save**: 500ms debounce after any state change
- **Graceful exit**: Saves on process termination
- **Error handling**: Creates default schema if file corrupted or missing
- **Date handling**: Dates stored as ISO strings, hydrated to Date objects

## Cross-Machine Sync

The `server/` directory contains an Express.js + Redis server for temporary data sharing:

- **Endpoints**: `/api/health`, `/api/data/export`, `/api/data/import`
- **Storage**: Upstash Redis with 5-minute TTL
- **Security**: bcrypt code hashing, rate limiting (100 req/15min)
- **Usage**: `ritual export` generates 8-character code, `ritual import` retrieves data

**Export Workflow**:

```bash
ritual export
# Output: Secret Key: aB3dE7fG, Expires: 1/4/2026, 7:45:30 PM
```

**Import Workflow**:

```bash
ritual import
# Enter your secret key: aB3dE7fG
# Choose (r)eplace or (m)erge with existing data
```

## Keyboard Shortcuts

### Global Navigation

- `1` / `2` / `3`: Switch directly to Calendar / Tasks / Timeline panes
- `Tab` / `Shift+Tab`: Cycle through panes
- `?`: Toggle help dialog
- `q`: Quit application

### Calendar Pane

- `j` / `k` (or `↓` / `↑`): Navigate weeks
- `h` / `l` (or `←` / `→`): Navigate days
- `n` / `p`: Next / Previous month
- `Enter`: Select date

### Tasks Pane

- `a`: Add new task
- `e`: Edit task title
- `d`: Delete task
- `Space`: Toggle completion status
- `s`: Start task (sets start time)
- `D`: Mark as delegated
- `x`: Mark as delayed/cancelled
- `Tab`: Indent task (convert to subtask)
- `Shift+Tab`: Unindent task
- `Enter`: Expand/Collapse subtasks

### Timeline Pane

- `j` / `k`: Scroll through activity history
- `t`: Toggle theme (Dark/Light)

## Testing Approach

- **Framework**: Vitest with happy-dom environment
- **Test location**: `__tests__/` directory
- **Coverage**: v8 provider with text, json, html, lcov reporters
- **Setup**: `__tests__/setup.ts` for test configuration
- **Path alias**: `@` maps to `./src`

## Important File Locations

### Core Application

- **Entry point**: `src/index.tsx` - CLI argument handling and app rendering
- **Main app**: `src/App.tsx` - Context providers and main layout
- **CLI commands**: `src/cli.ts` - export/import functionality
- **Keyboard navigation**: `src/hooks/useKeyboardNav.ts` - Global keyboard shortcuts
- **Terminal size**: `src/hooks/useTerminalSize.ts` - Responsive terminal handling

### Types

- `src/types/task.ts`: Task, TaskState, TaskTree, RecurrencePattern
- `src/types/timeline.ts`: TimelineEvent, TimelineEventType
- `src/types/calendar.ts`: CalendarView, CalendarDay
- `src/types/theme.ts`: Theme, ColorScheme
- `src/types/storage.ts`: StorageSchema, UserSettings
- `src/types/undo.ts`: Undo/redo types
- `src/types/app.ts`: App state types
- `src/types/recurring.ts`: Recurring task types

### Services

- `src/services/taskService.ts`: Task operations (create, update, delete, state changes)
- `src/services/timelineService.ts`: Event management
- `src/services/calendarService.ts`: Calendar calculations
- `src/services/storage.ts`: File I/O and persistence
- `src/services/recurringTaskService.ts`: Recurring task generation
- `src/services/taskMoveService.ts`: Task moving between dates

### Contexts

- `src/contexts/ThemeContext.tsx`: Theme provider
- `src/contexts/StorageContext.tsx`: Auto-save storage with debouncing
- `src/contexts/AppContext.tsx`: Global app state
- `src/contexts/UndoContext.tsx`: Undo/redo functionality

### Components

- `src/components/calendar/CalendarPane.tsx`: Calendar with month navigation
- `src/components/tasks/TasksPane.tsx`: Main task management
- `src/components/tasks/TaskList.tsx`: **Recursive** component for infinite nesting
- `src/components/timeline/TimelinePane.tsx`: Activity log viewer
- `src/components/common/HelpDialog.tsx`: Keyboard shortcuts
- `src/components/common/ThemeDialog.tsx`: Theme selection
- `src/components/common/SettingsDialog.tsx`: Settings management
- `src/components/common/RecurringTaskDialog.tsx`: Recurring task setup
- `src/components/common/RecurringEditDialog.tsx`: Recurring task editing
- `src/components/common/ClearTimelineDialog.tsx`: Timeline clearing
- `src/components/common/UpdateDialog.tsx`: Version update notifications
- `src/components/overview/OverviewScreen.tsx`: Overview screen
- `src/components/layout/ThreeColumnLayout.tsx`: Three-pane layout

### Utilities

- `src/utils/date.ts`: Date formatting, month generation
- `src/utils/tree.ts`: Tree traversal, task lookup
- `src/utils/validation.ts`: Time constraints, title validation
- `src/utils/task.ts`: Task utilities
- `src/utils/logger.ts`: Logging utilities
- `src/utils/version.ts`: Version management

## Development Workflow

1. **Branch naming**: Use prefixes `feat/`, `fix/`, `docs/`, `refactor/`, `test/`, `chore/`
2. **Pre-commit hook**: Automatically runs Prettier on staged files
3. **SonarQube**: Optional local analysis (non-blocking if server unavailable)
4. **Commit messages**: Use imperative mood (e.g., "feat: add recurring task support")

## Extension Points

### Adding a New Theme

1. Create `src/themes/mytheme.ts` with ColorScheme
2. Import and register in `src/themes/index.ts`
3. Add to appropriate theme collection (lightThemes or darkThemes)
4. Theme available in UI via theme toggle

### Adding Task State

1. Add to TaskState type in `src/types/task.ts`
2. Add colors to ColorScheme in themes
3. Update stateIcons/stateColors in TaskItem components
4. Update timelineService event types

### Adding Keyboard Shortcut

1. Modify `useKeyboardNav.ts` for global shortcuts
2. Add pane-specific shortcuts in respective Pane components
3. Update `HelpDialog.tsx` shortcuts reference

### Adding Recurring Task Pattern

1. Add to RecurrenceFrequency type in `src/types/task.ts`
2. Implement pattern logic in `recurringTaskService.ts`
3. Update UI dialogs to support new pattern

## Server Architecture

The sync server (`server/`) follows Express.js patterns:

- **Config**: `server/src/config/` - Redis, environment, Swagger
- **Controllers**: `server/src/controllers/` - Request handlers
- **Middleware**: `server/src/middleware/` - Express middleware
- **Routes**: `server/src/routes/` - API route definitions
- **Services**: `server/src/services/` - Business logic
- **Types**: `server/src/types/` - TypeScript types
- **Utils**: `server/src/utils/` - Utility functions

**Server Features**:

- Health check endpoint: `GET /api/health`
- Export endpoint: `POST /api/data/export`
- Import endpoint: `POST /api/data/import`
- Helmet for security headers
- Rate limiting (100 req/15min)
- Code hashing with bcrypt
- Comprehensive logging with Winston
- Swagger API documentation

## Environment Variables

**Main app** (`.env`):

- `RITUAL_SERVER_URL`: URL for cross-machine sync server

**Server** (`server/.env`):

- `UPSTASH_REDIS_REST_URL`: Upstash Redis URL
- `UPSTASH_REDIS_REST_TOKEN`: Upstash Redis token
- `HASH_SECRET`: Secret for code hashing
- `PORT`: Server port (default: 3000)

## Build Configuration

- **TypeScript**: ES2022 target, ESNext module, React JSX
- **Output**: `dist/` directory with ESM format
- **Package**: tsup for bundling, clean build on each compile
- **Binary**: `ritual` command points to `dist/index.js`

## Testing Checklist

- [ ] Calendar: Navigate months (n/p), select dates (Enter)
- [ ] Tasks: Add (a), complete (Space), delete (d)
- [ ] Nesting: Create subtasks (Tab), expand/collapse (Enter)
- [ ] Timeline: Auto-updates on task state changes
- [ ] Themes: Dark/light switch (t), colors apply correctly
- [ ] Persistence: Data survives app restart
- [ ] Help: All shortcuts documented and work as described
- [ ] Recurring tasks: Pattern generation and editing
- [ ] Undo/redo: Task operations can be undone
- [ ] Cross-machine sync: Export/import functionality

## Future Enhancements

- Task editing (e key not yet functional)
- Search and filter tasks
- Task priorities and categories
- Subtask indentation improvements
- Time estimate vs actual tracking
- Export to Markdown/CSV
- Custom user themes
- Advanced recurring patterns
- Task dependencies
