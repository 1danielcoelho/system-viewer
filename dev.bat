cd dist
START "" devserver --reload
cd ..
cargo watch -d 0.05 -- build.bat