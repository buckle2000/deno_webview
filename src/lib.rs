use deno_core::plugin_api::Buf;
use deno_core::plugin_api::Interface;
use deno_core::plugin_api::Op;
use deno_core::plugin_api::ZeroCopyBuf;

use futures::future::FutureExt;
use futures::future::poll_fn;

use serde::Deserialize;
use serde::Serialize;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::*;
use std::ptr::null_mut;

use webview_sys::*;

thread_local! {
    static INSTANCE_INDEX: RefCell<u32> = RefCell::new(0);
    static INSTANCE_MAP: RefCell<HashMap<u32, *mut CWebView>> = RefCell::new(HashMap::new());
}

#[no_mangle]
pub fn deno_plugin_init(interface: &mut dyn Interface) {
    interface.register_op("webview_new", op_webview_new);
    // interface.register_op("webview_free", op_webview_free);
    interface.register_op("webview_exit", op_webview_exit);
    interface.register_op("webview_eval", op_webview_eval);
    // interface.register_op("webview_dispatch", op_webview_dispatch);
    interface.register_op("webview_set_color", op_webview_set_color);
    interface.register_op("webview_set_title", op_webview_set_title);
    interface.register_op("webview_set_fullscreen", op_webview_set_fullscreen);
    interface.register_op("webview_loop", op_webview_loop);
    interface.register_op("webview_get_user_data", op_webview_get_user_data);
}


#[derive(Serialize)]
struct WebViewResponse<T> {
    err: Option<String>,
    ok: Option<T>,
}

#[derive(Deserialize)]
struct WebViewNewParams {
    title: String,
    url: String,
    width: i32,
    height: i32,
    resizable: bool,
    debug: bool,
    frameless: bool,
}

#[derive(Serialize)]
struct WebViewNewResult {
    id: u32,
}

fn op_webview_new(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    let mut response: WebViewResponse<WebViewNewResult> = WebViewResponse {
        err: None,
        ok: None,
    };

    let params: WebViewNewParams = serde_json::from_slice(data).unwrap();

    let mut instance_id: u32 = 0;
    INSTANCE_INDEX.with(|cell| {
        instance_id = cell.replace_with(|&mut i| i + 1);
    });

    unsafe {
        INSTANCE_MAP.with(|cell| {
            let title = CString::new(params.title).unwrap();
            let url = CString::new(params.url).unwrap();

            cell.borrow_mut().insert(
                instance_id,
                webview_new(
                    title.as_ptr(),
                    url.as_ptr(),
                    params.width,
                    params.height,
                    params.resizable as i32,
                    params.debug as i32,
                    params.frameless as i32,
                    None, // Some(ffi_invoke_handler),
                    null_mut(),
                ),
            );
        });
    }

    response.ok = Some(WebViewNewResult { id: instance_id });

    let result: Buf = serde_json::to_vec(&response).unwrap().into_boxed_slice();

    Op::Sync(result)
}

// extern "C" fn ffi_invoke_handler(webview: *mut CWebView, arg: *const c_char) {
//     unsafe {
//         let arg = CStr::from_ptr(arg).to_string_lossy().to_string();
// 
//         println!("{}", arg);
//     }
// }

#[derive(Deserialize)]
struct WebViewExitParams {
    id: u32,
}

#[derive(Serialize)]
struct WebViewExitResult {}

fn op_webview_exit(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewExitResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewExitParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();

                webview_exit(instance);

                response.ok = Some(WebViewExitResult {});
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewEvalParams {
    id: u32,
    js: String,
}

#[derive(Serialize)]
struct WebViewEvalResult {}

fn op_webview_eval(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewEvalResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewEvalParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();
                let js = CString::new(params.js).unwrap();

                match webview_eval(instance, js.as_ptr()) {
                    0 => {
                        response.ok = Some(WebViewEvalResult {});
                    }
                    _ => response.err = Some("Could not evaluate javascript".to_string()),
                }
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewSetColorParams {
    id: u32,
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

#[derive(Serialize)]
struct WebViewSetColorResult {}

fn op_webview_set_color(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewSetColorResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewSetColorParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();

                webview_set_color(instance, params.r, params.g, params.b, params.a);

                response.ok = Some(WebViewSetColorResult {});
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewSetTitleParams {
    id: u32,
    title: String,
}

#[derive(Serialize)]
struct WebViewSetTitleResult {}

fn op_webview_set_title(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewSetTitleResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewSetTitleParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();
                let title = CString::new(params.title).unwrap();

                webview_set_title(instance, title.as_ptr());

                response.ok = Some(WebViewSetTitleResult {});
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewSetFullscreenParams {
    id: u32,
    fullscreen: bool,
}

#[derive(Serialize)]
struct WebViewSetFullscreenResult {}

fn op_webview_set_fullscreen(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewSetFullscreenResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewSetFullscreenParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();

                webview_set_fullscreen(instance, params.fullscreen as i32);

                response.ok = Some(WebViewSetFullscreenResult {});
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewLoopParams {
    id: u32,
    blocking: i32,
}

#[derive(Serialize)]
struct WebViewLoopResult {
    code: i32,
}

fn op_webview_loop(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewLoopResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewLoopParams = serde_json::from_slice(data).unwrap();

        INSTANCE_MAP.with(|cell| {
            let instance_map = cell.borrow_mut();

            if !instance_map.contains_key(&params.id) {
                response.err = Some(format!("Could not find instance of id {}", &params.id))
            } else {
                let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();

                response.ok = Some(WebViewLoopResult {
                    code: webview_loop(instance, params.blocking),
                });
            }
        });

        Op::Sync(serde_json::to_vec(&response).unwrap().into_boxed_slice())
    }
}

#[derive(Deserialize)]
struct WebViewGetUserDataParams {
    id: u32,
}

#[derive(Serialize)]
struct WebViewGetUserDataResult {}

fn op_webview_get_user_data(
    _interface: &mut dyn Interface,
    data: &[u8],
    _zero_copy: Option<ZeroCopyBuf>,
) -> Op {
    unsafe {
        let mut response: WebViewResponse<WebViewGetUserDataResult> = WebViewResponse {
            err: None,
            ok: None,
        };

        let params: WebViewGetUserDataParams = serde_json::from_slice(data).unwrap();

        let fut = async move {
            INSTANCE_MAP.with(|cell| {
                let instance_map = cell.borrow_mut();

                if !instance_map.contains_key(&params.id) {
                    response.err = Some(format!("Could not find instance of id {}", &params.id))
                } else {
                    let instance: *mut CWebView = *instance_map.get(&params.id).unwrap();
                    response.ok = Some(WebViewGetUserDataResult {})
                }
            });

            serde_json::to_vec(&response).unwrap().into_boxed_slice()
        };

        Op::Async(fut.boxed())
    }
}
