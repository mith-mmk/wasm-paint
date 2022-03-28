use crate::Universe;
use crate::log;
use wml2::draw::CallbackResponse;
use wml2::draw::VerboseOptions;
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;


type Error = Box<dyn std::error::Error>;

pub(crate) fn write_log(str: &str,_: Option<VerboseOptions>) -> Result<Option<CallbackResponse>,Error> {
    if web_sys::window().is_some() {
        let window = web_sys::window().unwrap();
        if window.document().is_some() {
            let document = window.document().unwrap();
            if document.get_element_by_id("wasm_verbose").is_some() {
                let elmid = document.get_element_by_id("wasm_verbose").unwrap();
                if elmid.dyn_ref::<HtmlElement>().is_some() {
                    let elm = elmid.dyn_ref::<HtmlElement>().unwrap();
                    elm.set_inner_text(str);
                    return Ok(None)
                }
            }
        }
    }
    log(str);
    Ok(None)
}

#[wasm_bindgen(js_name = bindCanvas)]
pub fn bind_canvas(universe:&mut Universe,canvas:&str) {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id(canvas).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();
    universe.set_2d_context(context);
}

#[wasm_bindgen(js_name = drawCanvas)]
pub fn draw_canvas() {

}