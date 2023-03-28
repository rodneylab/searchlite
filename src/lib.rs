mod html_search;
mod utils;

//use html2text::from_read;
use html_search::search_html;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

// macro_rules! console_log {
//      ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
//  }

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn highlight_search_terms(input_string: &str, search_pattern: &str) -> String {
    search_html(input_string, search_pattern)
}

//#[wasm_bindgen]
//pub fn html_to_text(html: &str) -> String {
//    let output_text_columns: usize = 72;
//    let text = from_read(html.as_bytes(), output_text_columns);
//    text
//}
