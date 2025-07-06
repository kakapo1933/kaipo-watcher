# Known Issues

Currently, there are no known unresolved issues with the project. All previously identified issues have been fixed.

## Recently Resolved Issues

For reference, here are issues that were recently fixed:

### 1. Clap Argument Name Conflict (Fixed: 2025-07-06)
- **Issue**: Short flag `-i` conflict between `interface` and `interval` arguments
- **Resolution**: Used `-I` for interface and `-i` for interval

### 2. Integer Overflow in Bandwidth Calculation (Fixed: 2025-07-06)
- **Issue**: Dashboard crashed with overflow when network counters reset
- **Resolution**: Implemented saturating subtraction to handle counter resets gracefully

---

If you encounter any new issues, please report them by creating a new entry in this file.