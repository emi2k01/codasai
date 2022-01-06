mod guide;
mod index;
mod page;
mod workspace;

pub use guide::GuideContext;
pub use index::{Index, IndexEntry};
use serde::Serialize;
pub use workspace::{Directory, File, WorkspaceOutlineBuilder};
pub use page::PageContext;

#[derive(Serialize)]
pub struct GlobalContext<'a> {
    pub page: &'a PageContext,
    pub guide: &'a GuideContext,
}
