/// When moving (or validating, editing, etc..) cells, where do we go
pub enum SelectionDirection {
    /// Don't move at all
    Stay,
    /// Go to the next cell group (left)
    Next,
    /// Go back on the next line (down)
    Return,
}

pub enum Command {
    RedrawRequest,
    WriteCells {
        cells: crate::file::cell::CellRect,
        content: crate::file::cell::Cell,
        next_selection: SelectionDirection,
    },
    DeleteCells {
        cells: crate::file::cell::CellRect,
        next_selection: SelectionDirection,
    },
}
