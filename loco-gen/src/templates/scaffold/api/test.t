{% set file_name = name |  snake_case -%}
{% set module_name = file_name | pascal_case -%}
to: tests/requests/{{ file_name }}.rs
skip_exists: true
message: "Tests for controller `{{module_name}}` was added successfully. Run `cargo test`."
injections:
- into: tests/requests/mod.rs
  append: true
  content: "pub mod {{ file_name }};"
---
use {{pkg_name}}::app::App;
use loco_rs::testing::prelude::*;
use serial_test::serial;
use serde_json::json;

#[tokio::test]
#[serial]
async fn can_get_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/{{ name | plural | snake_case }}/").await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_one_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            {% for column in columns -%}
            {%- if column.2 == "IntegerNull" -%}
            "{{column.0}}": 1,
            {%- else -%}
            "{{column.0}}": {{column.1}},
            {%- endif %}
            {% endfor -%}
        });

        let res = request.get("/api/{{ name | plural | snake_case }}/1").await;
        assert_eq!(res.status_code(), 200);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_create_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            {% for column in columns -%}
            {%- if column.2 == "IntegerNull" -%}
            "{{column.0}}": 1,
            {%- else -%}
            "{{column.0}}": {{column.1}},
            {%- endif %}
            {% endfor -%}
        });


        let res = request.post("/api/{{ name | plural | snake_case }}/1")
                    .json(&payload)
                    .await;
        assert_eq!(res.status_code(), 201);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = json!({
            {% for column in columns -%}
            {%- if column.2 == "IntegerNull" -%}
            "{{column.0}}": 1,
            {%- else -%}
            "{{column.0}}": {{column.1}},
            {%- endif %}
            {% endfor -%}
        });

        let res = request.put("/api/{{ name | plural | snake_case }}/1").await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}


#[tokio::test]
#[serial]
async fn can_remove_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.delete("/api/{{ name | plural | snake_case }}/1").await;
        assert_eq!(res.status_code(), 204);

        // you can assert content like this:
        // assert_eq!(res.text(), "content");
    })
    .await;
}
