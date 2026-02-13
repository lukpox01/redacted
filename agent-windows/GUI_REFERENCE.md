# GUI Button Visual Reference

## Window Layout

The GUI button creates a simple Windows window that looks like this:

```
┌─────────────────────────────────────────┐
│ School Management Agent            [_][□][X] │
├─────────────────────────────────────────┤
│                                         │
│                                         │
│        ┌─────────────────────┐          │
│        │  Open Web Page     │          │
│        └─────────────────────┘          │
│                                         │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

## Window Specifications

- **Window Size**: 300 pixels wide × 150 pixels tall
- **Window Title**: "School Management Agent"
- **Window Style**: Standard Windows window with title bar and system buttons
- **Position**: Default Windows position (CW_USEDEFAULT)

## Button Specifications

- **Button Text**: "Open Web Page"
- **Button Size**: 200 pixels wide × 40 pixels tall
- **Button Position**: 50 pixels from left, 40 pixels from top
- **Button Style**: Standard Windows push button

## Visual Behavior

### When Agent Starts
1. The window appears on the desktop immediately
2. The window is visible on top of other windows initially
3. Users can move, minimize, or interact with the window

### When Button is Clicked
1. Button appears pressed (standard Windows button animation)
2. Default browser opens in a new window or tab
3. Browser navigates to the configured URL (default: https://www.google.com)
4. Agent window remains open and functional

### Error State
- If URL fails to open, message appears in console log
- Window and button remain functional
- User can click again to retry

## Example Screenshot Placeholder

On a real Windows system, the window would look similar to:

```
╔═════════════════════════════════════════╗
║ School Management Agent            ═ □ × ║
╠═════════════════════════════════════════╣
║                                         ║
║                                         ║
║        ╔═══════════════════╗            ║
║        ║  Open Web Page  ║            ║
║        ╚═══════════════════╝            ║
║                                         ║
║                                         ║
║                                         ║
╚═════════════════════════════════════════╝
```

## Customization Examples

### School Portal Button
Change URL to school portal:
```rust
const DEFAULT_URL: &str = "https://portal.school.edu";
```

Result: Clicking button opens school portal

### Help Desk Button
Change URL to IT help desk:
```rust
const DEFAULT_URL: &str = "https://help.school.edu/tickets";
```

Result: Clicking button opens help desk ticket system

### Emergency Information
Change URL to emergency procedures:
```rust
const DEFAULT_URL: &str = "https://school.edu/emergency";
```

Result: Clicking button opens emergency information page

## Multi-Monitor Support

The window will appear on the primary monitor by default. Windows will handle:
- Positioning on primary monitor
- User can drag to any monitor
- Window remembers position if moved by user (OS feature)

## Accessibility

The button follows Windows accessibility standards:
- Keyboard navigation supported (Tab to focus, Enter to activate)
- Screen reader compatible (button text is read aloud)
- High contrast mode supported (inherits Windows theme)
- Font scaling follows Windows settings

## Window Behavior

### Always Present
- Window stays open as long as agent is running
- Closing the window stops the GUI thread but agent continues
- Users can minimize the window to taskbar

### Not Intrusive
- Window can be minimized
- Window can be moved out of the way
- Window doesn't force focus or steal input
- Window doesn't stay on top of other windows

### Integration with Agent
- GUI runs in separate thread
- Agent operations continue normally
- Button remains responsive during agent tasks
- No performance impact on agent functionality
