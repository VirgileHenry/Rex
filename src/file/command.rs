pub enum Command {
    RedrawRequest,
    WriteCells {
        cells: crate::file::cell::CellRect,
        content: crate::file::cell::Cell,
        next_selection: Option<crate::file::cell::CellRect>,
    },
    DeleteCells {
        cells: crate::file::cell::CellRect,
        next_selection: Option<crate::file::cell::CellRect>,
    },
}
