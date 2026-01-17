use std::collections::HashMap;

use scraper::{Html, Selector};

// 병합된 셀 정보를 저장하는 구조체
#[derive(Debug, Clone)]
struct MergedCell {
    rowspan: usize,
    colspan: usize,
    content: String,
    attributes: HashMap<String, String>, // rowspan, colspan을 제외한 다른 속성들
}

pub fn transpose(html: &str) -> Result<String, String> {
    let document = Html::parse_document(html);
    
    let table_selector = Selector::parse("table").map_err(|e| format!("Failed to parse table selector: {}", e))?;
    let root = document.select(&table_selector).next()
        .ok_or("No <table> element found")?;
    
    let mut table_attributes: HashMap<String, String> = HashMap::new();
    for (attr_name, attr_value) in root.value().attrs() {
        table_attributes.insert(attr_name.to_string(), attr_value.to_string());
    }

    let tr_selector = Selector::parse("tr").map_err(|e| format!("Failed to parse tr selector: {}", e))?;
    let td_selector = Selector::parse("td, th").map_err(|e| format!("Failed to parse td/th selector: {}", e))?;

    let mut grid: Vec<Vec<Option<String>>> = Vec::new();
    let mut merged_cells: HashMap<(usize, usize), MergedCell> = HashMap::new();
    let mut cell_attributes: HashMap<(usize, usize), HashMap<String, String>> = HashMap::new();
    let mut occupied_positions: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    
    for (row_idx, row) in root.select(&tr_selector).enumerate() {
        if row_idx >= grid.len() {
            grid.push(Vec::new());
        }
        
        let mut col_idx = 0;
        
        while col_idx < grid[row_idx].len() && grid[row_idx][col_idx].is_some() {
            col_idx += 1;
        }
        
        for cell in row.select(&td_selector) {
            while col_idx < grid[row_idx].len() && grid[row_idx][col_idx].is_some() {
                col_idx += 1;
            }
            
            let rowspan = cell.value().attr("rowspan")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);
            let colspan = cell.value().attr("colspan")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(1);

            let content = cell.text().collect::<String>().trim().to_string();
            
            let mut attributes: HashMap<String, String> = HashMap::new();
            for (attr_name, attr_value) in cell.value().attrs() {
                let name = attr_name.to_string();
                if name != "rowspan" && name != "colspan" {
                    attributes.insert(name, attr_value.to_string());
                }
            }
            
            let needed_rows = row_idx + rowspan;
            let needed_cols = col_idx + colspan;
            
            while grid.len() < needed_rows {
                grid.push(Vec::new());
            }
            
            for r in 0..needed_rows {
                while grid[r].len() < needed_cols {
                    grid[r].push(None);
                }
            }
            
            grid[row_idx][col_idx] = Some(content.clone());
            
            if !attributes.is_empty() {
                cell_attributes.insert((row_idx, col_idx), attributes.clone());
            }
            
            if rowspan > 1 || colspan > 1 {
                merged_cells.insert((row_idx, col_idx), MergedCell {
                    rowspan,
                    colspan,
                    content: content.clone(),
                    attributes,
                });
            }
            
            for r in 0..rowspan {
                for c in 0..colspan {
                    if r == 0 && c == 0 {
                    } else {
                        grid[row_idx + r][col_idx + c] = Some("".to_string());
                        occupied_positions.insert((row_idx + r, col_idx + c));
                    }
                }
            }
            
            col_idx += colspan;
        }
    }
    
    let max_row = grid.len();
    let max_col = if max_row > 0 { grid[0].len() } else { 0 };

    let transposed_rows = max_col;
    let transposed_cols = max_row;
    let mut transposed_grid: Vec<Vec<Option<String>>> = vec![vec![None; transposed_cols]; transposed_rows];
    let mut transposed_merged_cells: HashMap<(usize, usize), MergedCell> = HashMap::new();
    let mut transposed_cell_attributes: HashMap<(usize, usize), HashMap<String, String>> = HashMap::new();
    let mut transposed_occupied_positions: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    for r in 0..max_row {
        for c in 0..max_col {
            transposed_grid[c][r] = grid[r][c].clone();
        }
    }

    for (row, col) in occupied_positions.iter() {
        transposed_occupied_positions.insert((*col, *row));
    }

    for ((row, col), attrs) in cell_attributes.iter() {
        if !merged_cells.contains_key(&(*row, *col)) {
            transposed_cell_attributes.insert((*col, *row), attrs.clone());
        }
    }

    for ((row, col), merged_cell) in merged_cells.iter() {
        transposed_merged_cells.insert((*col, *row), MergedCell {
            rowspan: merged_cell.colspan,
            colspan: merged_cell.rowspan,
            content: merged_cell.content.clone(),
            attributes: merged_cell.attributes.clone(),
        });
    }

    let mut html_output = String::from("<table");
    for (attr_name, attr_value) in &table_attributes {
        html_output.push_str(&format!(" {}=\"{}\"", attr_name, escape_attr_value(attr_value)));
    }
    html_output.push_str(">");
    
    for r in 0..transposed_rows {
        html_output.push_str("<tr>");
        
        let mut c = 0;
        while c < transposed_cols {
            if let Some(merged_cell) = transposed_merged_cells.get(&(r, c)) {
                html_output.push_str("<td");
                if merged_cell.rowspan > 1 {
                    html_output.push_str(&format!(" rowspan=\"{}\"", merged_cell.rowspan));
                }
                if merged_cell.colspan > 1 {
                    html_output.push_str(&format!(" colspan=\"{}\"", merged_cell.colspan));
                }
                for (attr_name, attr_value) in &merged_cell.attributes {
                    html_output.push_str(&format!(" {}=\"{}\"", attr_name, escape_attr_value(attr_value)));
                }
                html_output.push_str(">");
                html_output.push_str(&escape_html(&merged_cell.content));
                html_output.push_str("</td>");
                
                c += merged_cell.colspan;
            } else if let Some(Some(content)) = transposed_grid[r].get(c) {
                if !transposed_occupied_positions.contains(&(r, c)) {
                    html_output.push_str("<td");
                    if let Some(attrs) = transposed_cell_attributes.get(&(r, c)) {
                        for (attr_name, attr_value) in attrs {
                            html_output.push_str(&format!(" {}=\"{}\"", attr_name, escape_attr_value(attr_value)));
                        }
                    }
                    html_output.push_str(">");
                    html_output.push_str(&escape_html(content));
                    html_output.push_str("</td>");
                }
                c += 1;
            } else {
                html_output.push_str("<td></td>");
                c += 1;
            }
        }
        
        html_output.push_str("</tr>");
    }
    
    html_output.push_str("</table>");
    
    Ok(html_output)
}

