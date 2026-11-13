//! The optional console sink uses the server's release, without a backend.
#![cfg(all(target_arch = "wasm32", feature = "browser"))]

use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn console_json_has_the_servers_release_tag() {
    let setup = js_sys::Function::new_no_args(
        r"
        for (const [name, content] of [
            ['search-release-env', 'v9.8.7'], ['search-release-sha', 'abcdef0'],
            ['search-deployment-environment', 'test']
        ]) {
            const meta = document.createElement('meta');
            meta.name = name; meta.content = content; document.head.appendChild(meta);
        }
        window.__searchLogs = [];
        window.__searchConsole = console.log;
        console.log = line => window.__searchLogs.push(line);
    ",
    );
    setup.call0(&wasm_bindgen::JsValue::NULL).unwrap();
    search_front::telemetry::init_browser();
    search_front::telemetry::init_browser();
    let read = js_sys::Function::new_no_args(
        "console.log = window.__searchConsole; return JSON.stringify(window.__searchLogs)",
    );
    let text = read
        .call0(&wasm_bindgen::JsValue::NULL)
        .unwrap()
        .as_string()
        .unwrap();
    let lines: Vec<String> = serde_json::from_str(&text).unwrap();
    assert_eq!(lines.len(), 1);
    let log: serde_json::Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(log["event.name"], "browser.started");
    assert_eq!(log["service.namespace"], "palicanon");
    assert_eq!(log["service.name"], "search");
    assert_eq!(log["service.version"], "v9.8.7");
    assert_eq!(log["service.commit"], "abcdef0");
    assert_eq!(log["deployment.environment.name"], "test");
    assert!(log["timestamp"].as_str().unwrap().ends_with('Z'));
}
