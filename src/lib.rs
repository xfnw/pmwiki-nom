pub mod pmwiki;
pub mod parser;

pub mod prelude {
    pub use crate::parser::{pmwikis, try_pmwikis};
    // pub use crate::pmwiki::{Pmwiki, Pmwikis, };
    pub use crate::pmwiki::IPmwiki;
}
