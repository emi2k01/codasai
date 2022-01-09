mod guide;
mod index;
mod page;
mod workspace;

pub use guide::GuideContext;
pub use index::{Index, IndexEntry};
pub use page::PageContext;
use serde::Serialize;
pub use workspace::{Directory, File, WorkspaceOutlineBuilder};

#[derive(Serialize)]
pub struct GlobalContext<'a> {
    pub page: &'a PageContext,
    pub guide: &'a GuideContext,
}
