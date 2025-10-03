# 导出 API

memscope-rs 导出功能的完整参考文档。

## 导出接口

### `export_to_json()`

导出为 JSON 格式。

```rust
use memscope_rs::export::export_to_json;

export_to_json("analysis_result.json")?;
```

### `export_to_html()`

导出为交互式 HTML 仪表板。

```rust
use memscope_rs::export::export_to_html;

export_to_html("dashboard.html")?;
```

## 导出选项

### `ExportOptions`

配置导出行为的选项。

## 使用示例

详细的导出示例和配置说明。