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

use {{pkg_name}}::model::_entities::{{ name | plural | snake_case }};

#[tokio::test]
#[serial]
async fn can_get_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let res = request.get("/api/{{ name | plural | snake_case }}/")
            .add_header("Content-Type", "application/json")
            .await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}

fn load_payload() -> serde::Json {
    serde_json::json!({
              {% for column in columns -%}
              {%- if "Vec<" in column.1 -%}
              item.{{column.0}} = Set(self.{{column.0}}.clone());
              {%- elif column.2 == "IntegerNull" -%}
              item.{{column.0}} = Set(self.{{column.0}});
              {%- elif "i32" in column.1 or "i64" in column.1 or "i16" in column.1 or "Uuid" in column.1 or "f32" in column.1 or "f64" in column.1 or "Decimal" in column.1 or "bool" in column.1 or "Date" in column.1 or "DateTime" in column.1 or "DateTimeWithTimeZone" in column.1 -%}
              item.{{column.0}} = Set(self.{{column.0}});
              {%- else -%}
              item.{{column.0}} = Set(self.{{column.0}}.clone());
              {%- endif %}
              {% endfor -%}
        })
}

#[tokio::test]
#[serial]
async fn can_create_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = load_payload();

        let res = request.post("/api/{{ name | plural | snake_case }}/");
            .add_header("Content-Type", "application/json")
            .json(&payload)
            .await;
        assert_eq!(res.status_code(), 201);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_get_one_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = load_payload();

        let res_create = request.post("/api/{{ name | plural | snake_case }}/");
            .add_header("Content-Type", "application/json")
            .json(&payload)
            .await;
        assert_eq!(res_create.status_code(), 201);

        let obj: {{ name | plural | snake_case }}::Model = res_create.json();
        let id = obj.id;

        let res = request.get(format!("/api/{{ name | plural | snake_case }}/{}", id));
            .add_header("Content-Type", "application/json")
            .await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_update_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = load_payload();

        let res_create = request.post("/api/{{ name | plural | snake_case }}/");
            .add_header("Content-Type", "application/json")
            .json(&payload)
            .await;
        assert_eq!(res_create.status_code(), 201);

        let obj: {{ name | plural | snake_case }}::Model = res_create.json();
        let id = obj.id;

        let payload_update = load_payload();

        let res = request.put(format!("/api/{{ name | plural | snake_case }}/{}", id));
            .add_header("Content-Type", "application/json")
            .json(&payload_update)
            .await;
        assert_eq!(res.status_code(), 200);
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_remove_{{ name | plural | snake_case }}() {
    request::<App, _, _>(|request, _ctx| async move {
        let payload = load_payload();

        let res_create = request.post("/api/{{ name | plural | snake_case }}/");
            .add_header("Content-Type", "application/json")
            .json(&payload)
            .await;
        assert_eq!(res_create.status_code(), 201);

        let obj: {{ name | plural | snake_case }}::Model = res_create.json();
        let id = obj.id;

        let res = request.delete(format!("/api/{{ name | plural | snake_case }}/{}", id));
            .add_header("Content-Type", "application/json")
            .await;
        assert_eq!(res.status_code(), 204);
    })
    .await;
}
