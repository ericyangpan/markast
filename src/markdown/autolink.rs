use crate::RenderOptions;
use crate::markdown::render::post_autolink;

pub(crate) fn post_process_document_html(html: String, options: RenderOptions) -> String {
    if options.gfm {
        return post_autolink(&html);
    }
    html
}
