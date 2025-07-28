# Rex

A terminal based spread sheet editor

### Presentation

This project is a spreadheet editor, and aims at implementing most of the common features from such tools.

Ideally, we should open, interact and save spreadsheets, in all common formats.
Focus is added towards productivity, with no errors, super fast actions (no waiting time) and easy to use bindings for fast manipulation.

### Status

At the time of me writing this, we have the MVP, we can:

- Open a CSV file
- navigate the file
- select cells
- write / delete cells
- save the file

### Incoming features

This readme is mostly for me to track the things I want to add:

- undo / redo
- formulas
- menu with with actions
  - search
  - navigate to cell
  - change layout settings (cell size)
- read xls / xlsx, with their added complexity:
  - style in cells
  - load / save from / to multiple formats
  - filters / sort on columns
- FORMULAS!
  - I'm hyped for this
  - formula parser
  - solver
  - need to know when to recompute

### Tech stack

This is implemented in rust, using ratatui / crossterm for rendering.

No other libraries are used
