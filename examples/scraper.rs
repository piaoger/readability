use readability::extractor;

use std::os::raw::c_void;
use std::path::Path;
use std::{fs, io};

use std::fs::File;
use std::io::prelude::*;

// TODO:
//  1. try to integrate with html2md in the future
//     https://github.com/Adonai/html2md
//  2. seems neight readability and html2md nor are very robust
//

fn main() {
    let url = "https://blog.logrocket.com/rust-serialization-whats-ready-for-production-today/";
    match extractor::scrape(&url) {
        Ok(product) => {
            println!("------- html ------");
            println!("{}", product.content);

            println!("---- plain text ---");
            println!("{}", product.text);

            {
                let result = &product.content;
                let mut file = fs::File::create("result.html").unwrap();

                file.write_all(result.as_bytes()).unwrap();
            }

            {
                let result = &product.text;
                let mut file = fs::File::create("result.txt").unwrap();

                file.write_all(result.as_bytes()).unwrap();
            }
        }
        Err(_) => println!("error occured"),
    }
}
