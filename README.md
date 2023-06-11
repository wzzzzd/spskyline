# spskyline
1. Install Rust (https://www.rust-lang.org/learn/get-started)
2. Enter the following command in the shell to execute the query:
```bash
$ cargo run --release -- \
    -e Yago_small/edge.txt \
    -n Yago_small/node_keywords.txt \
    9567561,11381939,8757362 \
    11005257,8599768
```
