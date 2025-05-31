# Minimal Notes App - TODO List

## Features

### Search Improvements
- [ ] Add a clear search button
- [ ] Implement keyboard shortcuts for search (Ctrl+F/Cmd+F to focus)
- [ ] Add search result count indicator
- [ ] Implement more advanced search filters (by date, tags, etc.)
- [ ] Add search history

### Editor Enhancements
- [ ] Implement proper undo/redo functionality
- [ ] Add markdown support with preview
- [ ] Add text formatting options (bold, italic, etc.)
- [ ] Implement better data structures for editing (e.g., Rope or Piece Table)
- [ ] Add spell checking

### Note Organization
- [ ] Implement note categories/folders
- [ ] Add tagging system
- [ ] Add note pinning functionality
- [ ] Implement note sorting options (by date, title, etc.)
- [ ] Add note archiving

### UI/UX Improvements
- [ ] Add dark mode support
- [ ] Implement customizable themes
- [ ] Add keyboard shortcuts for common actions
- [ ] Improve accessibility features
- [ ] Add animations for smoother transitions

### Data Management
- [ ] Implement note export/import functionality
- [ ] Add backup and restore options
- [ ] Implement data encryption for sensitive notes
- [ ] Add sync capabilities (optional cloud integration)
- [ ] Improve performance for large notes/many notes

### Advanced Features
- [ ] Implement attachments (images, files)
- [ ] Add collaborative editing support
- [ ] Implement offline mode with sync
- [ ] Add reminders/notifications for notes
- [ ] Implement version history for notes

## Technical Improvements

### Performance
- [ ] Optimize search for large numbers of notes
- [ ] Implement lazy loading for note content
- [ ] Use better data structure for note content modification
- [ ] Improve startup time? Binary size?
- [ ] Optimize embedding

### Code Quality
- [ ] Add comprehensive test suite
- [ ] Improve error handling
- [ ] Add better logging
- [ ] Refactor code for better maintainability
- [ ] Document code more thoroughly

### Build & Deployment
- [ ] Set up CI/CD pipeline
- [ ] Add automated testing
- [ ] Implement auto-updates
- [ ] Create installers for different platforms
- [ ] Add telemetry for crash reporting (opt-in)

TODO
Figure out the source of these warnings and maybe add a production filter

2025-05-29 16:30:17.627 minimal-notes[99817:116559894] _TIPropertyValueIsValid called with 16 on nil context!
2025-05-29 16:30:17.627 minimal-notes[99817:116559894] imkxpc_getApplicationProperty:reply: called with incorrect property value 16, bailing.
2025-05-29 16:30:17.627 minimal-notes[99817:116559894] Text input context does not respond to _valueForTIProperty: