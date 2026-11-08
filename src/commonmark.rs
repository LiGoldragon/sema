//! The one place comrak, the CommonMark library under the Document, is
//! configured: the same options read the text and print it.

use comrak::Options;
use comrak::options::ListStyleType;

pub(crate) trait Configured {
    fn sema() -> Self;
}

impl Configured for Options<'static> {
    fn sema() -> Self {
        let mut options = Options::default();
        options.extension.table = true;
        options.render.width = 0;
        options.render.list_style = ListStyleType::Dash;
        options.render.prefer_fenced = true;
        options.render.experimental_minimize_commonmark = true;
        options
    }
}
