# html_transpose

A Rust library for transposing HTML tables while preserving merged cells, attributes, and structure.

## Overview

`html_transpose` is a lightweight Rust library that transposes HTML tables (swaps rows and columns) while correctly handling:
- Merged cells (`rowspan` and `colspan`)
- Cell and table attributes
- Complex table structures
- Empty cells

When a table is transposed, `rowspan` and `colspan` attributes are automatically swapped to maintain the correct visual structure.

## Features

- ✅ **Table Transposition**: Swaps rows and columns of HTML tables
- ✅ **Merged Cell Support**: Correctly handles `rowspan` and `colspan` attributes
- ✅ **Attribute Preservation**: Maintains all attributes on both table and cell elements
- ✅ **HTML Escaping**: Properly escapes HTML special characters
- ✅ **Error Handling**: Returns descriptive errors for invalid input
- ✅ **Well Tested**: Comprehensive test suite covering various edge cases

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
html_transpose = "0.1.0"
scraper = "0.25.0"
```

## Usage

```rust
use html_transpose::transpose;

let html_table = r#"
    <table>
        <tr><td>A</td><td>B</td></tr>
        <tr><td>C</td><td>D</td></tr>
    </table>
"#;

match transpose(html_table) {
    Ok(transposed) => println!("{}", transposed),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Examples

### Simple 2x2 Table

**Input:**
```html
<table>
    <tr><td>A</td><td>B</td></tr>
    <tr><td>C</td><td>D</td></tr>
</table>
```

**Output:**
```html
<table>
    <tr><td>A</td><td>C</td></tr>
    <tr><td>B</td><td>D</td></tr>
</table>
```

### Table with Colspan

**Input:**
```html
<table>
    <tr><td colspan="2">Header</td></tr>
    <tr><td>A</td><td>B</td></tr>
</table>
```

**Output:**
```html
<table>
    <tr><td rowspan="2">Header</td><td>A</td></tr>
    <tr><td>B</td></tr>
</table>
```

Note: `colspan` becomes `rowspan` after transposition.

### Table with Rowspan

**Input:**
```html
<table>
    <tr><td rowspan="2">A</td><td>B</td></tr>
    <tr><td>C</td></tr>
</table>
```

**Output:**
```html
<table>
    <tr><td colspan="2">A</td><td>B</td></tr>
    <tr><td>C</td></tr>
</table>
```

Note: `rowspan` becomes `colspan` after transposition.

### Table with Attributes

**Input:**
```html
<table border="1" class="my-table">
    <tr>
        <td class="header" style="color: red;">Name</td>
        <td class="data">John</td>
    </tr>
    <tr>
        <td class="header" style="color: blue;">Age</td>
        <td class="data">30</td>
    </tr>
</table>
```

**Output:**
```html
<table border="1" class="my-table">
    <tr>
        <td class="header" style="color: red;">Name</td>
        <td class="header" style="color: blue;">Age</td>
    </tr>
    <tr>
        <td class="data">John</td>
        <td class="data">30</td>
    </tr>
</table>
```

All attributes are preserved and correctly positioned after transposition.

## API

### `transpose(html: &str) -> Result<String, String>`

Transposes an HTML table string.

**Parameters:**
- `html`: A string containing an HTML table (must contain a `<table>` element)

**Returns:**
- `Ok(String)`: The transposed HTML table as a string
- `Err(String)`: An error message if the input is invalid

**Errors:**
- Returns an error if no `<table>` element is found in the input
- Returns an error if the HTML parser fails

## Testing

Run the test suite:

```bash
cargo test
```

The library includes comprehensive tests covering:
- Simple tables of various sizes
- Tables with `rowspan` and `colspan`
- Complex merged cell scenarios
- Tables with attributes
- Edge cases (empty tables, single cells, etc.)

## Implementation Details

The library works by:

1. **Parsing**: Uses the `scraper` crate to parse HTML
2. **Grid Construction**: Converts the HTML table into a 2D grid, expanding merged cells
3. **Transposition**: Swaps rows and columns in the grid
4. **Merged Cell Conversion**: Converts `rowspan` ↔ `colspan` appropriately
5. **Reconstruction**: Rebuilds the HTML table from the transposed grid

## Dependencies

- `scraper`: HTML parsing and CSS selector support

## License

This project is available for use under your preferred license.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
