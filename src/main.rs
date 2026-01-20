use std::fs;
use std::io::{self, Read};
use std::env;

use html_transpose::transpose;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    // 사용법 출력
    if args.len() < 2 {
        eprintln!("사용법:");
        eprintln!("  {} <입력파일> [출력파일]", args[0]);
        eprintln!("  {} - (stdin에서 입력받아 stdout으로 출력)", args[0]);
        eprintln!();
        eprintln!("예시:");
        eprintln!("  {} input.html output.html", args[0]);
        eprintln!("  {} input.html", args[0]);
        eprintln!("  cat input.html | {} -", args[0]);
        std::process::exit(1);
    }

    // HTML 입력 읽기
    let html_input = if args[1] == "-" {
        // stdin에서 읽기
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("stdin에서 읽기 실패");
        buffer
    } else {
        // 파일에서 읽기
        fs::read_to_string(&args[1])
            .unwrap_or_else(|_| panic!("파일 읽기 실패: {}", args[1]))
    };

    // HTML 테이블 전치 수행
    let transposed_html = match transpose(&html_input) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("에러: {}", e);
            std::process::exit(1);
        }
    };

    // 결과 출력
    if args.len() >= 3 {
        // 출력 파일로 저장
        fs::write(&args[2], &transposed_html)
            .unwrap_or_else(|_| panic!("파일 쓰기 실패: {}", args[2]));
        println!("전치된 HTML이 {} 파일에 저장되었습니다.", args[2]);
    } else if args[1] == "-" {
        // stdout으로 출력
        print!("{}", transposed_html);
    } else {
        // 입력 파일명에 .transposed.html 추가하여 저장
        let output_file = format!("{}.transposed.html", 
            args[1].trim_end_matches(".html").trim_end_matches(".htm"));
        fs::write(&output_file, &transposed_html)
            .unwrap_or_else(|_| panic!("파일 쓰기 실패: {}", output_file));
        println!("전치된 HTML이 {} 파일에 저장되었습니다.", output_file);
    }
}