// HTML 특수 문자 이스케이프
fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// HTML attribute 값 이스케이프
fn escape_attr_value(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    // 간단한 2x2 테이블 테스트
    #[test]
    fn test_simple_2x2_table() {
        let input = r#"<table>
            <tr><td>A</td><td>B</td></tr>
            <tr><td>C</td><td>D</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 전치 후: A C / B D
        assert!(result.contains("A") && result.contains("C") && result.contains("B") && result.contains("D"));
    }

    // 간단한 3x2 테이블 테스트
    #[test]
    fn test_simple_3x2_table() {
        let input = r#"<table>
            <tr><td>1</td><td>2</td></tr>
            <tr><td>3</td><td>4</td></tr>
            <tr><td>5</td><td>6</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 전치 후: 1 3 5 / 2 4 6
        assert!(result.contains("1") && result.contains("3") && result.contains("5"));
        assert!(result.contains("2") && result.contains("4") && result.contains("6"));
    }

    // colspan이 있는 케이스
    #[test]
    fn test_table_with_colspan() {
        let input = r#"<table>
            <tr><td colspan="2">Header</td></tr>
            <tr><td>A</td><td>B</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 전치 후: Header A / Header B (rowspan으로 변환됨)
        assert!(result.contains("Header"));
        assert!(result.contains("A") && result.contains("B"));
    }

    // rowspan이 있는 케이스
    #[test]
    fn test_table_with_rowspan() {
        let input = r#"<table>
            <tr><td rowspan="2">A</td><td>B</td></tr>
            <tr><td>C</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 전치 후: A B / A C (colspan으로 변환됨)
        assert!(result.contains("A"));
        assert!(result.contains("B") && result.contains("C"));
    }

    // rowspan과 colspan이 모두 있는 케이스
    #[test]
    fn test_table_with_rowspan_and_colspan() {
        let input = r#"<table>
            <tr><td rowspan="2" colspan="2">Merged</td><td>C</td></tr>
            <tr><td>F</td></tr>
            <tr><td>G</td><td>H</td><td>I</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        assert!(result.contains("Merged"));
        assert!(result.contains("C") && result.contains("F") && result.contains("G") && result.contains("H") && result.contains("I"));
    }

    // 복잡한 병합 셀이 많은 케이스
    #[test]
    fn test_complex_merged_cells() {
        let input = r#"<table>
            <tr><td rowspan="2">A</td><td colspan="2">BC</td></tr>
            <tr><td>D</td><td>E</td></tr>
            <tr><td>F</td><td rowspan="2" colspan="2">GH</td></tr>
            <tr><td>I</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        assert!(result.contains("A") && result.contains("BC") && result.contains("D") && result.contains("E"));
        assert!(result.contains("F") && result.contains("GH") && result.contains("I"));
    }

    // 매우 복잡한 케이스: 여러 병합 셀이 교차하는 경우
    #[test]
    fn test_very_complex_merged_cells() {
        let input = r#"<table>
            <tr><td rowspan="3" colspan="2">Big</td><td>C</td><td>D</td></tr>
            <tr><td>E</td><td>F</td></tr>
            <tr><td>G</td><td>H</td></tr>
            <tr><td>I</td><td rowspan="2">J</td><td>K</td><td>L</td></tr>
            <tr><td>M</td><td>N</td><td>O</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        assert!(result.contains("Big"));
        assert!(result.contains("C") && result.contains("D") && result.contains("E") && result.contains("F"));
        assert!(result.contains("G") && result.contains("H") && result.contains("I") && result.contains("J"));
        assert!(result.contains("K") && result.contains("L") && result.contains("M") && result.contains("N") && result.contains("O"));
    }

    // 루트 태그가 table이 아닌 경우 에러 테스트
    #[test]
    fn test_invalid_root_tag() {
        let result = transpose("<div>Hello</div>");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("No <table> element found"));
    }

    // 빈 테이블 테스트
    #[test]
    fn test_empty_table() {
        let input = "<table></table>";
        let result = transpose(input).unwrap();
        assert_eq!(result, "<table></table>");
    }

    // 단일 셀 테스트
    #[test]
    fn test_single_cell() {
        let input = "<table><tr><td>Only</td></tr></table>";
        let result = transpose(input).unwrap();
        assert!(result.contains("Only"));
    }

    // attribute가 있는 셀 테스트
    #[test]
    fn test_cell_with_attributes() {
        let input = r#"<table>
            <tr><td class="header" style="color: red;">이름</td><td class="data">홍길동</td></tr>
            <tr><td class="header" style="color: blue;">나이</td><td class="data">30</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // attribute가 유지되어야 함
        assert!(result.contains("class="));
        assert!(result.contains("style="));
        assert!(result.contains("header"));
        assert!(result.contains("data"));
        assert!(result.contains("이름") && result.contains("홍길동"));
        assert!(result.contains("나이") && result.contains("30"));
    }

    // 병합된 셀의 attribute 테스트
    #[test]
    fn test_merged_cell_with_attributes() {
        let input = r#"<table>
            <tr><td rowspan="2" class="merged" style="background: yellow;" data-id="1">A</td><td class="normal">B</td></tr>
            <tr><td class="normal">C</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 병합된 셀의 attribute가 유지되어야 함
        assert!(result.contains("class="));
        assert!(result.contains("style="));
        assert!(result.contains("data-id="));
        assert!(result.contains("merged"));
        assert!(result.contains("A") && result.contains("B") && result.contains("C"));
    }

    // 빈 셀이 있는 테이블 테스트
    #[test]
    fn test_table_with_empty_cell() {
        let input = r#"<table>
            <tr><td>A</td><td>B</td></tr>
            <tr><td>C</td><td></td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // 빈 셀이 유지되어야 함
        assert!(result.contains("A") && result.contains("B") && result.contains("C"));
        // 빈 셀도 출력되어야 함 (<td></td>)
        assert!(result.contains("<td></td>") || result.matches("<td").count() == 4);
    }

    // table 태그의 attribute 테스트
    #[test]
    fn test_table_with_attributes() {
        let input = r#"<table border="1" cellpadding="5" cellspacing="0" class="my-table" id="test-table">
            <tr><td>A</td><td>B</td></tr>
            <tr><td>C</td><td>D</td></tr>
        </table>"#;
        let result = transpose(input).unwrap();
        // table 태그의 attribute가 유지되어야 함
        assert!(result.contains("border="));
        assert!(result.contains("cellpadding="));
        assert!(result.contains("cellspacing="));
        assert!(result.contains("class="));
        assert!(result.contains("id="));
        assert!(result.contains("my-table"));
        assert!(result.contains("test-table"));
        assert!(result.contains("A") && result.contains("B") && result.contains("C") && result.contains("D"));
    }
}
